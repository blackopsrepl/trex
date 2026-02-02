use crate::directory::Directory;
use crate::process::{find_ai_processes, process_exists, read_process_state, AiProcessInfo};
use crate::tmux::{TmuxClient, TmuxSession, TmuxWindow};

// The current mode of the application.
#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Filtering,
    SelectingDirectory,
    NamingSession,
    ExpandedSession,
}

// Which UI area has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum FocusArea {
    Agents,
    #[default]
    Sessions,
}

// An action to perform after exiting the TUI.
#[derive(Debug, Clone)]
pub enum SessionAction {
    Attach(String),
    AttachWindow(String, u32),
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

    // Session naming state
    pub session_name_input: String,
    pub selected_dir_path: Option<std::path::PathBuf>,

    // Window expansion state
    pub expanded_session: Option<String>,
    pub expanded_windows: Vec<TmuxWindow>,
    pub selected_window_index: usize,

    // Preview state
    pub show_preview: bool,
    pub preview_lines: Vec<String>,

    // AI process detection
    pub ai_processes: Vec<AiProcessInfo>,

    // Focus tracking for agent/session navigation
    pub focus: FocusArea,
    pub agent_selected_index: usize,
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

        let ai_processes = find_ai_processes().unwrap_or_default();

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
            session_name_input: String::new(),
            selected_dir_path: None,
            expanded_session: None,
            expanded_windows: Vec::new(),
            selected_window_index: 0,
            show_preview: false,
            preview_lines: Vec::new(),
            ai_processes,
            focus: FocusArea::default(),
            agent_selected_index: 0,
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

    // Enters naming mode for the selected directory.
    // Pre-fills the session name with the sanitized directory name.
    pub fn enter_naming_mode(&mut self) {
        if let Some(dir) = self.selected_directory() {
            let name = dir.session_name();
            let path = dir.path.clone();
            self.session_name_input = name;
            self.selected_dir_path = Some(path);
            self.mode = AppMode::NamingSession;
        }
    }

    // Confirms session creation with the current name input.
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

    // Cancels naming mode and returns to directory selection.
    pub fn cancel_naming(&mut self) {
        self.session_name_input.clear();
        self.selected_dir_path = None;
        self.mode = AppMode::SelectingDirectory;
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

    // Toggles the preview panel.
    pub fn toggle_preview(&mut self) {
        self.show_preview = !self.show_preview;
        if self.show_preview {
            self.refresh_preview();
        } else {
            self.preview_lines.clear();
        }
    }

    // Refreshes the preview for the currently selected session.
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

    // ===== AI Process / Agent Methods =====

    // Returns the list of visible agents based on current mode.
    // In ExpandedSession mode or with preview, filters to agents in the selected/expanded session.
    pub fn visible_agents(&self) -> Vec<&AiProcessInfo> {
        let filter_session = match &self.mode {
            AppMode::ExpandedSession => self.expanded_session.as_ref(),
            _ if self.show_preview => self.selected_session().map(|s| &s.name),
            _ => None,
        };

        match filter_session {
            Some(session_name) => self
                .ai_processes
                .iter()
                .filter(|p| p.tmux_session.as_ref() == Some(session_name))
                .collect(),
            None => self.ai_processes.iter().collect(),
        }
    }

    // Refreshes the activity state of all known AI processes (fast operation).
    pub fn refresh_ai_process_states(&mut self) {
        for proc in &mut self.ai_processes {
            if process_exists(proc.pid) {
                proc.activity_state = read_process_state(proc.pid);
            }
        }
    }

    // Rescans for AI processes (detects new/exited processes).
    pub fn rescan_ai_processes(&mut self) {
        if let Ok(new_processes) = find_ai_processes() {
            self.ai_processes = new_processes;
            // Ensure agent selection is still valid
            let visible_count = self.visible_agents().len();
            if self.agent_selected_index >= visible_count && visible_count > 0 {
                self.agent_selected_index = visible_count - 1;
            }
        }
    }

    // Moves agent selection to the next agent.
    pub fn select_agent_next(&mut self) {
        let len = self.visible_agents().len();
        if len > 0 {
            self.agent_selected_index = (self.agent_selected_index + 1).min(len - 1);
        }
    }

    // Moves agent selection to the previous agent.
    pub fn select_agent_previous(&mut self) {
        self.agent_selected_index = self.agent_selected_index.saturating_sub(1);
    }

    // Moves agent selection to the first agent.
    pub fn select_agent_first(&mut self) {
        self.agent_selected_index = 0;
    }

    // Moves agent selection to the last agent.
    pub fn select_agent_last(&mut self) {
        let len = self.visible_agents().len();
        if len > 0 {
            self.agent_selected_index = len - 1;
        }
    }

    // Returns the currently selected agent, if any.
    pub fn selected_agent(&self) -> Option<&AiProcessInfo> {
        self.visible_agents()
            .get(self.agent_selected_index)
            .copied()
    }

    // Attaches to the tmux session of the selected agent.
    pub fn attach_selected_agent(&mut self) {
        if let Some(agent) = self.selected_agent()
            && let Some(session_name) = &agent.tmux_session
            // Don't attach to placeholder "(tmux)" session
            && session_name != "(tmux)"
        {
            self.action = Some(SessionAction::Attach(session_name.clone()));
            self.should_quit = true;
        }
    }

    // Checks if we're at the top of the session list (for navigation to agents).
    pub fn at_top_of_sessions(&self) -> bool {
        self.selected_index == 0
    }

    // Checks if we're at the bottom of the agent list (for navigation to sessions).
    pub fn at_bottom_of_agents(&self) -> bool {
        let len = self.visible_agents().len();
        len == 0 || self.agent_selected_index >= len.saturating_sub(1)
    }
}
