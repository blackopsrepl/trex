pub mod app;
pub mod events;
pub mod ui;

use crate::tmux::TmuxSession;
use crate::tui::app::{App, SessionAction};
use crate::tui::events::handle_key;
use crate::tui::ui::render;

use anyhow::Result;
use crossterm::{
    ExecutableCommand,
    event::{self, Event},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{Stdout, Write, stdout};
use std::time::{Duration, Instant};

// Runs the TUI with a specific session preselected.
//
// Sets up the terminal, runs the event loop, then restores the terminal.
// Returns the action selected by the user, if any.
pub fn run_tui_with_preselection(
    sessions: Vec<TmuxSession>,
    preselect_index: usize,
) -> Result<Option<SessionAction>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::with_preselection(sessions, preselect_index);
    let mut matcher = nucleo::Matcher::new(nucleo::Config::DEFAULT);
    let result = run_app(&mut terminal, &mut app, &mut matcher);

    drop(terminal);
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    stdout().flush()?;

    result?;
    Ok(app.action)
}

// Main event loop that renders the UI and handles input.
fn run_app(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
    matcher: &mut nucleo::Matcher,
) -> Result<()> {
    let mut last_state_refresh = Instant::now();
    let mut last_full_rescan = Instant::now();

    loop {
        terminal.draw(|frame| render(frame, app))?;

        if event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            handle_key(app, key.code, key.modifiers, matcher);
        }

        // Refresh process states every 100ms (real-time activity indicators)
        if last_state_refresh.elapsed() >= Duration::from_millis(100) {
            app.refresh_ai_process_states();
            last_state_refresh = Instant::now();
        }

        // Full rescan for new/exited processes every 2 seconds
        if last_full_rescan.elapsed() >= Duration::from_secs(2) {
            app.rescan_ai_processes();
            last_full_rescan = Instant::now();
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
