//! Windows 10 (Fluent) theme pack — accent blue, squared geometry.

use crate::packs::{shadow_ladder, ThemePack};
use crate::{ColorTokens, Theme, ThemeColor, ThemeMode};

/// Windows 10 Fluent design pack.
pub struct Windows10Pack;

impl ThemePack for Windows10Pack {
    fn name(&self) -> &'static str {
        "windows10"
    }

    fn light(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(255, 255, 255),
            foreground: ThemeColor::rgb(0, 0, 0),
            surface: ThemeColor::rgb(243, 243, 243),
            surface_foreground: ThemeColor::rgb(0, 0, 0),

            primary: ThemeColor::rgb(0, 120, 212), // Fluent accent #0078D4
            primary_foreground: ThemeColor::rgb(255, 255, 255),
            secondary: ThemeColor::rgb(238, 238, 238),
            secondary_foreground: ThemeColor::rgb(0, 0, 0),
            accent: ThemeColor::rgb(222, 236, 249),
            accent_foreground: ThemeColor::rgb(0, 90, 158),
            muted: ThemeColor::rgb(243, 243, 243),
            muted_foreground: ThemeColor::rgb(96, 96, 96),

            border: ThemeColor::rgb(229, 229, 229),
            input: ThemeColor::rgb(229, 229, 229),
            ring: ThemeColor::rgb(0, 120, 212),

            success: ThemeColor::rgb(16, 124, 16),
            warning: ThemeColor::rgb(157, 93, 0),
            danger: ThemeColor::rgb(232, 17, 35),
            info: ThemeColor::rgb(0, 120, 212),

            success_soft: ThemeColor::rgb(223, 246, 221),
            warning_soft: ThemeColor::rgb(255, 244, 206),
            danger_soft: ThemeColor::rgb(253, 231, 233),
            info_soft: ThemeColor::rgb(222, 236, 249),

            surface_elevated: ThemeColor::rgb(255, 255, 255),
            surface_overlay: ThemeColor::rgb(255, 255, 255),
            surface_tooltip: ThemeColor::rgb(43, 43, 43),

            primary_hover: ThemeColor::rgb(16, 110, 190),
            primary_pressed: ThemeColor::rgb(0, 90, 158),
            danger_hover: ThemeColor::rgb(196, 14, 30),
            danger_pressed: ThemeColor::rgb(160, 12, 24),
            ghost_hover: ThemeColor::rgb(238, 238, 238),
            ghost_pressed: ThemeColor::rgb(222, 222, 222),

            disabled_bg: ThemeColor::rgb(238, 238, 238),
            disabled_text: ThemeColor::rgb(160, 160, 160),
        };
        let mut t = Theme::from_colors(ThemeMode::Light, colors, shadow_ladder(0.05, 0.07, 0.09, 0.12));
        // Windows 10 is comparatively squared.
        t.radii.sm = 2.0;
        t.radii.md = 2.0;
        t.radii.lg = 4.0;
        t.radii.xl = 4.0;
        t
    }

    fn dark(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(32, 32, 32),
            foreground: ThemeColor::rgb(255, 255, 255),
            surface: ThemeColor::rgb(43, 43, 43),
            surface_foreground: ThemeColor::rgb(255, 255, 255),

            primary: ThemeColor::rgb(76, 194, 255), // dark accent #4CC2FF
            primary_foreground: ThemeColor::rgb(0, 0, 0),
            secondary: ThemeColor::rgb(53, 53, 53),
            secondary_foreground: ThemeColor::rgb(255, 255, 255),
            accent: ThemeColor::rgb(20, 40, 60),
            accent_foreground: ThemeColor::rgb(76, 194, 255),
            muted: ThemeColor::rgb(43, 43, 43),
            muted_foreground: ThemeColor::rgb(160, 160, 160),

            border: ThemeColor::rgb(64, 64, 64),
            input: ThemeColor::rgb(64, 64, 64),
            ring: ThemeColor::rgb(76, 194, 255),

            success: ThemeColor::rgb(108, 203, 95),
            warning: ThemeColor::rgb(252, 225, 0),
            danger: ThemeColor::rgb(255, 153, 164),
            info: ThemeColor::rgb(76, 194, 255),

            success_soft: ThemeColor::rgb(13, 40, 24),
            warning_soft: ThemeColor::rgb(45, 31, 4),
            danger_soft: ThemeColor::rgb(45, 10, 10),
            info_soft: ThemeColor::rgb(13, 27, 62),

            surface_elevated: ThemeColor::rgb(53, 53, 53),
            surface_overlay: ThemeColor::rgb(64, 64, 64),
            surface_tooltip: ThemeColor::rgb(64, 64, 64),

            primary_hover: ThemeColor::rgb(102, 205, 255),
            primary_pressed: ThemeColor::rgb(0, 120, 212),
            danger_hover: ThemeColor::rgb(255, 179, 187),
            danger_pressed: ThemeColor::rgb(232, 17, 35),
            ghost_hover: ThemeColor::rgb(53, 53, 53),
            ghost_pressed: ThemeColor::rgb(64, 64, 64),

            disabled_bg: ThemeColor::rgb(53, 53, 53),
            disabled_text: ThemeColor::rgb(102, 102, 102),
        };
        let mut t = Theme::from_colors(ThemeMode::Dark, colors, shadow_ladder(0.30, 0.35, 0.45, 0.55));
        t.radii.sm = 2.0;
        t.radii.md = 2.0;
        t.radii.lg = 4.0;
        t.radii.xl = 4.0;
        t
    }
}
