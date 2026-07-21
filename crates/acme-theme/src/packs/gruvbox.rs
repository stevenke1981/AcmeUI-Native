//! Gruvbox theme pack — retro groove warm palette with aqua/orange accents.

use crate::packs::{shadow_ladder, ThemePack};
use crate::{ColorTokens, Theme, ThemeColor, ThemeMode};

/// Gruvbox design pack.
pub struct GruvboxPack;

impl ThemePack for GruvboxPack {
    fn name(&self) -> &'static str {
        "gruvbox"
    }

    fn light(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(251, 241, 199), // bg
            foreground: ThemeColor::rgb(60, 56, 54),    // fg
            surface: ThemeColor::rgb(235, 219, 178),    // bg1
            surface_foreground: ThemeColor::rgb(60, 56, 54),

            primary: ThemeColor::rgb(69, 133, 136), // aqua/blue
            primary_foreground: ThemeColor::rgb(251, 241, 199),
            secondary: ThemeColor::rgb(235, 219, 178),
            secondary_foreground: ThemeColor::rgb(60, 56, 54),
            accent: ThemeColor::rgb(214, 93, 14), // orange
            accent_foreground: ThemeColor::rgb(251, 241, 199),
            muted: ThemeColor::rgb(235, 219, 178),
            muted_foreground: ThemeColor::rgb(124, 111, 100), // gray

            border: ThemeColor::rgb(235, 219, 178),
            input: ThemeColor::rgb(213, 196, 161),
            ring: ThemeColor::rgb(69, 133, 136),

            success: ThemeColor::rgb(152, 151, 26), // green
            warning: ThemeColor::rgb(215, 153, 33), // yellow
            danger: ThemeColor::rgb(204, 36, 29),   // red
            info: ThemeColor::rgb(69, 133, 136),    // blue

            success_soft: ThemeColor::rgb(235, 238, 200),
            warning_soft: ThemeColor::rgb(245, 232, 195),
            danger_soft: ThemeColor::rgb(248, 215, 205),
            info_soft: ThemeColor::rgb(215, 232, 232),

            surface_elevated: ThemeColor::rgb(251, 241, 199),
            surface_overlay: ThemeColor::rgb(251, 241, 199),
            surface_tooltip: ThemeColor::rgb(80, 73, 69),

            primary_hover: ThemeColor::rgb(90, 150, 153),
            primary_pressed: ThemeColor::rgb(55, 105, 108),
            danger_hover: ThemeColor::rgb(215, 60, 52),
            danger_pressed: ThemeColor::rgb(170, 30, 24),
            ghost_hover: ThemeColor::rgb(235, 219, 178),
            ghost_pressed: ThemeColor::rgb(213, 196, 161),

            disabled_bg: ThemeColor::rgb(235, 219, 178),
            disabled_text: ThemeColor::rgb(124, 111, 100),
        };
        Theme::from_colors(ThemeMode::Light, colors, shadow_ladder(0.05, 0.07, 0.09, 0.12))
    }

    fn dark(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(40, 40, 40),    // bg
            foreground: ThemeColor::rgb(235, 219, 178), // fg
            surface: ThemeColor::rgb(60, 56, 54),       // bg1
            surface_foreground: ThemeColor::rgb(235, 219, 178),

            primary: ThemeColor::rgb(69, 133, 136), // aqua/blue
            primary_foreground: ThemeColor::rgb(251, 241, 199),
            secondary: ThemeColor::rgb(60, 56, 54),
            secondary_foreground: ThemeColor::rgb(235, 219, 178),
            accent: ThemeColor::rgb(214, 93, 14), // orange
            accent_foreground: ThemeColor::rgb(251, 241, 199),
            muted: ThemeColor::rgb(60, 56, 54),
            muted_foreground: ThemeColor::rgb(168, 153, 132), // gray

            border: ThemeColor::rgb(60, 56, 54),
            input: ThemeColor::rgb(80, 73, 69),
            ring: ThemeColor::rgb(69, 133, 136),

            success: ThemeColor::rgb(152, 151, 26), // green
            warning: ThemeColor::rgb(215, 153, 33), // yellow
            danger: ThemeColor::rgb(204, 36, 29),   // red
            info: ThemeColor::rgb(69, 133, 136),    // blue

            success_soft: ThemeColor::rgb(45, 50, 25),
            warning_soft: ThemeColor::rgb(55, 45, 20),
            danger_soft: ThemeColor::rgb(55, 25, 22),
            info_soft: ThemeColor::rgb(28, 45, 48),

            surface_elevated: ThemeColor::rgb(60, 56, 54),
            surface_overlay: ThemeColor::rgb(80, 73, 69),
            surface_tooltip: ThemeColor::rgb(80, 73, 69),

            primary_hover: ThemeColor::rgb(90, 150, 153),
            primary_pressed: ThemeColor::rgb(55, 105, 108),
            danger_hover: ThemeColor::rgb(215, 60, 52),
            danger_pressed: ThemeColor::rgb(170, 30, 24),
            ghost_hover: ThemeColor::rgb(60, 56, 54),
            ghost_pressed: ThemeColor::rgb(80, 73, 69),

            disabled_bg: ThemeColor::rgb(60, 56, 54),
            disabled_text: ThemeColor::rgb(168, 153, 132),
        };
        Theme::from_colors(ThemeMode::Dark, colors, shadow_ladder(0.30, 0.35, 0.45, 0.55))
    }
}
