//! Theme‑aware color token that bridges style declarations and the live theme.
//!
//! A [`ColorToken`] can be either a direct [`ThemeColor`] or a semantic name
//! such as `Primary`, `Foreground`, `Surface`, etc.  At render time the token
//! is resolved against the active [`Theme`] so that light/dark switching works
//! automatically.

use acme_core::Color;
use acme_theme::{Theme, ThemeColor};

/// A theme‑aware color reference for use with [`Style`](crate::Style).
///
/// # Resolution
///
/// Call [`ColorToken::resolve`] with a [`Theme`] to obtain the concrete
/// [`ThemeColor`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ColorToken {
    /// A raw, explicit colour (bypasses the theme).
    Direct(ThemeColor),

    // ── Semantic surface tokens ─────────────────────────────────
    /// Page / window background.
    Background,
    /// Primary text on background.
    Foreground,
    /// Card / panel background (elevation‑1).
    Surface,
    /// Text on surface.
    SurfaceForeground,

    // ── Semantic pair tokens ────────────────────────────────────
    /// Primary accent colour.
    Primary,
    /// Text / icon on primary.
    PrimaryForeground,
    /// Secondary surface.
    Secondary,
    /// Text on secondary.
    SecondaryForeground,
    /// Soft highlight (sidebar, selected row).
    Accent,
    /// Text on accent.
    AccentForeground,
    /// Subtle hover / disabled bg.
    Muted,
    /// Secondary text / placeholder.
    MutedForeground,

    // ── Border & ring ───────────────────────────────────────────
    /// Default border colour.
    Border,
    /// Input border colour.
    Input,
    /// Focus ring colour.
    Ring,

    // ── Status ──────────────────────────────────────────────────
    /// Success / positive.
    Success,
    /// Warning / caution.
    Warning,
    /// Danger / destructive.
    Danger,
    /// Info / neutral.
    Info,

    // ── Soft status backgrounds ─────────────────────────────────
    /// Success soft background.
    SuccessSoft,
    /// Warning soft background.
    WarningSoft,
    /// Danger soft background.
    DangerSoft,
    /// Info soft background.
    InfoSoft,

    // ── Interactive states ──────────────────────────────────────
    /// Primary hover.
    PrimaryHover,
    /// Primary pressed.
    PrimaryPressed,
    /// Danger hover.
    DangerHover,
    /// Danger pressed.
    DangerPressed,
    /// Ghost hover.
    GhostHover,
    /// Ghost pressed.
    GhostPressed,

    // ── Disabled ────────────────────────────────────────────────
    /// Disabled background.
    DisabledBg,
    /// Disabled text.
    DisabledText,

    // ── Elevation surfaces ──────────────────────────────────────
    /// Elevation‑2 (popover / menu).
    SurfaceElevated,
    /// Elevation‑3 (dialog / modal).
    SurfaceOverlay,
    /// Elevation‑4 (tooltip).
    SurfaceTooltip,
}

impl ColorToken {
    /// Resolve this token against `theme` to a concrete [`ThemeColor`].
    pub fn resolve(&self, theme: &Theme) -> ThemeColor {
        use ColorToken::*;
        let c = &theme.colors;
        match self {
            Direct(c) => *c,

            Background => c.background,
            Foreground => c.foreground,
            Surface => c.surface,
            SurfaceForeground => c.surface_foreground,

            Primary => c.primary,
            PrimaryForeground => c.primary_foreground,
            Secondary => c.secondary,
            SecondaryForeground => c.secondary_foreground,
            Accent => c.accent,
            AccentForeground => c.accent_foreground,
            Muted => c.muted,
            MutedForeground => c.muted_foreground,

            Border => c.border,
            Input => c.input,
            Ring => c.ring,

            Success => c.success,
            Warning => c.warning,
            Danger => c.danger,
            Info => c.info,

            SuccessSoft => c.success_soft,
            WarningSoft => c.warning_soft,
            DangerSoft => c.danger_soft,
            InfoSoft => c.info_soft,

            PrimaryHover => c.primary_hover,
            PrimaryPressed => c.primary_pressed,
            DangerHover => c.danger_hover,
            DangerPressed => c.danger_pressed,
            GhostHover => c.ghost_hover,
            GhostPressed => c.ghost_pressed,

            DisabledBg => c.disabled_bg,
            DisabledText => c.disabled_text,

            SurfaceElevated => c.surface_elevated,
            SurfaceOverlay => c.surface_overlay,
            SurfaceTooltip => c.surface_tooltip,
        }
    }

    /// Convenience — create a `Direct` token from a `ThemeColor`.
    pub fn from_color(color: ThemeColor) -> Self {
        Self::Direct(color)
    }

    /// Convenience — create a `Direct` token from RGBA components (0‑1 range).
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::Direct(ThemeColor::rgba(r, g, b, a))
    }

    /// Convenience — create a `Direct` token from RGB bytes.
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::Direct(ThemeColor::rgb(r, g, b))
    }
}

impl From<ThemeColor> for ColorToken {
    fn from(c: ThemeColor) -> Self {
        Self::Direct(c)
    }
}

impl From<Color> for ColorToken {
    fn from(c: Color) -> Self {
        Self::Direct(ThemeColor::rgba(c.r, c.g, c.b, c.a))
    }
}

// ── Fast-path helpers ────────────────────────────────────────────────

/// Convenience conversions from common types.
impl From<[f32; 4]> for ColorToken {
    fn from(c: [f32; 4]) -> Self {
        Self::Direct(ThemeColor::rgba(c[0], c[1], c[2], c[3]))
    }
}

impl From<(u8, u8, u8)> for ColorToken {
    fn from(c: (u8, u8, u8)) -> Self {
        Self::Direct(ThemeColor::rgb(c.0, c.1, c.2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semantic_tokens_resolve_to_non_zero_colors() {
        let light = Theme::light();
        let dark = Theme::dark();
        let tokens = [
            ColorToken::Primary,
            ColorToken::Foreground,
            ColorToken::Surface,
            ColorToken::Border,
        ];
        for t in &tokens {
            let l = t.resolve(&light);
            let d = t.resolve(&dark);
            assert!(l.is_valid(), "light {t:?} must be valid");
            assert!(d.is_valid(), "dark {t:?} must be valid");
        }
    }

    #[test]
    fn direct_token_passes_through() {
        let theme = Theme::light();
        let color = ThemeColor::rgb(255, 128, 64);
        let token = ColorToken::Direct(color);
        assert_eq!(token.resolve(&theme), color);
    }

    #[test]
    fn semantic_tokens_differ_between_modes() {
        let light = Theme::light();
        let dark = Theme::dark();
        assert_ne!(
            ColorToken::Primary.resolve(&light),
            ColorToken::Primary.resolve(&dark)
        );
        assert_ne!(
            ColorToken::Background.resolve(&light),
            ColorToken::Background.resolve(&dark)
        );
    }
}
