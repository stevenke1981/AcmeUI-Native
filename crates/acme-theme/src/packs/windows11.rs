//! Windows 11 (Mica / WinUI 3) theme pack — accent blue, rounded geometry.

use crate::packs::{shadow_ladder, ThemePack};
use crate::{ColorTokens, Theme, ThemeColor, ThemeMode};

/// Windows 11 WinUI 3 design pack.
pub struct Windows11Pack;

impl ThemePack for Windows11Pack {
    fn name(&self) -> &'static str {
        "windows11"
    }

    fn light(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(243, 243, 243), // Mica base
            foreground: ThemeColor::rgb(26, 26, 26),
            surface: ThemeColor::rgb(251, 251, 251), // Card background
            surface_foreground: ThemeColor::rgb(26, 26, 26),

            primary: ThemeColor::rgb(0, 103, 192), // Win11 accent #0067C0
            primary_foreground: ThemeColor::rgb(255, 255, 255),
            secondary: ThemeColor::rgb(238, 238, 238),
            secondary_foreground: ThemeColor::rgb(26, 26, 26),
            accent: ThemeColor::rgb(217, 232, 247),
            accent_foreground: ThemeColor::rgb(0, 103, 192),
            muted: ThemeColor::rgb(243, 243, 243),
            muted_foreground: ThemeColor::rgb(96, 96, 96),

            border: ThemeColor::rgb(234, 234, 234),
            input: ThemeColor::rgb(234, 234, 234),
            ring: ThemeColor::rgb(0, 103, 192),

            success: ThemeColor::rgb(15, 123, 15),
            warning: ThemeColor::rgb(157, 93, 0),
            danger: ThemeColor::rgb(196, 43, 28),
            info: ThemeColor::rgb(0, 103, 192),

            success_soft: ThemeColor::rgb(223, 246, 221),
            warning_soft: ThemeColor::rgb(255, 244, 206),
            danger_soft: ThemeColor::rgb(253, 231, 233),
            info_soft: ThemeColor::rgb(217, 232, 247),

            surface_elevated: ThemeColor::rgb(255, 255, 255),
            surface_overlay: ThemeColor::rgb(255, 255, 255),
            surface_tooltip: ThemeColor::rgb(44, 44, 44),

            primary_hover: ThemeColor::rgb(25, 117, 199),
            primary_pressed: ThemeColor::rgb(0, 84, 153),
            danger_hover: ThemeColor::rgb(180, 39, 25),
            danger_pressed: ThemeColor::rgb(160, 35, 22),
            ghost_hover: ThemeColor::rgb(238, 238, 238),
            ghost_pressed: ThemeColor::rgb(222, 222, 222),

            disabled_bg: ThemeColor::rgb(238, 238, 238),
            disabled_text: ThemeColor::rgb(160, 160, 160),
        };
        let mut t = Theme::from_colors(ThemeMode::Light, colors, shadow_ladder(0.05, 0.08, 0.10, 0.14));
        // Windows 11 uses 4-8px rounded corners (WinUI).
        t.radii.sm = 4.0;
        t.radii.md = 4.0;
        t.radii.lg = 8.0;
        t.radii.xl = 8.0;
        t
    }

    fn dark(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(32, 32, 32), // Mica base (dark)
            foreground: ThemeColor::rgb(255, 255, 255),
            surface: ThemeColor::rgb(43, 43, 43), // Card background (dark)
            surface_foreground: ThemeColor::rgb(255, 255, 255),

            primary: ThemeColor::rgb(76, 194, 255), // dark accent #4CC2FF
            primary_foreground: ThemeColor::rgb(0, 0, 0),
            secondary: ThemeColor::rgb(53, 53, 53),
            secondary_foreground: ThemeColor::rgb(255, 255, 255),
            accent: ThemeColor::rgb(20, 40, 60),
            accent_foreground: ThemeColor::rgb(76, 194, 255),
            muted: ThemeColor::rgb(43, 43, 43),
            muted_foreground: ThemeColor::rgb(160, 160, 160),

            border: ThemeColor::rgb(61, 61, 61),
            input: ThemeColor::rgb(61, 61, 61),
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
            surface_overlay: ThemeColor::rgb(61, 61, 61),
            surface_tooltip: ThemeColor::rgb(61, 61, 61),

            primary_hover: ThemeColor::rgb(102, 205, 255),
            primary_pressed: ThemeColor::rgb(0, 103, 192),
            danger_hover: ThemeColor::rgb(255, 179, 187),
            danger_pressed: ThemeColor::rgb(196, 43, 28),
            ghost_hover: ThemeColor::rgb(53, 53, 53),
            ghost_pressed: ThemeColor::rgb(61, 61, 61),

            disabled_bg: ThemeColor::rgb(53, 53, 53),
            disabled_text: ThemeColor::rgb(102, 102, 102),
        };
        let mut t = Theme::from_colors(ThemeMode::Dark, colors, shadow_ladder(0.30, 0.35, 0.45, 0.55));
        t.radii.sm = 4.0;
        t.radii.md = 4.0;
        t.radii.lg = 8.0;
        t.radii.xl = 8.0;
        t
    }
}
