use crate::process::{find_ai_processes, process_exists, read_process_state, AiProcessInfo};

use super::{App, AppMode, SessionAction};

impl App {
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

    // Checks if we're at the bottom of the agent list (for navigation to sessions).
    pub fn at_bottom_of_agents(&self) -> bool {
        let len = self.visible_agents().len();
        len == 0 || self.agent_selected_index >= len.saturating_sub(1)
    }
}
