use crate::git::GitStatus;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

// Activity level based on time since last activity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivityLevel {
    Active,  // < 5 minutes
    Idle,    // 5-30 minutes
    Dormant, // > 30 minutes
}

// Represents a tmux session with its metadata.
#[derive(Debug, Clone)]
pub struct TmuxSession {
    pub name: String,
    pub attached: bool,
    pub windows: u32,
    pub path: Option<PathBuf>,
    pub last_activity: Option<u64>,
    pub git_status: Option<GitStatus>,
}

impl TmuxSession {
    // Returns a string suitable for fuzzy matching (name + path).
    pub fn match_string(&self) -> String {
        match &self.path {
            Some(p) => format!("{} {}", self.name, p.display()),
            None => self.name.clone(),
        }
    }

    // Returns the activity level based on time since last activity.
    pub fn activity_level(&self) -> Option<ActivityLevel> {
        let activity_ts = self.last_activity?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_secs();

        let elapsed_secs = now.saturating_sub(activity_ts);
        let elapsed_mins = elapsed_secs / 60;

        Some(if elapsed_mins < 5 {
            ActivityLevel::Active
        } else if elapsed_mins < 30 {
            ActivityLevel::Idle
        } else {
            ActivityLevel::Dormant
        })
    }

    // Returns a human-readable string for time since last activity.
    pub fn activity_ago_string(&self) -> Option<String> {
        let activity_ts = self.last_activity?;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()?
            .as_secs();

        let elapsed_secs = now.saturating_sub(activity_ts);

        Some(if elapsed_secs < 60 {
            format!("{}s", elapsed_secs)
        } else if elapsed_secs < 3600 {
            format!("{}m", elapsed_secs / 60)
        } else if elapsed_secs < 86400 {
            format!("{}h", elapsed_secs / 3600)
        } else {
            format!("{}d", elapsed_secs / 86400)
        })
    }
}

/* Finds the index of a session matching the current working directory.
 * First tries an exact path match, then falls back to matching the directory
 * name against session names. Returns 0 if no match is found. */
pub fn find_matching_session_index(sessions: &[TmuxSession]) -> usize {
    let cwd = match std::env::current_dir() {
        Ok(p) => p,
        Err(_) => return 0,
    };

    for (idx, session) in sessions.iter().enumerate() {
        if let Some(ref session_path) = session.path
            && session_path == &cwd
        {
            return idx;
        }
    }

    if let Some(dir_name) = cwd.file_name() {
        let dir_name_str = dir_name.to_string_lossy();
        for (idx, session) in sessions.iter().enumerate() {
            if session.name == dir_name_str {
                return idx;
            }
        }
    }

    0
}
