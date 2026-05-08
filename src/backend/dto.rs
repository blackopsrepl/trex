use serde::Serialize;

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
