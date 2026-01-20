# trex

A fast, minimal tmux session manager with fuzzy finding and an interactive TUI.

## Features

- **Interactive TUI** - Browse and manage tmux sessions with a clean, responsive interface
- **Fuzzy Finding** - Quickly filter sessions and directories with fuzzy search powered by `nucleo`
- **Smart Session Selection** - Automatically preselects sessions matching your current directory
- **Directory Discovery** - Find and create sessions in any directory with configurable filesystem scanning
- **Activity Indicators** - See at a glance which sessions are active, idle, or dormant
- **Git Integration** - View branch name, dirty file count, and ahead/behind status for git repos
- **Window Expansion** - Expand sessions to see and jump to specific windows without attaching first
- **Live Preview** - See the current pane content of any session before attaching
- **Vim-like Keybindings** - Navigate efficiently with familiar `j`/`k` movements
- **Session Management** - Create, attach, delete, and detach sessions with simple keyboard shortcuts
- **Lightweight** - Written in Rust with minimal dependencies and fast startup time

## Installation

### Prerequisites

- **tmux** must be installed and in your PATH

### Quick Install (Linux x86_64)

```bash
curl -fsSL https://github.com/blackopsrepl/trex/releases/latest/download/trex-linux-x86_64.tar.gz | tar -xzf - -C ~/.cargo/bin
```

### Build from Source

Requires the Rust toolchain.

```bash
git clone https://github.com/blackopsrepl/trex.git
cd trex
make install-user    # Install to ~/.cargo/bin
# or
sudo make install    # Install to /usr/local/bin
```

### Static Binary (Universal Linux)

Build a fully static binary that works on any Linux distribution:

```bash
make static          # Build static x86_64 binary
sudo make install-static
```

### All Make Targets

```
make              Build optimized release binary
make static       Build static x86_64 Linux binary (musl)
make static-arm   Build static aarch64 Linux binary (musl)
make install      Install to /usr/local/bin (may need sudo)
make install-user Install to ~/.cargo/bin
make dist         Create release archive
make test         Run tests
make help         Show all targets
```

## Usage

### Basic Usage

Simply run `trex` from outside tmux:

```bash
trex
```

### Zsh Keybinding (Ctrl+T)

Add trex to your Ctrl+T keybinding for quick access. Create a file at `~/.zsh/trex-keybinding.zsh`:

```zsh
# CTRL-T - Launch trex
trex-widget() {
  zle push-input
  BUFFER="trex"
  zle accept-line
}
zle -N trex-widget
bindkey -M emacs '^T' trex-widget
bindkey -M vicmd '^T' trex-widget
bindkey -M viins '^T' trex-widget
```

Then source it in your `~/.zshrc`:

```zsh
[ -f ~/.zsh/trex-keybinding.zsh ] && source ~/.zsh/trex-keybinding.zsh
```

Now pressing `Ctrl+T` will launch trex!

### Keybindings

#### Normal Mode (Session List)

| Key | Action |
|-----|--------|
| `j` / `Down` | Navigate down |
| `k` / `Up` | Navigate up |
| `g` / `Home` | Jump to first session |
| `G` / `End` | Jump to last session |
| `Enter` | Attach to selected session |
| `l` / `Right` | Expand session to show windows |
| `p` | Toggle live preview panel |
| `c` | Enter directory selection mode (create new session) |
| `d` | Delete selected session |
| `D` | Delete all sessions |
| `x` | Detach clients from selected session |
| `X` | Detach all clients from all sessions |
| `/` | Enter filter mode |
| `q` / `Esc` / `Ctrl-t` | Quit |

#### Expanded Session Mode (Window List)

| Key | Action |
|-----|--------|
| `j` / `Down` | Navigate down through windows |
| `k` / `Up` | Navigate up through windows |
| `Enter` | Attach to selected window |
| `h` / `Left` / `Esc` | Collapse and return to session list |
| `q` | Quit |

#### Filter Mode

| Key | Action |
|-----|--------|
| Type | Filter sessions by name/path |
| `Backspace` | Delete filter character |
| `Esc` | Exit filter mode |

#### Directory Selection Mode

| Key | Action |
|-----|--------|
| `j` / `Down` | Navigate down |
| `k` / `Up` | Navigate up |
| `g` / `Home` | Jump to first directory |
| `G` / `End` | Jump to last directory |
| `Enter` | Create session in selected directory |
| `+` | Increase directory scan depth (max 6) |
| `-` | Decrease directory scan depth (min 1) |
| `Tab` | Autocomplete filter with selected directory path |
| Type | Filter directories (fuzzy matching) |
| `Backspace` | Delete filter character |
| `Esc` | Cancel and return to session list |


## How It Works

### Session Display

Each session in the list shows rich information at a glance:

```
●* api-server (3 win) 2m  main +3 ↑2  ~/projects/api
○  frontend (2 win) 15m  feature/auth  ~/work/frontend
◌  old-project (1 win) 2h  ~/archive/old
```

- **Activity indicator**: `●` green (active < 5min), `○` yellow (idle 5-30min), `◌` gray (dormant > 30min)
- **Attached indicator**: `*` shown when a client is attached to the session
- **Window count**: Number of windows in the session
- **Time since activity**: How long since the last activity (e.g., "2m", "1h", "3d")
- **Git status**: Branch name, dirty count (+N), ahead/behind remote (↑N↓N)
- **Path**: Working directory of the session

### Session Preselection

When you launch trex, it automatically tries to select a session that matches your current working directory:

1. First, it looks for a session with an exact path match
2. If not found, it looks for a session whose name matches your current directory name
3. If still not found, it selects the first session

This makes it quick to jump back into the session you're likely working on.

### Window Expansion

Press `l` or `→` on any session to expand it and see all its windows:

```
▼ api-server (3 win)
    ● 0: vim [nvim]
      1: shell [zsh]
      2: logs [tail]
```

Navigate with `j`/`k` and press `Enter` to attach directly to a specific window. Press `h`, `←`, or `Esc` to collapse.

### Live Preview

Press `p` to toggle a preview panel showing the current pane content of the selected session:

```
┌─ Sessions ─────────────────┬─ Preview: api-server ─────┐
│ ●* api-server              │ $ npm run dev             │
│ ○  frontend (2 win)        │ > server listening :3000  │
│ ◌  database                │ > connected to postgres   │
└────────────────────────────┴───────────────────────────┘
```

The preview updates as you navigate between sessions, letting you peek at what's running before attaching.

### Directory Discovery

When creating a new session (press `c`), trex scans the filesystem to find directories:

- **Prioritizes:** Current working directory, home directory, and common project directories (`~/projects`, `~/work`, `~/dev`, `~/code`, `~/src`)
- **Configurable depth:** Scan from 1-6 levels deep (default: 3), adjustable with `+`/`-` keys
- **Skips symlinks:** Avoids infinite loops
- **Deduplicates:** Removes duplicate entries automatically
- **Fuzzy matching:** Quickly filter thousands of directories

### Session Naming

Session names are automatically derived from directory names and sanitized for tmux compatibility:

- Alphanumeric characters, hyphens, and underscores are preserved
- All other characters are replaced with underscores
- Example: `/home/user/my-project` → session name: `my-project`

## Development

```bash
make test    # Run tests
make lint    # Run clippy
make fmt     # Format code
make check   # Check without building
make clean   # Remove build artifacts
```

## Architecture

trex is organized into several modules:

- **`src/directory.rs`** - Directory discovery and fuzzy matching
- **`src/git.rs`** - Git repository status detection (branch, dirty files, ahead/behind)
- **`src/tmux/`** - Tmux integration
  - `commands.rs` - Tmux CLI wrapper (sessions, windows, pane capture)
  - `session.rs` - Session struct with activity tracking
  - `window.rs` - Window struct for expanded view
  - `parser.rs` - Output parsing
- **`src/tui/`** - Terminal UI
  - `app.rs` - Application state and business logic
  - `events.rs` - Keyboard event handling
  - `ui.rs` - Rendering (sessions, windows, preview panel)
- **`src/main.rs`** - Entry point and action handling

## Dependencies

- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal handling
- [nucleo](https://github.com/helix-editor/nucleo) - Fuzzy matching
- [anyhow](https://github.com/dtolnay/anyhow) - Error handling

## License
ISC License - see [LICENSE](LICENSE) for details.

## Why trex?

- **Fast:** Rust performance with minimal startup time
- **Simple:** Single binary, no configuration files needed
- **Interactive:** Visual interface beats remembering session names
- **Smart:** Automatic session preselection based on your current directory
- **Informative:** Activity status, git info, and live preview at a glance
- **Minimal:** Does one thing well - session management
