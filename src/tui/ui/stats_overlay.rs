use crate::health::HealthScore;
use crate::tui::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, BorderType, Clear, Paragraph, Wrap,
    },
};

pub fn render_stats_overlay(frame: &mut Frame, app: &App) {
    // Calculate overlay area (80% of screen, centered)
    let area = centered_rect(85, 85, frame.area());
    
    // Clear the area and render semi-transparent background
    frame.render_widget(Clear, area);
    
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .border_style(Style::default().fg(app.theme.primary))
        .title(" 📈 TREX STATS OVERLAY ")
        .title_alignment(Alignment::Center)
        .style(Style::default().bg(app.theme.bg_overlay));

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    // Split into sections
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),   // Top CPU consumers
            Constraint::Length(8),   // Top memory consumers
            Constraint::Length(8),   // Health status
            Constraint::Min(1),      // Activity timeline
            Constraint::Length(2),   // Help
        ])
        .split(inner);

    render_top_cpu(frame, app, chunks[0]);
    render_top_memory(frame, app, chunks[1]);
    render_health_summary(frame, app, chunks[2]);
    render_activity_timeline(frame, app, chunks[3]);
    render_overlay_help(frame, app, chunks[4]);
}

fn render_top_cpu(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.theme.warning))
        .title(" 🔥 Top CPU Consumers ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Collect and sort by CPU
    let mut sessions: Vec<_> = app.sessions.iter()
        .filter_map(|s| s.stats.as_ref().map(|stats| (s, stats)))
        .collect();
    sessions.sort_by(|a, b| b.1.cpu_percent.partial_cmp(&a.1.cpu_percent).unwrap());

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
            Span::styled(format!("{}. ", i + 1), Style::default().fg(app.theme.text_dim)),
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

fn render_top_memory(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.theme.info))
        .title(" 💾 Top Memory Consumers ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Collect and sort by memory
    let mut sessions: Vec<_> = app.sessions.iter()
        .filter_map(|s| s.stats.as_ref().map(|stats| (s, stats)))
        .collect();
    sessions.sort_by(|a, b| b.1.mem_mb.cmp(&a.1.mem_mb));

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
            Span::styled(format!("{}. ", i + 1), Style::default().fg(app.theme.text_dim)),
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

fn render_health_summary(frame: &mut Frame, app: &App, area: Rect) {
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
            crate::health::HealthLevel::Healthy => healthy += 1,
            crate::health::HealthLevel::Warning => warning += 1,
            crate::health::HealthLevel::Critical => {
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
            Span::styled(format!("{}", healthy), Style::default().fg(app.theme.text).add_modifier(Modifier::BOLD)),
            Span::styled("  🟡 Warning: ", Style::default().fg(app.theme.warning)),
            Span::styled(format!("{}", warning), Style::default().fg(app.theme.text).add_modifier(Modifier::BOLD)),
            Span::styled("  🔴 Critical: ", Style::default().fg(app.theme.error)),
            Span::styled(format!("{}", critical), Style::default().fg(app.theme.text).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
    ];

    if !critical_sessions.is_empty() {
        lines.push(Line::from(Span::styled(
            "Critical sessions:",
            Style::default().fg(app.theme.error).add_modifier(Modifier::BOLD),
        )));
        for (name, score) in critical_sessions {
            lines.push(Line::from(vec![
                Span::styled("  • ", Style::default().fg(app.theme.error)),
                Span::styled(name, Style::default().fg(app.theme.text)),
                Span::styled(format!(" (score: {})", score), Style::default().fg(app.theme.text_dim)),
            ]));
        }
    }

    let para = Paragraph::new(lines);
    frame.render_widget(para, inner);
}

fn render_activity_timeline(frame: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.theme.primary))
        .title(" ⏱️  Recent Activity Timeline ");

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Sort sessions by last activity
    let mut sessions_with_activity: Vec<_> = app.sessions.iter()
        .filter_map(|s| s.last_activity.map(|ts| (s, ts)))
        .collect();
    sessions_with_activity.sort_by(|a, b| b.1.cmp(&a.1));

    let mut lines = Vec::new();
    for (session, _) in sessions_with_activity.iter().take(10) {
        let activity_str = session.activity_ago_string().unwrap_or_else(|| "unknown".to_string());
        let (icon, color) = match session.activity_level() {
            Some(crate::tmux::ActivityLevel::Active) => ("●", app.theme.success),
            Some(crate::tmux::ActivityLevel::Idle) => ("○", app.theme.warning),
            Some(crate::tmux::ActivityLevel::Dormant) => ("◌", app.theme.text_dim),
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

fn render_overlay_help(frame: &mut Frame, app: &App, area: Rect) {
    let help = Paragraph::new(Line::from(vec![
        Span::styled("Press ", Style::default().fg(app.theme.text_dim)),
        Span::styled("S", Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled(" or ", Style::default().fg(app.theme.text_dim)),
        Span::styled("ESC", Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled(" to close overlay", Style::default().fg(app.theme.text_dim)),
    ]));
    
    frame.render_widget(help, area);
}

/// Helper function to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
