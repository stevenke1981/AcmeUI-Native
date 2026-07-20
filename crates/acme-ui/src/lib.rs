//! AcmeUI-Native Component Library — V2
//!
//! High-level UI components built on acme-widgets primitives.
//! Design system inspired by shadcn/ui, Material UI, and Ant Design.

pub mod charts;
pub mod foundations;
pub mod inputs;
pub mod layout;
pub mod overlay;

// Re-export common types
pub use acme_core::{NodeId, WidgetKey};
pub use acme_layout::{LayoutEngine, LayoutKind, LayoutNode, LayoutStyle, Length, Overflow};
pub use acme_theme::ThemeColor;
pub use acme_widgets::*;

// Re-export foundation types expected at the crate root
pub use foundations::icon::{icon, IconName};

/// Tone palette for semantic coloring — inspired by shadcn/ui status colors.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    #[default]
    Neutral,
    Primary,
    Success,
    Warning,
    Danger,
    Info,
}

/// Resolve tone → foreground/background pair from theme
pub struct ToneColors {
    pub bg: acme_theme::ThemeColor,
    pub fg: acme_theme::ThemeColor,
    pub soft_bg: Option<acme_theme::ThemeColor>,
    pub soft_fg: Option<acme_theme::ThemeColor>,
}

/// Resolve tone to color pair from theme
pub fn resolve_tone(theme: &acme_theme::Theme, tone: Tone, _solid: bool) -> ToneColors {
    let c = &theme.colors;
    match tone {
        Tone::Neutral => ToneColors {
            bg: c.secondary,
            fg: c.secondary_foreground,
            soft_bg: None,
            soft_fg: None,
        },
        Tone::Primary => ToneColors {
            bg: c.primary,
            fg: c.primary_foreground,
            soft_bg: Some(c.accent),
            soft_fg: Some(c.accent_foreground),
        },
        Tone::Success => ToneColors {
            bg: c.success,
            fg: ThemeColor::rgb(255, 255, 255),
            soft_bg: Some(c.success_soft),
            soft_fg: Some(c.success),
        },
        Tone::Warning => ToneColors {
            bg: c.warning,
            fg: ThemeColor::rgb(255, 255, 255),
            soft_bg: Some(c.warning_soft),
            soft_fg: Some(c.warning),
        },
        Tone::Danger => ToneColors {
            bg: c.danger,
            fg: ThemeColor::rgb(255, 255, 255),
            soft_bg: Some(c.danger_soft),
            soft_fg: Some(c.danger),
        },
        Tone::Info => ToneColors {
            bg: c.info,
            fg: ThemeColor::rgb(255, 255, 255),
            soft_bg: Some(c.info_soft),
            soft_fg: Some(c.info),
        },
    }
}

/// Control sizes matching V2 design system heights
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum ControlSize {
    Xs,      // 22px — badge, tag, small meta
    Sm,      // 28px — compact toolbar
    #[default]
    Md,      // 34px — default control
    Lg,      // 40px — primary action
    Xl,      // 48px — hero
}

impl ControlSize {
    /// Map to theme control heights
    pub fn height(&self, theme: &acme_theme::Theme) -> f32 {
        match self {
            Self::Xs => theme.control_heights.xs,
            Self::Sm => theme.control_heights.sm,
            Self::Md => theme.control_heights.md,
            Self::Lg => theme.control_heights.lg,
            Self::Xl => theme.control_heights.xl,
        }
    }
}

/// Access theme helper — retrieves the current theme's color for a tone.
pub fn tone_color(theme: &acme_theme::Theme, tone: Tone) -> acme_theme::ThemeColor {
    resolve_tone(theme, tone, true).bg
}
