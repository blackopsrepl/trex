use crate::tui::app::{App, AppMode, FocusArea};
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
        AppMode::NamingSession => handle_naming_mode(app, code),
        AppMode::ExpandedSession => handle_expanded_mode(app, code),
        AppMode::BarChartView => handle_barchart_mode(app, code),
        AppMode::StatsOverlay => handle_stats_overlay_mode(app, code),
    }
}

// Handles key events in normal mode (session list navigation and actions).
fn handle_normal_mode(app: &mut App, code: KeyCode, _matcher: &mut nucleo::Matcher) {
    match code {
        KeyCode::Char('j') | KeyCode::Down => {
            match app.focus {
                FocusArea::Agents => {
                    if app.at_bottom_of_agents() {
                        // Move focus to sessions
                        app.focus = FocusArea::Sessions;
                        app.select_first();
                    } else {
                        app.select_agent_next();
                    }
                }
                FocusArea::Sessions => {
                    app.select_next();
                }
            }
            app.refresh_preview();
        }
        KeyCode::Char('k') | KeyCode::Up => {
            match app.focus {
                FocusArea::Sessions => {
                    if app.at_top_of_sessions() && !app.visible_agents().is_empty() {
                        // Move focus to agents
                        app.focus = FocusArea::Agents;
                        app.select_agent_last();
                    } else {
                        app.select_previous();
                    }
                }
                FocusArea::Agents => {
                    app.select_agent_previous();
                }
            }
            app.refresh_preview();
        }
        KeyCode::Char('g') | KeyCode::Home => {
            match app.focus {
                FocusArea::Agents => app.select_agent_first(),
                FocusArea::Sessions => app.select_first(),
            }
            app.refresh_preview();
        }
        KeyCode::Char('G') | KeyCode::End => {
            match app.focus {
                FocusArea::Agents => app.select_agent_last(),
                FocusArea::Sessions => app.select_last(),
            }
            app.refresh_preview();
        }

        KeyCode::Enter => match app.focus {
            FocusArea::Agents => app.attach_selected_agent(),
            FocusArea::Sessions => app.attach_selected(),
        },
        KeyCode::Char('d') => {
            if app.focus == FocusArea::Sessions {
                app.delete_selected();
            }
        }
        KeyCode::Char('D') => {
            if app.focus == FocusArea::Sessions {
                app.delete_all();
            }
        }
        KeyCode::Char('x') => {
            if app.focus == FocusArea::Sessions {
                app.detach_selected();
            }
        }
        KeyCode::Char('X') => {
            if app.focus == FocusArea::Sessions {
                app.detach_all();
            }
        }
        KeyCode::Char('c') => app.mode = AppMode::SelectingDirectory,

        // Window expansion (only from session focus)
        KeyCode::Char('l') | KeyCode::Right => {
            if app.focus == FocusArea::Sessions {
                app.expand_selected();
            }
        }

        // Preview toggle
        KeyCode::Char('p') => app.toggle_preview(),

        KeyCode::Char('/') => app.mode = AppMode::Filtering,

        // Bar chart view toggle
        KeyCode::Char('b') => app.mode = AppMode::BarChartView,

        // Stats overlay toggle
        KeyCode::Char('s') => app.mode = AppMode::StatsOverlay,

        KeyCode::Esc | KeyCode::Char('q') => app.should_quit = true,

        _ => {}
    }
}

// Handles key events in filtering mode (session fuzzy search).
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

        KeyCode::Enter => app.enter_naming_mode(),

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

// Handles key events in session naming mode.
fn handle_naming_mode(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter => app.confirm_session_name(),
        KeyCode::Esc => app.cancel_naming(),
        KeyCode::Backspace => {
            app.session_name_input.pop();
        }
        KeyCode::Char(c) => {
            app.session_name_input.push(c);
        }
        _ => {}
    }
}

// Handles key events in expanded session mode (window list navigation).
fn handle_expanded_mode(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('j') | KeyCode::Down => app.select_next_window(),
        KeyCode::Char('k') | KeyCode::Up => app.select_previous_window(),

        KeyCode::Enter => app.attach_selected_window(),

        // Collapse back to normal mode
        KeyCode::Char('h') | KeyCode::Left | KeyCode::Esc => app.collapse_session(),

        KeyCode::Char('q') => app.should_quit = true,

        _ => {}
    }
}

// Handles key events in bar chart view mode.
fn handle_barchart_mode(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('b') | KeyCode::Esc => {
            app.mode = AppMode::Normal;
        }
        KeyCode::Char('q') => app.should_quit = true,
        _ => {}
    }
}

// Handles key events in stats overlay mode.
fn handle_stats_overlay_mode(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('s') | KeyCode::Esc => {
            app.mode = AppMode::Normal;
        }
        KeyCode::Char('q') => app.should_quit = true,
        _ => {}
    }
}
