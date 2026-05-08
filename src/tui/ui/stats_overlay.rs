mod activity;
mod health;
mod layout;
mod resources;

use activity::render_activity_timeline;
use health::render_health_summary;
use layout::{centered_rect, render_overlay_help};
use resources::{render_top_cpu, render_top_memory};

use crate::tui::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::Style,
    widgets::{Block, BorderType, Borders, Clear},
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
            Constraint::Length(8), // Top CPU consumers
            Constraint::Length(8), // Top memory consumers
            Constraint::Length(8), // Health status
            Constraint::Min(1),    // Activity timeline
            Constraint::Length(2), // Help
        ])
        .split(inner);

    render_top_cpu(frame, app, chunks[0]);
    render_top_memory(frame, app, chunks[1]);
    render_health_summary(frame, app, chunks[2]);
    render_activity_timeline(frame, app, chunks[3]);
    render_overlay_help(frame, app, chunks[4]);
}
