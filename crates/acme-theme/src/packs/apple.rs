//! Apple (macOS / iOS) theme pack — system blue, restrained grays, rounded geometry.

use crate::packs::{shadow_ladder, ThemePack};
use crate::{ColorTokens, Theme, ThemeColor, ThemeMode};

/// Apple design system pack.
pub struct ApplePack;

impl ThemePack for ApplePack {
    fn name(&self) -> &'static str {
        "apple"
    }

    fn light(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(242, 242, 247), // systemGroupedBackground
            foreground: ThemeColor::rgb(0, 0, 0),
            surface: ThemeColor::rgb(255, 255, 255), // secondarySystemGroupedBackground
            surface_foreground: ThemeColor::rgb(0, 0, 0),

            primary: ThemeColor::rgb(0, 122, 255), // systemBlue #007AFF
            primary_foreground: ThemeColor::rgb(255, 255, 255),
            secondary: ThemeColor::rgb(229, 229, 234), // systemGray5
            secondary_foreground: ThemeColor::rgb(28, 28, 30),
            accent: ThemeColor::rgb(224, 238, 255),
            accent_foreground: ThemeColor::rgb(0, 122, 255),
            muted: ThemeColor::rgb(242, 242, 247),
            muted_foreground: ThemeColor::rgb(142, 142, 147), // systemGray

            border: ThemeColor::rgb(229, 229, 234),
            input: ThemeColor::rgb(229, 229, 234),
            ring: ThemeColor::rgb(0, 122, 255),

            success: ThemeColor::rgb(52, 199, 89),  // systemGreen
            warning: ThemeColor::rgb(255, 149, 0),  // systemOrange
            danger: ThemeColor::rgb(255, 59, 48),   // systemRed
            info: ThemeColor::rgb(90, 200, 250),    // systemTeal

            success_soft: ThemeColor::rgb(232, 247, 236),
            warning_soft: ThemeColor::rgb(255, 242, 224),
            danger_soft: ThemeColor::rgb(255, 233, 231),
            info_soft: ThemeColor::rgb(233, 246, 254),

            surface_elevated: ThemeColor::rgb(255, 255, 255),
            surface_overlay: ThemeColor::rgb(255, 255, 255),
            surface_tooltip: ThemeColor::rgb(28, 28, 30),

            primary_hover: ThemeColor::rgb(0, 102, 214),
            primary_pressed: ThemeColor::rgb(0, 82, 173),
            danger_hover: ThemeColor::rgb(214, 49, 40),
            danger_pressed: ThemeColor::rgb(173, 40, 32),
            ghost_hover: ThemeColor::rgb(229, 229, 234),
            ghost_pressed: ThemeColor::rgb(209, 209, 214),

            disabled_bg: ThemeColor::rgb(229, 229, 234),
            disabled_text: ThemeColor::rgb(199, 199, 204),
        };
        let mut t = Theme::from_colors(ThemeMode::Light, colors, shadow_ladder(0.06, 0.08, 0.10, 0.14));
        // Apple uses generously rounded corners.
        t.radii.sm = 6.0;
        t.radii.md = 10.0;
        t.radii.lg = 14.0;
        t.radii.xl = 20.0;
        t
    }

    fn dark(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(0, 0, 0), // systemBackground (dark)
            foreground: ThemeColor::rgb(255, 255, 255),
            surface: ThemeColor::rgb(28, 28, 30), // secondarySystemBackground
            surface_foreground: ThemeColor::rgb(255, 255, 255),

            primary: ThemeColor::rgb(10, 132, 255), // systemBlue (dark) #0A84FF
            primary_foreground: ThemeColor::rgb(255, 255, 255),
            secondary: ThemeColor::rgb(44, 44, 46), // systemGray5 (dark)
            secondary_foreground: ThemeColor::rgb(235, 235, 245),
            accent: ThemeColor::rgb(20, 40, 69),
            accent_foreground: ThemeColor::rgb(10, 132, 255),
            muted: ThemeColor::rgb(28, 28, 30),
            muted_foreground: ThemeColor::rgb(142, 142, 147),

            border: ThemeColor::rgb(56, 56, 58),
            input: ThemeColor::rgb(56, 56, 58),
            ring: ThemeColor::rgb(10, 132, 255),

            success: ThemeColor::rgb(48, 209, 88),  // systemGreen (dark)
            warning: ThemeColor::rgb(255, 159, 10), // systemOrange (dark)
            danger: ThemeColor::rgb(255, 69, 58),   // systemRed (dark)
            info: ThemeColor::rgb(100, 210, 255),   // systemTeal (dark)

            success_soft: ThemeColor::rgb(13, 40, 24),
            warning_soft: ThemeColor::rgb(45, 31, 4),
            danger_soft: ThemeColor::rgb(45, 10, 10),
            info_soft: ThemeColor::rgb(13, 27, 62),

            surface_elevated: ThemeColor::rgb(44, 44, 46),
            surface_overlay: ThemeColor::rgb(58, 58, 60),
            surface_tooltip: ThemeColor::rgb(56, 56, 58),

            primary_hover: ThemeColor::rgb(64, 156, 255),
            primary_pressed: ThemeColor::rgb(0, 102, 214),
            danger_hover: ThemeColor::rgb(255, 105, 97),
            danger_pressed: ThemeColor::rgb(214, 49, 40),
            ghost_hover: ThemeColor::rgb(44, 44, 46),
            ghost_pressed: ThemeColor::rgb(58, 58, 60),

            disabled_bg: ThemeColor::rgb(44, 44, 46),
            disabled_text: ThemeColor::rgb(72, 72, 74),
        };
        let mut t = Theme::from_colors(ThemeMode::Dark, colors, shadow_ladder(0.30, 0.35, 0.45, 0.55));
        t.radii.sm = 6.0;
        t.radii.md = 10.0;
        t.radii.lg = 14.0;
        t.radii.xl = 20.0;
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apple_primary_is_system_blue() {
        let light = ApplePack.light();
        assert_eq!(light.colors.primary, ThemeColor::rgb(0, 122, 255));
    }
}
