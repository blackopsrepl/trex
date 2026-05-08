use super::{App, AppMode, SessionAction};
use crate::template::SessionTemplate;

impl App {
    // Pre-fills the session name with the sanitized directory name.
    pub fn enter_naming_mode(&mut self) {
        if let Some(dir) = self.selected_directory() {
            let name = dir.session_name();
            let path = dir.path.clone();
            self.session_name_input = name;
            self.selected_dir_path = Some(path);
            self.selected_template_index = 0;
            self.mode = AppMode::NamingSession;
        }
    }

    pub fn selected_template(&self) -> Option<&SessionTemplate> {
        self.templates
            .get(self.selected_template_index)
            .or_else(|| self.templates.first())
    }

    pub fn select_next_template(&mut self) {
        if !self.templates.is_empty() {
            self.selected_template_index =
                (self.selected_template_index + 1) % self.templates.len();
        }
    }

    pub fn select_previous_template(&mut self) {
        if !self.templates.is_empty() {
            self.selected_template_index = if self.selected_template_index == 0 {
                self.templates.len() - 1
            } else {
                self.selected_template_index - 1
            };
        }
    }

    pub fn confirm_session_name(&mut self) {
        if let (Some(template), Some(path)) = (
            self.selected_template().cloned(),
            self.selected_dir_path.take(),
        ) {
            let name = if self.session_name_input.is_empty() {
                "session".to_string()
            } else {
                crate::directory::sanitize_session_name(&self.session_name_input)
            };
            self.action = Some(SessionAction::Create {
                name,
                path,
                template,
            });
            self.should_quit = true;
        }
    }

    pub fn cancel_naming(&mut self) {
        self.session_name_input.clear();
        self.selected_dir_path = None;
        self.mode = AppMode::SelectingDirectory;
    }
}
