# Changelog

All notable changes to this project will be documented in this file. See [commit-and-tag-version](https://github.com/absolute-version/commit-and-tag-version) for commit guidelines.

## 0.4.1 (2026-02-10)


### Features

* add git repository status integration 0789183
* **health:** add session health scoring algorithm 0dc7175
* initialize project 20e933b
* **process:** add child_ai_names field to AiProcessInfo struct 63b9d82
* **process:** add openclaw to AI process detection e50033a
* **process:** detect parent-child relationships in find_ai_processes e194606
* **process:** filter child AI processes from results, return only root processes 5482b4c
* **process:** initialize child_ai_names field in get_process_info 20cf894
* **sysinfo:** add per-session CPU/memory stats via /proc scanning 6e62c75
* **theme:** add gradient_color() that interpolates success/warning/error 8312fe3
* **theme:** add Omarchy theme integration with automatic color scheme loading 39e2326
* **theme:** derive Omarchy background colors from theme instead of hardcoding 136eaa5
* **theme:** use eza-compatible ANSI named colors for default theme 82745ca
* **tmux:** add session activity tracking 20468a5
* **tui:** add ASCII T-Rex background decoration 219c61d
* **tui:** add bar chart view and stats overlay modes dd36e54
* **tui:** add directory name sanitization and TUI cues 9820a0b
* **tui:** add red eye to T-Rex ascii art 2c3336a
* **tui:** add window expansion and live preview 99d21c6
* **tui:** append child AI names after tmux icon in normal mode 8237303
* **tui:** apply theme colors to expanded, directory, and naming views b899f25
* **tui:** increase COL_WIDTH from 30 to 38 characters 633ff88
* **tui:** make T-Rex ASCII art optional at compile time via feature flag ba1a5ef
* **tui:** redesign normal mode with system overview, gauges, sparklines, and scrollbar 6b38ce5
* **tui:** use theme-aware green-to-amber gradient for T-Rex background art 4be776f
* **tui:** wire theme, stats, and health into app state and event loop e6e4ac7


### Bug Fixes

* **ci:** correct rust-toolchain action name in release workflow 449e1f0
* **ci:** replace dtolnay/rust-toolchain with rustup for Forgejo compatibility aea27d0
* **ci:** skip publish and release jobs on Forgejo runners 79931e7
* **directory:** use file_type() instead of metadata() for symlink detection f643bc7
* replace useless vec! with a static slice in expanded help fea97ee
* **tmux:** remove -a flag from list-panes to isolate PIDs per session aa341d7
* **tmux:** sanitize session names to handle dots in directory names 0432935
* **tui:** make ui helper functions public for internal module access eb68397
* **tui:** require '/' to enter filter mode a20de69
* **tui:** set gauge background to theme bg_primary to eliminate grey unfilled area 00288cc
* **tui:** set sparkline background to theme bg_primary to eliminate grey area 289b4f0

## 0.4.1 (2026-02-10)


### Features

* add git repository status integration 0789183
* **health:** add session health scoring algorithm 0dc7175
* initialize project 20e933b
* **process:** add child_ai_names field to AiProcessInfo struct 63b9d82
* **process:** add openclaw to AI process detection e50033a
* **process:** detect parent-child relationships in find_ai_processes e194606
* **process:** filter child AI processes from results, return only root processes 5482b4c
* **process:** initialize child_ai_names field in get_process_info 20cf894
* **sysinfo:** add per-session CPU/memory stats via /proc scanning 6e62c75
* **theme:** add gradient_color() that interpolates success/warning/error 8312fe3
* **theme:** add Omarchy theme integration with automatic color scheme loading 39e2326
* **theme:** derive Omarchy background colors from theme instead of hardcoding 136eaa5
* **theme:** use eza-compatible ANSI named colors for default theme 82745ca
* **tmux:** add session activity tracking 20468a5
* **tui:** add ASCII T-Rex background decoration 219c61d
* **tui:** add bar chart view and stats overlay modes dd36e54
* **tui:** add directory name sanitization and TUI cues 9820a0b
* **tui:** add red eye to T-Rex ascii art 2c3336a
* **tui:** add window expansion and live preview 99d21c6
* **tui:** append child AI names after tmux icon in normal mode 8237303
* **tui:** apply theme colors to expanded, directory, and naming views b899f25
* **tui:** increase COL_WIDTH from 30 to 38 characters 633ff88
* **tui:** make T-Rex ASCII art optional at compile time via feature flag ba1a5ef
* **tui:** redesign normal mode with system overview, gauges, sparklines, and scrollbar 6b38ce5
* **tui:** use theme-aware green-to-amber gradient for T-Rex background art 4be776f
* **tui:** wire theme, stats, and health into app state and event loop e6e4ac7


### Bug Fixes

* **ci:** correct rust-toolchain action name in release workflow 449e1f0
* **ci:** replace dtolnay/rust-toolchain with rustup for Forgejo compatibility aea27d0
* **ci:** skip publish and release jobs on Forgejo runners 79931e7
* **directory:** use file_type() instead of metadata() for symlink detection f643bc7
* replace useless vec! with a static slice in expanded help fea97ee
* **tmux:** remove -a flag from list-panes to isolate PIDs per session aa341d7
* **tmux:** sanitize session names to handle dots in directory names 0432935
* **tui:** make ui helper functions public for internal module access eb68397
* **tui:** require '/' to enter filter mode a20de69
* **tui:** set gauge background to theme bg_primary to eliminate grey unfilled area 00288cc
* **tui:** set sparkline background to theme bg_primary to eliminate grey area 289b4f0

## 0.4.1 (2026-02-10)


### Features

* add git repository status integration 0789183
* **health:** add session health scoring algorithm 0dc7175
* initialize project 20e933b
* **process:** add child_ai_names field to AiProcessInfo struct 63b9d82
* **process:** add openclaw to AI process detection e50033a
* **process:** detect parent-child relationships in find_ai_processes e194606
* **process:** filter child AI processes from results, return only root processes 5482b4c
* **process:** initialize child_ai_names field in get_process_info 20cf894
* **sysinfo:** add per-session CPU/memory stats via /proc scanning 6e62c75
* **theme:** add gradient_color() that interpolates success/warning/error 8312fe3
* **theme:** add Omarchy theme integration with automatic color scheme loading 39e2326
* **theme:** derive Omarchy background colors from theme instead of hardcoding 136eaa5
* **theme:** use eza-compatible ANSI named colors for default theme 82745ca
* **tmux:** add session activity tracking 20468a5
* **tui:** add ASCII T-Rex background decoration 219c61d
* **tui:** add bar chart view and stats overlay modes dd36e54
* **tui:** add directory name sanitization and TUI cues 9820a0b
* **tui:** add red eye to T-Rex ascii art 2c3336a
* **tui:** add window expansion and live preview 99d21c6
* **tui:** append child AI names after tmux icon in normal mode 8237303
* **tui:** apply theme colors to expanded, directory, and naming views b899f25
* **tui:** increase COL_WIDTH from 30 to 38 characters 633ff88
* **tui:** make T-Rex ASCII art optional at compile time via feature flag ba1a5ef
* **tui:** redesign normal mode with system overview, gauges, sparklines, and scrollbar 6b38ce5
* **tui:** use theme-aware green-to-amber gradient for T-Rex background art 4be776f
* **tui:** wire theme, stats, and health into app state and event loop e6e4ac7


### Bug Fixes

* **ci:** correct rust-toolchain action name in release workflow 449e1f0
* **ci:** replace dtolnay/rust-toolchain with rustup for Forgejo compatibility aea27d0
* **ci:** skip publish and release jobs on Forgejo runners 79931e7
* **directory:** use file_type() instead of metadata() for symlink detection f643bc7
* replace useless vec! with a static slice in expanded help fea97ee
* **tmux:** remove -a flag from list-panes to isolate PIDs per session aa341d7
* **tmux:** sanitize session names to handle dots in directory names 0432935
* **tui:** make ui helper functions public for internal module access eb68397
* **tui:** require '/' to enter filter mode a20de69
* **tui:** set gauge background to theme bg_primary to eliminate grey unfilled area 00288cc
* **tui:** set sparkline background to theme bg_primary to eliminate grey area 289b4f0

## 0.4.1 (2026-02-10)


### Features

* add git repository status integration 0789183
* **health:** add session health scoring algorithm 0dc7175
* initialize project 20e933b
* **process:** add child_ai_names field to AiProcessInfo struct 63b9d82
* **process:** add openclaw to AI process detection e50033a
* **process:** detect parent-child relationships in find_ai_processes e194606
* **process:** filter child AI processes from results, return only root processes 5482b4c
* **process:** initialize child_ai_names field in get_process_info 20cf894
* **sysinfo:** add per-session CPU/memory stats via /proc scanning 6e62c75
* **theme:** add gradient_color() that interpolates success/warning/error 8312fe3
* **theme:** add Omarchy theme integration with automatic color scheme loading 39e2326
* **theme:** derive Omarchy background colors from theme instead of hardcoding 136eaa5
* **theme:** use eza-compatible ANSI named colors for default theme 82745ca
* **tmux:** add session activity tracking 20468a5
* **tui:** add ASCII T-Rex background decoration 219c61d
* **tui:** add bar chart view and stats overlay modes dd36e54
* **tui:** add directory name sanitization and TUI cues 9820a0b
* **tui:** add red eye to T-Rex ascii art 2c3336a
* **tui:** add window expansion and live preview 99d21c6
* **tui:** append child AI names after tmux icon in normal mode 8237303
* **tui:** apply theme colors to expanded, directory, and naming views b899f25
* **tui:** increase COL_WIDTH from 30 to 38 characters 633ff88
* **tui:** make T-Rex ASCII art optional at compile time via feature flag ba1a5ef
* **tui:** redesign normal mode with system overview, gauges, sparklines, and scrollbar 6b38ce5
* **tui:** use theme-aware green-to-amber gradient for T-Rex background art 4be776f
* **tui:** wire theme, stats, and health into app state and event loop e6e4ac7


### Bug Fixes

* **ci:** correct rust-toolchain action name in release workflow 449e1f0
* **ci:** replace dtolnay/rust-toolchain with rustup for Forgejo compatibility aea27d0
* **ci:** skip publish and release jobs on Forgejo runners 79931e7
* **directory:** use file_type() instead of metadata() for symlink detection f643bc7
* replace useless vec! with a static slice in expanded help fea97ee
* **tmux:** remove -a flag from list-panes to isolate PIDs per session aa341d7
* **tmux:** sanitize session names to handle dots in directory names 0432935
* **tui:** make ui helper functions public for internal module access eb68397
* **tui:** require '/' to enter filter mode a20de69
* **tui:** set gauge background to theme bg_primary to eliminate grey unfilled area 00288cc
* **tui:** set sparkline background to theme bg_primary to eliminate grey area 289b4f0
