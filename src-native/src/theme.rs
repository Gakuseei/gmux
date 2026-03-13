use std::fs;
use std::path::Path;

use alacritty_terminal::term::color::Colors;
use alacritty_terminal::vte::ansi::{NamedColor, Rgb};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub name: String,
    pub foreground: Color,
    pub background: Color,
    pub cursor: Color,
    pub selection_foreground: Color,
    pub selection_background: Color,
    pub ansi: [Color; 16],
}

pub struct UiTheme {
    pub bg_primary: Color,
    pub bg_sidebar: Color,
    pub bg_surface: Color,
    pub border: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub accent: Color,
    pub diff_add: Color,
    pub diff_delete: Color,
    pub notification: Color,
    pub hover_overlay: Color,
    pub hover_overlay_alpha: f32,
    pub active_highlight_alpha: f32,
}

impl Color {
    pub fn from_hex(hex: &str) -> Self {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        let val = u32::from_str_radix(hex, 16).unwrap_or(0);
        Self {
            r: ((val >> 16) & 0xFF) as u8,
            g: ((val >> 8) & 0xFF) as u8,
            b: (val & 0xFF) as u8,
        }
    }

    pub fn to_iced(&self) -> iced::Color {
        iced::Color::from_rgb(
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
        )
    }

    pub fn to_iced_alpha(&self, alpha: f32) -> iced::Color {
        iced::Color::from_rgba(
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            alpha,
        )
    }

    fn to_rgb(&self) -> Rgb {
        Rgb {
            r: self.r,
            g: self.g,
            b: self.b,
        }
    }
}

impl ColorScheme {
    pub fn gmux_dark() -> Self {
        Self {
            name: String::from("gmux-dark"),
            foreground: Color::from_hex("#e5e5e5"),
            background: Color::from_hex("#171717"),
            cursor: Color::from_hex("#10a37f"),
            selection_foreground: Color::from_hex("#e5e5e5"),
            selection_background: Color::from_hex("#2d5a4a"),
            ansi: [
                Color::from_hex("#000000"),
                Color::from_hex("#cc0000"),
                Color::from_hex("#4e9a06"),
                Color::from_hex("#c4a000"),
                Color::from_hex("#3465a4"),
                Color::from_hex("#75507b"),
                Color::from_hex("#06989a"),
                Color::from_hex("#d3d7cf"),
                Color::from_hex("#555753"),
                Color::from_hex("#ef2929"),
                Color::from_hex("#8ae234"),
                Color::from_hex("#fce94f"),
                Color::from_hex("#729fcf"),
                Color::from_hex("#ad7fa8"),
                Color::from_hex("#34e2e2"),
                Color::from_hex("#eeeeec"),
            ],
        }
    }

    pub fn to_alacritty_colors(&self) -> Colors {
        let mut colors = Colors::default();

        for (i, ansi_color) in self.ansi.iter().enumerate() {
            colors[i] = Some(ansi_color.to_rgb());
        }

        colors[NamedColor::Foreground] = Some(self.foreground.to_rgb());
        colors[NamedColor::Background] = Some(self.background.to_rgb());
        colors[NamedColor::Cursor] = Some(self.cursor.to_rgb());

        colors
    }
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            bg_primary: Color::from_hex("#171717"),
            bg_sidebar: Color::from_hex("#0d0d0d"),
            bg_surface: Color::from_hex("#2a2a2a"),
            border: Color::from_hex("#303030"),
            text_primary: Color::from_hex("#e5e5e5"),
            text_secondary: Color::from_hex("#8e8ea0"),
            accent: Color::from_hex("#10a37f"),
            diff_add: Color::from_hex("#212922"),
            diff_delete: Color::from_hex("#3c170f"),
            notification: Color::from_hex("#3b82f6"),
            hover_overlay: Color { r: 255, g: 255, b: 255 },
            hover_overlay_alpha: 0.05,
            active_highlight_alpha: 0.15,
        }
    }
}

pub fn load_schemes_from_dir(dir: &Path) -> Vec<ColorScheme> {
    let mut schemes = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return schemes,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("ron") {
            if let Ok(contents) = fs::read_to_string(&path) {
                if let Ok(scheme) = ron::from_str::<ColorScheme>(&contents) {
                    schemes.push(scheme);
                }
            }
        }
    }
    schemes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_from_hex_with_hash() {
        let c = Color::from_hex("#ff8000");
        assert_eq!(c.r, 255);
        assert_eq!(c.g, 128);
        assert_eq!(c.b, 0);
    }

    #[test]
    fn color_from_hex_without_hash() {
        let c = Color::from_hex("10a37f");
        assert_eq!(c.r, 16);
        assert_eq!(c.g, 163);
        assert_eq!(c.b, 127);
    }

    #[test]
    fn color_to_iced_range() {
        let c = Color { r: 255, g: 0, b: 128 };
        let ic = c.to_iced();
        assert!((ic.r - 1.0).abs() < f32::EPSILON);
        assert!(ic.g.abs() < f32::EPSILON);
        assert!((ic.b - 128.0 / 255.0).abs() < 0.001);
    }

    #[test]
    fn gmux_dark_has_16_ansi_colors() {
        let scheme = ColorScheme::gmux_dark();
        assert_eq!(scheme.ansi.len(), 16);
        assert_eq!(scheme.name, "gmux-dark");
    }

    #[test]
    fn gmux_dark_to_alacritty_colors() {
        let scheme = ColorScheme::gmux_dark();
        let colors = scheme.to_alacritty_colors();
        assert!(colors[NamedColor::Foreground].is_some());
        assert!(colors[NamedColor::Background].is_some());
        assert!(colors[NamedColor::Cursor].is_some());
        assert!(colors[0].is_some());
        assert!(colors[15].is_some());
    }

    #[test]
    fn ui_theme_default_values() {
        let theme = UiTheme::default();
        assert_eq!(theme.bg_primary.r, 0x17);
        assert_eq!(theme.bg_primary.g, 0x17);
        assert_eq!(theme.bg_primary.b, 0x17);
        assert_eq!(theme.accent.r, 0x10);
        assert_eq!(theme.accent.g, 0xa3);
        assert_eq!(theme.accent.b, 0x7f);
    }

    #[test]
    fn color_scheme_ron_roundtrip() {
        let scheme = ColorScheme::gmux_dark();
        let pretty = ron::ser::PrettyConfig::default();
        let serialized = ron::ser::to_string_pretty(&scheme, pretty).expect("serialize scheme");
        let deserialized: ColorScheme = ron::from_str(&serialized).expect("deserialize scheme");
        assert_eq!(deserialized.name, scheme.name);
        assert_eq!(deserialized.foreground.r, scheme.foreground.r);
        assert_eq!(deserialized.ansi[0].r, scheme.ansi[0].r);
        assert_eq!(deserialized.ansi[15].b, scheme.ansi[15].b);
    }

    #[test]
    fn load_schemes_from_nonexistent_dir() {
        let schemes = load_schemes_from_dir(Path::new("/nonexistent/path/gmux/test"));
        assert!(schemes.is_empty());
    }

    #[test]
    fn load_schemes_from_temp_dir() {
        let dir = std::env::temp_dir().join(format!("gmux_theme_test_{}", std::process::id()));
        fs::create_dir_all(&dir).expect("create temp dir");

        let scheme = ColorScheme::gmux_dark();
        let pretty = ron::ser::PrettyConfig::default();
        let data = ron::ser::to_string_pretty(&scheme, pretty).expect("serialize");
        fs::write(dir.join("test.ron"), &data).expect("write scheme file");
        fs::write(dir.join("not_ron.txt"), "ignored").expect("write non-ron file");

        let schemes = load_schemes_from_dir(&dir);
        assert_eq!(schemes.len(), 1);
        assert_eq!(schemes[0].name, "gmux-dark");

        fs::remove_dir_all(&dir).ok();
    }
}
