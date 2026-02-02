use super::{App, AppMode};

impl App {
    /// Applies fuzzy filtering to the session list based on current input.
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

    /// Clears the session filter and returns to normal mode.
    pub fn clear_filter(&mut self, matcher: &mut nucleo::Matcher) {
        self.filter_input.clear();
        self.apply_filter(matcher);
        self.mode = AppMode::Normal;
    }
}
