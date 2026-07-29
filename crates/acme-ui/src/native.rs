//! Platform-aware adaptation for AcmeUI components.
//!
//! This module keeps operating-system differences behind stable, backend-neutral
//! types. Applications can use [`NativeProfile::current`] for automatic target
//! selection or [`NativeProfile::for_platform`] for deterministic previews and
//! tests.

use acme_core::WidgetKey;
use acme_layout::{Length, WidgetLayoutContext};
use acme_theme::{
    Theme, ThemeMode,
    packs::{
        ThemePack, apple::ApplePack, material::MaterialPack, ubuntu::UbuntuPack,
        windows11::Windows11Pack,
    },
};
use acme_widgets::style::prelude::Styled;
use acme_widgets::{Button, ButtonSize, button};

/// Operating-system family used by the component adaptation layer.
///
/// The enum intentionally contains no `winit`, Win32, AppKit, GTK, Android, or
/// browser types, so it is safe to expose from the cross-platform component API.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum NativePlatform {
    Windows,
    MacOs,
    Linux,
    Android,
    Ios,
    Web,
    #[default]
    Unknown,
}

impl NativePlatform {
    /// Resolve the platform from Rust target configuration.
    pub const fn current() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::MacOs
        } else if cfg!(target_os = "android") {
            Self::Android
        } else if cfg!(target_os = "ios") {
            Self::Ios
        } else if cfg!(target_arch = "wasm32") {
            Self::Web
        } else if cfg!(target_os = "linux") {
            Self::Linux
        } else {
            Self::Unknown
        }
    }

    /// Human-readable platform label for settings pages and diagnostics.
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Windows => "Windows",
            Self::MacOs => "macOS",
            Self::Linux => "Linux",
            Self::Android => "Android",
            Self::Ios => "iOS",
            Self::Web => "Web",
            Self::Unknown => "Unknown",
        }
    }

    /// Whether the platform should default to touch-sized controls.
    pub const fn is_touch_first(self) -> bool {
        matches!(self, Self::Android | Self::Ios)
    }

    /// Native primary shortcut modifier for labels such as Save or Copy.
    pub const fn primary_shortcut_modifier(self) -> PrimaryShortcutModifier {
        match self {
            Self::MacOs | Self::Ios => PrimaryShortcutModifier::Command,
            _ => PrimaryShortcutModifier::Control,
        }
    }
}

/// Primary modifier used when presenting platform-native keyboard shortcuts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrimaryShortcutModifier {
    Control,
    Command,
}

impl PrimaryShortcutModifier {
    /// Display label for this modifier.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Control => "Ctrl",
            Self::Command => "⌘",
        }
    }

    /// Format a shortcut using native platform conventions.
    pub fn format(self, key: &str) -> String {
        match self {
            Self::Control => format!("Ctrl+{key}"),
            Self::Command => format!("⌘{key}"),
        }
    }
}

/// Default information density for a native platform profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NativeDensity {
    Compact,
    Comfortable,
    Touch,
}

/// Geometry and typography values that make the same component tree feel at
/// home on different operating systems.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeMetrics {
    pub page_padding: f32,
    pub content_gap: f32,
    pub title_font_size: f32,
    pub body_font_size: f32,
    pub label_font_size: f32,
    pub line_height: f32,
    pub compact_control_height: f32,
    pub control_height: f32,
    pub large_control_height: f32,
    pub minimum_hit_target: f32,
    pub control_horizontal_padding: f32,
    pub control_radius: f32,
    pub surface_radius: f32,
    pub dialog_radius: f32,
}

/// Complete native adaptation profile for one platform family.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeProfile {
    pub platform: NativePlatform,
    pub density: NativeDensity,
    pub metrics: NativeMetrics,
}

impl NativeProfile {
    /// Resolve the profile for the current Rust compilation target.
    pub const fn current() -> Self {
        Self::for_platform(NativePlatform::current())
    }

    /// Resolve a deterministic profile for previews, tests, and design tools.
    pub const fn for_platform(platform: NativePlatform) -> Self {
        match platform {
            NativePlatform::Windows => Self {
                platform,
                density: NativeDensity::Comfortable,
                metrics: NativeMetrics {
                    page_padding: 24.0,
                    content_gap: 16.0,
                    title_font_size: 28.0,
                    body_font_size: 14.0,
                    label_font_size: 13.0,
                    line_height: 1.45,
                    compact_control_height: 28.0,
                    control_height: 32.0,
                    large_control_height: 40.0,
                    minimum_hit_target: 32.0,
                    control_horizontal_padding: 12.0,
                    control_radius: 4.0,
                    surface_radius: 8.0,
                    dialog_radius: 8.0,
                },
            },
            NativePlatform::MacOs => Self {
                platform,
                density: NativeDensity::Compact,
                metrics: NativeMetrics {
                    page_padding: 20.0,
                    content_gap: 12.0,
                    title_font_size: 26.0,
                    body_font_size: 13.0,
                    label_font_size: 13.0,
                    line_height: 1.4,
                    compact_control_height: 24.0,
                    control_height: 28.0,
                    large_control_height: 32.0,
                    minimum_hit_target: 28.0,
                    control_horizontal_padding: 12.0,
                    control_radius: 6.0,
                    surface_radius: 10.0,
                    dialog_radius: 12.0,
                },
            },
            NativePlatform::Linux => Self {
                platform,
                density: NativeDensity::Compact,
                metrics: NativeMetrics {
                    page_padding: 24.0,
                    content_gap: 12.0,
                    title_font_size: 28.0,
                    body_font_size: 14.0,
                    label_font_size: 13.0,
                    line_height: 1.45,
                    compact_control_height: 28.0,
                    control_height: 34.0,
                    large_control_height: 40.0,
                    minimum_hit_target: 34.0,
                    control_horizontal_padding: 12.0,
                    control_radius: 6.0,
                    surface_radius: 8.0,
                    dialog_radius: 10.0,
                },
            },
            NativePlatform::Android => Self {
                platform,
                density: NativeDensity::Touch,
                metrics: NativeMetrics {
                    page_padding: 16.0,
                    content_gap: 12.0,
                    title_font_size: 28.0,
                    body_font_size: 16.0,
                    label_font_size: 14.0,
                    line_height: 1.5,
                    compact_control_height: 40.0,
                    control_height: 48.0,
                    large_control_height: 56.0,
                    minimum_hit_target: 48.0,
                    control_horizontal_padding: 16.0,
                    control_radius: 12.0,
                    surface_radius: 16.0,
                    dialog_radius: 24.0,
                },
            },
            NativePlatform::Ios => Self {
                platform,
                density: NativeDensity::Touch,
                metrics: NativeMetrics {
                    page_padding: 16.0,
                    content_gap: 12.0,
                    title_font_size: 34.0,
                    body_font_size: 17.0,
                    label_font_size: 15.0,
                    line_height: 1.45,
                    compact_control_height: 36.0,
                    control_height: 44.0,
                    large_control_height: 50.0,
                    minimum_hit_target: 44.0,
                    control_horizontal_padding: 16.0,
                    control_radius: 10.0,
                    surface_radius: 14.0,
                    dialog_radius: 18.0,
                },
            },
            NativePlatform::Web => Self {
                platform,
                density: NativeDensity::Comfortable,
                metrics: NativeMetrics {
                    page_padding: 24.0,
                    content_gap: 16.0,
                    title_font_size: 30.0,
                    body_font_size: 16.0,
                    label_font_size: 14.0,
                    line_height: 1.5,
                    compact_control_height: 32.0,
                    control_height: 40.0,
                    large_control_height: 48.0,
                    minimum_hit_target: 40.0,
                    control_horizontal_padding: 14.0,
                    control_radius: 8.0,
                    surface_radius: 12.0,
                    dialog_radius: 16.0,
                },
            },
            NativePlatform::Unknown => Self {
                platform,
                density: NativeDensity::Comfortable,
                metrics: NativeMetrics {
                    page_padding: 24.0,
                    content_gap: 16.0,
                    title_font_size: 28.0,
                    body_font_size: 14.0,
                    label_font_size: 13.0,
                    line_height: 1.5,
                    compact_control_height: 28.0,
                    control_height: 34.0,
                    large_control_height: 40.0,
                    minimum_hit_target: 34.0,
                    control_horizontal_padding: 12.0,
                    control_radius: 6.0,
                    surface_radius: 8.0,
                    dialog_radius: 12.0,
                },
            },
        }
    }

    /// Stable theme-pack identifier selected for this platform.
    pub const fn theme_pack_name(self) -> &'static str {
        match self.platform {
            NativePlatform::Windows => "windows11",
            NativePlatform::MacOs | NativePlatform::Ios => "apple",
            NativePlatform::Linux => "ubuntu",
            NativePlatform::Android => "material",
            NativePlatform::Web | NativePlatform::Unknown => "default",
        }
    }

    /// Create a light or dark theme and apply this platform's geometry and
    /// typography metrics to the semantic token set.
    pub fn theme(self, mode: ThemeMode) -> Theme {
        let mut theme = match self.platform {
            NativePlatform::Windows => Windows11Pack.theme(mode),
            NativePlatform::MacOs | NativePlatform::Ios => ApplePack.theme(mode),
            NativePlatform::Linux => UbuntuPack.theme(mode),
            NativePlatform::Android => MaterialPack.theme(mode),
            NativePlatform::Web | NativePlatform::Unknown => match mode {
                ThemeMode::Light => Theme::light(),
                ThemeMode::Dark => Theme::dark(),
            },
        };
        self.apply_metrics(&mut theme);
        theme
    }

    /// Build the theme-aware layout context used by `WidgetNode` conversion.
    pub fn layout_context(self, scale_factor: f32) -> WidgetLayoutContext {
        let scale_factor = if scale_factor.is_finite() && scale_factor > 0.0 {
            scale_factor
        } else {
            1.0
        };
        WidgetLayoutContext {
            body_font_size: self.metrics.body_font_size,
            body_line_height: self.metrics.body_font_size * self.metrics.line_height,
            label_font_size: self.metrics.label_font_size,
            control_height: self.metrics.control_height,
            scale_factor,
        }
    }

    /// Format a primary-key shortcut using the current platform convention.
    pub fn shortcut_label(self, key: &str) -> String {
        self.platform.primary_shortcut_modifier().format(key)
    }

    fn apply_metrics(self, theme: &mut Theme) {
        let metrics = self.metrics;
        theme.typography.h1 = metrics.title_font_size;
        theme.typography.h2 = (metrics.title_font_size * 0.78).max(metrics.body_font_size + 4.0);
        theme.typography.h3 = metrics.body_font_size + 4.0;
        theme.typography.h4 = metrics.body_font_size + 2.0;
        theme.typography.body = metrics.body_font_size;
        theme.typography.body_sm = (metrics.body_font_size - 1.0).max(11.0);
        theme.typography.label = metrics.label_font_size;
        theme.typography.caption = (metrics.label_font_size - 1.0).max(10.0);
        theme.typography.small = (metrics.label_font_size - 2.0).max(9.0);
        theme.typography.line_height = metrics.line_height;

        theme.control_heights.xs = (metrics.compact_control_height - 6.0).max(20.0);
        theme.control_heights.sm = metrics.compact_control_height;
        theme.control_heights.md = metrics.control_height;
        theme.control_heights.lg = metrics.large_control_height;
        theme.control_heights.xl =
            (metrics.large_control_height + 8.0).max(metrics.minimum_hit_target);

        theme.radii.md = metrics.control_radius;
        theme.radii.lg = metrics.surface_radius;
        theme.radii.xl = metrics.dialog_radius;
    }
}

/// Resolve the current target's native profile.
pub const fn native_profile() -> NativeProfile {
    NativeProfile::current()
}

/// Resolve a platform-adapted semantic theme for the current target.
pub fn native_theme(mode: ThemeMode) -> Theme {
    native_profile().theme(mode)
}

/// Resolve the current target's layout context at the supplied scale factor.
pub fn native_layout_context(scale_factor: f32) -> WidgetLayoutContext {
    native_profile().layout_context(scale_factor)
}

/// Create a button with target-native height, padding, density, and corner
/// radius while preserving the existing `Button` builder API.
pub fn native_button<M>(key: impl Into<WidgetKey>, label: impl Into<String>) -> Button<M> {
    native_button_for(native_profile(), key, label)
}

/// Create a native button for an explicit platform profile. This is useful for
/// Gallery previews and deterministic cross-platform tests.
pub fn native_button_for<M>(
    profile: NativeProfile,
    key: impl Into<WidgetKey>,
    label: impl Into<String>,
) -> Button<M> {
    let size = match profile.density {
        NativeDensity::Compact => ButtonSize::Small,
        NativeDensity::Comfortable => ButtonSize::Medium,
        NativeDensity::Touch => ButtonSize::Large,
    };
    button(key, label)
        .size(size)
        .h(Length::px(profile.metrics.control_height))
        .px(profile.metrics.control_horizontal_padding)
        .rounded(profile.metrics.control_radius)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PLATFORMS: [NativePlatform; 7] = [
        NativePlatform::Windows,
        NativePlatform::MacOs,
        NativePlatform::Linux,
        NativePlatform::Android,
        NativePlatform::Ios,
        NativePlatform::Web,
        NativePlatform::Unknown,
    ];

    #[test]
    fn every_profile_produces_valid_light_and_dark_themes() {
        for platform in PLATFORMS {
            let profile = NativeProfile::for_platform(platform);
            assert!(profile.metrics.control_height > 0.0);
            assert!(profile.metrics.minimum_hit_target > 0.0);
            for mode in [ThemeMode::Light, ThemeMode::Dark] {
                let theme = profile.theme(mode);
                assert_eq!(theme.validate(), Ok(()));
                assert_eq!(theme.control_heights.md, profile.metrics.control_height);
                assert_eq!(theme.typography.body, profile.metrics.body_font_size);
            }
        }
    }

    #[test]
    fn apple_shortcuts_use_command_and_other_desktops_use_control() {
        assert_eq!(
            NativePlatform::MacOs.primary_shortcut_modifier(),
            PrimaryShortcutModifier::Command
        );
        assert_eq!(
            NativePlatform::Ios.primary_shortcut_modifier(),
            PrimaryShortcutModifier::Command
        );
        assert_eq!(
            NativePlatform::Windows.primary_shortcut_modifier(),
            PrimaryShortcutModifier::Control
        );
        assert_eq!(
            NativeProfile::for_platform(NativePlatform::MacOs).shortcut_label("S"),
            "⌘S"
        );
        assert_eq!(
            NativeProfile::for_platform(NativePlatform::Linux).shortcut_label("S"),
            "Ctrl+S"
        );
    }

    #[test]
    fn touch_profiles_enforce_touch_sized_controls() {
        for platform in [NativePlatform::Android, NativePlatform::Ios] {
            let profile = NativeProfile::for_platform(platform);
            assert_eq!(profile.density, NativeDensity::Touch);
            assert!(profile.metrics.control_height >= profile.metrics.minimum_hit_target);
        }
    }

    #[test]
    fn invalid_scale_factor_falls_back_to_one() {
        let profile = NativeProfile::for_platform(NativePlatform::Windows);
        assert_eq!(profile.layout_context(0.0).scale_factor, 1.0);
        assert_eq!(profile.layout_context(f32::NAN).scale_factor, 1.0);
        assert_eq!(profile.layout_context(2.0).scale_factor, 2.0);
    }

    #[test]
    fn native_button_applies_profile_geometry() {
        let profile = NativeProfile::for_platform(NativePlatform::Android);
        let button = native_button_for::<()>(profile, "save", "Save");
        assert_eq!(button.size, ButtonSize::Large);
        assert_eq!(
            button.style.height,
            Some(Length::px(profile.metrics.control_height))
        );
        assert_eq!(
            button.style.border_radius,
            Some(profile.metrics.control_radius)
        );
        let padding = button.style.padding.expect("native horizontal padding");
        assert_eq!(padding.left, profile.metrics.control_horizontal_padding);
        assert_eq!(padding.right, profile.metrics.control_horizontal_padding);
    }
}
