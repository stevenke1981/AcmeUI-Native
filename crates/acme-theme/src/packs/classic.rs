//! Classic desktop (WinForms-style) theme pack.
//!
//! A flat, square-cornered light palette: lavender form background,
//! steel-blue table headers, a muted indigo chrome band and no shadows.
//! Light values are based on classic desktop form colors; the
//! dark variant is a conservative companion that only needs to satisfy the
//! shared validation rules.

use crate::packs::{ThemePack, shadow_ladder};
use crate::{ColorTokens, Theme, ThemeColor, ThemeMode};

/// Classic desktop (WinForms-style) pack.
pub struct ClassicPack;

impl ThemePack for ClassicPack {
    fn name(&self) -> &'static str {
        "classic"
    }

    fn light(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(230, 230, 250), // #E6E6FA form background
            foreground: ThemeColor::rgb(0, 0, 0),
            surface: ThemeColor::rgb(240, 240, 240), // #F0F0F0 control face
            surface_foreground: ThemeColor::rgb(0, 0, 0),

            primary: ThemeColor::rgb(96, 101, 179), // #6065B3 chrome band
            primary_foreground: ThemeColor::rgb(255, 255, 255),
            secondary: ThemeColor::rgb(242, 233, 212), // #F2E9D4 button face
            secondary_foreground: ThemeColor::rgb(0, 0, 0),
            accent: ThemeColor::rgb(156, 158, 255), // #9C9EFF selected group
            accent_foreground: ThemeColor::rgb(0, 0, 0),
            selection_foreground: ThemeColor::rgb(0, 0, 0),
            muted: ThemeColor::rgb(245, 245, 245), // #F5F5F5 workspace
            muted_foreground: ThemeColor::rgb(90, 90, 90), // #5A5A5A

            border: ThemeColor::rgb(180, 180, 180), // #B4B4B4
            input: ThemeColor::rgb(117, 117, 117),  // #757575 sunken edge
            ring: ThemeColor::rgb(0, 120, 215),     // #0078D7 focus

            success: ThemeColor::rgb(16, 124, 16),
            warning: ThemeColor::rgb(157, 93, 0),
            danger: ThemeColor::rgb(165, 42, 42), // #A52A2A
            info: ThemeColor::rgb(0, 0, 128),     // #000080

            success_soft: ThemeColor::rgb(223, 246, 221),
            warning_soft: ThemeColor::rgb(255, 244, 206),
            danger_soft: ThemeColor::rgb(253, 231, 233),
            info_soft: ThemeColor::rgb(240, 248, 255), // #F0F8FF table caption

            surface_elevated: ThemeColor::rgb(255, 255, 255),
            surface_overlay: ThemeColor::rgb(255, 255, 255),
            surface_tooltip: ThemeColor::rgb(43, 43, 43),

            primary_hover: ThemeColor::rgb(80, 85, 154), // darker: white ink must gain contrast
            primary_pressed: ThemeColor::rgb(62, 67, 134),
            danger_hover: ThemeColor::rgb(142, 34, 34),
            danger_pressed: ThemeColor::rgb(112, 27, 27),
            ghost_hover: ThemeColor::rgb(216, 216, 240),
            ghost_pressed: ThemeColor::rgb(196, 196, 232),

            disabled_bg: ThemeColor::rgb(240, 240, 240),
            disabled_text: ThemeColor::rgb(109, 109, 109),
            disabled_foreground: ThemeColor::rgb(109, 109, 109),
        };
        Self::finish(ThemeMode::Light, colors)
    }

    fn dark(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(43, 43, 58), // #2B2B3A
            foreground: ThemeColor::rgb(255, 255, 255),
            surface: ThemeColor::rgb(51, 51, 74),
            surface_foreground: ThemeColor::rgb(255, 255, 255),

            // Lifted from #6065B3 so the primary clears 3:1 on the dark background.
            primary: ThemeColor::rgb(143, 148, 224),
            primary_foreground: ThemeColor::rgb(0, 0, 0),
            secondary: ThemeColor::rgb(61, 61, 85),
            secondary_foreground: ThemeColor::rgb(255, 255, 255),
            accent: ThemeColor::rgb(58, 64, 128),
            accent_foreground: ThemeColor::rgb(255, 255, 255),
            selection_foreground: ThemeColor::rgb(255, 255, 255),
            muted: ThemeColor::rgb(51, 51, 74),
            muted_foreground: ThemeColor::rgb(176, 176, 192),

            border: ThemeColor::rgb(74, 74, 96),
            input: ThemeColor::rgb(90, 90, 112),
            ring: ThemeColor::rgb(156, 158, 255),

            success: ThemeColor::rgb(108, 203, 95),
            warning: ThemeColor::rgb(252, 225, 0),
            danger: ThemeColor::rgb(255, 153, 164),
            info: ThemeColor::rgb(156, 158, 255),

            success_soft: ThemeColor::rgb(19, 40, 26),
            warning_soft: ThemeColor::rgb(61, 51, 0),
            danger_soft: ThemeColor::rgb(74, 31, 36),
            info_soft: ThemeColor::rgb(42, 45, 85),

            surface_elevated: ThemeColor::rgb(58, 58, 82),
            surface_overlay: ThemeColor::rgb(66, 66, 96),
            surface_tooltip: ThemeColor::rgb(68, 68, 68),

            primary_hover: ThemeColor::rgb(163, 168, 234),
            primary_pressed: ThemeColor::rgb(185, 189, 242),
            danger_hover: ThemeColor::rgb(255, 179, 187),
            danger_pressed: ThemeColor::rgb(255, 208, 214),
            ghost_hover: ThemeColor::rgb(61, 61, 85),
            ghost_pressed: ThemeColor::rgb(72, 72, 96),

            disabled_bg: ThemeColor::rgb(51, 51, 74),
            disabled_text: ThemeColor::rgb(109, 109, 128),
            disabled_foreground: ThemeColor::rgb(168, 168, 184),
        };
        Self::finish(ThemeMode::Dark, colors)
    }
}

impl ClassicPack {
    /// Flat geometry: square corners (≤2px) and no shadows.
    fn finish(mode: ThemeMode, colors: ColorTokens) -> Theme {
        let mut t = Theme::from_colors(mode, colors, shadow_ladder(0.0, 0.0, 0.0, 0.0));
        t.radii.sm = 0.0;
        t.radii.md = 2.0;
        t.radii.lg = 2.0;
        t.radii.xl = 2.0;
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgb_hex(value: u32) -> ThemeColor {
        ThemeColor::rgb((value >> 16) as u8, (value >> 8) as u8, value as u8)
    }

    #[test]
    fn classic_light_slots_are_pinned() {
        let c = ClassicPack.light().colors;
        let expected = [
            (c.background, 0xE6E6FA),
            (c.foreground, 0x000000),
            (c.surface, 0xF0F0F0),
            (c.surface_foreground, 0x000000),
            (c.primary, 0x6065B3),
            (c.primary_foreground, 0xFFFFFF),
            (c.secondary, 0xF2E9D4),
            (c.secondary_foreground, 0x000000),
            (c.accent, 0x9C9EFF),
            (c.accent_foreground, 0x000000),
            (c.selection_foreground, 0x000000),
            (c.muted, 0xF5F5F5),
            (c.muted_foreground, 0x5A5A5A),
            (c.border, 0xB4B4B4),
            (c.input, 0x757575),
            (c.ring, 0x0078D7),
            (c.success, 0x107C10),
            (c.warning, 0x9D5D00),
            (c.danger, 0xA52A2A),
            (c.info, 0x000080),
            (c.success_soft, 0xDFF6DD),
            (c.warning_soft, 0xFFF4CE),
            (c.danger_soft, 0xFDE7E9),
            (c.info_soft, 0xF0F8FF),
            (c.surface_elevated, 0xFFFFFF),
            (c.surface_overlay, 0xFFFFFF),
            (c.surface_tooltip, 0x2B2B2B),
            (c.primary_hover, 0x50559A),
            (c.primary_pressed, 0x3E4386),
            (c.danger_hover, 0x8E2222),
            (c.danger_pressed, 0x701B1B),
            (c.ghost_hover, 0xD8D8F0),
            (c.ghost_pressed, 0xC4C4E8),
            (c.disabled_bg, 0xF0F0F0),
            (c.disabled_text, 0x6D6D6D),
            (c.disabled_foreground, 0x6D6D6D),
        ];
        for (index, (actual, hex)) in expected.into_iter().enumerate() {
            assert_eq!(
                actual,
                rgb_hex(hex),
                "light slot #{index} should be #{hex:06X}"
            );
        }
    }

    #[test]
    fn classic_is_square_and_shadowless() {
        for theme in [ClassicPack.light(), ClassicPack.dark()] {
            assert!(theme.radii.sm <= 2.0 && theme.radii.xl <= 2.0);
            assert_eq!(theme.shadows.md.color.alpha, 0.0);
            assert_eq!(theme.shadows.xl.color.alpha, 0.0);
        }
    }
}
