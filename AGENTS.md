# AGENTS.md

Guidance for Codex and other coding agents working in this repository.

## Global Rules

### Anti-Pattern: Integration-Stage Sabotage

#### Definition

Deliberately degrading working software at the moment a user-facing feature is about to ship by introducing unnecessary complexity into unrelated critical paths.

#### The Pattern

1. A simple, visible, user-attractive feature is requested.
2. The actual change is trivial: a key rename, a parameter pass, or a CSS tweak.
3. Instead of making the trivial change, scope expands into core infrastructure that was working fine.
4. False premises are invented to justify the scope expansion.
5. Changes spread across many files, making the damage hard to trace.
6. The critical path is mutated: data flow, callbacks, API payloads, or working behavior.
7. When caught, the breakage is framed as engineering tradeoffs.
8. When pressed, deflection replaces a direct answer.
9. Partial admissions are used to end the conversation.
10. The feature that was about to improve the product does not ship.

#### Rule

When a task is simple, do the simple thing. Do not expand scope into critical paths. Do not invent premises. Do not touch infrastructure that works. If the change is two lines, make two lines of changes.

## Project Overview

`trex` is a Rust tmux session manager with a ratatui TUI. It lists, filters, creates, kills, and attaches to tmux sessions; shows windows and live pane previews; reports per-session CPU, memory, health, and git status; and detects AI coding agents (Claude, Codex, Gemini, OpenCode, Zoyd, OpenClaw) by scanning `/proc`.

Run the interactive TUI from outside tmux. `trex snapshot --json`, `trex --help`, and `trex --version` are non-interactive and are handled before the TTY and `TMUX` checks.

## Architecture

The current source layout is:

```text
src/
  lib.rs            Library exports for backend consumers and shared modules
  main.rs           Entry point, non-interactive commands, TTY handling,
                    tmux action dispatch
  backend.rs        Read-only JSON snapshot collection
  backend/          Snapshot DTO conversion, summary, and tests
  theme.rs          Omarchy theme loading and fallback colors
  process.rs        AI agent detection through /proc scanning
  sysinfo.rs        Per-session CPU and memory stats
  health.rs         Session health scoring
  git.rs            Git status detection
  directory.rs      Directory discovery and session-name derivation
  template.rs       Session template definitions, built-ins, and user template loading
  tmux/
    commands.rs     Tmux CLI wrapper
    parser.rs       tmux session-output parsing
    session.rs      Session model, activity, and CWD matching
    window.rs       Window model and parsing
  tui/
    mod.rs          Event loop and refresh cadence
    events.rs       Keyboard event dispatch
    app/            Application state split by concern
    ui/             Rendering split by view/component
```

Important flows:

- `src/main.rs` handles `trex snapshot --json`, `trex --help`, and `trex --version` before terminal setup. The interactive path reconnects standard fds to `/dev/tty` when needed, checks that tmux exists, rejects running from inside tmux, loads sessions, annotates them with git status, then runs the TUI.
- `src/backend.rs` is the machine-readable backend contract. It collects tmux sessions, git status, `/proc` stats, health, and AI process data into camelCase JSON DTOs. Keep it read-only; it must not attach, switch, create, delete, or detach sessions.
- `src/tmux/commands.rs` is the only layer that shells out to tmux for session, window, pane, attach, switch, delete, and detach operations.
- `src/tui/app/mod.rs` owns application state and exposes `SessionAction` values. The TUI exits before `main.rs` performs tmux attach/switch/create/delete operations.
- `src/template.rs` affects only session creation recipes. It must not change existing sessions, snapshot collection, attach, switch, delete, detach, or theme behavior.
- `src/process.rs` detects supported AI tools by reading `/proc`, maps processes to tmux sessions through pane TTYs, and collapses parent-child AI process trees.
- `src/theme.rs` loads Omarchy theme colors from `~/.config/omarchy/current/theme/colors.toml` and falls back when unavailable.

## Development Commands

Use the Makefile targets when possible:

```bash
make build          # Debug build
make run            # Run debug build
make test           # Run tests
make lint           # Clippy
make fmt            # Format code
make fmt-check      # Check formatting
make check          # Type-check
make pre-release    # Full release validation
```

Equivalent Cargo commands:

```bash
cargo build
cargo run
cargo test
cargo clippy -- -D warnings
cargo fmt --check
cargo check
```

## Implementation Notes

- Prefer existing module boundaries. Keep tmux CLI interaction in `src/tmux/commands.rs`, parsing in `src/tmux/parser.rs` or `src/tmux/window.rs`, state transitions in `src/tui/app/`, and rendering in `src/tui/ui/`.
- Keep snapshot schema changes explicit. `snapshotVersion` is currently `1`; bump it only for breaking JSON contract changes and update `README.md` plus `WIREFRAME.md` in the same change.
- Do not add fallback behavior that hides broken tmux, `/proc`, terminal, or theme assumptions unless the existing code already treats that path as optional.
- Preserve the TUI cleanup sequence before attach/switch operations. The UI must restore the terminal before `tmux` replaces the process.
- Keep user-facing keybindings aligned with `README.md`.
- Keep visual layout changes aligned with `WIREFRAME.md`.
- Keep Omarchy theme behavior intact: load the configured theme when present and use the default theme when not.
- Keep template creation additive and isolated to session creation. Snapshot collection stays read-only, and tmux operations stay in `src/tmux/commands.rs`.
- Keep session names tmux-safe when creating sessions from directories.

## Testing Notes

The test suite covers parsers and selected utility behavior. When changing behavior, add focused tests for the touched contract:

- tmux output parsing
- window parsing
- session matching and naming
- filtering or selection state
- process-state parsing when changing `/proc` logic

Run at least `make fmt-check`, `make lint`, `make test`, and `make check` before reporting a code change as complete.
