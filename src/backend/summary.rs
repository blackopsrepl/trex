use super::dto::{BackendSession, SnapshotStatus, SnapshotSummary};

pub(super) fn summarize(sessions: &[BackendSession], agent_count: usize) -> SnapshotSummary {
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

pub(super) fn snapshot_status(
    no_errors: bool,
    no_sessions: bool,
    no_agents: bool,
) -> SnapshotStatus {
    if no_errors {
        SnapshotStatus::Healthy
    } else if no_sessions && no_agents {
        SnapshotStatus::Error
    } else {
        SnapshotStatus::Partial
    }
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
