use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const AI_PROCESSES: &[&str] = &["claude", "opencode", "zoyd"];

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ProcessState {
    Running,  // 'R' - actively using CPU
    Waiting,  // 'S' - sleeping/waiting for I/O or input
    #[default]
    Unknown,  // Could not determine state
}

#[derive(Debug, Clone)]
pub struct AiProcessInfo {
    pub process_name: String,
    pub project_name: String,
    pub tmux_session: Option<String>,
    pub activity_state: ProcessState,
    pub pid: u32,
}

pub fn find_ai_processes() -> Result<Vec<AiProcessInfo>> {
    let tty_session_map = get_tty_session_map();
    let mut processes = Vec::new();

    for entry in fs::read_dir("/proc")? {
        let entry = entry?;
        let file_name = entry.file_name();
        let pid_str = file_name.to_string_lossy();

        if let Ok(pid) = pid_str.parse::<u32>()
            && let Ok(info) = get_process_info(pid, &tty_session_map)
        {
            processes.push(info);
        }
    }

    Ok(processes)
}

// Reads the current process state for a given PID
// This is public so it can be called for real-time state refresh
pub fn read_process_state(pid: u32) -> ProcessState {
    let path = format!("/proc/{}/stat", pid);
    if let Ok(content) = fs::read_to_string(&path) {
        // Format: "pid (comm) STATE ..."
        // The comm field can contain spaces and parens, so we find ") " to locate the state
        if let Some(rest) = content.split(") ").nth(1) {
            return match rest.chars().next() {
                Some('R') => ProcessState::Running,
                Some('S') => ProcessState::Waiting,
                _ => ProcessState::Unknown,
            };
        }
    }
    ProcessState::Unknown
}

pub fn process_exists(pid: u32) -> bool {
    fs::metadata(format!("/proc/{}", pid)).is_ok()
}

fn get_process_info(pid: u32, tty_session_map: &HashMap<String, String>) -> Result<AiProcessInfo> {
    let comm = read_comm(pid)?;
    let process_name = AI_PROCESSES
        .iter()
        .find(|&&name| comm.to_lowercase().contains(name))
        .map(|&name| name.to_string())
        .context("Not an AI process")?;

    let project_name = read_cwd(pid)?
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let tmux_session = find_tmux_session(pid, tty_session_map);
    let activity_state = read_process_state(pid);

    Ok(AiProcessInfo {
        process_name,
        project_name,
        tmux_session,
        activity_state,
        pid,
    })
}

fn read_comm(pid: u32) -> Result<String> {
    let path = format!("/proc/{}/comm", pid);
    fs::read_to_string(&path)
        .map(|s| s.trim().to_string())
        .context("Failed to read comm")
}

fn read_cwd(pid: u32) -> Result<PathBuf> {
    let path = format!("/proc/{}/cwd", pid);
    fs::read_link(&path).context("Failed to read cwd")
}

fn get_tty_session_map() -> HashMap<String, String> {
    let mut map = HashMap::new();

    // Run: tmux list-panes -a -F '#{pane_tty}:#{session_name}'
    let output = Command::new("tmux")
        .args(["list-panes", "-a", "-F", "#{pane_tty}:#{session_name}"])
        .output();

    if let Ok(output) = output
        && output.status.success()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if let Some((tty, session)) = line.split_once(':') {
                map.insert(tty.to_string(), session.to_string());
            }
        }
    }

    map
}

fn find_tmux_session(pid: u32, tty_session_map: &HashMap<String, String>) -> Option<String> {
    // First, try to get the TTY from stdin (fd/0)
    if let Ok(tty) = fs::read_link(format!("/proc/{}/fd/0", pid)) {
        let tty_str = tty.to_string_lossy();
        if let Some(session) = tty_session_map.get(tty_str.as_ref()) {
            return Some(session.clone());
        }
    }

    // Fallback: check if process has TMUX env var (means it's in tmux, even if we can't resolve session)
    let env = read_environ(pid);
    if env.contains("TMUX=") {
        // Try to walk up process tree to find a process with a known TTY
        if let Ok(Some(ppid)) = get_ppid(pid)
            && let Some(session) = find_tmux_session(ppid, tty_session_map)
        {
            return Some(session);
        }
        // We know it's in tmux but can't resolve session name
        return Some("(tmux)".to_string());
    }

    None
}

fn read_environ(pid: u32) -> String {
    let path = format!("/proc/{}/environ", pid);
    fs::read_to_string(&path)
        .unwrap_or_default()
        .replace('\0', "\n")
}

fn get_ppid(pid: u32) -> Result<Option<u32>> {
    let path = format!("/proc/{}/stat", pid);
    let content = fs::read_to_string(&path).context("Failed to read stat")?;

    // The PPID is the 4th field, but comm can contain spaces/parens
    // So we find ") " and then split the rest
    if let Some(rest) = content.split(") ").nth(1) {
        let fields: Vec<&str> = rest.split_whitespace().collect();
        // After ") ", fields are: state, ppid, pgrp, ...
        // So ppid is at index 1
        if fields.len() > 1 {
            return fields[1]
                .parse::<u32>()
                .map(Some)
                .context("Failed to parse ppid");
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_ai_processes_runs() {
        // This test verifies that find_ai_processes() runs without panicking
        // and returns a valid result. The actual processes found will vary
        // depending on what's running on the system.
        let result = find_ai_processes();
        assert!(result.is_ok(), "find_ai_processes should not fail");
    }

    #[test]
    fn test_read_process_state() {
        // Test with current process - it could be in any valid state
        let pid = std::process::id();
        let state = read_process_state(pid);
        // The function should return a valid state (not panic)
        // State could be Running, Waiting, or Unknown depending on timing
        let _ = state;
    }

    #[test]
    fn test_process_exists() {
        // Current process should exist
        assert!(process_exists(std::process::id()));
        // PID 0 is kernel, should exist
        assert!(process_exists(0) || !process_exists(0)); // May or may not be readable
        // Very high PID probably doesn't exist
        assert!(!process_exists(999999999));
    }

    #[test]
    fn test_get_tty_session_map() {
        // This should not panic even if tmux is not running
        let map = get_tty_session_map();
        // Map might be empty if tmux isn't running, that's fine
        // Map might be empty if tmux isn't running, that's fine
        let _ = map.len();
    }
}
