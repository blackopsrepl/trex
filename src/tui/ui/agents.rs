use crate::process::ProcessState;
use crate::tui::app::{App, AppMode, FocusArea};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Paragraph},
};

pub fn render_agent_box(frame: &mut Frame, app: &App, area: Rect) {
    let visible_agents = app.visible_agents();

    // Determine title based on mode
    let title = match &app.mode {
        AppMode::ExpandedSession => {
            let session_name = app.expanded_session.as_deref().unwrap_or("session");
            format!(" AGENTS IN: {} ", session_name)
        }
        _ if app.show_preview => {
            if let Some(session) = app.selected_session() {
                format!(" AGENTS IN: {} ", session.name)
            } else {
                " RUNNING AGENTS ".to_string()
            }
        }
        _ => " RUNNING AGENTS ".to_string(),
    };

    let border_color = if app.focus == FocusArea::Agents {
        app.theme.secondary
    } else {
        app.theme.text_dim
    };

    let border_type = if app.focus == FocusArea::Agents {
        BorderType::Double
    } else {
        BorderType::Rounded
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(border_color))
        .title(title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Handle empty state
    if visible_agents.is_empty() {
        let empty_text = Paragraph::new(" No agents running").style(
            Style::default()
                .fg(app.theme.text_dim)
                .add_modifier(Modifier::ITALIC),
        );
        frame.render_widget(empty_text, inner);
        return;
    }

    const COL_WIDTH: usize = 38;
    const MAX_ROWS: usize = 5;

    let process_count = visible_agents.len();
    let num_cols = (inner.width as usize / COL_WIDTH).max(1);
    let max_display = num_cols * MAX_ROWS;
    let display_count = process_count.min(max_display);
    let num_rows = display_count.min(MAX_ROWS);
    let show_more = process_count > max_display;

    // Build lines with dynamic columns (column-first: fill top-to-bottom, then left-to-right)
    let mut lines: Vec<Line> = Vec::new();
    for row in 0..num_rows {
        let mut spans = Vec::new();

        for col in 0..num_cols {
            let idx = col * MAX_ROWS + row;
            if idx < display_count {
                let proc = visible_agents[idx];
                let is_selected = app.focus == FocusArea::Agents && idx == app.agent_selected_index;

                // Activity indicator based on process state
                let (activity_icon, activity_color) = match proc.activity_state {
                    ProcessState::Running => ("▶", app.theme.success),
                    ProcessState::Waiting => ("⏸", app.theme.warning),
                    ProcessState::Unknown => ("◼", app.theme.text_dim),
                };

                // Tmux indicator
                let tmux_icon = if proc.tmux_session.is_some() {
                    "●"
                } else {
                    "○"
                };

                // Project name (truncated)
                let display_name = if proc.project_name.len() > 12 {
                    format!("{}...", &proc.project_name[..12])
                } else {
                    proc.project_name.clone()
                };

                // Text color: theme primary when selected, otherwise dimmed
                let text_color = if is_selected {
                    app.theme.primary
                } else {
                    app.theme.text_dim
                };

                let text_style = if is_selected {
                    Style::default().fg(text_color).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(text_color)
                };

                // For activity icon, we need separate spans to color it
                spans.push(Span::styled(" ", text_style));
                spans.push(Span::styled(
                    activity_icon,
                    Style::default().fg(activity_color),
                ));

                // Build the main display text with optional child AI names
                let main_text = if proc.child_ai_names.is_empty() {
                    format!(" {}:{} {}", proc.process_name, display_name, tmux_icon)
                } else {
                    let child_names = proc.child_ai_names.join(", ");
                    format!(
                        " {}:{} {} ({})",
                        proc.process_name, display_name, tmux_icon, child_names
                    )
                };

                spans.push(Span::styled(main_text.clone(), text_style));

                // Add padding to reach column width
                let current_len = 1 + activity_icon.chars().count() + main_text.chars().count();
                if current_len < COL_WIDTH {
                    spans.push(Span::raw(" ".repeat(COL_WIDTH - current_len)));
                }
            }
        }

        if !spans.is_empty() {
            lines.push(Line::from(spans));
        }
    }

    let ai_paragraph = Paragraph::new(lines);
    frame.render_widget(ai_paragraph, inner);

    // Render "+N more" at bottom-right only if processes don't fit
    if show_more {
        let extra = process_count - max_display;
        let more_text = format!("+{} more ", extra);
        let more_width = more_text.len() as u16;
        let more_area = Rect {
            x: inner.x + inner.width.saturating_sub(more_width),
            y: inner.y + inner.height.saturating_sub(1),
            width: more_width,
            height: 1,
        };
        let more_paragraph =
            Paragraph::new(more_text).style(Style::default().fg(app.theme.text_dim));
        frame.render_widget(more_paragraph, more_area);
    }
}
