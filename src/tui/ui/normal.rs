use crate::process::ProcessState;
use crate::tmux::ActivityLevel;
use crate::tui::app::{App, AppMode, FocusArea};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, BorderType, Gauge, Paragraph, Sparkline,
    },
};

/// Braille-based vertical scrollbar characters
const SCROLL_TRACK: &str = "│";
const SCROLL_THUMB: &str = "┃";

/// Pulsing dot animation frames
const PULSE_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Smooth gradient: green (0%) → yellow (50%) → red (100%)
fn gradient_color(percent: f64) -> Color {
    let t = (percent / 100.0).clamp(0.0, 1.0);
    if t < 0.5 {
        // Green to Yellow
        let f = t * 2.0;
        Color::Rgb(
            (50.0 + 205.0 * f) as u8,
            (200.0 + 55.0 * (1.0 - f * 0.3)) as u8,
            (50.0 * (1.0 - f)) as u8,
        )
    } else {
        // Yellow to Red
        let f = (t - 0.5) * 2.0;
        Color::Rgb(
            255,
            (220.0 * (1.0 - f)) as u8,
            0,
        )
    }
}

pub fn render_normal_mode(frame: &mut Frame, app: &App) {
    let visible_agents = app.visible_agents();
    let agent_rows = if visible_agents.is_empty() {
        1
    } else {
        visible_agents.len().min(5)
    } as u16;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),              // System overview
            Constraint::Length(agent_rows + 2), // Agent box (content + borders)
            Constraint::Min(1),                 // Sessions
            Constraint::Length(2),              // Enhanced help
        ])
        .split(frame.area());

    render_system_overview(frame, app, chunks[0]);
    render_agent_box(frame, app, chunks[1]);

    // If preview is enabled, split the session area
    if app.show_preview {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[2]);
        render_session_list(frame, app, main_chunks[0]);
        render_preview(frame, app, main_chunks[1]);
    } else {
        render_session_list(frame, app, chunks[2]);
    }
    render_help(frame, app, chunks[3]);
}

pub fn render_system_overview(frame: &mut Frame, app: &App, area: Rect) {
    // Calculate totals across all sessions
    let mut total_cpu = 0.0;
    let mut total_mem_mb = 0u64;
    let mut active_sessions = 0;
    
    for session in &app.sessions {
        if let Some(ref stats) = session.stats {
            total_cpu += stats.cpu_percent;
            total_mem_mb += stats.mem_mb;
        }
        if session.attached {
            active_sessions += 1;
        }
    }

    // Pulsing spinner shows the app is alive and sampling
    let pulse = PULSE_FRAMES[(app.tick as usize / 2) % PULSE_FRAMES.len()];

    let overview_line = Line::from(vec![
        Span::styled(format!("{} ", pulse), Style::default().fg(app.theme.success)),
        Span::styled("TREX ", Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)),
        Span::styled("│ ", Style::default().fg(app.theme.text_dim)),
        Span::styled(format!("{} sessions", app.sessions.len()), Style::default().fg(app.theme.info)),
        Span::styled(" │ ", Style::default().fg(app.theme.text_dim)),
        Span::styled(format!("{} attached", active_sessions), Style::default().fg(app.theme.success)),
        Span::styled(" │ ", Style::default().fg(app.theme.text_dim)),
        Span::styled(format!("CPU: {:.1}%", total_cpu), Style::default().fg(
            if total_cpu > 300.0 { app.theme.error }
            else if total_cpu > 150.0 { app.theme.warning }
            else { app.theme.success }
        )),
        Span::styled(" │ ", Style::default().fg(app.theme.text_dim)),
        Span::styled(format!("MEM: {}MB", total_mem_mb), Style::default().fg(
            if total_mem_mb > 4096 { app.theme.error }
            else if total_mem_mb > 2048 { app.theme.warning }
            else { app.theme.info }
        )),
        Span::styled(" │ ", Style::default().fg(app.theme.text_dim)),
        Span::styled(format!("{} agents", app.ai_processes.len()), Style::default().fg(app.theme.secondary)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(app.theme.primary));

    let para = Paragraph::new(overview_line)
        .block(block)
        .style(Style::default().bg(app.theme.bg_primary));
    
    frame.render_widget(para, area);
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
        let more_paragraph = Paragraph::new(more_text).style(Style::default().fg(app.theme.text_dim));
        frame.render_widget(more_paragraph, more_area);
    }
}

pub fn render_session_list(frame: &mut Frame, app: &App, area: Rect) {
    let title = match app.mode {
        AppMode::Filtering => format!(
            " ⚡ Sessions ({}) > {} ",
            app.filtered_indices.len(),
            app.filter_input
        ),
        _ => format!(" ⚡ Sessions ({}) • ●=active ○=idle ◌=dormant ★=attached 🟢🟡🔴=health ", app.sessions.len()),
    };

    let border_color = if app.focus == FocusArea::Sessions {
        app.theme.info
    } else {
        app.theme.text_dim
    };

    let border_type = if app.focus == FocusArea::Sessions {
        BorderType::Double
    } else {
        BorderType::Rounded
    };

    if app.filtered_indices.is_empty() {
        let empty_msg = if app.sessions.is_empty() {
            "No tmux sessions found. Press 'c' to create one."
        } else {
            "No sessions match your filter"
        };
        let paragraph = Paragraph::new(empty_msg)
            .style(Style::default().fg(app.theme.text_dim))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(border_type)
                    .border_style(Style::default().fg(border_color))
                    .title(title),
            );
        frame.render_widget(paragraph, area);
        return;
    }

    // Each session takes 4 lines: header, CPU gauge, MEM gauge, sparklines
    const LINES_PER_SESSION: u16 = 5;
    
    // Calculate how many sessions we can show
    let inner_height = area.height.saturating_sub(2); // minus borders
    let max_visible = (inner_height / LINES_PER_SESSION) as usize;
    
    // Create scrollable window
    let start_idx = if app.selected_index >= max_visible {
        app.selected_index - max_visible + 1
    } else {
        0
    };
    let end_idx = (start_idx + max_visible).min(app.filtered_indices.len());

    // Scroll position indicator in title
    let scroll_info = if app.filtered_indices.len() > max_visible {
        format!(" [{}/{}]", app.selected_index + 1, app.filtered_indices.len())
    } else {
        String::new()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(border_color))
        .title(format!("{}{} ", title, scroll_info));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Render scrollbar on the right edge if content overflows
    if app.filtered_indices.len() > max_visible && inner.height > 0 {
        let total = app.filtered_indices.len();
        let track_h = inner.height as usize;
        let thumb_size = ((max_visible as f64 / total as f64) * track_h as f64)
            .ceil()
            .max(1.0) as usize;
        let thumb_pos = if total <= max_visible {
            0
        } else {
            ((start_idx as f64 / (total - max_visible) as f64) * (track_h - thumb_size) as f64) as usize
        };

        for row in 0..track_h {
            let ch = if row >= thumb_pos && row < thumb_pos + thumb_size {
                SCROLL_THUMB
            } else {
                SCROLL_TRACK
            };
            let color = if row >= thumb_pos && row < thumb_pos + thumb_size {
                app.theme.primary
            } else {
                app.theme.text_dim
            };
            let sb_area = Rect {
                x: inner.x + inner.width.saturating_sub(1),
                y: inner.y + row as u16,
                width: 1,
                height: 1,
            };
            frame.render_widget(
                Paragraph::new(ch).style(Style::default().fg(color)),
                sb_area,
            );
        }
    }

    let mut y_offset = 0;
    
    for display_idx in start_idx..end_idx {
        let &session_idx = &app.filtered_indices[display_idx];
        let session = &app.sessions[session_idx];
        let is_selected = app.focus == FocusArea::Sessions && display_idx == app.selected_index;

        // Session header line
        let (activity_icon, activity_color) = match session.activity_level() {
            Some(ActivityLevel::Active) => ("●", app.theme.success),
            Some(ActivityLevel::Idle) => ("○", app.theme.warning),
            Some(ActivityLevel::Dormant) => ("◌", app.theme.text_dim),
            None => ("○", app.theme.text_dim),
        };

        let attached_indicator = if session.attached { "★" } else { "☆" };
        let name_style = if is_selected {
            Style::default()
                .fg(app.theme.primary)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.text)
        };

        let activity_ago = session.activity_ago_string().unwrap_or_default();
        let git_badge = session
            .git_status
            .as_ref()
            .and_then(|gs| gs.badge())
            .unwrap_or_default();

        // Calculate health score
        let health = crate::health::HealthScore::calculate(session);
        let health_icon = health.icon();
        let health_color = match health.level() {
            crate::health::HealthLevel::Healthy => app.theme.success,
            crate::health::HealthLevel::Warning => app.theme.warning,
            crate::health::HealthLevel::Critical => app.theme.error,
        };

        let header_line = Line::from(vec![
            Span::styled(activity_icon, Style::default().fg(activity_color)),
            Span::raw(" "),
            Span::styled(attached_indicator, Style::default().fg(app.theme.warning)),
            Span::raw(" "),
            Span::styled(&session.name, name_style),
            Span::raw(" "),
            Span::styled(health_icon, Style::default().fg(health_color)),
            Span::styled(
                format!(" ({} win)", session.windows),
                Style::default().fg(app.theme.text_dim),
            ),
            Span::styled(format!(" {}", activity_ago), Style::default().fg(activity_color)),
            if !git_badge.is_empty() {
                Span::styled(format!(" {}", git_badge), Style::default().fg(app.theme.secondary))
            } else {
                Span::raw("")
            },
        ]);

        let header_area = Rect {
            x: inner.x,
            y: inner.y + y_offset,
            width: inner.width,
            height: 1,
        };
        
        let bg_style = if is_selected {
            Style::default().bg(app.theme.bg_highlight)
        } else {
            Style::default()
        };
        
        let header_para = Paragraph::new(header_line).style(bg_style);
        frame.render_widget(header_para, header_area);
        y_offset += 1;

        // CPU and Memory stats with gauges and sparklines
        if let Some(ref stats) = session.stats {
            // CPU Gauge with smooth gradient color
            let cpu_ratio = (stats.cpu_percent / 100.0).min(1.0);
            let cpu_color = gradient_color(stats.cpu_percent);

            let cpu_gauge = Gauge::default()
                .block(Block::default())
                .gauge_style(Style::default().fg(cpu_color))
                .label(format!("CPU {:5.1}%", stats.cpu_percent))
                .ratio(cpu_ratio);

            let cpu_area = Rect {
                x: inner.x,
                y: inner.y + y_offset,
                width: inner.width / 2,
                height: 1,
            };
            frame.render_widget(cpu_gauge, cpu_area);

            // Memory Gauge with smooth gradient color
            let mem_ratio = (stats.mem_percent / 100.0).min(1.0);
            let mem_color = gradient_color(stats.mem_percent);

            let mem_gauge = Gauge::default()
                .block(Block::default())
                .gauge_style(Style::default().fg(mem_color))
                .label(format!("MEM {:4}MB", stats.mem_mb))
                .ratio(mem_ratio);

            let mem_area = Rect {
                x: inner.x + inner.width / 2,
                y: inner.y + y_offset,
                width: inner.width / 2,
                height: 1,
            };
            frame.render_widget(mem_gauge, mem_area);
            y_offset += 1;

            // CPU Sparkline
            if !session.cpu_history.is_empty() {
                let cpu_sparkline = Sparkline::default()
                    .block(Block::default())
                    .data(&session.cpu_history)
                    .style(Style::default().fg(cpu_color));

                let cpu_spark_area = Rect {
                    x: inner.x,
                    y: inner.y + y_offset,
                    width: inner.width / 2,
                    height: 1,
                };
                frame.render_widget(cpu_sparkline, cpu_spark_area);
            }

            // Memory Sparkline
            if !session.mem_history.is_empty() {
                let mem_sparkline = Sparkline::default()
                    .block(Block::default())
                    .data(&session.mem_history)
                    .style(Style::default().fg(mem_color));

                let mem_spark_area = Rect {
                    x: inner.x + inner.width / 2,
                    y: inner.y + y_offset,
                    width: inner.width / 2,
                    height: 1,
                };
                frame.render_widget(mem_sparkline, mem_spark_area);
            }
            y_offset += 1;
        } else {
            // No stats available yet
            let waiting_line = Line::from(vec![
                Span::styled("  ⏳ Gathering metrics...", Style::default().fg(app.theme.text_dim)),
            ]);
            let waiting_area = Rect {
                x: inner.x,
                y: inner.y + y_offset,
                width: inner.width,
                height: 1,
            };
            frame.render_widget(Paragraph::new(waiting_line), waiting_area);
            y_offset += 1;
        }

        // Add spacing between sessions
        y_offset += 1;
    }
}

pub fn render_preview(frame: &mut Frame, app: &App, area: Rect) {
    let session_name = app
        .selected_session()
        .map(|s| s.name.as_str())
        .unwrap_or("No session");

    let title = format!(" Preview: {} ", session_name);

    let content = if app.preview_lines.is_empty() {
        vec![Line::from(Span::styled(
            "No content to preview",
            Style::default().fg(app.theme.text_dim),
        ))]
    } else {
        app.preview_lines
            .iter()
            .map(|line| Line::from(Span::raw(line.as_str())))
            .collect()
    };

    let paragraph = Paragraph::new(content)
        .style(Style::default().fg(app.theme.text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.info))
                .title(title),
        );

    frame.render_widget(paragraph, area);
}

pub fn render_help(frame: &mut Frame, app: &App, area: Rect) {
    let help_items: Vec<(&str, &str)> = match app.mode {
        AppMode::Normal => vec![
            ("j/k", "nav"), ("l", "expand"), ("p", "preview"),
            ("b", "charts"), ("s", "stats"),
            ("↵", "attach"), ("c", "create"), ("d", "delete"),
            ("/", "filter"), ("q", "quit"),
        ],
        AppMode::Filtering => vec![
            ("type", "filter"), ("↵", "attach"), ("Esc", "clear"), ("Tab", "nav"),
        ],
        AppMode::SelectingDirectory | AppMode::NamingSession => vec![
            ("type", "filter"), ("Tab", "complete"), ("+/-", "depth"),
            ("↵", "name"), ("Esc", "cancel"),
        ],
        AppMode::ExpandedSession => vec![
            ("j/k", "nav"), ("↵", "attach"), ("h/Esc", "back"), ("q", "quit"),
        ],
        AppMode::BarChartView => vec![
            ("b/Esc", "back"), ("q", "quit"),
        ],
        AppMode::StatsOverlay => vec![
            ("s/Esc", "close"), ("q", "quit"),
        ],
    };

    let mut spans = Vec::new();
    
    for (i, (key, action)) in help_items.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" │ ", Style::default().fg(app.theme.text_dim)));
        }
        spans.push(Span::styled(*key, Style::default().fg(app.theme.primary).add_modifier(Modifier::BOLD)));
        spans.push(Span::styled(format!(" {}", action), Style::default().fg(app.theme.text)));
    }

    let help_line = Line::from(spans);
    
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(app.theme.text_dim));

    let para = Paragraph::new(help_line)
        .block(block)
        .style(Style::default().bg(app.theme.bg_primary));

    frame.render_widget(para, area);
}
