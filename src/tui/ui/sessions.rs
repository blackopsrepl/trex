use crate::tui::app::{App, AppMode, FocusArea};
use ratatui::{
    layout::Rect,
    style::Style,
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};

use super::session_row::{render_session_gauges, render_session_header};

// Braille-based vertical scrollbar characters
const SCROLL_TRACK: &str = "│";
const SCROLL_THUMB: &str = "┃";

// Each session takes 5 lines: header, gauges, sparklines, spacing, separator
const LINES_PER_SESSION: u16 = 5;

pub fn render_session_list(frame: &mut Frame, app: &App, area: Rect) {
    let title = match app.mode {
        AppMode::Filtering => format!(
            " ⚡ Sessions ({}) > {} ",
            app.filtered_indices.len(),
            app.filter_input
        ),
        _ => format!(
            " ⚡ Sessions ({}) • ●=active ○=idle ◌=dormant ★=attached 🟢🟡🔴=health ",
            app.sessions.len()
        ),
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
        format!(
            " [{}/{}]",
            app.selected_index + 1,
            app.filtered_indices.len()
        )
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
    render_scrollbar(frame, app, inner, start_idx, max_visible);

    let mut y_offset = 0;

    for display_idx in start_idx..end_idx {
        let &session_idx = &app.filtered_indices[display_idx];
        let session = &app.sessions[session_idx];
        let is_selected = app.focus == FocusArea::Sessions && display_idx == app.selected_index;

        render_session_header(frame, app, inner, &mut y_offset, session, is_selected);
        render_session_gauges(frame, app, inner, &mut y_offset, session);

        // Add spacing between sessions
        y_offset += 1;
    }
}

fn render_scrollbar(
    frame: &mut Frame,
    app: &App,
    inner: Rect,
    start_idx: usize,
    max_visible: usize,
) {
    let total = app.filtered_indices.len();
    if total <= max_visible || inner.height == 0 {
        return;
    }

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
