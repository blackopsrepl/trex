use crate::health::{HealthLevel, HealthScore};
use crate::tui::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub(super) fn render_health_summary(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.theme.secondary))
        .title(" 🏥 Session Health Status ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut healthy = 0;
    let mut warning = 0;
    let mut critical = 0;
    let mut critical_sessions = Vec::new();

    for session in &app.sessions {
        let health = HealthScore::calculate(session);
        match health.level() {
            HealthLevel::Healthy => healthy += 1,
            HealthLevel::Warning => warning += 1,
            HealthLevel::Critical => {
                critical += 1;
                if critical_sessions.len() < 3 {
                    critical_sessions.push((session.name.as_str(), health.score));
                }
            }
        }
    }

    let mut lines = vec![
        Line::from(vec![
            Span::styled("🟢 Healthy: ", Style::default().fg(app.theme.success)),
            Span::styled(
                format!("{}", healthy),
                Style::default()
                    .fg(app.theme.text)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  🟡 Warning: ", Style::default().fg(app.theme.warning)),
            Span::styled(
                format!("{}", warning),
                Style::default()
                    .fg(app.theme.text)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  🔴 Critical: ", Style::default().fg(app.theme.error)),
            Span::styled(
                format!("{}", critical),
                Style::default()
                    .fg(app.theme.text)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
    ];

    if !critical_sessions.is_empty() {
        lines.push(Line::from(Span::styled(
            "Critical sessions:",
            Style::default()
                .fg(app.theme.error)
                .add_modifier(Modifier::BOLD),
        )));
        for (name, score) in critical_sessions {
            lines.push(Line::from(vec![
                Span::styled("  • ", Style::default().fg(app.theme.error)),
                Span::styled(name, Style::default().fg(app.theme.text)),
                Span::styled(
                    format!(" (score: {})", score),
                    Style::default().fg(app.theme.text_dim),
                ),
            ]));
        }
    }

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner);
}
