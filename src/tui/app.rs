use crate::directory::Directory;
use crate::tmux::TmuxSession;

// The current mode of the application.
#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Filtering,
    SelectingDirectory,
}

// An action to perform after exiting the TUI.
#[derive(Debug, Clone)]
pub enum SessionAction {
    Attach(String),
    Create(String, std::path::PathBuf),
    Delete(String),
    DeleteAll,
    Detach(String),
    DetachAll,
}

// Application state for the TUI.
pub struct App {
    pub sessions: Vec<TmuxSession>,
    pub filtered_indices: Vec<usize>,
    pub selected_index: usize,
    pub filter_input: String,
    pub mode: AppMode,
    pub should_quit: bool,
    pub action: Option<SessionAction>,

    pub directories: Vec<Directory>,
    pub dir_filtered_indices: Vec<usize>,
    pub dir_selected_index: usize,
    pub dir_filter_input: String,
    pub dir_scan_depth: u32,
}

impl App {
    // Creates a new app with a preselected session index.
    pub fn with_preselection(sessions: Vec<TmuxSession>, preselect_index: usize) -> Self {
        let filtered_indices: Vec<usize> = (0..sessions.len()).collect();
        let selected_index = if preselect_index < sessions.len() {
            preselect_index
        } else {
            0
        };

        let dir_scan_depth = crate::directory::DEFAULT_DEPTH;
        let directories = crate::directory::discover_directories_with_depth(dir_scan_depth);
        let dir_filtered_indices: Vec<usize> = (0..directories.len()).collect();

        Self {
            sessions,
            filtered_indices,
            selected_index,
            filter_input: String::new(),
            mode: AppMode::Normal,
            should_quit: false,
            action: None,
            directories,
            dir_filtered_indices,
            dir_selected_index: 0,
            dir_filter_input: String::new(),
            dir_scan_depth,
        }
    }

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

    // Moves selection to the next directory (wraps around).
    pub fn select_dir_next(&mut self) {
        if !self.dir_filtered_indices.is_empty() {
            self.dir_selected_index =
                (self.dir_selected_index + 1) % self.dir_filtered_indices.len();
        }
    }

    // Moves selection to the previous directory (wraps around).
    pub fn select_dir_previous(&mut self) {
        if !self.dir_filtered_indices.is_empty() {
            self.dir_selected_index = if self.dir_selected_index == 0 {
                self.dir_filtered_indices.len() - 1
            } else {
                self.dir_selected_index - 1
            };
        }
    }

    // Moves selection to the first directory.
    pub fn select_dir_first(&mut self) {
        self.dir_selected_index = 0;
    }

    // Moves selection to the last directory.
    pub fn select_dir_last(&mut self) {
        if !self.dir_filtered_indices.is_empty() {
            self.dir_selected_index = self.dir_filtered_indices.len() - 1;
        }
    }

    // Returns the currently selected directory, if any.
    pub fn selected_directory(&self) -> Option<&Directory> {
        self.dir_filtered_indices
            .get(self.dir_selected_index)
            .and_then(|&idx| self.directories.get(idx))
    }

    // Sets action to create a session in the selected directory and quits.
    pub fn create_session_in_directory(&mut self) {
        if let Some(dir) = self.selected_directory() {
            let name = dir.session_name();
            self.action = Some(SessionAction::Create(name, dir.path.clone()));
            self.should_quit = true;
        }
    }

    // Applies fuzzy filtering to the directory list based on current input.
    pub fn apply_dir_filter(&mut self, matcher: &mut nucleo::Matcher) {
        if self.dir_filter_input.is_empty() {
            self.dir_filtered_indices = (0..self.directories.len()).collect();
        } else {
            use nucleo::pattern::{CaseMatching, Normalization, Pattern};

            let pattern = Pattern::parse(
                &self.dir_filter_input,
                CaseMatching::Smart,
                Normalization::Smart,
            );

            let mut results: Vec<(usize, u32)> = self
                .directories
                .iter()
                .enumerate()
                .filter_map(|(idx, dir)| {
                    let haystack = dir.match_string();
                    let mut buf = Vec::new();
                    let haystack_utf32 = nucleo::Utf32Str::new(&haystack, &mut buf);
                    pattern
                        .score(haystack_utf32, matcher)
                        .map(|score| (idx, score))
                })
                .collect();

            results.sort_by(|a, b| b.1.cmp(&a.1));
            self.dir_filtered_indices = results.into_iter().map(|(idx, _)| idx).collect();
        }

        self.dir_selected_index = 0;
    }

    // Clears the directory filter and returns to normal mode.
    pub fn clear_dir_filter(&mut self, matcher: &mut nucleo::Matcher) {
        self.dir_filter_input.clear();
        self.apply_dir_filter(matcher);
        self.mode = AppMode::Normal;
    }

    // Increases the directory scan depth and refreshes the list.
    pub fn increase_depth(&mut self, matcher: &mut nucleo::Matcher) {
        if self.dir_scan_depth < crate::directory::MAX_DEPTH {
            self.dir_scan_depth += 1;
            self.refresh_directories(matcher);
        }
    }

    // Decreases the directory scan depth and refreshes the list.
    pub fn decrease_depth(&mut self, matcher: &mut nucleo::Matcher) {
        if self.dir_scan_depth > crate::directory::MIN_DEPTH {
            self.dir_scan_depth -= 1;
            self.refresh_directories(matcher);
        }
    }

    // Refreshes the directory list with the current scan depth.
    fn refresh_directories(&mut self, matcher: &mut nucleo::Matcher) {
        self.directories = crate::directory::discover_directories_with_depth(self.dir_scan_depth);
        self.dir_filtered_indices = (0..self.directories.len()).collect();
        self.dir_selected_index = 0;
        if !self.dir_filter_input.is_empty() {
            self.apply_dir_filter(matcher);
        }
    }

    // Sets the directory filter input to the selected directory's path.
    pub fn tab_complete_directory(&mut self) {
        if let Some(dir) = self.selected_directory() {
            self.dir_filter_input = dir.path.display().to_string();
        }
    }

    // Applies fuzzy filtering to the session list based on current input.
    pub fn apply_filter(&mut self, matcher: &mut nucleo::Matcher) {
        if self.filter_input.is_empty() {
            self.filtered_indices = (0..self.sessions.len()).collect();
        } else {
            use nucleo::pattern::{CaseMatching, Normalization, Pattern};

            let pattern = Pattern::parse(
                &self.filter_input,
                CaseMatching::Smart,
                Normalization::Smart,
            );

            let mut results: Vec<(usize, u32)> = self
                .sessions
                .iter()
                .enumerate()
                .filter_map(|(idx, session)| {
                    let haystack = session.match_string();
                    let mut buf = Vec::new();
                    let haystack_utf32 = nucleo::Utf32Str::new(&haystack, &mut buf);
                    pattern
                        .score(haystack_utf32, matcher)
                        .map(|score| (idx, score))
                })
                .collect();

            results.sort_by(|a, b| b.1.cmp(&a.1));
            self.filtered_indices = results.into_iter().map(|(idx, _)| idx).collect();
        }

        self.selected_index = 0;
    }

    // Clears the session filter and returns to normal mode.
    pub fn clear_filter(&mut self, matcher: &mut nucleo::Matcher) {
        self.filter_input.clear();
        self.apply_filter(matcher);
        self.mode = AppMode::Normal;
    }
}
