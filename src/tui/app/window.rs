use crate::tmux::{TmuxClient, TmuxWindow};

use super::{App, AppMode, SessionAction};

impl App {
    // Expands the selected session to show its windows.
    pub fn expand_selected(&mut self) {
        if let Some(session) = self.selected_session() {
            let session_name = session.name.clone();
            if let Ok(windows) = TmuxClient::list_windows(&session_name) {
                self.expanded_session = Some(session_name);
                self.expanded_windows = windows;
                self.selected_window_index = 0;
                self.mode = AppMode::ExpandedSession;
            }
        }
    }

    // Collapses the expanded session view.
    pub fn collapse_session(&mut self) {
        self.expanded_session = None;
        self.expanded_windows.clear();
        self.selected_window_index = 0;
        self.mode = AppMode::Normal;
    }

    // Moves selection to the next window (wraps around).
    pub fn select_next_window(&mut self) {
        if !self.expanded_windows.is_empty() {
            self.selected_window_index =
                (self.selected_window_index + 1) % self.expanded_windows.len();
        }
    }

    // Moves selection to the previous window (wraps around).
    pub fn select_previous_window(&mut self) {
        if !self.expanded_windows.is_empty() {
            self.selected_window_index = if self.selected_window_index == 0 {
                self.expanded_windows.len() - 1
            } else {
                self.selected_window_index - 1
            };
        }
    }

    // Returns the currently selected window, if any.
    pub fn selected_window(&self) -> Option<&TmuxWindow> {
        self.expanded_windows.get(self.selected_window_index)
    }

    // Attaches to the selected window.
    pub fn attach_selected_window(&mut self) {
        if let (Some(session_name), Some(window)) = (&self.expanded_session, self.selected_window())
        {
            self.action = Some(SessionAction::AttachWindow(
                session_name.clone(),
                window.index,
            ));
            self.should_quit = true;
        }
    }
}
