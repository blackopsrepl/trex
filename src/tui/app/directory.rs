use crate::directory::Directory;
use super::{App, AppMode};

impl App {
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
}
