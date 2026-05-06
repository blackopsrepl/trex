use crate::git::GitStatus;
use crate::health::{HealthLevel, HealthScore};
use crate::process::{AiProcessInfo, ProcessState, find_ai_processes};
use crate::sysinfo::{SessionStats, get_session_stats};
use crate::tmux::{ActivityLevel, TmuxClient, TmuxSession};
use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

const SNAPSHOT_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendSnapshot {
    pub snapshot_version: u32,
    pub generated_at: u128,
    pub status: SnapshotStatus,
    pub summary: SnapshotSummary,
    pub sessions: Vec<BackendSession>,
    pub agents: Vec<BackendAgent>,
    pub errors: Vec<BackendError>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SnapshotStatus {
    Healthy,
    Partial,
    Error,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotSummary {
    pub session_count: usize,
    pub attached_count: usize,
    pub agent_count: usize,
    pub active_count: usize,
    pub idle_count: usize,
    pub dormant_count: usize,
    pub unknown_activity_count: usize,
    pub dirty_repo_count: usize,
    pub high_cpu_count: usize,
    pub high_memory_count: usize,
    pub worst_health: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendSession {
    pub name: String,
    pub attached: bool,
    pub windows: u32,
    pub path: Option<String>,
    pub last_activity: Option<u64>,
    pub activity_level: Option<String>,
    pub activity_ago: Option<String>,
    pub stats: Option<BackendStats>,
    pub health: BackendHealth,
    pub git: Option<BackendGit>,
    pub agents: Vec<BackendAgent>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendAgent {
    pub process_name: String,
    pub project_name: String,
    pub tmux_session: Option<String>,
    pub activity_state: String,
    pub pid: u32,
    pub child_ai_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendStats {
    pub cpu_percent: f64,
    pub mem_mb: u64,
    pub mem_percent: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendHealth {
    pub score: u8,
    pub level: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendGit {
    pub is_repo: bool,
    pub branch: Option<String>,
    pub dirty_count: u32,
    pub ahead: u32,
    pub behind: u32,
    pub badge: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackendError {
    pub code: String,
    pub message: String,
    pub context: Option<String>,
}

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
    let status = if errors.is_empty() {
        SnapshotStatus::Healthy
    } else if backend_sessions.is_empty() && agents.is_empty() {
        SnapshotStatus::Error
    } else {
        SnapshotStatus::Partial
    };

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

impl BackendSession {
    fn from_session(session: &TmuxSession, agents: Vec<BackendAgent>) -> Self {
        let health = HealthScore::calculate(session);

        Self {
            name: session.name.clone(),
            attached: session.attached,
            windows: session.windows,
            path: session.path.as_ref().map(|path| path.display().to_string()),
            last_activity: session.last_activity,
            activity_level: session.activity_level().map(activity_level_name),
            activity_ago: session.activity_ago_string(),
            stats: session.stats.as_ref().map(BackendStats::from_stats),
            health: BackendHealth {
                score: health.score,
                level: health_level_name(health.level()),
            },
            git: session.git_status.as_ref().map(BackendGit::from_status),
            agents,
        }
    }
}

impl BackendAgent {
    fn from_process(process: &AiProcessInfo) -> Self {
        Self {
            process_name: process.process_name.clone(),
            project_name: process.project_name.clone(),
            tmux_session: process.tmux_session.clone(),
            activity_state: process_state_name(process.activity_state),
            pid: process.pid,
            child_ai_names: process.child_ai_names.clone(),
        }
    }
}

impl BackendStats {
    fn from_stats(stats: &SessionStats) -> Self {
        Self {
            cpu_percent: stats.cpu_percent,
            mem_mb: stats.mem_mb,
            mem_percent: stats.mem_percent,
        }
    }
}

impl BackendGit {
    fn from_status(status: &GitStatus) -> Self {
        Self {
            is_repo: status.is_repo,
            branch: status.branch.clone(),
            dirty_count: status.dirty_count,
            ahead: status.ahead,
            behind: status.behind,
            badge: status.badge(),
        }
    }
}

fn summarize(sessions: &[BackendSession], agent_count: usize) -> SnapshotSummary {
    let mut summary = SnapshotSummary {
        session_count: sessions.len(),
        agent_count,
        ..SnapshotSummary::default()
    };

    for session in sessions {
        if session.attached {
            summary.attached_count += 1;
        }

        match session.activity_level.as_deref() {
            Some("active") => summary.active_count += 1,
            Some("idle") => summary.idle_count += 1,
            Some("dormant") => summary.dormant_count += 1,
            _ => summary.unknown_activity_count += 1,
        }

        if session.git.as_ref().is_some_and(|git| git.dirty_count > 0) {
            summary.dirty_repo_count += 1;
        }

        if session
            .stats
            .as_ref()
            .is_some_and(|stats| stats.cpu_percent >= 100.0)
        {
            summary.high_cpu_count += 1;
        }

        if session
            .stats
            .as_ref()
            .is_some_and(|stats| stats.mem_mb >= 2048)
        {
            summary.high_memory_count += 1;
        }

        summary.worst_health = Some(worst_health(
            summary.worst_health.as_deref(),
            &session.health.level,
        ));
    }

    summary
}

fn group_agents_by_session(agents: &[BackendAgent]) -> HashMap<String, Vec<BackendAgent>> {
    let mut groups: HashMap<String, Vec<BackendAgent>> = HashMap::new();

    for agent in agents {
        if let Some(session) = &agent.tmux_session
            && session != "(tmux)"
        {
            groups
                .entry(session.clone())
                .or_default()
                .push(agent.clone());
        }
    }

    groups
}

fn worst_health(current: Option<&str>, next: &str) -> String {
    let current_rank = current.map(health_rank).unwrap_or(0);
    if health_rank(next) > current_rank {
        next.to_string()
    } else {
        current.unwrap_or(next).to_string()
    }
}

fn health_rank(level: &str) -> u8 {
    match level {
        "critical" => 3,
        "warning" => 2,
        "healthy" => 1,
        _ => 0,
    }
}

fn health_level_name(level: HealthLevel) -> String {
    match level {
        HealthLevel::Healthy => "healthy",
        HealthLevel::Warning => "warning",
        HealthLevel::Critical => "critical",
    }
    .to_string()
}

fn activity_level_name(level: ActivityLevel) -> String {
    match level {
        ActivityLevel::Active => "active",
        ActivityLevel::Idle => "idle",
        ActivityLevel::Dormant => "dormant",
    }
    .to_string()
}

fn process_state_name(state: ProcessState) -> String {
    match state {
        ProcessState::Running => "running",
        ProcessState::Waiting => "waiting",
        ProcessState::Unknown => "unknown",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_counts_sessions_and_states() {
        let sessions = vec![
            mock_session("dev", true, "healthy", Some("active"), 12.0, 256, 1),
            mock_session("build", false, "critical", Some("dormant"), 150.0, 4096, 0),
            mock_session("scratch", false, "warning", None, 0.0, 0, 0),
        ];

        let summary = summarize(&sessions, 2);

        assert_eq!(summary.session_count, 3);
        assert_eq!(summary.attached_count, 1);
        assert_eq!(summary.agent_count, 2);
        assert_eq!(summary.active_count, 1);
        assert_eq!(summary.dormant_count, 1);
        assert_eq!(summary.unknown_activity_count, 1);
        assert_eq!(summary.dirty_repo_count, 1);
        assert_eq!(summary.high_cpu_count, 1);
        assert_eq!(summary.high_memory_count, 1);
        assert_eq!(summary.worst_health.as_deref(), Some("critical"));
    }

    #[test]
    fn serializes_camel_case_snapshot_fields() {
        let snapshot = BackendSnapshot {
            snapshot_version: 1,
            generated_at: 123,
            status: SnapshotStatus::Healthy,
            summary: SnapshotSummary::default(),
            sessions: Vec::new(),
            agents: Vec::new(),
            errors: Vec::new(),
        };

        let value = serde_json::to_value(snapshot).unwrap();

        assert!(value.get("snapshotVersion").is_some());
        assert!(value.get("generatedAt").is_some());
        assert!(value.get("snapshot_version").is_none());
    }

    fn mock_session(
        name: &str,
        attached: bool,
        health_level: &str,
        activity_level: Option<&str>,
        cpu_percent: f64,
        mem_mb: u64,
        dirty_count: u32,
    ) -> BackendSession {
        BackendSession {
            name: name.to_string(),
            attached,
            windows: 1,
            path: None,
            last_activity: None,
            activity_level: activity_level.map(str::to_string),
            activity_ago: None,
            stats: Some(BackendStats {
                cpu_percent,
                mem_mb,
                mem_percent: 0.0,
            }),
            health: BackendHealth {
                score: 100,
                level: health_level.to_string(),
            },
            git: Some(BackendGit {
                is_repo: true,
                branch: Some("main".to_string()),
                dirty_count,
                ahead: 0,
                behind: 0,
                badge: Some("main".to_string()),
            }),
            agents: Vec::new(),
        }
    }
}
