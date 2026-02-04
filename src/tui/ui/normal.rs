use crate::process::ProcessState;
use crate::tmux::ActivityLevel;
use crate::tui::app::{App, AppMode, FocusArea};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use super::constants::EMERALD_GREEN;

pub fn render_normal_mode(frame: &mut Frame, app: &App) {
    let visible_agents = app.visible_agents();
    let agent_rows = if visible_agents.is_empty() { 1 } else { visible_agents.len().min(5) } as u16;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(agent_rows + 2), // Agent box (content + borders)
            Constraint::Min(1),                 // Sessions
            Constraint::Length(1),              // Help
        ])
        .split(frame.area());

    render_agent_box(frame, app, chunks[0]);

    // If preview is enabled, split the session area
    if app.show_preview {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);
        render_session_list(frame, app, main_chunks[0]);
        render_preview(frame, app, main_chunks[1]);
    } else {
        render_session_list(frame, app, chunks[1]);
    }
    render_help(frame, app, chunks[2]);
}

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
        Color::Magenta
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(title);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Handle empty state
    if visible_agents.is_empty() {
        let empty_text = Paragraph::new(" No agents running")
            .style(Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC));
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
                    ProcessState::Running => ("▶", Color::Green),
                    ProcessState::Waiting => ("⏸", Color::Yellow),
                    ProcessState::Unknown => ("◼", Color::DarkGray),
                };

                // Tmux indicator
                let tmux_icon = if proc.tmux_session.is_some() { "●" } else { "○" };

                // Project name (truncated)
                let display_name = if proc.project_name.len() > 12 {
                    format!("{}...", &proc.project_name[..12])
                } else {
                    proc.project_name.clone()
                };

                // Text color: emerald green when selected, otherwise based on activity
                let text_color = if is_selected {
                    EMERALD_GREEN
                } else {
                    Color::DarkGray
                };

                let text_style = if is_selected {
                    Style::default().fg(text_color).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(text_color)
                };

                // For activity icon, we need separate spans to color it
                spans.push(Span::styled(" ", text_style));
                spans.push(Span::styled(activity_icon, Style::default().fg(activity_color)));
                spans.push(Span::styled(
                    format!(" {}:{} {}", proc.process_name, display_name, tmux_icon),
                    text_style,
                ));

                // Add padding to reach column width
                let current_len = 1 + activity_icon.chars().count() + 1 + proc.process_name.len() + 1 + display_name.len() + 1 + tmux_icon.chars().count();
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
            Paragraph::new(more_text).style(Style::default().fg(Color::DarkGray));
        frame.render_widget(more_paragraph, more_area);
    }
}

// Renders the session list with selection highlighting.
pub fn render_session_list(frame: &mut Frame, app: &App, area: Rect) {
    // Title includes session count and filter input if filtering
    let title = match app.mode {
        AppMode::Filtering => format!(" Sessions ({}) > {} ", app.filtered_indices.len(), app.filter_input),
        _ => format!(" Sessions ({}) ", app.sessions.len()),
    };

    let border_color = if app.focus == FocusArea::Sessions {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    if app.filtered_indices.is_empty() {
        let empty_msg = if app.sessions.is_empty() {
            "No tmux sessions found. Press 'c' to create one."
        } else {
            "No sessions match your filter"
        };
        let paragraph = Paragraph::new(empty_msg)
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .title(title),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = app
        .filtered_indices
        .iter()
        .enumerate()
        .map(|(idx, &session_idx)| {
            let session = &app.sessions[session_idx];
            let is_selected = app.focus == FocusArea::Sessions && idx == app.selected_index;

            // Activity indicator with color based on activity level
            let (activity_icon, activity_color) = match session.activity_level() {
                Some(ActivityLevel::Active) => ("●", Color::Green),
                Some(ActivityLevel::Idle) => ("○", Color::Yellow),
                Some(ActivityLevel::Dormant) => ("◌", Color::DarkGray),
                None => ("○", Color::DarkGray),
            };

            let attached_indicator = if session.attached { "*" } else { " " };

            let name_style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let activity_ago = session
                .activity_ago_string()
                .map(|s| format!(" {}", s))
                .unwrap_or_default();

            let git_badge = session
                .git_status
                .as_ref()
                .and_then(|gs| gs.badge())
                .map(|b| format!("  {}", b))
                .unwrap_or_default();

            let path_display = session
                .path
                .as_ref()
                .map(|p| {
                    let path_str = p.display().to_string();
                    if let Ok(home) = std::env::var("HOME")
                        && path_str.starts_with(&home)
                    {
                        return format!("  ~{}", &path_str[home.len()..]);
                    }
                    format!("  {}", path_str)
                })
                .unwrap_or_default();

            let line = Line::from(vec![
                Span::styled(activity_icon, Style::default().fg(activity_color)),
                Span::styled(
                    attached_indicator,
                    if session.attached {
                        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::DarkGray)
                    },
                ),
                Span::raw(" "),
                Span::styled(&session.name, name_style),
                Span::styled(
                    format!(" ({} win)", session.windows),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(activity_ago, Style::default().fg(activity_color)),
                Span::styled(git_badge, Style::default().fg(Color::Magenta)),
                Span::styled(path_display, Style::default().fg(Color::Cyan)),
            ]);

            let item_style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            ListItem::new(line).style(item_style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .title(title),
    );

    frame.render_widget(list, area);
}

// Renders the preview panel showing captured pane content.
pub fn render_preview(frame: &mut Frame, app: &App, area: Rect) {
    let session_name = app
        .selected_session()
        .map(|s| s.name.as_str())
        .unwrap_or("No session");

    let title = format!(" Preview: {} ", session_name);

    let content = if app.preview_lines.is_empty() {
        vec![Line::from(Span::styled(
            "No content to preview",
            Style::default().fg(Color::DarkGray),
        ))]
    } else {
        app.preview_lines
            .iter()
            .map(|line| Line::from(Span::raw(line.as_str())))
            .collect()
    };

    let paragraph = Paragraph::new(content)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue))
                .title(title),
        );

    frame.render_widget(paragraph, area);
}

// Renders the help line showing available keybindings.
pub fn render_help(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.mode {
        AppMode::Normal => {
            "j/k: nav | l: expand | p: preview | Enter: attach | c: create | d/D: delete | x/X: detach | /: filter | q: quit"
        }
        AppMode::Filtering => "Type to filter | Enter: attach | Esc: clear | Tab: navigate",
        AppMode::SelectingDirectory | AppMode::NamingSession => {
            "Type: filter | Tab: complete | +/-: depth | Enter: name session | Esc: cancel"
        }
        AppMode::ExpandedSession => {
            "j/k: navigate | Enter: attach window | h/Esc: collapse | q: quit"
        }
    };

    let paragraph = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));

    frame.render_widget(paragraph, area);
}
