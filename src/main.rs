pub use trex_cli::{directory, git, health, process, sysinfo, template, theme, tmux};
mod tui;

use crate::git::GitStatus;
use crate::tmux::{TmuxClient, find_matching_session_index};
use crate::tui::app::SessionAction;

use anyhow::{Result, bail};
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StartupCommand {
    Interactive,
    SnapshotJson,
    Help,
    Version,
}

fn parse_startup_command(args: &[String]) -> StartupCommand {
    match args {
        [] => StartupCommand::Interactive,
        [arg] if arg == "-h" || arg == "--help" => StartupCommand::Help,
        [arg] if arg == "-V" || arg == "--version" => StartupCommand::Version,
        [command, flag] if command == "snapshot" && flag == "--json" => {
            StartupCommand::SnapshotJson
        }
        _ => StartupCommand::Interactive,
    }
}

fn print_help() {
    println!(
        concat!(
            "trex {} - tmux session manager\n\n",
            "Usage:\n",
            "  trex\n",
            "  trex snapshot --json\n",
            "  trex --help\n",
            "  trex --version\n\n",
            "Commands:\n",
            "  snapshot --json    Emit a read-only JSON snapshot\n\n",
            "Options:\n",
            "  -h, --help         Show this help\n",
            "  -V, --version      Show version"
        ),
        env!("CARGO_PKG_VERSION")
    );
}

fn print_version() {
    println!("trex {}", env!("CARGO_PKG_VERSION"));
}

/* Ensures stdin, stdout, and stderr are connected to a TTY.
 *
 * When running from keybindings or other non-standard contexts, the standard
 * file descriptors may not be connected to a terminal. This function detects
 * that condition and reconnects them to `/dev/tty`. */
fn ensure_terminal() -> Result<()> {
    let stdin_is_tty = unsafe { libc::isatty(0) };
    let stdout_is_tty = unsafe { libc::isatty(1) };
    let stderr_is_tty = unsafe { libc::isatty(2) };

    if stdin_is_tty == 1 && stdout_is_tty == 1 && stderr_is_tty == 1 {
        return Ok(());
    }

    let tty = OpenOptions::new().read(true).write(true).open("/dev/tty")?;
    let tty_fd = tty.as_raw_fd();

    unsafe {
        libc::dup2(tty_fd, 0);
        libc::dup2(tty_fd, 1);
        libc::dup2(tty_fd, 2);

        if tty_fd > 2 {
            libc::close(tty_fd);
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match parse_startup_command(&args) {
        StartupCommand::SnapshotJson => {
            let snapshot = trex_cli::backend::collect_snapshot()?;
            println!("{}", serde_json::to_string(&snapshot)?);
            return Ok(());
        }
        StartupCommand::Help => {
            print_help();
            return Ok(());
        }
        StartupCommand::Version => {
            print_version();
            return Ok(());
        }
        StartupCommand::Interactive => {}
    }

    ensure_terminal()?;

    TmuxClient::check_installed()?;

    if TmuxClient::is_inside_tmux() {
        bail!(
            "trex cannot be run from inside a tmux session.\nPlease run trex from outside tmux to manage your sessions."
        );
    }

    let mut sessions = TmuxClient::list_sessions()?;

    // Fetch git status for all sessions with paths
    for session in &mut sessions {
        if let Some(ref path) = session.path {
            session.git_status = Some(GitStatus::for_path(path));
        }
    }

    let preselect_index = find_matching_session_index(&sessions);

    match tui::run_tui_with_preselection(sessions, preselect_index)? {
        Some(SessionAction::Attach(name)) => {
            TmuxClient::attach_or_switch(&name)?;
        }

        Some(SessionAction::AttachWindow(session_name, window_index)) => {
            TmuxClient::attach_or_switch_window(&session_name, window_index)?;
        }

        Some(SessionAction::Create {
            name,
            path,
            template,
        }) => {
            let existing_sessions = TmuxClient::list_sessions()?;
            let session_exists = existing_sessions.iter().any(|s| s.name == name);

            if !session_exists {
                TmuxClient::new_session_from_template(&name, &path, &template)?;
            }

            TmuxClient::attach(&name)?;
        }

        Some(SessionAction::Delete(name)) => {
            TmuxClient::delete_session(&name)?;
            println!("Deleted session: {}", name);
        }

        Some(SessionAction::DeleteAll) => {
            TmuxClient::delete_all_sessions()?;
            println!("Deleted all sessions");
        }

        Some(SessionAction::Detach(name)) => {
            TmuxClient::detach_session(&name)?;
            println!("Detached from session: {}", name);
        }

        Some(SessionAction::DetachAll) => {
            TmuxClient::detach_all_sessions()?;
            println!("Detached all clients");
        }

        None => {}
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    #[test]
    fn parses_non_interactive_startup_commands() {
        assert_eq!(
            parse_startup_command(&args(&["snapshot", "--json"])),
            StartupCommand::SnapshotJson
        );
        assert_eq!(
            parse_startup_command(&args(&["--help"])),
            StartupCommand::Help
        );
        assert_eq!(parse_startup_command(&args(&["-h"])), StartupCommand::Help);
        assert_eq!(
            parse_startup_command(&args(&["--version"])),
            StartupCommand::Version
        );
        assert_eq!(
            parse_startup_command(&args(&["-V"])),
            StartupCommand::Version
        );
    }

    #[test]
    fn defaults_to_interactive_for_unknown_args() {
        assert_eq!(parse_startup_command(&[]), StartupCommand::Interactive);
        assert_eq!(
            parse_startup_command(&args(&["snapshot"])),
            StartupCommand::Interactive
        );
        assert_eq!(
            parse_startup_command(&args(&["--unknown"])),
            StartupCommand::Interactive
        );
    }
}
