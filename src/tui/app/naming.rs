use super::{App, AppMode, SessionAction};

impl App {
    /// Enters naming mode for the selected directory.
    /// Pre-fills the session name with the sanitized directory name.
    pub fn enter_naming_mode(&mut self) {
        if let Some(dir) = self.selected_directory() {
            let name = dir.session_name();
            let path = dir.path.clone();
            self.session_name_input = name;
            self.selected_dir_path = Some(path);
            self.mode = AppMode::NamingSession;
        }
    }

    /// Confirms session creation with the current name input.
    pub fn confirm_session_name(&mut self) {
        if let Some(path) = self.selected_dir_path.take() {
            let name = if self.session_name_input.is_empty() {
                "session".to_string()
            } else {
                crate::directory::sanitize_session_name(&self.session_name_input)
            };
            self.action = Some(SessionAction::Create(name, path));
            self.should_quit = true;
        }
    }

    /// Cancels naming mode and returns to directory selection.
    pub fn cancel_naming(&mut self) {
        self.session_name_input.clear();
        self.selected_dir_path = None;
        self.mode = AppMode::SelectingDirectory;
    }
}
