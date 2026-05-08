use super::dto::{BackendAgent, BackendGit, BackendHealth, BackendSession, BackendStats};
use crate::git::GitStatus;
use crate::health::{HealthLevel, HealthScore};
use crate::process::{AiProcessInfo, ProcessState};
use crate::sysinfo::SessionStats;
use crate::tmux::{ActivityLevel, TmuxSession};
use std::collections::HashMap;

impl BackendSession {
    pub(super) fn from_session(session: &TmuxSession, agents: Vec<BackendAgent>) -> Self {
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
    pub(super) fn from_process(process: &AiProcessInfo) -> Self {
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

pub(super) fn group_agents_by_session(
    agents: &[BackendAgent],
) -> HashMap<String, Vec<BackendAgent>> {
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
