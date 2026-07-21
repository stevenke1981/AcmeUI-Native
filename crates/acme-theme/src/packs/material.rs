//! Material Design 3 theme pack — baseline purple, tonal surfaces.

use crate::packs::{shadow_ladder, ThemePack};
use crate::{ColorTokens, Theme, ThemeColor, ThemeMode};

/// Material Design 3 (baseline) design pack.
pub struct MaterialPack;

impl ThemePack for MaterialPack {
    fn name(&self) -> &'static str {
        "material"
    }

    fn light(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(255, 251, 254), // M3 surface
            foreground: ThemeColor::rgb(28, 27, 31),
            surface: ThemeColor::rgb(255, 251, 254),
            surface_foreground: ThemeColor::rgb(28, 27, 31),

            primary: ThemeColor::rgb(103, 80, 164), // M3 primary #6750A4
            primary_foreground: ThemeColor::rgb(255, 255, 255),
            secondary: ThemeColor::rgb(232, 222, 248), // secondaryContainer
            secondary_foreground: ThemeColor::rgb(29, 25, 43),
            accent: ThemeColor::rgb(232, 222, 248),
            accent_foreground: ThemeColor::rgb(103, 80, 164),
            muted: ThemeColor::rgb(231, 224, 236),
            muted_foreground: ThemeColor::rgb(73, 69, 79),

            border: ThemeColor::rgb(202, 196, 208),
            input: ThemeColor::rgb(202, 196, 208),
            ring: ThemeColor::rgb(103, 80, 164),

            success: ThemeColor::rgb(56, 119, 55),
            warning: ThemeColor::rgb(125, 83, 6),
            danger: ThemeColor::rgb(179, 38, 30), // M3 error #B3261E
            info: ThemeColor::rgb(103, 80, 164),

            success_soft: ThemeColor::rgb(220, 239, 215),
            warning_soft: ThemeColor::rgb(255, 242, 216),
            danger_soft: ThemeColor::rgb(249, 222, 220),
            info_soft: ThemeColor::rgb(232, 222, 248),

            surface_elevated: ThemeColor::rgb(255, 251, 254),
            surface_overlay: ThemeColor::rgb(255, 251, 254),
            surface_tooltip: ThemeColor::rgb(49, 48, 51),

            primary_hover: ThemeColor::rgb(88, 68, 140),
            primary_pressed: ThemeColor::rgb(73, 56, 117),
            danger_hover: ThemeColor::rgb(153, 32, 25),
            danger_pressed: ThemeColor::rgb(127, 27, 21),
            ghost_hover: ThemeColor::rgb(232, 222, 248),
            ghost_pressed: ThemeColor::rgb(214, 202, 233),

            disabled_bg: ThemeColor::rgb(231, 224, 236),
            disabled_text: ThemeColor::rgb(150, 144, 156),
        };
        let mut t = Theme::from_colors(ThemeMode::Light, colors, shadow_ladder(0.05, 0.08, 0.11, 0.15));
        // M3 uses fully rounded pill shapes for many components.
        t.radii.sm = 8.0;
        t.radii.md = 12.0;
        t.radii.lg = 16.0;
        t.radii.xl = 28.0;
        t
    }

    fn dark(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(28, 27, 31), // M3 surface (dark)
            foreground: ThemeColor::rgb(230, 225, 229),
            surface: ThemeColor::rgb(28, 27, 31),
            surface_foreground: ThemeColor::rgb(230, 225, 229),

            primary: ThemeColor::rgb(208, 188, 255), // M3 primary (dark) #D0BCFF
            primary_foreground: ThemeColor::rgb(56, 30, 114),
            secondary: ThemeColor::rgb(74, 66, 92),
            secondary_foreground: ThemeColor::rgb(232, 222, 248),
            accent: ThemeColor::rgb(74, 66, 92),
            accent_foreground: ThemeColor::rgb(208, 188, 255),
            muted: ThemeColor::rgb(49, 48, 51),
            muted_foreground: ThemeColor::rgb(202, 196, 208),

            border: ThemeColor::rgb(73, 69, 79),
            input: ThemeColor::rgb(73, 69, 79),
            ring: ThemeColor::rgb(208, 188, 255),

            success: ThemeColor::rgb(138, 222, 132),
            warning: ThemeColor::rgb(255, 180, 105),
            danger: ThemeColor::rgb(242, 184, 181), // M3 error (dark)
            info: ThemeColor::rgb(208, 188, 255),

            success_soft: ThemeColor::rgb(13, 40, 24),
            warning_soft: ThemeColor::rgb(45, 31, 4),
            danger_soft: ThemeColor::rgb(45, 10, 10),
            info_soft: ThemeColor::rgb(30, 22, 50),

            surface_elevated: ThemeColor::rgb(49, 48, 51),
            surface_overlay: ThemeColor::rgb(59, 57, 62),
            surface_tooltip: ThemeColor::rgb(73, 69, 79),

            primary_hover: ThemeColor::rgb(218, 203, 255),
            primary_pressed: ThemeColor::rgb(103, 80, 164),
            danger_hover: ThemeColor::rgb(247, 200, 197),
            danger_pressed: ThemeColor::rgb(179, 38, 30),
            ghost_hover: ThemeColor::rgb(74, 66, 92),
            ghost_pressed: ThemeColor::rgb(89, 80, 107),

            disabled_bg: ThemeColor::rgb(49, 48, 51),
            disabled_text: ThemeColor::rgb(90, 87, 95),
        };
        let mut t = Theme::from_colors(ThemeMode::Dark, colors, shadow_ladder(0.30, 0.35, 0.45, 0.55));
        t.radii.sm = 8.0;
        t.radii.md = 12.0;
        t.radii.lg = 16.0;
        t.radii.xl = 28.0;
        t
    }
}
