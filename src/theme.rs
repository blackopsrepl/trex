use ratatui::style::Color;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OmarchyTheme {
    accent: String,
    cursor: String,
    foreground: String,
    background: String,
    selection_foreground: String,
    selection_background: String,
    color0: String,
    color1: String,
    color2: String,
    color3: String,
    color4: String,
    color5: String,
    color6: String,
    color7: String,
    color8: String,
    color9: String,
    color10: String,
    color11: String,
    color12: String,
    color13: String,
    color14: String,
    color15: String,
}

pub struct ThemeColors {
    pub primary: Color,      // Main accent/selection color
    pub secondary: Color,    // Secondary accent
    pub text: Color,         // Normal text
    pub text_dim: Color,     // Dimmed/secondary text
    pub border: Color,       // Borders
    pub success: Color,      // Green/success
    pub warning: Color,      // Yellow/warning
    pub error: Color,        // Red/error
    pub info: Color,         // Blue/info
    pub highlight: Color,    // Highlight/selection background
    pub bg_primary: Color,   // Primary background (bars, panels)
    pub bg_highlight: Color, // Highlighted selection background
    pub bg_overlay: Color,   // Overlay/modal background
}

impl Default for ThemeColors {
    fn default() -> Self {
        // Fallback: eza-compatible jungle theme using ANSI named colors.
        // These adapt to the terminal's configured palette, matching how
        // eza renders its output. Green/Cyan/Yellow/Red/Blue map directly
        // to eza's core color assignments.
        ThemeColors {
            primary: Color::Green,
            secondary: Color::Cyan,
            text: Color::White,
            text_dim: Color::DarkGray,
            border: Color::Green,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            info: Color::Blue,
            highlight: Color::DarkGray,
            bg_primary: Color::Black,
            bg_highlight: Color::DarkGray,
            bg_overlay: Color::Black,
        }
    }
}

// Extract RGB components from a Color, using a fallback for ANSI named colors
pub(crate) fn extract_rgb(color: Color, fallback: (f64, f64, f64)) -> (f64, f64, f64) {
    match color {
        Color::Rgb(r, g, b) => (r as f64, g as f64, b as f64),
        _ => fallback,
    }
}

// Linearly interpolate between two RGB triples
pub(crate) fn lerp_rgb(a: (f64, f64, f64), b: (f64, f64, f64), t: f64) -> Color {
    Color::Rgb(
        (a.0 + (b.0 - a.0) * t) as u8,
        (a.1 + (b.1 - a.1) * t) as u8,
        (a.2 + (b.2 - a.2) * t) as u8,
    )
}

impl ThemeColors {
    // Smooth gradient flowing through success -> warning -> error.
    // With Omarchy: interpolates the theme's actual hex colors.
    // Without Omarchy: uses jungle-toned RGB fallbacks (green -> amber -> orange).
    pub fn gradient_color(&self, percent: f64) -> Color {
        let t = (percent / 100.0).clamp(0.0, 1.0);

        let s = extract_rgb(self.success, (50.0, 200.0, 50.0));
        let w = extract_rgb(self.warning, (255.0, 200.0, 30.0));
        let e = extract_rgb(self.error, (255.0, 140.0, 0.0));

        if t < 0.5 {
            lerp_rgb(s, w, t * 2.0)
        } else {
            lerp_rgb(w, e, (t - 0.5) * 2.0)
        }
    }
}

fn parse_hex_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some(Color::Rgb(r, g, b))
}

fn load_omarchy_theme() -> Option<OmarchyTheme> {
    let home = std::env::var("HOME").ok()?;
    let theme_path = PathBuf::from(home).join(".config/omarchy/current/theme/colors.toml");

    let contents = fs::read_to_string(theme_path).ok()?;
    toml::from_str(&contents).ok()
}

pub fn load_theme() -> ThemeColors {
    if let Some(omarchy) = load_omarchy_theme() {
        ThemeColors {
            primary: parse_hex_color(&omarchy.accent).unwrap_or(Color::Green),
            secondary: parse_hex_color(&omarchy.color2).unwrap_or(Color::Cyan),
            text: parse_hex_color(&omarchy.foreground).unwrap_or(Color::White),
            text_dim: parse_hex_color(&omarchy.color8).unwrap_or(Color::DarkGray),
            border: parse_hex_color(&omarchy.accent).unwrap_or(Color::Green),
            success: parse_hex_color(&omarchy.color2).unwrap_or(Color::Green),
            warning: parse_hex_color(&omarchy.color3).unwrap_or(Color::Yellow),
            error: parse_hex_color(&omarchy.color1).unwrap_or(Color::Red),
            info: parse_hex_color(&omarchy.color4).unwrap_or(Color::Blue),
            highlight: parse_hex_color(&omarchy.selection_background).unwrap_or(Color::DarkGray),
            bg_primary: parse_hex_color(&omarchy.background).unwrap_or(Color::Black),
            bg_highlight: parse_hex_color(&omarchy.selection_background).unwrap_or(Color::DarkGray),
            bg_overlay: parse_hex_color(&omarchy.background).unwrap_or(Color::Black),
        }
    } else {
        ThemeColors::default()
    }
}
