mod directory;
mod git;
mod tmux;
mod tui;

use crate::git::GitStatus;
use crate::tmux::{TmuxClient, find_matching_session_index};
use crate::tui::app::SessionAction;

use anyhow::{Result, bail};
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;

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

        Some(SessionAction::Create(name, path)) => {
            let existing_sessions = TmuxClient::list_sessions()?;
            let session_exists = existing_sessions.iter().any(|s| s.name == name);

            if !session_exists {
                TmuxClient::new_session(&name, &path, true)?;
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
