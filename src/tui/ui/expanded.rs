use crate::tui::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use super::normal::render_agent_box;

/// Renders the expanded session mode layout.
pub fn render_expanded_mode(frame: &mut Frame, app: &App) {
    let visible_agents = app.visible_agents();
    let agent_rows = if visible_agents.is_empty() { 1 } else { visible_agents.len().min(5) } as u16;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(agent_rows + 2), // Agent box (filtered to session)
            Constraint::Min(1),                 // Windows
            Constraint::Length(1),              // Help
        ])
        .split(frame.area());

    render_agent_box(frame, app, chunks[0]);
    render_window_list(frame, app, chunks[1]);
    render_help_expanded(frame, chunks[2]);
}

/// Renders the window list for expanded session mode.
pub fn render_window_list(frame: &mut Frame, app: &App, area: Rect) {
    let session_name = app.expanded_session.as_deref().unwrap_or("session");
    let title = format!(" {} - {} windows ", session_name, app.expanded_windows.len());

    if app.expanded_windows.is_empty() {
        let paragraph = Paragraph::new("No windows found")
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Blue))
                    .title(title),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = app
        .expanded_windows
        .iter()
        .enumerate()
        .map(|(idx, window)| {
            let is_selected = idx == app.selected_window_index;

            let active_icon = if window.active { "●" } else { " " };
            let active_style = if window.active {
                Style::default().fg(Color::Green)
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

            let line = Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(active_icon, active_style),
                Span::raw(" "),
                Span::styled(format!("{}: ", window.index), Style::default().fg(Color::DarkGray)),
                Span::styled(&window.name, name_style),
                Span::styled(
                    format!(" [{}]", window.current_command),
                    Style::default().fg(Color::Cyan),
                ),
            ]);

            let item_style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            ListItem::new(line).style(item_style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue))
            .title(title),
    );

    frame.render_widget(list, area);
}

/// Renders the help line for expanded session mode.
pub fn render_help_expanded(frame: &mut Frame, area: Rect) {
    let help_text = "j/k: navigate | Enter: attach window | h/Esc: collapse | q: quit";
    let paragraph = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));

    frame.render_widget(paragraph, area);
}
