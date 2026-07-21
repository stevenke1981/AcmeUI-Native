//! Solarized theme pack — Ethan Schoonover's precision light/dark palette.

use crate::packs::{shadow_ladder, ThemePack};
use crate::{ColorTokens, Theme, ThemeColor, ThemeMode};

/// Solarized design pack.
pub struct SolarizedPack;

impl ThemePack for SolarizedPack {
    fn name(&self) -> &'static str {
        "solarized"
    }

    fn light(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(253, 246, 227), // base3
            foreground: ThemeColor::rgb(101, 123, 131), // base00
            surface: ThemeColor::rgb(238, 232, 213),    // base2
            surface_foreground: ThemeColor::rgb(101, 123, 131),

            primary: ThemeColor::rgb(38, 139, 210), // blue
            primary_foreground: ThemeColor::rgb(253, 246, 227),
            secondary: ThemeColor::rgb(42, 161, 152), // cyan
            secondary_foreground: ThemeColor::rgb(253, 246, 227),
            accent: ThemeColor::rgb(238, 235, 245),
            accent_foreground: ThemeColor::rgb(108, 113, 196), // violet
            muted: ThemeColor::rgb(238, 232, 213),
            muted_foreground: ThemeColor::rgb(147, 161, 161), // base1

            border: ThemeColor::rgb(238, 232, 213),
            input: ThemeColor::rgb(238, 232, 213),
            ring: ThemeColor::rgb(38, 139, 210),

            success: ThemeColor::rgb(133, 153, 0), // green
            warning: ThemeColor::rgb(181, 137, 0), // yellow
            danger: ThemeColor::rgb(220, 50, 47),  // red
            info: ThemeColor::rgb(42, 161, 152),   // cyan

            success_soft: ThemeColor::rgb(238, 243, 208),
            warning_soft: ThemeColor::rgb(245, 238, 205),
            danger_soft: ThemeColor::rgb(250, 225, 220),
            info_soft: ThemeColor::rgb(222, 240, 238),

            surface_elevated: ThemeColor::rgb(253, 246, 227),
            surface_overlay: ThemeColor::rgb(253, 246, 227),
            surface_tooltip: ThemeColor::rgb(7, 54, 66), // base02

            primary_hover: ThemeColor::rgb(60, 155, 220),
            primary_pressed: ThemeColor::rgb(28, 110, 170),
            danger_hover: ThemeColor::rgb(228, 80, 77),
            danger_pressed: ThemeColor::rgb(190, 40, 37),
            ghost_hover: ThemeColor::rgb(238, 232, 213),
            ghost_pressed: ThemeColor::rgb(225, 218, 198),

            disabled_bg: ThemeColor::rgb(238, 232, 213),
            disabled_text: ThemeColor::rgb(147, 161, 161),
        };
        Theme::from_colors(ThemeMode::Light, colors, shadow_ladder(0.05, 0.07, 0.09, 0.12))
    }

    fn dark(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(0, 43, 54),    // base03
            foreground: ThemeColor::rgb(131, 148, 150), // base0
            surface: ThemeColor::rgb(7, 54, 66),       // base02
            surface_foreground: ThemeColor::rgb(131, 148, 150),

            primary: ThemeColor::rgb(38, 139, 210), // blue
            primary_foreground: ThemeColor::rgb(253, 246, 227),
            secondary: ThemeColor::rgb(7, 54, 66),
            secondary_foreground: ThemeColor::rgb(131, 148, 150),
            accent: ThemeColor::rgb(7, 54, 66),
            accent_foreground: ThemeColor::rgb(42, 161, 152), // cyan
            muted: ThemeColor::rgb(7, 54, 66),
            muted_foreground: ThemeColor::rgb(88, 110, 117), // base01

            border: ThemeColor::rgb(7, 54, 66),
            input: ThemeColor::rgb(88, 110, 117),
            ring: ThemeColor::rgb(38, 139, 210),

            success: ThemeColor::rgb(133, 153, 0), // green
            warning: ThemeColor::rgb(181, 137, 0), // yellow
            danger: ThemeColor::rgb(220, 50, 47),  // red
            info: ThemeColor::rgb(42, 161, 152),   // cyan

            success_soft: ThemeColor::rgb(10, 45, 30),
            warning_soft: ThemeColor::rgb(45, 40, 15),
            danger_soft: ThemeColor::rgb(50, 25, 25),
            info_soft: ThemeColor::rgb(12, 45, 50),

            surface_elevated: ThemeColor::rgb(7, 54, 66),
            surface_overlay: ThemeColor::rgb(15, 60, 72),
            surface_tooltip: ThemeColor::rgb(88, 110, 117),

            primary_hover: ThemeColor::rgb(60, 155, 220),
            primary_pressed: ThemeColor::rgb(28, 110, 170),
            danger_hover: ThemeColor::rgb(228, 80, 77),
            danger_pressed: ThemeColor::rgb(190, 40, 37),
            ghost_hover: ThemeColor::rgb(7, 54, 66),
            ghost_pressed: ThemeColor::rgb(15, 60, 72),

            disabled_bg: ThemeColor::rgb(7, 54, 66),
            disabled_text: ThemeColor::rgb(88, 110, 117),
        };
        Theme::from_colors(ThemeMode::Dark, colors, shadow_ladder(0.30, 0.35, 0.45, 0.55))
    }
}
