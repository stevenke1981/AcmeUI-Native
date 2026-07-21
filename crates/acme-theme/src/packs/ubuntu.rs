//! Ubuntu (Yaru) theme pack — Ubuntu orange, aubergine accent.

use crate::packs::{shadow_ladder, ThemePack};
use crate::{ColorTokens, Theme, ThemeColor, ThemeMode};

/// Ubuntu Yaru design pack.
pub struct UbuntuPack;

impl ThemePack for UbuntuPack {
    fn name(&self) -> &'static str {
        "ubuntu"
    }

    fn light(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(250, 250, 250),
            foreground: ThemeColor::rgb(17, 17, 17),
            surface: ThemeColor::rgb(255, 255, 255),
            surface_foreground: ThemeColor::rgb(17, 17, 17),

            primary: ThemeColor::rgb(233, 84, 32), // Ubuntu orange #E95420
            primary_foreground: ThemeColor::rgb(255, 255, 255),
            secondary: ThemeColor::rgb(238, 238, 238),
            secondary_foreground: ThemeColor::rgb(17, 17, 17),
            accent: ThemeColor::rgb(119, 41, 83), // aubergine #772953
            accent_foreground: ThemeColor::rgb(255, 255, 255),
            muted: ThemeColor::rgb(244, 244, 244),
            muted_foreground: ThemeColor::rgb(102, 102, 102),

            border: ThemeColor::rgb(224, 224, 224),
            input: ThemeColor::rgb(224, 224, 224),
            ring: ThemeColor::rgb(233, 84, 32),

            success: ThemeColor::rgb(14, 134, 78),
            warning: ThemeColor::rgb(249, 154, 0),
            danger: ThemeColor::rgb(218, 34, 34),
            info: ThemeColor::rgb(0, 115, 229),

            success_soft: ThemeColor::rgb(224, 244, 233),
            warning_soft: ThemeColor::rgb(254, 243, 217),
            danger_soft: ThemeColor::rgb(251, 226, 226),
            info_soft: ThemeColor::rgb(217, 234, 250),

            surface_elevated: ThemeColor::rgb(255, 255, 255),
            surface_overlay: ThemeColor::rgb(255, 255, 255),
            surface_tooltip: ThemeColor::rgb(51, 51, 51),

            primary_hover: ThemeColor::rgb(205, 73, 27),
            primary_pressed: ThemeColor::rgb(177, 63, 23),
            danger_hover: ThemeColor::rgb(190, 30, 30),
            danger_pressed: ThemeColor::rgb(160, 25, 25),
            ghost_hover: ThemeColor::rgb(238, 238, 238),
            ghost_pressed: ThemeColor::rgb(222, 222, 222),

            disabled_bg: ThemeColor::rgb(238, 238, 238),
            disabled_text: ThemeColor::rgb(170, 170, 170),
        };
        let mut t = Theme::from_colors(ThemeMode::Light, colors, shadow_ladder(0.05, 0.07, 0.09, 0.12));
        t.radii.sm = 4.0;
        t.radii.md = 6.0;
        t.radii.lg = 8.0;
        t.radii.xl = 12.0;
        t
    }

    fn dark(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(44, 44, 44), // Yaru dark bg
            foreground: ThemeColor::rgb(255, 255, 255),
            surface: ThemeColor::rgb(60, 60, 60),
            surface_foreground: ThemeColor::rgb(255, 255, 255),

            primary: ThemeColor::rgb(233, 84, 32), // Ubuntu orange
            primary_foreground: ThemeColor::rgb(255, 255, 255),
            secondary: ThemeColor::rgb(72, 72, 72),
            secondary_foreground: ThemeColor::rgb(255, 255, 255),
            accent: ThemeColor::rgb(153, 73, 110), // lighter aubergine
            accent_foreground: ThemeColor::rgb(255, 255, 255),
            muted: ThemeColor::rgb(60, 60, 60),
            muted_foreground: ThemeColor::rgb(170, 170, 170),

            border: ThemeColor::rgb(80, 80, 80),
            input: ThemeColor::rgb(80, 80, 80),
            ring: ThemeColor::rgb(233, 84, 32),

            success: ThemeColor::rgb(83, 216, 137),
            warning: ThemeColor::rgb(255, 181, 28),
            danger: ThemeColor::rgb(255, 99, 99),
            info: ThemeColor::rgb(94, 164, 255),

            success_soft: ThemeColor::rgb(13, 40, 24),
            warning_soft: ThemeColor::rgb(45, 31, 4),
            danger_soft: ThemeColor::rgb(45, 10, 10),
            info_soft: ThemeColor::rgb(13, 27, 62),

            surface_elevated: ThemeColor::rgb(72, 72, 72),
            surface_overlay: ThemeColor::rgb(80, 80, 80),
            surface_tooltip: ThemeColor::rgb(80, 80, 80),

            primary_hover: ThemeColor::rgb(240, 105, 58),
            primary_pressed: ThemeColor::rgb(205, 73, 27),
            danger_hover: ThemeColor::rgb(255, 129, 129),
            danger_pressed: ThemeColor::rgb(218, 34, 34),
            ghost_hover: ThemeColor::rgb(72, 72, 72),
            ghost_pressed: ThemeColor::rgb(80, 80, 80),

            disabled_bg: ThemeColor::rgb(72, 72, 72),
            disabled_text: ThemeColor::rgb(110, 110, 110),
        };
        let mut t = Theme::from_colors(ThemeMode::Dark, colors, shadow_ladder(0.30, 0.35, 0.45, 0.55));
        t.radii.sm = 4.0;
        t.radii.md = 6.0;
        t.radii.lg = 8.0;
        t.radii.xl = 12.0;
        t
    }
}
