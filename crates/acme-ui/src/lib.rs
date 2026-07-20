//! High-level UI component library built on AcmeUI-Native primitives.
//! Ported from GPUI-based AcmeUIKit design patterns.

pub mod charts;
pub mod foundations;
pub mod inputs;
pub mod layout;
pub mod overlay;

// Re-export common types
pub use acme_core::{NodeId, WidgetKey};
pub use acme_layout::{LayoutEngine, LayoutKind, LayoutNode, LayoutStyle, Length, Overflow};
pub use acme_widgets::*;

// Re-export foundation types expected at the crate root
pub use foundations::icon::{icon, IconName};

/// Tone palette for semantic coloring.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tone {
    #[default]
    Neutral,
    Primary,
    Success,
    Warning,
    Danger,
}

/// Size presets matching AcmeUIKit design tokens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ControlSize {
    Xs, // 24px
    Sm, // 28px
    Md, // 32px
    Lg, // 38px
    Xl, // 44px
}

impl ControlSize {
    pub fn px(&self) -> f32 {
        match self {
            Self::Xs => 24.0,
            Self::Sm => 28.0,
            Self::Md => 32.0,
            Self::Lg => 38.0,
            Self::Xl => 44.0,
        }
    }
}

/// Access theme helper — retrieves the current theme's color for a tone.
pub fn tone_color(theme: &acme_theme::Theme, tone: Tone) -> acme_theme::ThemeColor {
    let c = &theme.colors;
    match tone {
        Tone::Neutral => c.text,
        Tone::Primary => c.accent,
        Tone::Success => c.input_validation_success,
        Tone::Warning => c.input_validation_error,
        Tone::Danger => c.danger,
    }
}
