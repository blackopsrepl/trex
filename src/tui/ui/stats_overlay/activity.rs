use crate::tmux::ActivityLevel;
use crate::tui::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph, Wrap},
};

pub(super) fn render_activity_timeline(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.theme.primary))
        .title(" ⏱️  Recent Activity Timeline ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut sessions_with_activity: Vec<_> = app
        .sessions
        .iter()
        .filter_map(|s| s.last_activity.map(|ts| (s, ts)))
        .collect();
    sessions_with_activity.sort_by_key(|item| std::cmp::Reverse(item.1));

    let mut lines = Vec::new();
    for (session, _) in sessions_with_activity.iter().take(10) {
        let activity_str = session
            .activity_ago_string()
            .unwrap_or_else(|| "unknown".to_string());
        let (icon, color) = match session.activity_level() {
            Some(ActivityLevel::Active) => ("●", app.theme.success),
            Some(ActivityLevel::Idle) => ("○", app.theme.warning),
            Some(ActivityLevel::Dormant) => ("◌", app.theme.text_dim),
            None => ("?", app.theme.text_dim),
        };

        let attach_icon = if session.attached { " ★" } else { "" };

        lines.push(Line::from(vec![
            Span::styled(icon, Style::default().fg(color)),
            Span::styled(" ", Style::default()),
            Span::styled(
                format!("{:.<25}", &session.name),
                Style::default().fg(app.theme.text),
            ),
            Span::styled(
                format!("{:>6} ago", activity_str),
                Style::default().fg(app.theme.info),
            ),
            Span::styled(attach_icon, Style::default().fg(app.theme.primary)),
        ]));
    }

    if lines.is_empty() {
        lines.push(Line::from(Span::styled(
            "No activity data available",
            Style::default().fg(app.theme.text_dim),
        )));
    }

    let para = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(para, inner);
}
