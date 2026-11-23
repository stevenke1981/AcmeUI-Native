//! High-contrast accessibility theme pack — WCAG AAA text contrast on pure surfaces.

use crate::packs::{ThemePack, shadow_ladder};
use crate::{ColorTokens, Theme, ThemeColor, ThemeMode};

/// High-contrast accessibility theme pack (WCAG AA/AAA compliant).
pub struct HighContrastPack;

impl ThemePack for HighContrastPack {
    fn name(&self) -> &'static str {
        "high-contrast"
    }

    fn light(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(255, 255, 255),
            foreground: ThemeColor::rgb(0, 0, 0),
            surface: ThemeColor::rgb(245, 245, 245),
            surface_foreground: ThemeColor::rgb(0, 0, 0),

            primary: ThemeColor::rgb(0, 51, 153),
            primary_foreground: ThemeColor::rgb(255, 255, 255),
            secondary: ThemeColor::rgb(230, 230, 230),
            secondary_foreground: ThemeColor::rgb(0, 0, 0),
            accent: ThemeColor::rgb(214, 228, 255),
            accent_foreground: ThemeColor::rgb(0, 34, 102),
            selection_foreground: ThemeColor::rgb(0, 34, 102),
            muted: ThemeColor::rgb(235, 235, 235),
            muted_foreground: ThemeColor::rgb(51, 51, 51),

            border: ThemeColor::rgb(0, 0, 0),
            input: ThemeColor::rgb(0, 0, 0),
            ring: ThemeColor::rgb(0, 51, 153),

            success: ThemeColor::rgb(0, 102, 0),
            warning: ThemeColor::rgb(122, 61, 0),
            danger: ThemeColor::rgb(168, 0, 0),
            info: ThemeColor::rgb(0, 51, 153),

            success_soft: ThemeColor::rgb(225, 247, 225),
            warning_soft: ThemeColor::rgb(255, 243, 214),
            danger_soft: ThemeColor::rgb(255, 229, 229),
            info_soft: ThemeColor::rgb(224, 236, 255),

            surface_elevated: ThemeColor::rgb(255, 255, 255),
            surface_overlay: ThemeColor::rgb(255, 255, 255),
            surface_tooltip: ThemeColor::rgb(0, 0, 0),

            primary_hover: ThemeColor::rgb(0, 38, 122),
            primary_pressed: ThemeColor::rgb(0, 25, 92),
            danger_hover: ThemeColor::rgb(138, 0, 0),
            danger_pressed: ThemeColor::rgb(107, 0, 0),
            ghost_hover: ThemeColor::rgb(224, 224, 224),
            ghost_pressed: ThemeColor::rgb(199, 199, 199),

            disabled_bg: ThemeColor::rgb(235, 235, 235),
            disabled_text: ThemeColor::rgb(89, 89, 89),
            disabled_foreground: ThemeColor::rgb(72, 72, 72),
        };
        Theme::from_colors(
            ThemeMode::Light,
            colors,
            shadow_ladder(0.12, 0.16, 0.20, 0.25),
        )
    }

    fn dark(&self) -> Theme {
        let colors = ColorTokens {
            background: ThemeColor::rgb(0, 0, 0),
            foreground: ThemeColor::rgb(255, 255, 255),
            surface: ThemeColor::rgb(13, 13, 13),
            surface_foreground: ThemeColor::rgb(255, 255, 255),

            primary: ThemeColor::rgb(26, 235, 255),
            primary_foreground: ThemeColor::rgb(0, 0, 0),
            secondary: ThemeColor::rgb(26, 26, 26),
            secondary_foreground: ThemeColor::rgb(255, 255, 255),
            accent: ThemeColor::rgb(0, 59, 77),
            accent_foreground: ThemeColor::rgb(179, 245, 255),
            selection_foreground: ThemeColor::rgb(179, 245, 255),
            muted: ThemeColor::rgb(20, 20, 20),
            muted_foreground: ThemeColor::rgb(212, 212, 212),

            border: ThemeColor::rgb(255, 255, 255),
            input: ThemeColor::rgb(255, 255, 255),
            ring: ThemeColor::rgb(255, 255, 0),

            success: ThemeColor::rgb(61, 255, 117),
            warning: ThemeColor::rgb(255, 235, 0),
            danger: ThemeColor::rgb(255, 102, 117),
            info: ThemeColor::rgb(26, 235, 255),

            success_soft: ThemeColor::rgb(0, 48, 18),
            warning_soft: ThemeColor::rgb(48, 38, 0),
            danger_soft: ThemeColor::rgb(56, 0, 10),
            info_soft: ThemeColor::rgb(0, 40, 52),

            surface_elevated: ThemeColor::rgb(18, 18, 18),
            surface_overlay: ThemeColor::rgb(24, 24, 24),
            surface_tooltip: ThemeColor::rgb(36, 36, 36),

            primary_hover: ThemeColor::rgb(102, 242, 255),
            primary_pressed: ThemeColor::rgb(0, 204, 224),
            danger_hover: ThemeColor::rgb(255, 138, 150),
            danger_pressed: ThemeColor::rgb(235, 66, 84),
            ghost_hover: ThemeColor::rgb(36, 36, 36),
            ghost_pressed: ThemeColor::rgb(56, 56, 56),

            disabled_bg: ThemeColor::rgb(20, 20, 20),
            disabled_text: ThemeColor::rgb(140, 140, 140),
            disabled_foreground: ThemeColor::rgb(170, 170, 170),
        };
        Theme::from_colors(
            ThemeMode::Dark,
            colors,
            shadow_ladder(0.40, 0.50, 0.60, 0.70),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ContrastLevel;

    #[test]
    fn high_contrast_pack_name_and_variants_validate() {
        assert_eq!(HighContrastPack.name(), "high-contrast");
        for theme in [HighContrastPack.light(), HighContrastPack.dark()] {
            assert_eq!(theme.validate(), Ok(()));
            assert!(theme.meets_wcag_aa());
        }
    }

    #[test]
    fn high_contrast_pack_meets_wcag_aa_everywhere_and_aaa_on_primary_surfaces() {
        for theme in [HighContrastPack.light(), HighContrastPack.dark()] {
            let report = theme.wcag_report();
            for (pair, ratio, level) in &report {
                assert!(
                    *ratio >= 4.5 && level.meets_aa(),
                    "{:?} pair {pair} ratio {ratio} must meet WCAG AA",
                    theme.mode
                );
                assert!(
                    *ratio >= 7.0 && *level == ContrastLevel::Aaa,
                    "{:?} pair {pair} ratio {ratio} should meet WCAG AAA",
                    theme.mode
                );
            }
            let fg_bg = report
                .iter()
                .find(|(name, _, _)| *name == "foreground/background")
                .expect("foreground/background pair present");
            assert!(fg_bg.1 >= 7.0);
            let surf = report
                .iter()
                .find(|(name, _, _)| *name == "surface_fg/surface")
                .expect("surface_fg/surface pair present");
            assert!(surf.1 >= 7.0);
        }
    }
}
