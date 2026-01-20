use crate::tmux::parser::parse_sessions;
use crate::tmux::session::TmuxSession;
use crate::tmux::window::{TmuxWindow, parse_windows};
use anyhow::{Result, bail};
use std::os::unix::process::CommandExt;
use std::process::Command;

// Client for interacting with tmux via CLI commands.
pub struct TmuxClient;

impl TmuxClient {
    // Verifies that tmux is installed and available in PATH.
    pub fn check_installed() -> Result<()> {
        which::which("tmux")
            .map_err(|_| anyhow::anyhow!("tmux is not installed or not in PATH"))?;
        Ok(())
    }

    // Returns true if currently running inside a tmux session.
    pub fn is_inside_tmux() -> bool {
        std::env::var("TMUX").is_ok()
    }

    // Lists all tmux sessions with their metadata.
    pub fn list_sessions() -> Result<Vec<TmuxSession>> {
        let output = Command::new("tmux")
            .args([
                "list-sessions",
                "-F",
                "#{session_name}|#{session_attached}|#{session_windows}|#{session_path}|#{session_activity}",
            ])
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(parse_sessions(&stdout))
    }

    // Attaches to a session, replacing the current process via exec.
    pub fn attach(session_name: &str) -> Result<()> {
        let err = Command::new("tmux")
            .args(["attach-session", "-t", session_name])
            .exec();

        bail!("Failed to attach to session: {}: {}", session_name, err);
    }

    // Switches the current tmux client to a different session.
    pub fn switch_client(session_name: &str) -> Result<()> {
        let status = Command::new("tmux")
            .args(["switch-client", "-t", session_name])
            .status()?;

        if !status.success() {
            bail!("Failed to switch to session: {}", session_name);
        }
        Ok(())
    }

    // Attaches or switches to a session depending on whether we're inside tmux.
    pub fn attach_or_switch(session_name: &str) -> Result<()> {
        if Self::is_inside_tmux() {
            Self::switch_client(session_name)
        } else {
            Self::attach(session_name)
        }
    }

    // Creates a new tmux session with the given name and working directory.
    pub fn new_session(name: &str, working_dir: &std::path::Path, detached: bool) -> Result<()> {
        let dir_str = working_dir.to_string_lossy();
        let mut args = vec!["new-session", "-s", name, "-c", &dir_str];

        if detached {
            args.push("-d");
        }

        let status = Command::new("tmux").args(&args).status()?;

        if !status.success() {
            bail!("Failed to create session: {}", name);
        }
        Ok(())
    }

    // Deletes a session by name.
    pub fn delete_session(session_name: &str) -> Result<()> {
        let status = Command::new("tmux")
            .args(["kill-session", "-t", session_name])
            .status()?;

        if !status.success() {
            bail!("Failed to delete session: {}", session_name);
        }
        Ok(())
    }

    // Deletes all sessions by killing the tmux server.
    pub fn delete_all_sessions() -> Result<()> {
        let status = Command::new("tmux").args(["kill-server"]).status()?;

        if !status.success() {
            bail!("Failed to delete all sessions");
        }
        Ok(())
    }

    // Detaches all clients from a specific session.
    pub fn detach_session(session_name: &str) -> Result<()> {
        let status = Command::new("tmux")
            .args(["detach-client", "-s", session_name])
            .status()?;

        if !status.success() {
            bail!("Failed to detach from session: {}", session_name);
        }
        Ok(())
    }

    // Detaches all clients from all sessions.
    pub fn detach_all_sessions() -> Result<()> {
        let status = Command::new("tmux")
            .args(["detach-client", "-a"])
            .status()?;

        if !status.success() {
            bail!("Failed to detach all clients");
        }
        Ok(())
    }

    // Lists all windows in a session.
    pub fn list_windows(session_name: &str) -> Result<Vec<TmuxWindow>> {
        let output = Command::new("tmux")
            .args([
                "list-windows",
                "-t",
                session_name,
                "-F",
                "#{window_index}|#{window_name}|#{window_active}|#{pane_current_command}",
            ])
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(parse_windows(&stdout))
    }

    // Attaches to a specific window in a session, replacing the current process via exec.
    pub fn attach_window(session_name: &str, window_index: u32) -> Result<()> {
        let target = format!("{}:{}", session_name, window_index);
        let err = Command::new("tmux")
            .args(["attach-session", "-t", &target])
            .exec();

        bail!("Failed to attach to window: {}: {}", target, err);
    }

    // Switches to a specific window in a session.
    pub fn switch_to_window(session_name: &str, window_index: u32) -> Result<()> {
        let target = format!("{}:{}", session_name, window_index);
        let status = Command::new("tmux")
            .args(["switch-client", "-t", &target])
            .status()?;

        if !status.success() {
            bail!("Failed to switch to window: {}", target);
        }
        Ok(())
    }

    // Attaches or switches to a specific window depending on whether we're inside tmux.
    pub fn attach_or_switch_window(session_name: &str, window_index: u32) -> Result<()> {
        if Self::is_inside_tmux() {
            Self::switch_to_window(session_name, window_index)
        } else {
            Self::attach_window(session_name, window_index)
        }
    }

    // Captures the content of the current pane in a session.
    pub fn capture_pane(session_name: &str, lines: usize) -> Result<Vec<String>> {
        let start_line = format!("-{}", lines);
        let output = Command::new("tmux")
            .args([
                "capture-pane",
                "-t",
                &format!("{}:", session_name),
                "-p",
                "-S",
                &start_line,
            ])
            .output()?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|l| l.to_string()).collect())
    }
}
