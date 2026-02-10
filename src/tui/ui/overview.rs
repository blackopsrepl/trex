use crate::tui::app::App;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

// Pulsing dot animation frames
const PULSE_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn render_system_overview(frame: &mut Frame, app: &App, area: Rect) {
    // Calculate totals across all sessions
    let mut total_cpu = 0.0;
    let mut total_mem_mb = 0u64;
    let mut active_sessions = 0;

    for session in &app.sessions {
        if let Some(ref stats) = session.stats {
            total_cpu += stats.cpu_percent;
            total_mem_mb += stats.mem_mb;
        }
        if session.attached {
            active_sessions += 1;
        }
    }

    // Pulsing spinner shows the app is alive and sampling
    let pulse = PULSE_FRAMES[(app.tick as usize / 2) % PULSE_FRAMES.len()];

    let overview_line = Line::from(vec![
        Span::styled(
            format!("{} ", pulse),
            Style::default().fg(app.theme.success),
        ),
        Span::styled(
            "TREX ",
            Style::default()
                .fg(app.theme.primary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("│ ", Style::default().fg(app.theme.text_dim)),
        Span::styled(
            format!("{} sessions", app.sessions.len()),
            Style::default().fg(app.theme.info),
        ),
        Span::styled(" │ ", Style::default().fg(app.theme.text_dim)),
        Span::styled(
            format!("{} attached", active_sessions),
            Style::default().fg(app.theme.success),
        ),
        Span::styled(" │ ", Style::default().fg(app.theme.text_dim)),
        Span::styled(
            format!("CPU: {:.1}%", total_cpu),
            Style::default().fg(if total_cpu > 300.0 {
                app.theme.error
            } else if total_cpu > 150.0 {
                app.theme.warning
            } else {
                app.theme.success
            }),
        ),
        Span::styled(" │ ", Style::default().fg(app.theme.text_dim)),
        Span::styled(
            format!("MEM: {}MB", total_mem_mb),
            Style::default().fg(if total_mem_mb > 4096 {
                app.theme.error
            } else if total_mem_mb > 2048 {
                app.theme.warning
            } else {
                app.theme.info
            }),
        ),
        Span::styled(" │ ", Style::default().fg(app.theme.text_dim)),
        Span::styled(
            format!("{} agents", app.ai_processes.len()),
            Style::default().fg(app.theme.secondary),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(app.theme.primary));

    let para = Paragraph::new(overview_line)
        .block(block)
        .style(Style::default().bg(app.theme.bg_primary));

    frame.render_widget(para, area);
}
