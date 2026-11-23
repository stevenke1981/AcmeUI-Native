//! Solarized theme pack — Ethan Schoonover's precision light/dark palette.

use crate::packs::{ThemePack, shadow_ladder};
use crate::{ColorTokens, Theme, ThemeColor, ThemeMode};

/// Solarized design pack.
pub struct SolarizedPack;

impl ThemePack for SolarizedPack {
    fn name(&self) -> &'static str {
        "solarized"
    }

    fn light(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(253, 246, 227),       // base3
            foreground: ThemeColor::rgb(80, 97, 104), // base00 darkened to #506168: AA on base2 and ghost_pressed, 1.1:1 above muted (062 review)
            surface: ThemeColor::rgb(238, 232, 213),  // base2
            surface_foreground: ThemeColor::rgb(80, 97, 104), // = foreground (062 review)

            primary: ThemeColor::rgb(32, 116, 176), // blue #268BD2 darkened to #2074B0 for AA base3 text (062)
            primary_foreground: ThemeColor::rgb(253, 246, 227),
            secondary: ThemeColor::rgb(32, 124, 117), // cyan #2AA198 darkened to #207C75 for AA base3 text (062)
            secondary_foreground: ThemeColor::rgb(253, 246, 227),
            accent: ThemeColor::rgb(238, 235, 245),
            accent_foreground: ThemeColor::rgb(95, 99, 172), // violet, darkened for AA on accent (060)
            selection_foreground: ThemeColor::rgb(95, 99, 172),
            muted: ThemeColor::rgb(238, 232, 213),
            muted_foreground: ThemeColor::rgb(92, 106, 106), // base1 darkened to #5C6A6A for AA on base2 (062)

            border: ThemeColor::rgb(238, 232, 213),
            input: ThemeColor::rgb(238, 232, 213),
            ring: ThemeColor::rgb(38, 139, 210),

            success: ThemeColor::rgb(121, 139, 0), // green darkened to #798B00 for 3:1 graphic (062)
            warning: ThemeColor::rgb(164, 124, 0), // yellow darkened to #A47C00 for 3:1 graphic (062)
            danger: ThemeColor::rgb(215, 39, 36), // red darkened to #D72724, AA base3 ink (062 review)
            info: ThemeColor::rgb(38, 145, 137),  // cyan darkened to #269189 for 3:1 graphic (062)

            success_soft: ThemeColor::rgb(238, 243, 208),
            warning_soft: ThemeColor::rgb(245, 238, 205),
            danger_soft: ThemeColor::rgb(250, 225, 220),
            info_soft: ThemeColor::rgb(222, 240, 238),

            surface_elevated: ThemeColor::rgb(253, 246, 227),
            surface_overlay: ThemeColor::rgb(253, 246, 227),
            surface_tooltip: ThemeColor::rgb(7, 54, 66), // base02

            primary_hover: ThemeColor::rgb(29, 105, 159), // darker under base3 text (062 review)
            primary_pressed: ThemeColor::rgb(26, 95, 144), // darker again (062 review)
            danger_hover: ThemeColor::rgb(190, 40, 37), // darker: old pressed #BE2825 (062 review)
            danger_pressed: ThemeColor::rgb(173, 36, 34), // darker again (062 review)
            ghost_hover: ThemeColor::rgb(238, 232, 213),
            ghost_pressed: ThemeColor::rgb(225, 218, 198),

            disabled_bg: ThemeColor::rgb(238, 232, 213),
            disabled_text: ThemeColor::rgb(147, 161, 161),
            disabled_foreground: ThemeColor::rgb(96, 105, 105),
        };
        Theme::from_colors(
            ThemeMode::Light,
            colors,
            shadow_ladder(0.05, 0.07, 0.09, 0.12),
        )
    }

    fn dark(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(0, 43, 54),             // base03
            foreground: ThemeColor::rgb(157, 170, 170), // base1 lightened to #9DAAAA: 1.1:1 above muted (062 review)
            surface: ThemeColor::rgb(7, 54, 66),        // base02
            surface_foreground: ThemeColor::rgb(157, 170, 170), // = foreground (062 review)

            primary: ThemeColor::rgb(38, 139, 210), // blue
            primary_foreground: ThemeColor::rgb(0, 32, 41), // base03 darkened to #002029, AA on blue (062)
            secondary: ThemeColor::rgb(7, 54, 66),
            secondary_foreground: ThemeColor::rgb(147, 161, 161), // base1 (062)
            accent: ThemeColor::rgb(7, 54, 66),
            accent_foreground: ThemeColor::rgb(45, 171, 161), // cyan lightened to #2DABA1 for AA (062)
            selection_foreground: ThemeColor::rgb(45, 171, 161),
            muted: ThemeColor::rgb(7, 54, 66),
            muted_foreground: ThemeColor::rgb(147, 161, 161), // base1, AA on base02 (062)

            border: ThemeColor::rgb(7, 54, 66),
            input: ThemeColor::rgb(88, 110, 117),
            ring: ThemeColor::rgb(38, 139, 210),

            success: ThemeColor::rgb(133, 153, 0), // green
            warning: ThemeColor::rgb(181, 137, 0), // yellow
            danger: ThemeColor::rgb(226, 87, 84), // red lightened to #E25754, AA dark ink (062 review)
            info: ThemeColor::rgb(42, 161, 152),  // cyan

            success_soft: ThemeColor::rgb(10, 45, 30),
            warning_soft: ThemeColor::rgb(45, 40, 15),
            danger_soft: ThemeColor::rgb(50, 25, 25),
            info_soft: ThemeColor::rgb(12, 45, 50),

            surface_elevated: ThemeColor::rgb(7, 54, 66),
            surface_overlay: ThemeColor::rgb(15, 60, 72),
            surface_tooltip: ThemeColor::rgb(88, 110, 117),

            primary_hover: ThemeColor::rgb(60, 155, 220),
            primary_pressed: ThemeColor::rgb(82, 167, 224), // lighter than hover, away from dark ink (062 review)
            danger_hover: ThemeColor::rgb(230, 108, 105),   // lighter (062 review)
            danger_pressed: ThemeColor::rgb(233, 126, 123), // lighter again (062 review)
            ghost_hover: ThemeColor::rgb(7, 54, 66),
            ghost_pressed: ThemeColor::rgb(15, 60, 72),

            disabled_bg: ThemeColor::rgb(7, 54, 66),
            disabled_text: ThemeColor::rgb(88, 110, 117),
            disabled_foreground: ThemeColor::rgb(143, 158, 163),
        };
        Theme::from_colors(
            ThemeMode::Dark,
            colors,
            shadow_ladder(0.30, 0.35, 0.45, 0.55),
        )
    }
}
