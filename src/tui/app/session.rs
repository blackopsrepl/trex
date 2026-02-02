use crate::tmux::TmuxSession;

use super::{App, SessionAction};

impl App {
    // Moves selection to the next session (wraps around).
    pub fn select_next(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_indices.len();
        }
    }

    // Moves selection to the previous session (wraps around).
    pub fn select_previous(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected_index = if self.selected_index == 0 {
                self.filtered_indices.len() - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    // Moves selection to the first session.
    pub fn select_first(&mut self) {
        self.selected_index = 0;
    }

    // Moves selection to the last session.
    pub fn select_last(&mut self) {
        if !self.filtered_indices.is_empty() {
            self.selected_index = self.filtered_indices.len() - 1;
        }
    }

    // Returns the currently selected session, if any.
    pub fn selected_session(&self) -> Option<&TmuxSession> {
        self.filtered_indices
            .get(self.selected_index)
            .and_then(|&idx| self.sessions.get(idx))
    }

    // Sets action to attach to the selected session and quits.
    pub fn attach_selected(&mut self) {
        if let Some(session) = self.selected_session() {
            self.action = Some(SessionAction::Attach(session.name.clone()));
            self.should_quit = true;
        }
    }

    // Sets action to delete the selected session and quits.
    pub fn delete_selected(&mut self) {
        if let Some(session) = self.selected_session() {
            self.action = Some(SessionAction::Delete(session.name.clone()));
            self.should_quit = true;
        }
    }

    // Sets action to delete all sessions and quits.
    pub fn delete_all(&mut self) {
        self.action = Some(SessionAction::DeleteAll);
        self.should_quit = true;
    }

    // Sets action to detach the selected session and quits.
    pub fn detach_selected(&mut self) {
        if let Some(session) = self.selected_session() {
            self.action = Some(SessionAction::Detach(session.name.clone()));
            self.should_quit = true;
        }
    }

    // Sets action to detach all sessions and quits.
    pub fn detach_all(&mut self) {
        self.action = Some(SessionAction::DetachAll);
        self.should_quit = true;
    }

    // Checks if we're at the top of the session list (for navigation to agents).
    pub fn at_top_of_sessions(&self) -> bool {
        self.selected_index == 0
    }
}
