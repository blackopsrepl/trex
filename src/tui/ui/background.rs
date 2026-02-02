use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::constants::{EYE_CHAR, EYE_LINE, GREEN_GRADIENT, TREX_ASCII};

/// Renders the decorative T-Rex background in the bottom-right corner.
pub fn render_background_trex(frame: &mut Frame, area: Rect) {
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
