use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::tui::app::App;

/* Renders the session naming mode layout.
 *
 * This mode is entered after the user selects a directory for creating a new session.
 * It consists of three sections:
 * - Header: Shows naming prompt and current input with cursor
 * - Preview: Shows directory path and sanitized session name
 * - Help: Shows available keybindings */
pub fn render_naming_mode(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(frame.area());

    render_header_naming(frame, app, chunks[0]);
    render_naming_preview(frame, app, chunks[1]);
    render_help_naming(frame, app, chunks[2]);
}

/* Renders the header for session naming mode.
 *
 * Shows the naming prompt with the current input and a blinking cursor.
 * The header has a magenta border to indicate the naming mode. */
pub fn render_header_naming(frame: &mut Frame, app: &App, area: Rect) {
    let cursor = "_";
    let title = Line::from(vec![
        Span::styled(
            " Name session ",
            Style::default()
                .fg(app.theme.secondary)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("> ", Style::default().fg(app.theme.secondary)),
        Span::styled(
            &app.session_name_input,
            Style::default()
                .fg(app.theme.text)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            cursor,
            Style::default()
                .fg(app.theme.secondary)
                .add_modifier(Modifier::RAPID_BLINK),
        ),
        Span::raw(" "),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.secondary))
        .title(title);

    frame.render_widget(block, area);
}

/* Renders the preview panel for session naming mode.
 *
 * Shows:
 * - The selected directory path (with home directory abbreviated to ~)
 * - The sanitized session name that will be used
 * - Whether the name was sanitized (if different from input)
 * - Instructions for creating or going back */
pub fn render_naming_preview(frame: &mut Frame, app: &App, area: Rect) {
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
            Span::styled("    Directory  ", Style::default().fg(app.theme.text_dim)),
            Span::styled(&path_display, Style::default().fg(app.theme.info)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    Session    ", Style::default().fg(app.theme.text_dim)),
            Span::styled(
                &sanitized_name,
                Style::default()
                    .fg(app.theme.warning)
                    .add_modifier(Modifier::BOLD),
            ),
            if name_changed {
                Span::styled(
                    " (sanitized)",
                    Style::default()
                        .fg(app.theme.text_dim)
                        .add_modifier(Modifier::ITALIC),
                )
            } else {
                Span::raw("")
            },
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    ", Style::default()),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(app.theme.border)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to create  ", Style::default().fg(app.theme.text_dim)),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(app.theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to go back", Style::default().fg(app.theme.text_dim)),
        ]),
    ];

    let paragraph = Paragraph::new(lines).block(Block::default().borders(Borders::ALL));

    frame.render_widget(paragraph, area);
}

/* Renders the help line for session naming mode.
 *
 * Shows available keybindings:
 * - Type: Enter session name
 * - Enter: Create the session
 * - Esc: Go back to directory selection */
pub fn render_help_naming(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = "Type session name | Enter: create | Esc: back to directories";
    let paragraph = Paragraph::new(help_text).style(Style::default().fg(app.theme.text_dim));

    frame.render_widget(paragraph, area);
}
