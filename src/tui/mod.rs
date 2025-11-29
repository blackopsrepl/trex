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
    loop {
        terminal.draw(|frame| render(frame, app))?;

        if event::poll(std::time::Duration::from_millis(100))?
            && let Event::Key(key) = event::read()?
        {
            handle_key(app, key.code, key.modifiers, matcher);
        }

        if app.should_quit {
            break;
        }
    }
    Ok(())
}
