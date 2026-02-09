use crate::tui::app::{App, AppMode};
use ratatui::Frame;

// Submodules
mod background;
mod barchart;
mod constants;
mod directory;
mod expanded;
mod naming;
mod normal;
mod stats_overlay;

// Re-export only the main rendering function that's called from render()
// Helper functions are internal and not re-exported
use background::render_background_trex;
use barchart::render_barchart_view;
use directory::render_directory_mode;
use expanded::render_expanded_mode;
use naming::render_naming_mode;
use normal::render_normal_mode;
use stats_overlay::render_stats_overlay;

/// Renders the entire TUI based on the current app state.
pub fn render(frame: &mut Frame, app: &App) {
    // Render decorative T-Rex background first (behind everything)
    render_background_trex(frame, app, frame.area());

    match app.mode {
        AppMode::SelectingDirectory => render_directory_mode(frame, app),
        AppMode::NamingSession => render_naming_mode(frame, app),
        AppMode::ExpandedSession => render_expanded_mode(frame, app),
        AppMode::BarChartView => render_barchart_view(frame, app),
        AppMode::StatsOverlay => render_stats_overlay(frame, app),
        _ => render_normal_mode(frame, app),
    }
}
