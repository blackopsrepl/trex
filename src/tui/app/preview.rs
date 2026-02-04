use crate::tmux::TmuxClient;

use super::App;

impl App {
    pub fn toggle_preview(&mut self) {
        self.show_preview = !self.show_preview;
        if self.show_preview {
            self.refresh_preview();
        } else {
            self.preview_lines.clear();
        }
    }

    pub fn refresh_preview(&mut self) {
        if !self.show_preview {
            return;
        }
        if let Some(session) = self.selected_session() {
            if let Ok(lines) = TmuxClient::capture_pane(&session.name, 30) {
                self.preview_lines = lines;
            } else {
                self.preview_lines.clear();
            }
        } else {
            self.preview_lines.clear();
        }
    }
}
