//! Dracula theme pack — vibrant dark palette with purple accent.

use crate::packs::{shadow_ladder, ThemePack};
use crate::{ColorTokens, Theme, ThemeColor, ThemeMode};

/// Dracula design pack.
pub struct DraculaPack;

impl ThemePack for DraculaPack {
    fn name(&self) -> &'static str {
        "dracula"
    }

    fn light(&self) -> Theme {
        // Derived "Dracula light" (inverted background).
        let colors = ColorTokens {
            background: ThemeColor::rgb(248, 248, 242), // foreground as bg
            foreground: ThemeColor::rgb(40, 42, 54),    // background as fg
            surface: ThemeColor::rgb(255, 255, 255),
            surface_foreground: ThemeColor::rgb(40, 42, 54),

            primary: ThemeColor::rgb(189, 147, 249), // purple
            primary_foreground: ThemeColor::rgb(40, 42, 54),
            secondary: ThemeColor::rgb(235, 235, 230),
            secondary_foreground: ThemeColor::rgb(40, 42, 54),
            accent: ThemeColor::rgb(235, 228, 248),
            accent_foreground: ThemeColor::rgb(120, 80, 200),
            muted: ThemeColor::rgb(235, 235, 230),
            muted_foreground: ThemeColor::rgb(98, 114, 164), // comment

            border: ThemeColor::rgb(220, 220, 214),
            input: ThemeColor::rgb(220, 220, 214),
            ring: ThemeColor::rgb(189, 147, 249),

            success: ThemeColor::rgb(80, 250, 123),
            warning: ThemeColor::rgb(241, 250, 140),
            danger: ThemeColor::rgb(255, 85, 85),
            info: ThemeColor::rgb(139, 233, 253),

            success_soft: ThemeColor::rgb(224, 247, 229),
            warning_soft: ThemeColor::rgb(250, 248, 214),
            danger_soft: ThemeColor::rgb(255, 228, 228),
            info_soft: ThemeColor::rgb(222, 245, 250),

            surface_elevated: ThemeColor::rgb(255, 255, 255),
            surface_overlay: ThemeColor::rgb(255, 255, 255),
            surface_tooltip: ThemeColor::rgb(68, 71, 90),

            primary_hover: ThemeColor::rgb(200, 165, 250),
            primary_pressed: ThemeColor::rgb(150, 110, 220),
            danger_hover: ThemeColor::rgb(255, 110, 110),
            danger_pressed: ThemeColor::rgb(220, 60, 60),
            ghost_hover: ThemeColor::rgb(235, 235, 230),
            ghost_pressed: ThemeColor::rgb(220, 220, 214),

            disabled_bg: ThemeColor::rgb(235, 235, 230),
            disabled_text: ThemeColor::rgb(170, 170, 165),
        };
        Theme::from_colors(ThemeMode::Light, colors, shadow_ladder(0.05, 0.07, 0.09, 0.12))
    }

    fn dark(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(40, 42, 54),   // Dracula bg
            foreground: ThemeColor::rgb(248, 248, 242), // foreground
            surface: ThemeColor::rgb(68, 71, 90),      // current line
            surface_foreground: ThemeColor::rgb(248, 248, 242),

            primary: ThemeColor::rgb(189, 147, 249), // purple
            primary_foreground: ThemeColor::rgb(40, 42, 54),
            secondary: ThemeColor::rgb(68, 71, 90),
            secondary_foreground: ThemeColor::rgb(248, 248, 242),
            accent: ThemeColor::rgb(68, 71, 90),
            accent_foreground: ThemeColor::rgb(255, 121, 198), // pink
            muted: ThemeColor::rgb(68, 71, 90),
            muted_foreground: ThemeColor::rgb(98, 114, 164), // comment

            border: ThemeColor::rgb(98, 114, 164),
            input: ThemeColor::rgb(98, 114, 164),
            ring: ThemeColor::rgb(189, 147, 249),

            success: ThemeColor::rgb(80, 250, 123),  // green
            warning: ThemeColor::rgb(241, 250, 140), // yellow
            danger: ThemeColor::rgb(255, 85, 85),    // red
            info: ThemeColor::rgb(139, 233, 253),    // cyan

            success_soft: ThemeColor::rgb(30, 50, 38),
            warning_soft: ThemeColor::rgb(50, 50, 30),
            danger_soft: ThemeColor::rgb(50, 25, 30),
            info_soft: ThemeColor::rgb(28, 45, 55),

            surface_elevated: ThemeColor::rgb(68, 71, 90),
            surface_overlay: ThemeColor::rgb(80, 83, 105),
            surface_tooltip: ThemeColor::rgb(98, 114, 164),

            primary_hover: ThemeColor::rgb(200, 165, 250),
            primary_pressed: ThemeColor::rgb(150, 110, 220),
            danger_hover: ThemeColor::rgb(255, 110, 110),
            danger_pressed: ThemeColor::rgb(220, 60, 60),
            ghost_hover: ThemeColor::rgb(68, 71, 90),
            ghost_pressed: ThemeColor::rgb(80, 83, 105),

            disabled_bg: ThemeColor::rgb(68, 71, 90),
            disabled_text: ThemeColor::rgb(98, 114, 164),
        };
        Theme::from_colors(ThemeMode::Dark, colors, shadow_ladder(0.30, 0.35, 0.45, 0.55))
    }
}
