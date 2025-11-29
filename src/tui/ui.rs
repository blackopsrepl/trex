use crate::tui::app::{App, AppMode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

// Renders the entire TUI based on the current app state.
pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    match app.mode {
        AppMode::SelectingDirectory => {
            render_header_dir(frame, app, chunks[0]);
            render_directory_list(frame, app, chunks[1]);
            render_help_dir(frame, chunks[2]);
        }
        _ => {
            render_header(frame, app, chunks[0]);
            render_session_list(frame, app, chunks[1]);
            render_help(frame, app, chunks[2]);
        }
    }
}

// Renders the header bar showing mode and filter input.
fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let (title, style) = match app.mode {
        AppMode::Normal => (
            format!(" trex - {} sessions ", app.sessions.len()),
            Style::default().fg(Color::Cyan),
        ),
        AppMode::Filtering => (
            format!(" > {} ", app.filter_input),
            Style::default().fg(Color::Yellow),
        ),
        AppMode::SelectingDirectory => (
            format!(" Select directory > {} ", app.dir_filter_input),
            Style::default().fg(Color::Green),
        ),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(style)
        .title(title);

    frame.render_widget(block, area);
}

// Renders the session list with selection highlighting.
fn render_session_list(frame: &mut Frame, app: &App, area: Rect) {
    if app.filtered_indices.is_empty() {
        let empty_msg = if app.sessions.is_empty() {
            "No tmux sessions found. Create one with: tmux new -s <name>"
        } else {
            "No sessions match your filter"
        };
        let paragraph = Paragraph::new(empty_msg)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = app
        .filtered_indices
        .iter()
        .enumerate()
        .map(|(idx, &session_idx)| {
            let session = &app.sessions[session_idx];
            let is_selected = idx == app.selected_index;

            let indicator = if session.attached { "*" } else { " " };
            let indicator_style = if session.attached {
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let name_style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let path_display = session
                .path
                .as_ref()
                .map(|p| {
                    let path_str = p.display().to_string();
                    if let Ok(home) = std::env::var("HOME")
                        && path_str.starts_with(&home)
                    {
                        return format!(" [~{}]", &path_str[home.len()..]);
                    }
                    format!(" [{}]", path_str)
                })
                .unwrap_or_default();

            let line = Line::from(vec![
                Span::styled(indicator, indicator_style),
                Span::raw(" "),
                Span::styled(&session.name, name_style),
                Span::styled(
                    format!(" ({} win)", session.windows),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(path_display, Style::default().fg(Color::Cyan)),
            ]);

            let item_style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            ListItem::new(line).style(item_style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL));

    frame.render_widget(list, area);
}

// Renders the help line showing available keybindings.
fn render_help(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.mode {
        AppMode::Normal => {
            "j/k: navigate | Enter: attach | c: create | d: delete | D: delete all | x: detach | X: detach all | /: filter | q/C-t: quit"
        }
        AppMode::Filtering => "Type to filter | Enter: attach | Esc: clear | Tab: navigate",
        AppMode::SelectingDirectory => {
            "Type: filter | Tab: complete | +/-: depth | Enter: create | Esc: cancel"
        }
    };

    let paragraph = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));

    frame.render_widget(paragraph, area);
}

// Renders the header bar for directory selection mode.
fn render_header_dir(frame: &mut Frame, app: &App, area: Rect) {
    let title = format!(
        " Select directory (depth: {}) > {} ",
        app.dir_scan_depth, app.dir_filter_input
    );
    let dir_count = format!(" {} dirs ", app.dir_filtered_indices.len());
    let style = Style::default().fg(Color::Green);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(style)
        .title(title)
        .title_bottom(dir_count);

    frame.render_widget(block, area);
}

// Renders the directory list with selection highlighting.
fn render_directory_list(frame: &mut Frame, app: &App, area: Rect) {
    if app.dir_filtered_indices.is_empty() {
        let paragraph = Paragraph::new("No directories found")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = app
        .dir_filtered_indices
        .iter()
        .enumerate()
        .map(|(idx, &dir_idx)| {
            let dir = &app.directories[dir_idx];
            let is_selected = idx == app.dir_selected_index;

            let path_str = dir.path.display().to_string();
            let display_name = dir
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path_str.clone());

            let name_style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let item_style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::styled(display_name, name_style),
                Span::styled(format!(" [{}]", path_str), Style::default().fg(Color::Cyan)),
            ]);

            ListItem::new(line).style(item_style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL));

    frame.render_widget(list, area);
}

// Renders the help line for directory selection mode.
fn render_help_dir(frame: &mut Frame, area: Rect) {
    let help_text = "Type: filter | Tab: complete | +/-: depth | Enter: create | Esc: cancel";
    let paragraph = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));

    frame.render_widget(paragraph, area);
}
