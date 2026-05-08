# Changelog

All notable changes to this project are documented in this file. The format follows Conventional Commits.

## 0.5.0 (2026-05-08)

### Features

- Expose `trex snapshot --json` as a read-only backend for desktop and status-bar integrations.
- Export library modules through `src/lib.rs` so the snapshot backend can share the same tmux, git, health, process, sysinfo, and theme contracts as the TUI.

### Bug Fixes

- Detect Codex processes in the AI agent scanner.

### Documentation

- Add current agent guidance in `AGENTS.md`.
- Add mascot artwork to the README.
- Add the current JSON snapshot contract to the README.
- Add `WIREFRAME.md` as the shipped visual and integration contract.
- Remove the completed historical `PRD.md` so there is no stale active-plan document in the project root.
- Remove stale Claude project config and old 0.3.0 launch-copy collateral.
- Update remaining launch copy for Codex detection, JSON snapshots, and the `trex-cli` crate name.

### Maintenance

- Optimize the mascot PNG for repository and release hygiene.
- Simplify TUI ordering handlers.

## 0.4.1 (2026-02-10)

### Features

- Add git repository status integration.
- Add session health scoring.
- Add per-session CPU and memory stats through `/proc` scanning.
- Add Omarchy theme integration with automatic color scheme loading.
- Add tmux session activity tracking.
- Add optional ASCII T-Rex background decoration through the `ascii-art` feature.
- Add bar chart and stats overlay modes.
- Add directory selection and safe session-name derivation.
- Add window expansion and live pane preview.
- Add grouped AI process display for parent-child agent relationships.
- Redesign normal mode with system overview, gauges, sparklines, and scrollbar.

### Bug Fixes

- Correct release workflow toolchain setup.
- Skip publish and release jobs on Forgejo runners.
- Use `file_type()` instead of `metadata()` for symlink detection.
- Remove `tmux list-panes -a` from per-session PID lookups.
- Sanitize tmux session names derived from directory names.
- Require `/` to enter filter mode.
- Use theme backgrounds for gauges and sparklines.
