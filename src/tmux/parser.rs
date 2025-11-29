use crate::tmux::session::TmuxSession;
use std::path::PathBuf;

// Parses the output of `tmux list-sessions` into a list of sessions.
pub fn parse_sessions(output: &str) -> Vec<TmuxSession> {
    output
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(parse_session_line)
        .collect()
}

// Parses a single line of `tmux list-sessions` output.
//
// Expected format: `name|attached|windows|path`
fn parse_session_line(line: &str) -> Option<TmuxSession> {
    let parts: Vec<&str> = line.split('|').collect();

    if parts.len() < 4 {
        return None;
    }

    let name = parts[0].to_string();
    let attached = parts[1] == "1";
    let windows: u32 = parts[2].parse().unwrap_or(0);
    let path = if parts[3].is_empty() {
        None
    } else {
        Some(PathBuf::from(parts[3]))
    };

    Some(TmuxSession {
        name,
        attached,
        windows,
        path,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_session_line() {
        let line = "dev|1|3|/home/user/project";
        let session = parse_session_line(line).unwrap();
        assert_eq!(session.name, "dev");
        assert!(session.attached);
        assert_eq!(session.windows, 3);
        assert_eq!(session.path, Some(PathBuf::from("/home/user/project")));
    }

    #[test]
    fn test_parse_session_no_path() {
        let line = "scratch|0|1|";
        let session = parse_session_line(line).unwrap();
        assert_eq!(session.name, "scratch");
        assert!(!session.attached);
        assert_eq!(session.windows, 1);
        assert_eq!(session.path, None);
    }

    #[test]
    fn test_parse_sessions() {
        let output = "dev|1|3|/home/user/project\nscratch|0|1|\n";
        let sessions = parse_sessions(output);
        assert_eq!(sessions.len(), 2);
    }
}
