mod convert;
mod dto;
mod summary;

#[cfg(test)]
mod tests;

pub use dto::{
    BackendAgent, BackendError, BackendGit, BackendHealth, BackendSession, BackendSnapshot,
    BackendStats, SnapshotStatus, SnapshotSummary,
};

use crate::git::GitStatus;
use crate::process::find_ai_processes;
use crate::sysinfo::get_session_stats;
use crate::tmux::TmuxClient;
use anyhow::Result;
use std::time::{SystemTime, UNIX_EPOCH};

use convert::group_agents_by_session;
use summary::{snapshot_status, summarize};

const SNAPSHOT_VERSION: u32 = 1;

pub fn collect_snapshot() -> Result<BackendSnapshot> {
    TmuxClient::check_installed()?;

    let generated_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let mut errors = Vec::new();
    let mut sessions = TmuxClient::list_sessions()?;

    let ai_processes = match find_ai_processes() {
        Ok(processes) => processes,
        Err(error) => {
            errors.push(BackendError {
                code: "ai-process-scan-failed".to_string(),
                message: error.to_string(),
                context: None,
            });
            Vec::new()
        }
    };

    let agents = ai_processes
        .iter()
        .map(BackendAgent::from_process)
        .collect::<Vec<_>>();
    let agents_by_session = group_agents_by_session(&agents);

    let mut backend_sessions = Vec::with_capacity(sessions.len());
    for session in &mut sessions {
        if let Some(path) = &session.path {
            session.git_status = Some(GitStatus::for_path(path));
        }

        match get_session_stats(&session.name) {
            Ok(stats) => session.stats = Some(stats),
            Err(error) => errors.push(BackendError {
                code: "session-stats-failed".to_string(),
                message: error.to_string(),
                context: Some(session.name.clone()),
            }),
        }

        backend_sessions.push(BackendSession::from_session(
            session,
            agents_by_session
                .get(&session.name)
                .cloned()
                .unwrap_or_default(),
        ));
    }

    let summary = summarize(&backend_sessions, agents.len());
    let status = snapshot_status(
        errors.is_empty(),
        backend_sessions.is_empty(),
        agents.is_empty(),
    );

    Ok(BackendSnapshot {
        snapshot_version: SNAPSHOT_VERSION,
        generated_at,
        status,
        summary,
        sessions: backend_sessions,
        agents,
        errors,
    })
}
