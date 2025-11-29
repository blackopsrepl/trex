use crate::tui::app::{App, AppMode};
use crossterm::event::{KeyCode, KeyModifiers};

// Handles a key event and updates the app state accordingly.
pub fn handle_key(
    app: &mut App,
    code: KeyCode,
    modifiers: KeyModifiers,
    matcher: &mut nucleo::Matcher,
) {
    if modifiers.contains(KeyModifiers::CONTROL) {
        match code {
            KeyCode::Char('c') | KeyCode::Char('t') => {
                app.should_quit = true;
                return;
            }
            _ => {}
        }
    }

    match app.mode {
        AppMode::Normal => handle_normal_mode(app, code, matcher),
        AppMode::Filtering => handle_filter_mode(app, code, matcher),
        AppMode::SelectingDirectory => handle_dir_mode(app, code, matcher),
    }
}

// Handles key events in normal mode (session list navigation and actions).
fn handle_normal_mode(app: &mut App, code: KeyCode, matcher: &mut nucleo::Matcher) {
    match code {
        KeyCode::Char('j') | KeyCode::Down => app.select_next(),
        KeyCode::Char('k') | KeyCode::Up => app.select_previous(),
        KeyCode::Char('g') | KeyCode::Home => app.select_first(),
        KeyCode::Char('G') | KeyCode::End => app.select_last(),

        KeyCode::Enter => app.attach_selected(),
        KeyCode::Char('d') => app.delete_selected(),
        KeyCode::Char('D') => app.delete_all(),
        KeyCode::Char('x') => app.detach_selected(),
        KeyCode::Char('X') => app.detach_all(),
        KeyCode::Char('c') => app.mode = AppMode::SelectingDirectory,

        KeyCode::Char('/') => app.mode = AppMode::Filtering,

        KeyCode::Esc | KeyCode::Char('q') => app.should_quit = true,

        KeyCode::Char(c) if c.is_alphanumeric() || c == '-' || c == '_' => {
            app.mode = AppMode::Filtering;
            app.filter_input.push(c);
            app.apply_filter(matcher);
        }

        _ => {}
    }
}

/// Handles key events in filtering mode (session fuzzy search).
fn handle_filter_mode(app: &mut App, code: KeyCode, matcher: &mut nucleo::Matcher) {
    match code {
        KeyCode::Esc => {
            app.clear_filter(matcher);
        }
        KeyCode::Enter => {
            app.attach_selected();
        }
        KeyCode::Backspace => {
            app.filter_input.pop();
            app.apply_filter(matcher);
            if app.filter_input.is_empty() {
                app.mode = AppMode::Normal;
            }
        }
        KeyCode::Char(c) => {
            app.filter_input.push(c);
            app.apply_filter(matcher);
        }
        KeyCode::Down | KeyCode::Tab => app.select_next(),
        KeyCode::Up | KeyCode::BackTab => app.select_previous(),
        _ => {}
    }
}

// Handles key events in directory selection mode (creating new sessions).
fn handle_dir_mode(app: &mut App, code: KeyCode, matcher: &mut nucleo::Matcher) {
    match code {
        KeyCode::Down => app.select_dir_next(),
        KeyCode::Up => app.select_dir_previous(),
        KeyCode::Home => app.select_dir_first(),
        KeyCode::End => app.select_dir_last(),

        KeyCode::Enter => app.create_session_in_directory(),

        KeyCode::Char('+') | KeyCode::Char('=') => app.increase_depth(matcher),
        KeyCode::Char('-') | KeyCode::Char('_') => app.decrease_depth(matcher),

        KeyCode::Tab => {
            app.tab_complete_directory();
            app.apply_dir_filter(matcher);
        }

        KeyCode::Esc => {
            app.clear_dir_filter(matcher);
        }

        KeyCode::Backspace => {
            app.dir_filter_input.pop();
            app.apply_dir_filter(matcher);
        }
        KeyCode::Char(c) => {
            app.dir_filter_input.push(c);
            app.apply_dir_filter(matcher);
        }

        _ => {}
    }
}
