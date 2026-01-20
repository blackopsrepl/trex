use crate::tmux::ActivityLevel;
use crate::tui::app::{App, AppMode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

// The majestic T-Rex king, holding his tmux windows
const TREX_ASCII: &str = r#"                  \/
              \  _||_  /
               \/*||*\/
             .-=-==--==--.
       ..-=="  ,'o`)      `.
     ,'         `"'         \
    :  (                     `.__...._
    |                  )    /         `-=-.
    :       ,vv.-._   /    /               `---==-._
     \/\/\/VV ^ d88`;'    /                         `.
         ``  ^/d88P!'    /             ,              `._
            ^/    !'   ,.      ,      /                  "-,,__,,--'""""-.
           ^/    !'  ,'  \ . .(      (         _           )  ) ) ) ))_,-.\
          ^(__ ,!',"'   ;:+.:%:a.     \:.. . ,'          )  )  ) ) ,"'    '
          ',,,'','     /o:::":%:%a.    \:.:.:         .    )  ) _,'
           """'       ;':::'' `+%%%a._  \%:%|         ;.). _,-""
                  ,-='_.-'      ``:%::)  )%:|        /:._,"
                 (/(/"        .-----. ,'%%%:       (_,'
                             |  $   | ___;        \
                             | vim  |\   `         `
                              `-----' `.   `.        :
                        .-----.  \. . .\    : . . . :
                       | htop  \  \. . .:    `.. . .:
                       |  $$   |   `..:.:\     \:...\
                        `-----'     ;:.:.;      ::...:
                                    ):%::       :::::;
                                __,::%:(        :::::
                             ,;:%%%%%%%:        ;:%::
                               ;,--""-.`\  ,=--':%:%:\
                              /"       "| /-".:%%%%%%%\
                                              ;,-"'`)%%)
                                             /"      "|"#;

// The eye is on line 4 (0-indexed), character 'o'
const EYE_LINE: usize = 4;
const EYE_CHAR: char = 'o';

// Green rainbow gradient - from dark to lime to bright (lolcat-style)
const GREEN_GRADIENT: [(u8, u8, u8); 8] = [
    (0, 60, 20),    // Dark forest
    (0, 90, 30),    // Deep green
    (0, 120, 40),   // Forest green
    (20, 150, 50),  // Green
    (40, 180, 60),  // Bright green
    (80, 200, 80),  // Lime
    (40, 180, 60),  // Bright green (back down)
    (20, 150, 50),  // Green
];

// Renders the entire TUI based on the current app state.
pub fn render(frame: &mut Frame, app: &App) {
    // Render decorative T-Rex background first (behind everything)
    render_background_trex(frame, frame.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    match app.mode {
        AppMode::SelectingDirectory => {
            render_header_dir(frame, app, chunks[0]);
            render_directory_list(frame, app, chunks[1]);
            render_help_dir(frame, chunks[2]);
        }
        AppMode::NamingSession => {
            render_header_naming(frame, app, chunks[0]);
            render_naming_preview(frame, app, chunks[1]);
            render_help_naming(frame, chunks[2]);
        }
        AppMode::ExpandedSession => {
            render_header_expanded(frame, app, chunks[0]);
            render_window_list(frame, app, chunks[1]);
            render_help_expanded(frame, chunks[2]);
        }
        _ => {
            render_header(frame, app, chunks[0]);
            // If preview is enabled, split the main area
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
    }
}

// Renders the header bar showing mode and filter input.
fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let preview_indicator = if app.show_preview { " [preview] " } else { "" };
    let (title, style) = match app.mode {
        AppMode::Normal => (
            format!(" trex - {} sessions{}", app.sessions.len(), preview_indicator),
            Style::default().fg(Color::Cyan),
        ),
        AppMode::Filtering => (
            format!(" > {}{}", app.filter_input, preview_indicator),
            Style::default().fg(Color::Yellow),
        ),
        AppMode::SelectingDirectory | AppMode::NamingSession => (
            format!(" Select directory > {} ", app.dir_filter_input),
            Style::default().fg(Color::Green),
        ),
        AppMode::ExpandedSession => (
            format!(
                " {} - {} windows ",
                app.expanded_session.as_deref().unwrap_or(""),
                app.expanded_windows.len()
            ),
            Style::default().fg(Color::Blue),
        ),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(style)
        .title(title);

    frame.render_widget(block, area);
}

// Renders the session list with selection highlighting.
fn render_session_list(frame: &mut Frame, app: &App, area: Rect) {
    if app.filtered_indices.is_empty() {
        let empty_msg = if app.sessions.is_empty() {
            "No tmux sessions found. Create one with: tmux new -s <name>"
        } else {
            "No sessions match your filter"
        };
        let paragraph = Paragraph::new(empty_msg)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = app
        .filtered_indices
        .iter()
        .enumerate()
        .map(|(idx, &session_idx)| {
            let session = &app.sessions[session_idx];
            let is_selected = idx == app.selected_index;

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

    let list = List::new(items).block(Block::default().borders(Borders::ALL));

    frame.render_widget(list, area);
}

// Renders the help line showing available keybindings.
fn render_help(frame: &mut Frame, app: &App, area: Rect) {
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

// Renders the header bar for directory selection mode.
fn render_header_dir(frame: &mut Frame, app: &App, area: Rect) {
    let title = format!(
        " Select directory (depth: {}) > {} ",
        app.dir_scan_depth, app.dir_filter_input
    );
    let dir_count = format!(" {} dirs ", app.dir_filtered_indices.len());
    let style = Style::default().fg(Color::Green);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(style)
        .title(title)
        .title_bottom(dir_count);

    frame.render_widget(block, area);
}

// Renders the directory list with selection highlighting.
fn render_directory_list(frame: &mut Frame, app: &App, area: Rect) {
    if app.dir_filtered_indices.is_empty() {
        let paragraph = Paragraph::new("No directories found")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = app
        .dir_filtered_indices
        .iter()
        .enumerate()
        .map(|(idx, &dir_idx)| {
            let dir = &app.directories[dir_idx];
            let is_selected = idx == app.dir_selected_index;

            let path_str = dir.path.display().to_string();
            let display_name = dir
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path_str.clone());

            let name_style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let item_style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::styled(display_name, name_style),
                Span::styled(format!(" [{}]", path_str), Style::default().fg(Color::Cyan)),
            ]);

            ListItem::new(line).style(item_style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL));

    frame.render_widget(list, area);
}

// Renders the help line for directory selection mode.
fn render_help_dir(frame: &mut Frame, area: Rect) {
    let help_text = "Type: filter | Tab: complete | +/-: depth | Enter: name session | Esc: cancel";
    let paragraph = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));

    frame.render_widget(paragraph, area);
}

// Renders the header for session naming mode.
fn render_header_naming(frame: &mut Frame, app: &App, area: Rect) {
    let cursor = "_";
    let title = Line::from(vec![
        Span::styled(" Name session ", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
        Span::styled("> ", Style::default().fg(Color::Magenta)),
        Span::styled(&app.session_name_input, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        Span::styled(cursor, Style::default().fg(Color::Magenta).add_modifier(Modifier::RAPID_BLINK)),
        Span::raw(" "),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Magenta))
        .title(title);

    frame.render_widget(block, area);
}

// Renders the preview panel for session naming mode.
fn render_naming_preview(frame: &mut Frame, app: &App, area: Rect) {
    let path_display = app
        .selected_dir_path
        .as_ref()
        .map(|p| {
            let path_str = p.display().to_string();
            if let Ok(home) = std::env::var("HOME")
                && path_str.starts_with(&home)
            {
                return format!("~{}", &path_str[home.len()..]);
            }
            path_str
        })
        .unwrap_or_default();

    let sanitized_name = if app.session_name_input.is_empty() {
        "session".to_string()
    } else {
        crate::directory::sanitize_session_name(&app.session_name_input)
    };

    let name_changed = app.session_name_input != sanitized_name;

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("    Directory  ", Style::default().fg(Color::DarkGray)),
            Span::styled(&path_display, Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    Session    ", Style::default().fg(Color::DarkGray)),
            Span::styled(&sanitized_name, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            if name_changed {
                Span::styled(" (sanitized)", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC))
            } else {
                Span::raw("")
            },
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled("Enter", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(" to create  ", Style::default().fg(Color::DarkGray)),
            Span::styled("Esc", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
            Span::styled(" to go back", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(Block::default().borders(Borders::ALL));

    frame.render_widget(paragraph, area);
}

// Renders the help line for session naming mode.
fn render_help_naming(frame: &mut Frame, area: Rect) {
    let help_text = "Type session name | Enter: create | Esc: back to directories";
    let paragraph = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));

    frame.render_widget(paragraph, area);
}

// Renders the header for expanded session mode.
fn render_header_expanded(frame: &mut Frame, app: &App, area: Rect) {
    let session_name = app.expanded_session.as_deref().unwrap_or("Unknown");
    let title = format!(" {} - {} windows ", session_name, app.expanded_windows.len());
    let style = Style::default().fg(Color::Blue);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(style)
        .title(title);

    frame.render_widget(block, area);
}

// Renders the window list for expanded session mode.
fn render_window_list(frame: &mut Frame, app: &App, area: Rect) {
    if app.expanded_windows.is_empty() {
        let paragraph = Paragraph::new("No windows found")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(paragraph, area);
        return;
    }

    let items: Vec<ListItem> = app
        .expanded_windows
        .iter()
        .enumerate()
        .map(|(idx, window)| {
            let is_selected = idx == app.selected_window_index;

            let active_icon = if window.active { "●" } else { " " };
            let active_style = if window.active {
                Style::default().fg(Color::Green)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let name_style = if is_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            let line = Line::from(vec![
                Span::styled("  ", Style::default()),
                Span::styled(active_icon, active_style),
                Span::raw(" "),
                Span::styled(format!("{}: ", window.index), Style::default().fg(Color::DarkGray)),
                Span::styled(&window.name, name_style),
                Span::styled(
                    format!(" [{}]", window.current_command),
                    Style::default().fg(Color::Cyan),
                ),
            ]);

            let item_style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            ListItem::new(line).style(item_style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL));

    frame.render_widget(list, area);
}

// Renders the help line for expanded session mode.
fn render_help_expanded(frame: &mut Frame, area: Rect) {
    let help_text = "j/k: navigate | Enter: attach window | h/Esc: collapse | q: quit";
    let paragraph = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));

    frame.render_widget(paragraph, area);
}

// Renders the preview panel showing captured pane content.
fn render_preview(frame: &mut Frame, app: &App, area: Rect) {
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

// Renders the decorative T-Rex background in the bottom-right corner.
fn render_background_trex(frame: &mut Frame, area: Rect) {
    // Only render if terminal is large enough
    if area.width < 80 || area.height < 25 {
        return;
    }

    let lines: Vec<Line> = TREX_ASCII
        .lines()
        .enumerate()
        .map(|(idx, line)| {
            let (r, g, b) = GREEN_GRADIENT[idx % GREEN_GRADIENT.len()];
            let green_style = Style::default().fg(Color::Rgb(r, g, b));
            // Reddish eye with slight green tint
            let eye_style = Style::default().fg(Color::Rgb(220, 50, 30));

            // Build spans with per-character coloring for the red eye
            let spans: Vec<Span> = line
                .chars()
                .map(|c| {
                    if idx == EYE_LINE && c == EYE_CHAR {
                        Span::styled(c.to_string(), eye_style)
                    } else {
                        Span::styled(c.to_string(), green_style)
                    }
                })
                .collect();

            Line::from(spans)
        })
        .collect();

    // Calculate art dimensions
    let art_height = lines.len() as u16;
    let art_width = TREX_ASCII.lines().map(|l| l.len()).max().unwrap_or(0) as u16;

    // Position in bottom-right corner
    let x = area.width.saturating_sub(art_width + 1);
    let y = area.height.saturating_sub(art_height);

    let art_area = Rect::new(x, y, art_width.min(area.width), art_height.min(area.height));

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, art_area);
}
