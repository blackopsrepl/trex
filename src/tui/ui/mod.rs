use crate::tui::app::{App, AppMode};
use ratatui::Frame;

// Submodules
mod constants;
mod background;
mod normal;
mod directory;
mod naming;
mod expanded;

// Re-export constants for potential external use
pub use constants::{EMERALD_GREEN, TREX_ASCII, EYE_LINE, EYE_CHAR, GREEN_GRADIENT};

// Re-export all rendering functions
pub use background::render_background_trex;
pub use normal::{render_normal_mode, render_agent_box, render_session_list, render_preview, render_help};
pub use directory::{render_directory_mode, render_header_dir, render_directory_list, render_help_dir};
pub use naming::{render_naming_mode, render_header_naming, render_naming_preview, render_help_naming};
pub use expanded::{render_expanded_mode, render_window_list, render_help_expanded};

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
