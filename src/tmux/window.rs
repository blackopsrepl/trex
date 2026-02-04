#[derive(Debug, Clone)]
pub struct TmuxWindow {
    pub index: u32,
    pub name: String,
    pub active: bool,
    pub current_command: String,
}

// Parses the output of `tmux list-windows` into a list of windows.
pub fn parse_windows(output: &str) -> Vec<TmuxWindow> {
    output
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(parse_window_line)
        .collect()
}

// Parses a single line of window output.
// Format: index|name|active|command
fn parse_window_line(line: &str) -> Option<TmuxWindow> {
    let parts: Vec<&str> = line.split('|').collect();

    if parts.len() < 4 {
        return None;
    }

    let index: u32 = parts[0].parse().ok()?;
    let name = parts[1].to_string();
    let active = parts[2] == "1";
    let current_command = parts[3].to_string();

    Some(TmuxWindow {
        index,
        name,
        active,
        current_command,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_window_line() {
        let line = "0|vim|1|nvim";
        let window = parse_window_line(line).unwrap();
        assert_eq!(window.index, 0);
        assert_eq!(window.name, "vim");
        assert!(window.active);
        assert_eq!(window.current_command, "nvim");
    }

    #[test]
    fn test_parse_windows() {
        let output = "0|vim|1|nvim\n1|shell|0|zsh\n";
        let windows = parse_windows(output);
        assert_eq!(windows.len(), 2);
        assert!(windows[0].active);
        assert!(!windows[1].active);
    }
}
