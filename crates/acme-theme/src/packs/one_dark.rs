//! One Dark theme pack — Atom's One Dark / One Light palette.

use crate::packs::{shadow_ladder, ThemePack};
use crate::{ColorTokens, Theme, ThemeColor, ThemeMode};

/// One Dark design pack.
pub struct OneDarkPack;

impl ThemePack for OneDarkPack {
    fn name(&self) -> &'static str {
        "one-dark"
    }

    fn light(&self) -> Theme {
        // One Light variant.
        let colors = ColorTokens {
            background: ThemeColor::rgb(250, 250, 250),
            foreground: ThemeColor::rgb(56, 58, 66),
            surface: ThemeColor::rgb(255, 255, 255),
            surface_foreground: ThemeColor::rgb(56, 58, 66),

            primary: ThemeColor::rgb(64, 120, 242), // blue
            primary_foreground: ThemeColor::rgb(255, 255, 255),
            secondary: ThemeColor::rgb(240, 240, 242),
            secondary_foreground: ThemeColor::rgb(56, 58, 66),
            accent: ThemeColor::rgb(166, 38, 164), // purple
            accent_foreground: ThemeColor::rgb(255, 255, 255),
            muted: ThemeColor::rgb(240, 240, 242),
            muted_foreground: ThemeColor::rgb(160, 161, 167),

            border: ThemeColor::rgb(219, 219, 220),
            input: ThemeColor::rgb(219, 219, 220),
            ring: ThemeColor::rgb(64, 120, 242),

            success: ThemeColor::rgb(80, 161, 79),  // green
            warning: ThemeColor::rgb(193, 132, 1),  // yellow
            danger: ThemeColor::rgb(228, 86, 73),   // red
            info: ThemeColor::rgb(1, 132, 188),     // cyan

            success_soft: ThemeColor::rgb(224, 240, 222),
            warning_soft: ThemeColor::rgb(248, 238, 214),
            danger_soft: ThemeColor::rgb(250, 225, 220),
            info_soft: ThemeColor::rgb(214, 238, 246),

            surface_elevated: ThemeColor::rgb(255, 255, 255),
            surface_overlay: ThemeColor::rgb(255, 255, 255),
            surface_tooltip: ThemeColor::rgb(56, 58, 66),

            primary_hover: ThemeColor::rgb(90, 140, 245),
            primary_pressed: ThemeColor::rgb(48, 98, 210),
            danger_hover: ThemeColor::rgb(234, 110, 98),
            danger_pressed: ThemeColor::rgb(198, 68, 57),
            ghost_hover: ThemeColor::rgb(240, 240, 242),
            ghost_pressed: ThemeColor::rgb(225, 225, 228),

            disabled_bg: ThemeColor::rgb(240, 240, 242),
            disabled_text: ThemeColor::rgb(160, 161, 167),
        };
        Theme::from_colors(ThemeMode::Light, colors, shadow_ladder(0.05, 0.07, 0.09, 0.12))
    }

    fn dark(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(40, 44, 52),    // One Dark bg
            foreground: ThemeColor::rgb(171, 178, 191), // fg
            surface: ThemeColor::rgb(33, 37, 43),
            surface_foreground: ThemeColor::rgb(171, 178, 191),

            primary: ThemeColor::rgb(97, 175, 239), // blue
            primary_foreground: ThemeColor::rgb(33, 37, 43),
            secondary: ThemeColor::rgb(33, 37, 43),
            secondary_foreground: ThemeColor::rgb(171, 178, 191),
            accent: ThemeColor::rgb(198, 120, 221), // purple
            accent_foreground: ThemeColor::rgb(40, 44, 52),
            muted: ThemeColor::rgb(33, 37, 43),
            muted_foreground: ThemeColor::rgb(92, 99, 112),

            border: ThemeColor::rgb(62, 68, 81),
            input: ThemeColor::rgb(62, 68, 81),
            ring: ThemeColor::rgb(97, 175, 239),

            success: ThemeColor::rgb(152, 195, 121), // green
            warning: ThemeColor::rgb(229, 192, 123), // yellow
            danger: ThemeColor::rgb(224, 108, 117),  // red
            info: ThemeColor::rgb(86, 182, 194),     // cyan

            success_soft: ThemeColor::rgb(35, 50, 38),
            warning_soft: ThemeColor::rgb(52, 48, 35),
            danger_soft: ThemeColor::rgb(52, 35, 38),
            info_soft: ThemeColor::rgb(32, 48, 52),

            surface_elevated: ThemeColor::rgb(33, 37, 43),
            surface_overlay: ThemeColor::rgb(62, 68, 81),
            surface_tooltip: ThemeColor::rgb(62, 68, 81),

            primary_hover: ThemeColor::rgb(120, 190, 245),
            primary_pressed: ThemeColor::rgb(70, 145, 210),
            danger_hover: ThemeColor::rgb(232, 130, 138),
            danger_pressed: ThemeColor::rgb(195, 85, 94),
            ghost_hover: ThemeColor::rgb(33, 37, 43),
            ghost_pressed: ThemeColor::rgb(62, 68, 81),

            disabled_bg: ThemeColor::rgb(33, 37, 43),
            disabled_text: ThemeColor::rgb(92, 99, 112),
        };
        Theme::from_colors(ThemeMode::Dark, colors, shadow_ladder(0.30, 0.35, 0.45, 0.55))
    }
}
