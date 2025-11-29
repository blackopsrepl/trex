use std::path::PathBuf;

// Represents a tmux session with its metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct TmuxSession {
    pub name: String,
    pub attached: bool,
    pub windows: u32,
    pub path: Option<PathBuf>,
}

impl TmuxSession {
    // Returns a string suitable for fuzzy matching (name + path).
    pub fn match_string(&self) -> String {
        match &self.path {
            Some(p) => format!("{} {}", self.name, p.display()),
            None => self.name.clone(),
        }
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
