use crate::tui::app::App;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph},
    Frame,
};

use super::agents::render_agent_box;
use super::normal::render_help;
use super::overview::render_system_overview;

pub fn render_expanded_mode(frame: &mut Frame, app: &App) {
    let visible_agents = app.visible_agents();
    let agent_rows = if visible_agents.is_empty() {
        1
    } else {
        visible_agents.len().min(5)
    } as u16;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),              // System overview
            Constraint::Length(agent_rows + 2), // Agent box (filtered to session)
            Constraint::Min(1),                 // Windows
            Constraint::Length(2),              // Help
        ])
        .split(frame.area());

    render_system_overview(frame, app, chunks[0]);
    render_agent_box(frame, app, chunks[1]);
    render_window_list(frame, app, chunks[2]);
    render_help(frame, app, chunks[3]);
}

pub fn render_window_list(frame: &mut Frame, app: &App, area: Rect) {
    let session_name = app.expanded_session.as_deref().unwrap_or("session");
    let title = format!(
        " 🪟 {} - {} windows ",
        session_name,
        app.expanded_windows.len()
    );

    if app.expanded_windows.is_empty() {
        let paragraph = Paragraph::new("No windows found")
            .style(Style::default().fg(app.theme.text_dim))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Double)
                    .border_style(Style::default().fg(app.theme.info))
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

            let active_icon = if window.active { "⚡" } else { "○" };
            let active_style = if window.active {
                Style::default().fg(app.theme.primary)
            } else {
                Style::default().fg(app.theme.text_dim)
            };

            let name_style = if is_selected {
                Style::default()
                    .fg(app.theme.primary)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(app.theme.text)
            };

            let line = Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(active_icon, active_style),
                Span::raw(" "),
                Span::styled(
                    format!("#{} ", window.index),
                    Style::default().fg(app.theme.text_dim),
                ),
                Span::styled(&window.name, name_style),
                Span::styled(
                    format!(" ⟨{}⟩", window.current_command),
                    Style::default().fg(app.theme.info),
                ),
            ]);

            let item_style = if is_selected {
                Style::default().bg(app.theme.bg_highlight)
            } else {
                Style::default()
            };

            ListItem::new(line).style(item_style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(app.theme.primary))
            .title(title),
    );

    frame.render_widget(list, area);
}
