//! Curated brand theme packs.
//!
//! Each pack implements [`ThemePack`] and provides light + dark variants built
//! from real brand palettes. Use [`theme_by_name`] for dynamic lookup (e.g. from
//! a settings page or Gallery) or the pack structs directly.

use crate::{ShadowDef, ShadowTokens, Theme, ThemeColor, ThemeMode};

pub mod apple;
pub mod dracula;
pub mod gruvbox;
pub mod material;
pub mod nord;
pub mod one_dark;
pub mod solarized;
pub mod ubuntu;
pub mod windows10;
pub mod windows11;

/// A named brand theme pack providing light and dark variants.
pub trait ThemePack {
    /// Stable identifier used by [`theme_by_name`].
    fn name(&self) -> &'static str;
    /// The light variant.
    fn light(&self) -> Theme;
    /// The dark variant.
    fn dark(&self) -> Theme;
    /// Resolve a variant by mode.
    fn theme(&self, mode: ThemeMode) -> Theme {
        match mode {
            ThemeMode::Light => self.light(),
            ThemeMode::Dark => self.dark(),
        }
    }
}

/// Build a standard 4-level shadow ladder from per-level opacities.
pub(crate) fn shadow_ladder(a_sm: f32, a_md: f32, a_lg: f32, a_xl: f32) -> ShadowTokens {
    let base = ThemeColor::rgba(0.0, 0.0, 0.0, 1.0);
    ShadowTokens {
        sm: ShadowDef {
            offset_x: 0.0,
            offset_y: 1.0,
            blur: 2.0,
            spread: 0.0,
            color: base.with_alpha(a_sm),
        },
        md: ShadowDef {
            offset_x: 0.0,
            offset_y: 4.0,
            blur: 12.0,
            spread: 0.0,
            color: base.with_alpha(a_md),
        },
        lg: ShadowDef {
            offset_x: 0.0,
            offset_y: 8.0,
            blur: 24.0,
            spread: 0.0,
            color: base.with_alpha(a_lg),
        },
        xl: ShadowDef {
            offset_x: 0.0,
            offset_y: 16.0,
            blur: 48.0,
            spread: 0.0,
            color: base.with_alpha(a_xl),
        },
    }
}

/// Look up a theme pack by name and resolve a mode variant.
///
/// Returns `None` for unknown names. Names are stable identifiers, not
/// display labels.
pub fn theme_by_name(name: &str, mode: ThemeMode) -> Option<Theme> {
    match name {
        "apple" => Some(apple::ApplePack.theme(mode)),
        "windows10" => Some(windows10::Windows10Pack.theme(mode)),
        "windows11" => Some(windows11::Windows11Pack.theme(mode)),
        "ubuntu" => Some(ubuntu::UbuntuPack.theme(mode)),
        "material" => Some(material::MaterialPack.theme(mode)),
        "nord" => Some(nord::NordPack.theme(mode)),
        "dracula" => Some(dracula::DraculaPack.theme(mode)),
        "solarized" => Some(solarized::SolarizedPack.theme(mode)),
        "gruvbox" => Some(gruvbox::GruvboxPack.theme(mode)),
        "one-dark" | "onedark" => Some(one_dark::OneDarkPack.theme(mode)),
        _ => None,
    }
}

/// All registered theme pack names (stable identifiers).
pub fn available_themes() -> &'static [&'static str] {
    &[
        "apple",
        "windows10",
        "windows11",
        "ubuntu",
        "material",
        "nord",
        "dracula",
        "solarized",
        "gruvbox",
        "one-dark",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_registered_themes_resolve_and_validate() {
        for name in available_themes() {
            for mode in [ThemeMode::Light, ThemeMode::Dark] {
                let theme = theme_by_name(name, mode)
                    .unwrap_or_else(|| panic!("theme {name} should resolve"));
                assert_eq!(
                    theme.validate(),
                    Ok(()),
                    "theme {name} ({mode:?}) should be valid"
                );
            }
        }
    }

    #[test]
    fn unknown_theme_returns_none() {
        assert!(theme_by_name("does-not-exist", ThemeMode::Light).is_none());
    }

    #[test]
    fn light_and_dark_variants_differ() {
        for name in available_themes() {
            let light = theme_by_name(name, ThemeMode::Light).unwrap();
            let dark = theme_by_name(name, ThemeMode::Dark).unwrap();
            assert_ne!(
                light.colors.background, dark.colors.background,
                "theme {name} light/dark backgrounds should differ"
            );
        }
    }
}
