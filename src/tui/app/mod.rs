use crate::directory::Directory;
use crate::process::{AiProcessInfo, find_ai_processes};
use crate::theme::ThemeColors;
use crate::tmux::{TmuxSession, TmuxWindow};

// Submodules
mod agent;
mod directory;
mod filter;
mod naming;
mod preview;
mod session;
mod window;

// The current mode of the application.
#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Normal,
    Filtering,
    SelectingDirectory,
    NamingSession,
    ExpandedSession,
    BarChartView,
    StatsOverlay,
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

    // Theme colors
    pub theme: ThemeColors,

    // Tick counter for animations (incremented each render cycle)
    pub tick: u64,
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
        let theme = crate::theme::load_theme();

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
            theme,
            tick: 0,
        }
    }
}
