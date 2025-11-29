use std::path::PathBuf;

/// Minimum scan depth for directory discovery.
pub const MIN_DEPTH: u32 = 1;

/// Maximum scan depth for directory discovery.
pub const MAX_DEPTH: u32 = 6;

/// Default scan depth for directory discovery.
pub const DEFAULT_DEPTH: u32 = 3;

/// A directory that can be used to create a new tmux session.
#[derive(Debug, Clone, PartialEq)]
pub struct Directory {
    pub path: PathBuf,
}

impl Directory {
    // Creates a new `Directory` from a path.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    // Returns a string suitable for fuzzy matching (display name + full path).
    pub fn match_string(&self) -> String {
        let display_name = self
            .path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| self.path.display().to_string());
        format!("{} {}", display_name, self.path.display())
    }

    // Derives a session name from the directory's basename.
    pub fn session_name(&self) -> String {
        self.path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "session".to_string())
    }
}

/* Discovers directories from the filesystem for session creation.
 * Prioritizes the current working directory, home directory, and common
 * subdirectories (projects, work, dev, code, src), then recursively scans
 * from root up to `max_depth` levels. Skips symlinks to avoid infinite loops. */
pub fn discover_directories_with_depth(max_depth: u32) -> Vec<Directory> {
    let mut dirs = Vec::new();
    let mut seen = std::collections::HashSet::new();

    if let Ok(cwd) = std::env::current_dir()
        && seen.insert(cwd.clone())
    {
        dirs.push(Directory::new(cwd));
    }

    if let Ok(home) = std::env::var("HOME")
        && let Ok(home_path) = std::fs::canonicalize(&home)
    {
        if seen.insert(home_path.clone()) {
            dirs.push(Directory::new(home_path.clone()));
        }

        let common_subdirs = vec!["projects", "work", "dev", "code", "src"];
        for subdir in common_subdirs {
            let path = home_path.join(subdir);
            if path.is_dir()
                && let Ok(canonical) = std::fs::canonicalize(&path)
                && seen.insert(canonical.clone())
            {
                dirs.push(Directory::new(canonical));
            }
        }
    }

    scan_directories("/", 0, max_depth, &mut dirs, &mut seen);

    dirs
}

// Recursively scans directories up to `max_depth`, collecting them into `dirs`.
fn scan_directories(
    path: &str,
    current_depth: u32,
    max_depth: u32,
    dirs: &mut Vec<Directory>,
    seen: &mut std::collections::HashSet<std::path::PathBuf>,
) {
    if current_depth >= max_depth {
        return;
    }

    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_symlink() {
                    continue;
                }

                if metadata.is_dir()
                    && let Ok(canonical) = std::fs::canonicalize(entry.path())
                {
                    if seen.insert(canonical.clone()) {
                        dirs.push(Directory::new(canonical));
                    }

                    if let Some(path_str) = entry.path().to_str() {
                        scan_directories(path_str, current_depth + 1, max_depth, dirs, seen);
                    }
                }
            }
        }
    }
}
