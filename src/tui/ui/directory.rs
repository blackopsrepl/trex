use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::tui::app::App;

/// Renders the directory selection mode layout.
///
/// This function creates a three-section vertical layout:
/// - Top: Header with depth indicator and filter input
/// - Middle: Directory list with selection highlighting
/// - Bottom: Help line with available keybindings
pub fn render_directory_mode(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header_dir(frame, app, chunks[0]);
    render_directory_list(frame, app, chunks[1]);
    render_help_dir(frame, chunks[2]);
}

/// Renders the header bar for directory selection mode.
///
/// Displays:
/// - Title with current scan depth and filter input
/// - Bottom title with filtered directory count
pub fn render_header_dir(frame: &mut Frame, app: &App, area: Rect) {
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

/// Renders the directory list with selection highlighting.
///
/// Each directory item shows:
/// - Directory name (highlighted in yellow if selected)
/// - Full path in cyan within square brackets
///
/// Selected item has a dark gray background.
/// Empty state shows "No directories found" message.
pub fn render_directory_list(frame: &mut Frame, app: &App, area: Rect) {
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

/// Renders the help line for directory selection mode.
///
/// Shows available keybindings:
/// - Type: filter directories (fuzzy matching)
/// - Tab: complete filter with selected directory path
/// - +/-: increase/decrease scan depth
/// - Enter: proceed to session naming
/// - Esc: cancel and return to normal mode
pub fn render_help_dir(frame: &mut Frame, area: Rect) {
    let help_text = "Type: filter | Tab: complete | +/-: depth | Enter: name session | Esc: cancel";
    let paragraph = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));

    frame.render_widget(paragraph, area);
}
