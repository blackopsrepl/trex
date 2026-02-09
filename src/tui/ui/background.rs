use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use super::constants::{EYE_CHAR, EYE_LINE, TREX_ASCII};
use crate::tui::app::App;

// Generate a color gradient based on a base color
fn generate_gradient(base_color: Color, steps: usize) -> Vec<Color> {
    let (r, g, b) = match base_color {
        Color::Rgb(r, g, b) => (r as f32, g as f32, b as f32),
        _ => (80.0, 200.0, 120.0), // fallback
    };

    (0..steps)
        .map(|i| {
            let factor = 0.5 + (i as f32 / steps as f32) * 0.5; // 50% to 100% intensity
            Color::Rgb(
                (r * factor) as u8,
                (g * factor) as u8,
                (b * factor) as u8,
            )
        })
        .collect()
}

pub fn render_background_trex(frame: &mut Frame, app: &App, area: Rect) {
    // Only render if terminal is large enough
    if area.width < 80 || area.height < 25 {
        return;
    }

    let gradient = generate_gradient(app.theme.primary, 8);

    let lines: Vec<Line> = TREX_ASCII
        .lines()
        .enumerate()
        .map(|(idx, line)| {
            let base_style = Style::default().fg(gradient[idx % gradient.len()]);

            // Eye uses the error color (typically red)
            let eye_style = Style::default().fg(app.theme.error);

            // Build spans with per-character coloring for the eye
            let spans: Vec<Span> = line
                .chars()
                .map(|c| {
                    if idx == EYE_LINE && c == EYE_CHAR {
                        Span::styled(c.to_string(), eye_style)
                    } else {
                        Span::styled(c.to_string(), base_style)
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
