# WIREFRAME.md

This document describes the shipped trex interface and the read-only JSON snapshot contract that companion tools consume.

## Product Shape

trex has two public surfaces:

- `trex`: interactive ratatui tmux session manager.
- `trex snapshot --json`: non-interactive backend snapshot for status bars, launchers, and desktop integrations.

The TUI is stateful and can request tmux actions after terminal cleanup. The snapshot command is read-only and must not create, attach, switch, delete, or detach tmux sessions.

## Normal Mode

```text
+------------------------------------------------------------------------------+
| trex system overview: sessions, agents, CPU, memory, health                  |
+------------------------------------------------------------------------------+
| RUNNING AGENTS                                                               |
|  > codex:trex ●        claude:api ○        zoyd:ui ● (claude)                |
+------------------------------------------------------------------------------+
| Sessions (N) - activity, attached marker, health, git                         |
| > ● ★ trex 🟢 (2 win) 12s main +2                                             |
|   CPU  12.5% [██████      ]  MEM  512MB [███         ]                       |
|   ▁▂▃▅▂▁                         ▁▁▂▂▃▂                                    |
|                                                                              |
|   ○ ☆ scratch 🟡 (1 win) 4m                                                    |
|   CPU   0.0% [            ]  MEM   96MB [█           ]                       |
|   ▁▁▁▁▁▁                         ▁▁▁▁▁▁                                    |
+------------------------------------------------------------------------------+
| j/k nav | l expand | p preview | b charts | s stats | enter attach | q quit |
+------------------------------------------------------------------------------+
```

Responsibilities:

- `src/tui/ui/overview.rs` renders the top system summary.
- `src/tui/ui/agents.rs` renders the agent panel and parent-child agent labels.
- `src/tui/ui/sessions.rs` renders the session list frame and scrollbar.
- `src/tui/ui/session_row.rs` renders session headers, gauges, health, git badges, and sparklines.
- `src/tui/ui/normal.rs` assembles the normal layout and help line.

## Focus Model

```text
Agents focus
  j/Down at bottom -> Sessions focus

Sessions focus
  k/Up at top with agents present -> Agents focus
```

The focused panel uses a stronger border. `Enter` attaches to the selected agent's tmux session when agent focus is active, or to the selected session when session focus is active.

## Preview Mode

```text
+--------------------------------------+---------------------------------------+
| Sessions                             | Preview: selected-session             |
| > selected session rows              | latest captured pane lines             |
|   other session rows                 |                                       |
+--------------------------------------+---------------------------------------+
```

`p` toggles preview. When preview is enabled, the session area splits horizontally. The agent panel narrows to agents in the selected session.

## Expanded Session Mode

```text
+------------------------------------------------------------------------------+
| AGENTS IN: selected-session                                                   |
|  codex:selected-session ●                                                     |
+------------------------------------------------------------------------------+
| Windows: selected-session                                                     |
| > 1 editor                                                                    |
|   2 tests                                                                     |
|   3 shell                                                                     |
+------------------------------------------------------------------------------+
| j/k nav | enter attach | h/Esc back | q quit                                  |
+------------------------------------------------------------------------------+
```

`l` or Right expands the selected session. `Enter` attaches to the selected window. `h`, Left, or Esc collapses back to normal mode.

## Filter Mode

```text
+------------------------------------------------------------------------------+
| Sessions (filtered-count) > query                                             |
| > matching session                                                            |
|   another match                                                               |
+------------------------------------------------------------------------------+
| type filter | enter attach | Esc clear | Tab nav                              |
+------------------------------------------------------------------------------+
```

`/` enters filter mode. The session list uses fuzzy matching through `nucleo`.

## Directory Selection

```text
+------------------------------------------------------------------------------+
| Select directory (depth: D) > query                                           |
+------------------------------------------------------------------------------+
| > project-name  [/path/to/project]                                            |
|   other-project [/path/to/other-project]                                      |
+------------------------------------------------------------------------------+
| type filter | Tab complete | +/- depth | enter name | Esc cancel              |
+------------------------------------------------------------------------------+
```

`c` enters directory selection. The default directory list includes the current directory, home directory, and discovered child directories up to the configured scan depth.

## Session Naming

```text
+------------------------------------------------------------------------------+
| Name tmux session                                                             |
+------------------------------------------------------------------------------+
| Directory      /path/to/project                                               |
| Session name   project_name                                                   |
| Sanitized      project_name                                                   |
+------------------------------------------------------------------------------+
| type edit | Backspace delete | enter create | Esc back                        |
+------------------------------------------------------------------------------+
```

After selecting a directory, trex asks for a tmux-safe session name. The final name is sanitized by `src/directory.rs` before creating the session.

## Bar Chart View

```text
+------------------------------------------------------------------------------+
| CPU by session                                                                |
| trex       █████████████ 125                                                  |
| api        ███ 30                                                             |
| scratch    0                                                                  |
+------------------------------------------------------------------------------+
| Memory by session                                                             |
| trex       █████ 512 MB                                                       |
| api        ███ 300 MB                                                         |
+------------------------------------------------------------------------------+
| b/Esc back | q quit                                                           |
+------------------------------------------------------------------------------+
```

`b` opens the chart view. It ranks sessions by current CPU and memory values from `src/sysinfo.rs`.

## Stats Overlay

```text
+------------------------------------------------------------------------------+
| Stats                                                                         |
| Top CPU sessions                                                              |
| Top memory sessions                                                           |
| Health summary: healthy / warning / critical                                  |
| Activity timeline                                                             |
+------------------------------------------------------------------------------+
| s/Esc close | q quit                                                          |
+------------------------------------------------------------------------------+
```

`s` opens the stats overlay. The overlay summarizes resource use, health levels, and activity across all sessions.

## Snapshot JSON

```text
trex snapshot --json
  |
  +-- TmuxClient::check_installed()
  +-- TmuxClient::list_sessions()
  +-- GitStatus::for_path()
  +-- get_session_stats()
  +-- find_ai_processes()
  +-- HealthScore::calculate()
  |
  +-- BackendSnapshot { snapshotVersion: 1, status, summary, sessions, agents, errors }
```

Top-level fields:

- `snapshotVersion`: schema version, currently `1`.
- `generatedAt`: Unix epoch milliseconds.
- `status`: `healthy`, `partial`, or `error`.
- `summary`: aggregate counts for sessions, agents, activity, dirty repositories, high resource usage, and worst health.
- `sessions`: session records with stats, health, git, and session-local agents.
- `agents`: root AI agent process records.
- `errors`: non-fatal collection errors with code, message, and optional context.

Snapshot collection may return `partial` when optional enrichments fail. It still surfaces collection errors instead of hiding them behind fallback data.

## Theme Contract

trex reads Omarchy colors from `~/.config/omarchy/current/theme/colors.toml` when available. The fallback theme remains green-forward and terminal-native. TUI visual changes should use `ThemeColors` rather than hard-coded colors unless the color is an intentional semantic marker already present in the code.

## Layout Invariants

- The top overview is always three rows.
- The agent panel displays up to five rows before showing `+N more`.
- Session rows use activity, attached, health, window count, age, git badge, CPU gauge, memory gauge, and sparklines.
- Preview mode splits only the session area; overview, agents, and help remain full width.
- Bar chart and stats modes are temporary views and must return to normal mode with their toggle key or Esc.
- Interactive tmux actions are performed only after the TUI restores the terminal.
