use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use super::constants::{EYE_CHAR, EYE_LINE, TREX_ASCII};
use crate::theme::{extract_rgb, lerp_rgb};
use crate::tui::app::App;

pub fn render_background_trex(frame: &mut Frame, app: &App, area: Rect) {
    // Only render if terminal is large enough
    if area.width < 80 || area.height < 25 {
        return;
    }

    // Gradient flows from primary (head) to warning (feet):
    // jungle green canopy fading to warm amber ground.
    // With Omarchy these are the theme's actual hex colors;
    // without, the fallbacks give a jungle green-to-amber tone.
    let top = extract_rgb(app.theme.primary, (80.0, 200.0, 120.0));
    let bottom = extract_rgb(app.theme.warning, (255.0, 200.0, 30.0));

    let total_lines = TREX_ASCII.lines().count().max(1);

    let lines: Vec<Line> = TREX_ASCII
        .lines()
        .enumerate()
        .map(|(idx, line)| {
            let t = idx as f64 / (total_lines - 1).max(1) as f64;
            // Dim to 40%-80% intensity so the art stays subtle behind the UI
            let dim = 0.4 + t * 0.4;
            let blended = lerp_rgb(top, bottom, t);
            let (r, g, b) = extract_rgb(blended, (80.0, 200.0, 120.0));
            let dimmed =
                ratatui::style::Color::Rgb((r * dim) as u8, (g * dim) as u8, (b * dim) as u8);
            let base_style = Style::default().fg(dimmed);

            // Eye uses the error color
            let eye_style = Style::default().fg(app.theme.error);

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
