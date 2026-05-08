use crate::tui::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub(super) fn render_top_cpu(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.theme.warning))
        .title(" 🔥 Top CPU Consumers ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut sessions: Vec<_> = app
        .sessions
        .iter()
        .filter_map(|s| s.stats.as_ref().map(|stats| (s, stats)))
        .collect();
    sessions.sort_by(|a, b| b.1.cpu_percent.total_cmp(&a.1.cpu_percent));

    let mut lines = Vec::new();
    for (i, (session, stats)) in sessions.iter().take(5).enumerate() {
        let color = if stats.cpu_percent > 200.0 {
            app.theme.error
        } else if stats.cpu_percent > 100.0 {
            app.theme.warning
        } else {
            app.theme.success
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!("{}. ", i + 1),
                Style::default().fg(app.theme.text_dim),
            ),
            Span::styled(
                format!("{:.<20}", &session.name),
                Style::default().fg(app.theme.text),
            ),
            Span::styled(
                format!("{:>8.1}%", stats.cpu_percent),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "No session data available",
            Style::default().fg(app.theme.text_dim),
        )));
    }

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner);
}

pub(super) fn render_top_memory(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.theme.info))
        .title(" 💾 Top Memory Consumers ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut sessions: Vec<_> = app
        .sessions
        .iter()
        .filter_map(|s| s.stats.as_ref().map(|stats| (s, stats)))
        .collect();
    sessions.sort_by_key(|item| std::cmp::Reverse(item.1.mem_mb));

    let mut lines = Vec::new();
    for (i, (session, stats)) in sessions.iter().take(5).enumerate() {
        let color = if stats.mem_mb > 4096 {
            app.theme.error
        } else if stats.mem_mb > 2048 {
            app.theme.warning
        } else {
            app.theme.info
        };

        lines.push(Line::from(vec![
            Span::styled(
                format!("{}. ", i + 1),
                Style::default().fg(app.theme.text_dim),
            ),
            Span::styled(
                format!("{:.<20}", &session.name),
                Style::default().fg(app.theme.text),
            ),
            Span::styled(
                format!("{:>8} MB", stats.mem_mb),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ),
        ]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "No session data available",
            Style::default().fg(app.theme.text_dim),
        )));
    }

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner);
}
