use crate::tui::app::{App, AppMode};
use ratatui::Frame;

// Submodules
mod constants;
mod background;
mod normal;
mod directory;
mod naming;
mod expanded;

// Re-export only the main rendering function that's called from render()
// Helper functions are internal and not re-exported
use background::render_background_trex;
use normal::render_normal_mode;
use directory::render_directory_mode;
use naming::render_naming_mode;
use expanded::render_expanded_mode;

/// Renders the entire TUI based on the current app state.
pub fn render(frame: &mut Frame, app: &App) {
    // Render decorative T-Rex background first (behind everything)
    render_background_trex(frame, frame.area());

    match app.mode {
        AppMode::SelectingDirectory => render_directory_mode(frame, app),
        AppMode::NamingSession => render_naming_mode(frame, app),
        AppMode::ExpandedSession => render_expanded_mode(frame, app),
        _ => render_normal_mode(frame, app),
    }
}
