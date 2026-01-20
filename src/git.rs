use std::path::Path;
use std::process::Command;

// Git repository status for a session's working directory.
#[derive(Debug, Clone, Default)]
pub struct GitStatus {
    pub is_repo: bool,
    pub branch: Option<String>,
    pub dirty_count: u32,
    pub ahead: u32,
    pub behind: u32,
}

impl GitStatus {
    // Fetches git status for the given path.
    pub fn for_path(path: &Path) -> Self {
        if !path.exists() {
            return Self::default();
        }

        // Check if it's a git repo
        let is_repo = Command::new("git")
            .args(["-C", &path.display().to_string(), "rev-parse", "--git-dir"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !is_repo {
            return Self::default();
        }

        let branch = Self::get_branch(path);
        let dirty_count = Self::get_dirty_count(path);
        let (ahead, behind) = Self::get_ahead_behind(path);

        Self {
            is_repo: true,
            branch,
            dirty_count,
            ahead,
            behind,
        }
    }

    // Gets the current branch name.
    fn get_branch(path: &Path) -> Option<String> {
        let output = Command::new("git")
            .args([
                "-C",
                &path.display().to_string(),
                "rev-parse",
                "--abbrev-ref",
                "HEAD",
            ])
            .output()
            .ok()?;

        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if branch.is_empty() || branch == "HEAD" {
                // Detached HEAD - get short commit hash instead
                let hash_output = Command::new("git")
                    .args([
                        "-C",
                        &path.display().to_string(),
                        "rev-parse",
                        "--short",
                        "HEAD",
                    ])
                    .output()
                    .ok()?;
                if hash_output.status.success() {
                    return Some(String::from_utf8_lossy(&hash_output.stdout).trim().to_string());
                }
                None
            } else {
                Some(branch)
            }
        } else {
            None
        }
    }

    // Counts dirty files (modified, staged, untracked).
    fn get_dirty_count(path: &Path) -> u32 {
        let output = Command::new("git")
            .args([
                "-C",
                &path.display().to_string(),
                "status",
                "--porcelain",
            ])
            .output()
            .ok();

        output
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .filter(|l| !l.is_empty())
                    .count() as u32
            })
            .unwrap_or(0)
    }

    // Gets commits ahead/behind upstream.
    fn get_ahead_behind(path: &Path) -> (u32, u32) {
        // First check if there's an upstream branch
        let upstream = Command::new("git")
            .args([
                "-C",
                &path.display().to_string(),
                "rev-parse",
                "--abbrev-ref",
                "@{upstream}",
            ])
            .output();

        if upstream.map(|o| !o.status.success()).unwrap_or(true) {
            return (0, 0);
        }

        let output = Command::new("git")
            .args([
                "-C",
                &path.display().to_string(),
                "rev-list",
                "--left-right",
                "--count",
                "HEAD...@{upstream}",
            ])
            .output()
            .ok();

        output
            .and_then(|o| {
                if o.status.success() {
                    let s = String::from_utf8_lossy(&o.stdout);
                    let parts: Vec<&str> = s.split_whitespace().collect();
                    if parts.len() == 2 {
                        let ahead = parts[0].parse().unwrap_or(0);
                        let behind = parts[1].parse().unwrap_or(0);
                        return Some((ahead, behind));
                    }
                }
                None
            })
            .unwrap_or((0, 0))
    }

    // Returns a formatted badge string for display.
    // Format: "main +3 ↑2↓1" or just "main" if clean
    pub fn badge(&self) -> Option<String> {
        if !self.is_repo {
            return None;
        }

        let branch = self.branch.as_ref()?;
        let mut parts = vec![branch.clone()];

        if self.dirty_count > 0 {
            parts.push(format!("+{}", self.dirty_count));
        }

        if self.ahead > 0 || self.behind > 0 {
            let mut sync = String::new();
            if self.ahead > 0 {
                sync.push_str(&format!("↑{}", self.ahead));
            }
            if self.behind > 0 {
                sync.push_str(&format!("↓{}", self.behind));
            }
            if !sync.is_empty() {
                parts.push(sync);
            }
        }

        Some(parts.join(" "))
    }
}
