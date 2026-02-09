use crate::tui::app::{App, AppMode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::agents::render_agent_box;
use super::overview::render_system_overview;
use super::sessions::render_session_list;

pub fn render_normal_mode(frame: &mut Frame, app: &App) {
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
            Constraint::Length(agent_rows + 2), // Agent box (content + borders)
            Constraint::Min(1),                 // Sessions
            Constraint::Length(2),              // Enhanced help
        ])
        .split(frame.area());

    render_system_overview(frame, app, chunks[0]);
    render_agent_box(frame, app, chunks[1]);

    // If preview is enabled, split the session area
    if app.show_preview {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[2]);
        render_session_list(frame, app, main_chunks[0]);
        render_preview(frame, app, main_chunks[1]);
    } else {
        render_session_list(frame, app, chunks[2]);
    }
    render_help(frame, app, chunks[3]);
}

fn render_preview(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let session_name = app
        .selected_session()
        .map(|s| s.name.as_str())
        .unwrap_or("No session");

    let title = format!(" Preview: {} ", session_name);

    let content = if app.preview_lines.is_empty() {
        vec![Line::from(Span::styled(
            "No content to preview",
            Style::default().fg(app.theme.text_dim),
        ))]
    } else {
        app.preview_lines
            .iter()
            .map(|line| Line::from(Span::raw(line.as_str())))
            .collect()
    };

    let paragraph = Paragraph::new(content)
        .style(Style::default().fg(app.theme.text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.info))
                .title(title),
        );

    frame.render_widget(paragraph, area);
}

pub fn render_help(frame: &mut Frame, app: &App, area: ratatui::layout::Rect) {
    let help_items: Vec<(&str, &str)> = match app.mode {
        AppMode::Normal => vec![
            ("j/k", "nav"),
            ("l", "expand"),
            ("p", "preview"),
            ("b", "charts"),
            ("s", "stats"),
            ("↵", "attach"),
            ("c", "create"),
            ("d", "delete"),
            ("/", "filter"),
            ("q", "quit"),
        ],
        AppMode::Filtering => vec![
            ("type", "filter"),
            ("↵", "attach"),
            ("Esc", "clear"),
            ("Tab", "nav"),
        ],
        AppMode::SelectingDirectory | AppMode::NamingSession => vec![
            ("type", "filter"),
            ("Tab", "complete"),
            ("+/-", "depth"),
            ("↵", "name"),
            ("Esc", "cancel"),
        ],
        AppMode::ExpandedSession => vec![
            ("j/k", "nav"),
            ("↵", "attach"),
            ("h/Esc", "back"),
            ("q", "quit"),
        ],
        AppMode::BarChartView => vec![("b/Esc", "back"), ("q", "quit")],
        AppMode::StatsOverlay => vec![("s/Esc", "close"), ("q", "quit")],
    };

    let mut spans = Vec::new();

    for (i, (key, action)) in help_items.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" │ ", Style::default().fg(app.theme.text_dim)));
        }
        spans.push(Span::styled(
            *key,
            Style::default()
                .fg(app.theme.primary)
                .add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(" {}", action),
            Style::default().fg(app.theme.text),
        ));
    }

    let help_line = Line::from(spans);

    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(app.theme.text_dim));

    let para = Paragraph::new(help_line)
        .block(block)
        .style(Style::default().bg(app.theme.bg_primary));

    frame.render_widget(para, area);
}
