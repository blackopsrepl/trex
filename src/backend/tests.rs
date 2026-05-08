use super::summary::summarize;
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
