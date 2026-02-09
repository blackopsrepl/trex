use crate::tmux::{TmuxSession, ActivityLevel};

/// Health score from 0-100 based on multiple factors
#[derive(Debug, Clone, Copy)]
pub struct HealthScore {
    pub score: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HealthLevel {
    Healthy,   // 70-100
    Warning,   // 40-69
    Critical,  // 0-39
}

impl HealthScore {
    /// Calculate health score for a session
    pub fn calculate(session: &TmuxSession) -> Self {
        // CPU penalty (0-40 points)
        let cpu_penalty: u8 = if let Some(ref stats) = session.stats {
            if stats.cpu_percent > 200.0 {
                40
            } else if stats.cpu_percent > 150.0 {
                30
            } else if stats.cpu_percent > 100.0 {
                20
            } else if stats.cpu_percent > 50.0 {
                10
            } else {
                0
            }
        } else {
            0
        };

        // Memory penalty (0-40 points)
        let mem_penalty: u8 = if let Some(ref stats) = session.stats {
            if stats.mem_mb > 8192 {
                40
            } else if stats.mem_mb > 4096 {
                30
            } else if stats.mem_mb > 2048 {
                20
            } else if stats.mem_mb > 1024 {
                10
            } else {
                0
            }
        } else {
            0
        };

        // Activity penalty (0-20 points)
        let activity_penalty: u8 = match session.activity_level() {
            Some(ActivityLevel::Dormant) => 20,
            Some(ActivityLevel::Idle) => 10,
            Some(ActivityLevel::Active) => 0,
            None => 5,
        };

        let total_penalty = cpu_penalty.saturating_add(mem_penalty).saturating_add(activity_penalty);
        let score = 100u8.saturating_sub(total_penalty);

        Self { score }
    }

    pub fn level(&self) -> HealthLevel {
        if self.score >= 70 {
            HealthLevel::Healthy
        } else if self.score >= 40 {
            HealthLevel::Warning
        } else {
            HealthLevel::Critical
        }
    }

    pub fn icon(&self) -> &'static str {
        match self.level() {
            HealthLevel::Healthy => "🟢",
            HealthLevel::Warning => "🟡",
            HealthLevel::Critical => "🔴",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sysinfo::SessionStats;
    use std::path::PathBuf;

    fn mock_session(cpu: f64, mem_mb: u64, activity: Option<ActivityLevel>) -> TmuxSession {
        TmuxSession {
            name: "test".to_string(),
            attached: false,
            windows: 1,
            path: Some(PathBuf::from("/tmp")),
            last_activity: if activity.is_some() { Some(0) } else { None },
            git_status: None,
            stats: Some(SessionStats {
                cpu_percent: cpu,
                mem_mb,
                mem_percent: 0.0,
            }),
            cpu_history: Vec::new(),
            mem_history: Vec::new(),
        }
    }

    #[test]
    fn test_healthy_session() {
        let session = mock_session(10.0, 500, Some(ActivityLevel::Active));
        let health = HealthScore::calculate(&session);
        assert_eq!(health.level(), HealthLevel::Healthy);
        assert!(health.score >= 70);
    }

    #[test]
    fn test_warning_session() {
        let session = mock_session(150.0, 2500, Some(ActivityLevel::Idle));
        let health = HealthScore::calculate(&session);
        assert_eq!(health.level(), HealthLevel::Warning);
    }

    #[test]
    fn test_critical_session() {
        let session = mock_session(300.0, 9000, Some(ActivityLevel::Dormant));
        let health = HealthScore::calculate(&session);
        assert_eq!(health.level(), HealthLevel::Critical);
    }
}
