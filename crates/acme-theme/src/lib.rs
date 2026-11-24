//! AcmeUI-Native Design System V2
//!
//! Synthesized from shadcn/ui, Material UI, and Ant Design best practices.
//! All colors, spacing, typography via semantic design tokens — no hardcoded values.
//!
//! Architecture: Seed Tokens → Derived Tokens → Component Tokens

#![forbid(unsafe_op_in_unsafe_fn)]

/// Pure Rust JSON serialization and parsing for [`Theme`].
pub mod json;
/// Curated brand theme packs (Apple, Windows, Ubuntu, Material, editor themes).
pub mod packs;

pub use json::ThemeJsonError;

/// The built-in appearance variants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

// ═════════════════════════════════════════════════════════════════════════════
// Color Primitives
// ═════════════════════════════════════════════════════════════════════════════

/// Framework-owned color value. All components must consume semantic fields.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThemeColor {
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub alpha: f32,
}

impl ThemeColor {
    pub const fn rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
    pub const fn rgb(red: u8, green: u8, blue: u8) -> Self {
        Self {
            red: red as f32 / 255.0,
            green: green as f32 / 255.0,
            blue: blue as f32 / 255.0,
            alpha: 1.0,
        }
    }
    pub fn is_valid(self) -> bool {
        [self.red, self.green, self.blue, self.alpha]
            .into_iter()
            .all(|v| v.is_finite() && (0.0..=1.0).contains(&v))
    }
    /// Linear interpolation between two colors
    pub fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            red: self.red + (other.red - self.red) * t,
            green: self.green + (other.green - self.green) * t,
            blue: self.blue + (other.blue - self.blue) * t,
            alpha: self.alpha + (other.alpha - self.alpha) * t,
        }
    }
    /// Darken by reducing lightness
    pub fn darken(self, factor: f32) -> Self {
        Self {
            red: (self.red * (1.0 - factor)).max(0.0),
            green: (self.green * (1.0 - factor)).max(0.0),
            blue: (self.blue * (1.0 - factor)).max(0.0),
            alpha: self.alpha,
        }
    }
    /// Lighten by increasing lightness
    pub fn lighten(self, factor: f32) -> Self {
        Self {
            red: (self.red + (1.0 - self.red) * factor).min(1.0),
            green: (self.green + (1.0 - self.green) * factor).min(1.0),
            blue: (self.blue + (1.0 - self.blue) * factor).min(1.0),
            alpha: self.alpha,
        }
    }
    /// With modified alpha
    pub fn with_alpha(self, alpha: f32) -> Self {
        Self { alpha, ..self }
    }
    /// WCAG 2.1 relative luminance (0.0 = black, 1.0 = white).
    ///
    /// Alpha is ignored; luminance is computed from the RGB channels only.
    pub fn relative_luminance(self) -> f32 {
        fn channel(c: f32) -> f32 {
            if c <= 0.03928 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * channel(self.red) + 0.7152 * channel(self.green) + 0.0722 * channel(self.blue)
    }
}

/// WCAG contrast ratio between two colors, in the range 1.0–21.0.
///
/// 1.0 means no contrast (identical luminance); 21.0 is black-on-white.
/// Alpha is ignored.
pub fn contrast_ratio(a: ThemeColor, b: ThemeColor) -> f32 {
    let la = a.relative_luminance();
    let lb = b.relative_luminance();
    let (lighter, darker) = if la >= lb { (la, lb) } else { (lb, la) };
    (lighter + 0.05) / (darker + 0.05)
}

/// WCAG conformance level for a contrast ratio.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContrastLevel {
    /// ≥ 7.0 — enhanced (AAA) for normal text.
    Aaa,
    /// ≥ 4.5 — minimum (AA) for normal text.
    Aa,
    /// ≥ 3.0 — AA for large text / UI components only.
    AaLarge,
    /// < 3.0 — fails WCAG.
    Fail,
}

impl ContrastLevel {
    /// Classify a contrast ratio.
    pub fn from_ratio(ratio: f32) -> Self {
        if ratio >= 7.0 {
            Self::Aaa
        } else if ratio >= 4.5 {
            Self::Aa
        } else if ratio >= 3.0 {
            Self::AaLarge
        } else {
            Self::Fail
        }
    }

    /// True if this level meets WCAG AA for normal text (≥ 4.5).
    pub fn meets_aa(self) -> bool {
        matches!(self, Self::Aa | Self::Aaa)
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// V2 Color Tokens — inspired by shadcn/ui foreground/background pairs
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorTokens {
    // ── Base surface tokens (foreground/background pairs) ──
    pub background: ThemeColor,         // Page/window bg
    pub foreground: ThemeColor,         // Text on background
    pub surface: ThemeColor,            // Card, panel bg (elevation-1)
    pub surface_foreground: ThemeColor, // Text on surface

    // ── Semantic surface pairs ──
    pub primary: ThemeColor,
    pub primary_foreground: ThemeColor,
    pub secondary: ThemeColor,
    pub secondary_foreground: ThemeColor,
    pub accent: ThemeColor,
    pub accent_foreground: ThemeColor,
    /// Text on the accent surface for a selected item.
    pub selection_foreground: ThemeColor,
    pub muted: ThemeColor,
    pub muted_foreground: ThemeColor,

    // ── Border & ring ──
    pub border: ThemeColor,
    pub input: ThemeColor,
    pub ring: ThemeColor,

    // ── Status colors (solid) ──
    pub success: ThemeColor,
    pub warning: ThemeColor,
    pub danger: ThemeColor,
    pub info: ThemeColor,

    // ── Status soft backgrounds ──
    pub success_soft: ThemeColor,
    pub warning_soft: ThemeColor,
    pub danger_soft: ThemeColor,
    pub info_soft: ThemeColor,

    // ── Elevation surface ladder ──
    pub surface_elevated: ThemeColor, // elevation-2: popover, menu
    pub surface_overlay: ThemeColor,  // elevation-3: dialog, modal
    pub surface_tooltip: ThemeColor,  // elevation-4: tooltip

    // ── Component-specific ──
    pub primary_hover: ThemeColor,
    pub primary_pressed: ThemeColor,
    pub danger_hover: ThemeColor,
    pub danger_pressed: ThemeColor,
    pub ghost_hover: ThemeColor,
    pub ghost_pressed: ThemeColor,

    // ── Disabled ──
    pub disabled_bg: ThemeColor,
    /// Legacy disabled text token.
    pub disabled_text: ThemeColor,
    /// Accessible disabled text on disabled_bg.
    pub disabled_foreground: ThemeColor,
}

// ═════════════════════════════════════════════════════════════════════════════
// Spacing, Radius, Typography, Shadow, Animation
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpacingTokens {
    /// 2px — icon gap, tight packing
    pub half: f32,
    /// 4px — inner padding (badge, tag)
    pub px: f32,
    /// 6px — dense button icon gap
    pub px1: f32,
    /// 8px — component gap, form spacing
    pub px2: f32,
    /// 12px — card padding, section gap
    pub px3: f32,
    /// 16px — panel padding, large gap
    pub px4: f32,
    /// 20px — section margin
    pub px5: f32,
    /// 24px — page margin, card group gap
    pub px6: f32,
    /// 32px — major section separation
    pub px8: f32,
    /// 40px — page padding
    pub px10: f32,
}

impl SpacingTokens {
    pub fn get(&self, n: u32) -> f32 {
        match n {
            0 => 0.0,
            1 => self.px,
            2 => self.px2,
            3 => self.px3,
            4 => self.px4,
            5 => self.px5,
            6 => self.px6,
            8 => self.px8,
            10 => self.px10,
            _ => n as f32 * 4.0, // fallback: 4px grid
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RadiusTokens {
    /// 0px — sharp elements
    pub none: f32,
    /// 4px — inputs, cards in data view
    pub sm: f32,
    /// 6px — buttons, default components
    pub md: f32,
    /// 8px — cards, dialogs
    pub lg: f32,
    /// 12px — modals, large surfaces
    pub xl: f32,
    /// 999px — badges, pills, avatars
    pub full: f32,
}

/// Shadow definition with offset and blur
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowDef {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: ThemeColor,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowTokens {
    /// 0 1px 2px rgba(0,0,0,0.04) — subtle card
    pub sm: ShadowDef,
    /// 0 4px 12px rgba(0,0,0,0.06) — popover, menu
    pub md: ShadowDef,
    /// 0 8px 24px rgba(0,0,0,0.08) — dialog, modal
    pub lg: ShadowDef,
    /// 0 16px 48px rgba(0,0,0,0.10) — notification, tooltip
    pub xl: ShadowDef,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypographyTokens {
    /// 28px — page title
    pub h1: f32,
    /// 22px — section title
    pub h2: f32,
    /// 18px — card title
    pub h3: f32,
    /// 16px — subsection
    pub h4: f32,
    /// 14px — default body text
    pub body: f32,
    /// 13px — compact body
    pub body_sm: f32,
    /// 13px — form label
    pub label: f32,
    /// 12px — helper text, badges
    pub caption: f32,
    /// 11px — legal, timestamps
    pub small: f32,
    /// 13px — monospace code
    pub code: f32,
    /// Line height ratio for body text (1.5)
    pub line_height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimationTokens {
    /// 100ms — hover, press
    pub fast_ms: u32,
    /// 200ms — transition, toggle
    pub normal_ms: u32,
    /// 300ms — enter/leave
    pub slow_ms: u32,
    /// 250ms — panel slide
    pub slide_ms: u32,
}

/// Control height presets for interactive components
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ControlHeights {
    /// 22px — badge, tag, small meta
    pub xs: f32,
    /// 28px — compact toolbar
    pub sm: f32,
    /// 34px — default control
    pub md: f32,
    /// 40px — primary action
    pub lg: f32,
    /// 48px — hero
    pub xl: f32,
}

// ═════════════════════════════════════════════════════════════════════════════
// Main Theme Struct
// ═════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub mode: ThemeMode,
    pub colors: ColorTokens,
    pub spacing: SpacingTokens,
    pub radii: RadiusTokens,
    pub typography: TypographyTokens,
    pub shadows: ShadowTokens,
    pub animation: AnimationTokens,
    pub control_heights: ControlHeights,
}

/// Reason a [`Theme`] failed validation.
///
/// Marked `#[non_exhaustive]` so new checks can be added without breaking
/// downstream `match` expressions.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum ThemeValidationError {
    /// A color channel is invalid.
    InvalidColor,
    /// `selection_foreground` has less than 4.5:1 contrast against `accent`.
    SelectionContrastTooLow {
        ratio: f32,
    },
    InvalidSpacing,
    InvalidRadius,
    InvalidTypography,
}

impl std::fmt::Display for ThemeValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SelectionContrastTooLow { ratio } => write!(
                f,
                "selection_foreground has insufficient contrast against accent ({ratio:.2}:1, requires 4.5:1)"
            ),
            _ => write!(f, "{self:?}"),
        }
    }
}

impl Theme {
    pub fn light() -> Self {
        Self::builtin(ThemeMode::Light)
    }
    pub fn dark() -> Self {
        Self::builtin(ThemeMode::Dark)
    }

    /// Resolve border color for component state
    pub fn border_color(&self, focused: bool, invalid: bool) -> ThemeColor {
        if invalid {
            self.colors.danger
        } else if focused {
            self.colors.ring
        } else {
            self.colors.border
        }
    }

    /// Get control height by index (0=xs, 1=sm, 2=md, 3=lg, 4=xl)
    pub fn control_height(&self, level: usize) -> f32 {
        match level {
            0 => self.control_heights.xs,
            1 => self.control_heights.sm,
            2 => self.control_heights.md,
            3 => self.control_heights.lg,
            4 => self.control_heights.xl,
            _ => self.control_heights.md,
        }
    }

    /// WCAG contrast ratio between two colors in this theme's palette.
    pub fn contrast(&self, fg: ThemeColor, bg: ThemeColor) -> f32 {
        contrast_ratio(fg, bg)
    }

    /// Token pair for a selected item in a list or nav, as `(bg, fg)`.
    ///
    /// Custom themes supply [`ColorTokens::selection_foreground`] with at least
    /// 4.5:1 contrast on [`ColorTokens::accent`] for WCAG AA normal text.
    pub fn selected_item_colors(&self) -> (ThemeColor, ThemeColor) {
        (self.colors.accent, self.colors.selection_foreground)
    }

    /// Audit the key foreground/background pairs for WCAG conformance.
    ///
    /// Returns a list of `(pair_name, ratio, level)` for the most important
    /// text-on-surface combinations, including the selected-item pair from
    /// [`Theme::selected_item_colors`]. Use this to surface accessibility
    /// issues in a theme pack or a Gallery preview.
    pub fn wcag_report(&self) -> Vec<(&'static str, f32, ContrastLevel)> {
        let c = &self.colors;
        let (selected_bg, selected_fg) = self.selected_item_colors();
        let pairs = [
            ("foreground/background", c.foreground, c.background),
            ("surface_fg/surface", c.surface_foreground, c.surface),
            ("primary_fg/primary", c.primary_foreground, c.primary),
            (
                "secondary_fg/secondary",
                c.secondary_foreground,
                c.secondary,
            ),
            ("muted_fg/muted", c.muted_foreground, c.muted),
            ("accent_fg/accent", c.accent_foreground, c.accent),
            ("selected_fg/selected_bg", selected_fg, selected_bg),
            (
                "selection_foreground/accent",
                c.selection_foreground,
                c.accent,
            ),
            (
                "disabled_foreground/disabled_bg",
                c.disabled_foreground,
                c.disabled_bg,
            ),
        ];
        pairs
            .into_iter()
            .map(|(name, fg, bg)| {
                let ratio = contrast_ratio(fg, bg);
                (name, ratio, ContrastLevel::from_ratio(ratio))
            })
            .collect()
    }

    /// True if all key text pairs meet at least WCAG AA (≥ 4.5:1).
    pub fn meets_wcag_aa(&self) -> bool {
        self.wcag_report()
            .iter()
            .all(|(_, _, level)| level.meets_aa())
    }

    /// Validate token values and require 4.5:1 contrast for selected text on the accent.
    pub fn validate(&self) -> Result<(), ThemeValidationError> {
        let c = &self.colors;
        let color_fields = [
            c.background,
            c.foreground,
            c.surface,
            c.surface_foreground,
            c.primary,
            c.primary_foreground,
            c.secondary,
            c.secondary_foreground,
            c.accent,
            c.accent_foreground,
            c.muted,
            c.muted_foreground,
            c.border,
            c.input,
            c.ring,
            c.success,
            c.warning,
            c.danger,
            c.info,
            c.success_soft,
            c.warning_soft,
            c.danger_soft,
            c.info_soft,
            c.surface_elevated,
            c.surface_overlay,
            c.surface_tooltip,
            c.primary_hover,
            c.primary_pressed,
            c.danger_hover,
            c.danger_pressed,
            c.ghost_hover,
            c.ghost_pressed,
            c.disabled_bg,
            c.disabled_text,
            c.disabled_foreground,
            c.selection_foreground,
        ];
        if !color_fields.into_iter().all(ThemeColor::is_valid) {
            return Err(ThemeValidationError::InvalidColor);
        }
        let s = &self.spacing;
        if [
            s.half, s.px, s.px1, s.px2, s.px3, s.px4, s.px5, s.px6, s.px8, s.px10,
        ]
        .into_iter()
        .any(|v| !v.is_finite() || v < 0.0)
        {
            return Err(ThemeValidationError::InvalidSpacing);
        }
        let r = &self.radii;
        if [r.none, r.sm, r.md, r.lg, r.xl, r.full]
            .into_iter()
            .any(|v| !v.is_finite() || v < 0.0)
        {
            return Err(ThemeValidationError::InvalidRadius);
        }
        let t = &self.typography;
        if [
            t.h1,
            t.h2,
            t.h3,
            t.h4,
            t.body,
            t.body_sm,
            t.label,
            t.caption,
            t.small,
            t.code,
            t.line_height,
        ]
        .into_iter()
        .any(|v| !v.is_finite() || v <= 0.0)
        {
            return Err(ThemeValidationError::InvalidTypography);
        }
        let (selected_bg, selected_fg) = self.selected_item_colors();
        let ratio = contrast_ratio(selected_fg, selected_bg);
        if !ContrastLevel::from_ratio(ratio).meets_aa() {
            return Err(ThemeValidationError::SelectionContrastTooLow { ratio });
        }
        Ok(())
    }

    fn builtin(mode: ThemeMode) -> Self {
        let (colors, shadows) = match mode {
            ThemeMode::Light => Self::light_colors(),
            ThemeMode::Dark => Self::dark_colors(),
        };
        Self::from_colors(mode, colors, shadows)
    }

    /// Build a theme from explicit color and shadow tokens, using the default
    /// spacing, radius, typography, animation, and control-height scales.
    ///
    /// Theme packs use this to supply brand palettes without repeating the
    /// shared structural tokens. Callers may further mutate the returned
    /// `Theme` (e.g. adjust `radii`) to match a brand's geometry.
    pub fn from_colors(mode: ThemeMode, colors: ColorTokens, shadows: ShadowTokens) -> Self {
        Self {
            mode,
            colors,
            spacing: SpacingTokens {
                half: 2.0,
                px: 4.0,
                px1: 6.0,
                px2: 8.0,
                px3: 12.0,
                px4: 16.0,
                px5: 20.0,
                px6: 24.0,
                px8: 32.0,
                px10: 40.0,
            },
            radii: RadiusTokens {
                none: 0.0,
                sm: 4.0,
                md: 6.0,
                lg: 8.0,
                xl: 12.0,
                full: 999.0,
            },
            typography: TypographyTokens {
                h1: 28.0,
                h2: 22.0,
                h3: 18.0,
                h4: 16.0,
                body: 14.0,
                body_sm: 13.0,
                label: 13.0,
                caption: 12.0,
                small: 11.0,
                code: 13.0,
                line_height: 1.5,
            },
            shadows,
            animation: AnimationTokens {
                fast_ms: 100,
                normal_ms: 200,
                slow_ms: 300,
                slide_ms: 250,
            },
            control_heights: ControlHeights {
                xs: 22.0,
                sm: 28.0,
                md: 34.0,
                lg: 40.0,
                xl: 48.0,
            },
        }
    }

    // ── Light Palette --------------------------------------------------------

    fn light_colors() -> (ColorTokens, ShadowTokens) {
        let c = ColorTokens {
            // Base
            background: ThemeColor::rgb(250, 251, 252), // #FAFBFC
            foreground: ThemeColor::rgb(15, 20, 25),    // #0F1419
            surface: ThemeColor::rgb(255, 255, 255),    // #FFFFFF
            surface_foreground: ThemeColor::rgb(15, 20, 25), // #0F1419

            // Semantic
            primary: ThemeColor::rgb(37, 99, 235), // #2563EB
            primary_foreground: ThemeColor::rgb(255, 255, 255), // #FFFFFF
            secondary: ThemeColor::rgb(240, 242, 245), // #F0F2F5
            secondary_foreground: ThemeColor::rgb(31, 41, 55), // #1F2937
            accent: ThemeColor::rgb(232, 240, 254), // #E8F0FE
            accent_foreground: ThemeColor::rgb(29, 78, 216), // #1D4ED8
            selection_foreground: ThemeColor::rgb(29, 78, 216),
            muted: ThemeColor::rgb(244, 245, 247), // #F4F5F7
            muted_foreground: ThemeColor::rgb(104, 111, 125), // #686F7D, #6B7280 darkened for AA on muted (062)

            // Border & ring
            border: ThemeColor::rgb(226, 229, 234), // #E2E5EA
            input: ThemeColor::rgb(226, 229, 234),  // #E2E5EA
            ring: ThemeColor::rgb(37, 99, 235),     // #2563EB

            // Status solid
            success: ThemeColor::rgb(22, 163, 74), // #16A34A
            warning: ThemeColor::rgb(217, 119, 6), // #D97706
            danger: ThemeColor::rgb(220, 38, 38),  // #DC2626
            info: ThemeColor::rgb(37, 99, 235),    // #2563EB

            // Status soft
            success_soft: ThemeColor::rgb(240, 253, 244), // #F0FDF4
            warning_soft: ThemeColor::rgb(255, 251, 235), // #FFFBEB
            danger_soft: ThemeColor::rgb(254, 242, 242),  // #FEF2F2
            info_soft: ThemeColor::rgb(239, 246, 255),    // #EFF6FF

            // Elevation
            surface_elevated: ThemeColor::rgb(255, 255, 255), // #FFFFFF
            surface_overlay: ThemeColor::rgb(255, 255, 255),  // #FFFFFF
            surface_tooltip: ThemeColor::rgb(31, 41, 55),     // #1F2937 (dark for contrast)

            // Interactive states
            primary_hover: ThemeColor::rgb(29, 78, 216), // #1D4ED8
            primary_pressed: ThemeColor::rgb(30, 64, 175), // #1E40AF
            danger_hover: ThemeColor::rgb(185, 28, 28),  // #B91C1C
            danger_pressed: ThemeColor::rgb(153, 27, 27), // #991B1B
            ghost_hover: ThemeColor::rgb(240, 242, 245), // #F0F2F5
            ghost_pressed: ThemeColor::rgb(226, 229, 234), // #E2E5EA

            // Disabled
            disabled_bg: ThemeColor::rgb(240, 241, 243), // #F0F1F3
            disabled_text: ThemeColor::rgb(156, 163, 175), // #9CA3AF
            disabled_foreground: ThemeColor::rgb(103, 108, 116),
        };

        let shadow_base = ThemeColor::rgba(0.0, 0.0, 0.0, 1.0);
        let sh = ShadowTokens {
            sm: ShadowDef {
                offset_x: 0.0,
                offset_y: 1.0,
                blur: 2.0,
                spread: 0.0,
                color: shadow_base.with_alpha(0.04),
            },
            md: ShadowDef {
                offset_x: 0.0,
                offset_y: 4.0,
                blur: 12.0,
                spread: 0.0,
                color: shadow_base.with_alpha(0.06),
            },
            lg: ShadowDef {
                offset_x: 0.0,
                offset_y: 8.0,
                blur: 24.0,
                spread: 0.0,
                color: shadow_base.with_alpha(0.08),
            },
            xl: ShadowDef {
                offset_x: 0.0,
                offset_y: 16.0,
                blur: 48.0,
                spread: 0.0,
                color: shadow_base.with_alpha(0.10),
            },
        };

        (c, sh)
    }

    // ── Dark Palette ---------------------------------------------------------

    fn dark_colors() -> (ColorTokens, ShadowTokens) {
        let c = ColorTokens {
            // Base
            background: ThemeColor::rgb(14, 18, 23), // #0E1217
            foreground: ThemeColor::rgb(240, 242, 245), // #F0F2F5
            surface: ThemeColor::rgb(21, 27, 35),    // #151B23
            surface_foreground: ThemeColor::rgb(240, 242, 245), // #F0F2F5

            // Semantic
            primary: ThemeColor::rgb(91, 141, 239), // #5B8DEF
            primary_foreground: ThemeColor::rgb(15, 20, 25), // #0F1419
            secondary: ThemeColor::rgb(28, 33, 40), // #1C2128
            secondary_foreground: ThemeColor::rgb(226, 229, 234), // #E2E5EA
            accent: ThemeColor::rgb(25, 40, 67),    // #192843
            accent_foreground: ThemeColor::rgb(139, 180, 248), // #8BB4F8
            selection_foreground: ThemeColor::rgb(139, 180, 248),
            muted: ThemeColor::rgb(22, 27, 34), // #161B22
            muted_foreground: ThemeColor::rgb(139, 146, 154), // #8B929A

            // Border & ring
            border: ThemeColor::rgb(45, 51, 59), // #2D333B
            input: ThemeColor::rgb(45, 51, 59),  // #2D333B
            ring: ThemeColor::rgb(91, 141, 239), // #5B8DEF

            // Status solid
            success: ThemeColor::rgb(74, 222, 128), // #4ADE80
            warning: ThemeColor::rgb(251, 191, 36), // #FBBF24
            danger: ThemeColor::rgb(248, 113, 113), // #F87171
            info: ThemeColor::rgb(96, 165, 250),    // #60A5FA

            // Status soft
            success_soft: ThemeColor::rgb(13, 40, 24), // #0D2818
            warning_soft: ThemeColor::rgb(45, 31, 4),  // #2D1F04
            danger_soft: ThemeColor::rgb(45, 10, 10),  // #2D0A0A
            info_soft: ThemeColor::rgb(13, 27, 62),    // #0D1B3E

            // Elevation
            surface_elevated: ThemeColor::rgb(28, 36, 51), // #1C2433
            surface_overlay: ThemeColor::rgb(34, 48, 68),  // #223044
            surface_tooltip: ThemeColor::rgb(45, 51, 59),  // #2D333B

            // Interactive states
            primary_hover: ThemeColor::rgb(115, 161, 242), // #73A1F2
            primary_pressed: ThemeColor::rgb(74, 122, 231), // #4072E6 lightened, AA dark text (062 review)
            danger_hover: ThemeColor::rgb(252, 129, 129),   // #FC8181
            danger_pressed: ThemeColor::rgb(235, 80, 80),   // #EB5050
            ghost_hover: ThemeColor::rgb(28, 33, 40),       // #1C2128
            ghost_pressed: ThemeColor::rgb(45, 51, 59),     // #2D333B

            // Disabled
            disabled_bg: ThemeColor::rgb(28, 33, 40), // #1C2128
            disabled_text: ThemeColor::rgb(75, 85, 99), // #4B5563
            disabled_foreground: ThemeColor::rgb(131, 138, 147),
        };

        let shadow_base = ThemeColor::rgba(0.0, 0.0, 0.0, 1.0);
        let sh = ShadowTokens {
            sm: ShadowDef {
                offset_x: 0.0,
                offset_y: 1.0,
                blur: 2.0,
                spread: 0.0,
                color: shadow_base.with_alpha(0.30),
            },
            md: ShadowDef {
                offset_x: 0.0,
                offset_y: 4.0,
                blur: 12.0,
                spread: 0.0,
                color: shadow_base.with_alpha(0.35),
            },
            lg: ShadowDef {
                offset_x: 0.0,
                offset_y: 8.0,
                blur: 24.0,
                spread: 0.0,
                color: shadow_base.with_alpha(0.45),
            },
            xl: ShadowDef {
                offset_x: 0.0,
                offset_y: 16.0,
                blur: 48.0,
                spread: 0.0,
                color: shadow_base.with_alpha(0.55),
            },
        };

        (c, sh)
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// Tests
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtins_are_complete_and_valid() {
        assert_eq!(Theme::light().validate(), Ok(()));
        assert_eq!(Theme::dark().validate(), Ok(()));
        assert_ne!(
            Theme::light().colors.background,
            Theme::dark().colors.background
        );
    }

    #[test]
    fn light_dark_have_distinct_palettes() {
        let light = Theme::light();
        let dark = Theme::dark();
        assert_ne!(light.colors.primary, dark.colors.primary);
        assert_ne!(light.colors.surface, dark.colors.surface);
        assert_eq!(
            light.colors.primary_foreground,
            ThemeColor::rgb(255, 255, 255)
        );
    }

    #[test]
    fn control_heights_are_monotonic() {
        let t = Theme::light();
        assert!(t.control_heights.xs < t.control_heights.sm);
        assert!(t.control_heights.sm < t.control_heights.md);
        assert!(t.control_heights.md < t.control_heights.lg);
        assert!(t.control_heights.lg < t.control_heights.xl);
    }

    #[test]
    fn color_lerp_and_darken_produce_valid_color() {
        let red = ThemeColor::rgb(255, 0, 0);
        let blue = ThemeColor::rgb(0, 0, 255);
        let purple = red.lerp(blue, 0.5);
        assert!(purple.is_valid());
        assert!(purple.red > 0.0 && purple.blue > 0.0);

        let darker = red.darken(0.2);
        assert!(darker.red < 1.0);
        assert!(darker.is_valid());
    }

    #[test]
    fn spacing_get_returns_expected_values() {
        let t = Theme::light();
        assert_eq!(t.spacing.get(0), 0.0);
        assert_eq!(t.spacing.get(1), t.spacing.px);
        assert_eq!(t.spacing.get(2), t.spacing.px2);
        assert_eq!(t.spacing.get(10), 40.0);
    }

    #[test]
    fn border_color_focused_returns_ring() {
        let t = Theme::light();
        assert_eq!(t.border_color(true, false), t.colors.ring);
        assert_eq!(t.border_color(false, true), t.colors.danger);
        assert_eq!(t.border_color(false, false), t.colors.border);
    }

    #[test]
    fn rejects_invalid_custom_values() {
        let mut t = Theme::light();
        t.colors.primary.alpha = 1.5;
        assert_eq!(t.validate(), Err(ThemeValidationError::InvalidColor));

        let mut bad_spacing = Theme::light();
        bad_spacing.spacing.px4 = -1.0;
        assert_eq!(
            bad_spacing.validate(),
            Err(ThemeValidationError::InvalidSpacing)
        );
        bad_spacing.spacing.px4 = f32::NAN;
        assert_eq!(
            bad_spacing.validate(),
            Err(ThemeValidationError::InvalidSpacing)
        );

        let mut bad_radius = Theme::light();
        bad_radius.radii.md = -2.0;
        assert_eq!(
            bad_radius.validate(),
            Err(ThemeValidationError::InvalidRadius)
        );

        let mut bad_type = Theme::light();
        bad_type.typography.body = 0.0;
        assert_eq!(
            bad_type.validate(),
            Err(ThemeValidationError::InvalidTypography)
        );
    }

    #[test]
    fn validate_rejects_selection_contrast_below_aa_for_normal_text() {
        let mut theme = Theme::light();
        theme.colors.accent = ThemeColor::rgb(0, 0, 0);
        theme.colors.selection_foreground = ThemeColor::rgb(112, 112, 112);
        let ratio = contrast_ratio(theme.colors.selection_foreground, theme.colors.accent);
        assert_eq!(
            theme.validate(),
            Err(ThemeValidationError::SelectionContrastTooLow { ratio })
        );

        theme.colors.selection_foreground = ThemeColor::rgb(117, 117, 117);
        assert_eq!(theme.validate(), Ok(()));
    }

    #[test]
    fn contrast_ratio_black_white_is_21() {
        let black = ThemeColor::rgb(0, 0, 0);
        let white = ThemeColor::rgb(255, 255, 255);
        let ratio = contrast_ratio(black, white);
        assert!((ratio - 21.0).abs() < 0.1, "got {ratio}");
    }

    #[test]
    fn contrast_ratio_identical_is_1() {
        let c = ThemeColor::rgb(120, 120, 120);
        assert!((contrast_ratio(c, c) - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn relative_luminance_extremes() {
        assert!(ThemeColor::rgb(255, 255, 255).relative_luminance() > 0.99);
        assert!(ThemeColor::rgb(0, 0, 0).relative_luminance() < 0.01);
    }

    #[test]
    fn contrast_level_classification() {
        assert_eq!(ContrastLevel::from_ratio(8.0), ContrastLevel::Aaa);
        assert_eq!(ContrastLevel::from_ratio(5.0), ContrastLevel::Aa);
        assert_eq!(ContrastLevel::from_ratio(3.5), ContrastLevel::AaLarge);
        assert_eq!(ContrastLevel::from_ratio(2.0), ContrastLevel::Fail);
        assert!(ContrastLevel::Aa.meets_aa());
        assert!(!ContrastLevel::AaLarge.meets_aa());
    }

    #[test]
    fn builtin_themes_main_text_meets_wcag_aa() {
        // The primary body-text pair must be readable in both built-in themes.
        for theme in [Theme::light(), Theme::dark()] {
            let ratio = contrast_ratio(theme.colors.foreground, theme.colors.background);
            assert!(
                ratio >= 4.5,
                "{:?} foreground/background ratio {ratio} should meet AA",
                theme.mode
            );
        }
    }

    #[test]
    fn wcag_report_covers_key_pairs() {
        let report = Theme::light().wcag_report();
        assert_eq!(report.len(), 9);
        assert!(
            report
                .iter()
                .any(|(name, _, _)| *name == "foreground/background")
        );
    }

    /// Every built-in theme plus every registered pack, in both modes.
    fn all_pack_themes() -> Vec<(String, Theme)> {
        let mut themes = vec![
            ("default/light".to_string(), Theme::light()),
            ("default/dark".to_string(), Theme::dark()),
        ];
        for name in packs::available_themes() {
            for mode in [ThemeMode::Light, ThemeMode::Dark] {
                let theme = packs::theme_by_name(name, mode).expect("registered pack resolves");
                themes.push((format!("{name}/{mode:?}"), theme));
            }
        }
        themes
    }

    #[test]
    fn selected_item_colors_meet_aa_in_every_pack_and_mode() {
        let themes = all_pack_themes();
        assert_eq!(themes.len(), 26, "2 built-ins + 12 packs x 2 modes");
        for (label, theme) in themes {
            let (bg, fg) = theme.selected_item_colors();
            let ratio = contrast_ratio(fg, bg);
            assert!(
                ratio >= 4.5,
                "{label}: selected item fg/bg ratio {ratio:.2} must be >= 4.5"
            );
        }
    }

    #[test]
    fn state_foreground_tokens_have_aa_contrast_in_every_pack_and_mode() {
        for (label, theme) in all_pack_themes() {
            let c = theme.colors;
            let report = theme.wcag_report();
            for (name, fg, bg) in [
                (
                    "selection_foreground/accent",
                    c.selection_foreground,
                    c.accent,
                ),
                (
                    "disabled_foreground/disabled_bg",
                    c.disabled_foreground,
                    c.disabled_bg,
                ),
            ] {
                let ratio = contrast_ratio(fg, bg);
                assert!(ratio >= 4.5, "{label} {name}: {ratio:.2}");
                assert!(report.contains(&(name, ratio, ContrastLevel::from_ratio(ratio))));
            }
        }
    }

    #[test]
    fn selected_item_colors_resolve_the_selection_foreground_token() {
        for (label, theme) in all_pack_themes() {
            let c = theme.colors;
            assert_eq!(
                theme.selected_item_colors(),
                (c.accent, c.selection_foreground),
                "{label}"
            );
        }
        let mut custom = Theme::light();
        custom.colors.selection_foreground = ThemeColor::rgb(12, 34, 56);
        assert_eq!(
            custom.selected_item_colors().1,
            custom.colors.selection_foreground
        );
    }

    #[test]
    fn wcag_report_includes_selected_item_pair() {
        for (label, theme) in all_pack_themes() {
            let (bg, fg) = theme.selected_item_colors();
            let report = theme.wcag_report();
            let row = report
                .iter()
                .find(|(name, _, _)| *name == "selected_fg/selected_bg")
                .unwrap_or_else(|| panic!("{label}: selected_fg/selected_bg row missing"));
            let expected = contrast_ratio(fg, bg);
            assert!((row.1 - expected).abs() < f32::EPSILON, "{label}");
            assert_eq!(row.2, ContrastLevel::from_ratio(expected));
        }
    }

    type Rgb062 = (u8, u8, u8);
    /// ROADMAP 062 retune record: `(pack/mode, token, before, after, pair,
    /// pair ratio before, pair ratio after)`. "Before" ratios are measured on
    /// the pre-062 palette (every 062 change for that pack/mode reverted).
    /// The table is the source of truth for `logs/token-changes.md`.
    #[allow(clippy::type_complexity, clippy::approx_constant)] // measured ratios, e.g. 3.14
    #[rustfmt::skip]
    const ROADMAP_062_CHANGES: &[(&str, &str, Rgb062, Rgb062, &str, f32, f32)] = &[
        ("default/light", "muted_foreground", (107, 114, 128), (104, 111, 125), "muted_fg/muted", 4.43, 4.63),
        ("default/dark", "primary_pressed", (64, 114, 230), (74, 122, 231), "primary_fg/primary_pressed", 4.20, 4.61),
        ("default/dark", "primary_pressed", (64, 114, 230), (74, 122, 231), "primary_pressed/primary", 1.36, 1.24),
        ("apple/Light", "primary", (0, 122, 255), (0, 112, 235), "primary_fg/primary", 4.02, 4.65),
        ("apple/Light", "primary", (0, 122, 255), (0, 112, 235), "primary/background", 3.60, 4.17),
        ("apple/Light", "accent_foreground", (0, 122, 255), (0, 102, 214), "accent_fg/accent", 3.41, 4.61),
        ("apple/Light", "muted_foreground", (142, 142, 147), (109, 109, 114), "muted_fg/muted", 2.92, 4.61),
        ("apple/Light", "success", (52, 199, 89), (41, 157, 70), "success/surface", 2.22, 3.50),
        ("apple/Light", "success", (52, 199, 89), (41, 157, 70), "success/background", 1.99, 3.13),
        ("apple/Light", "warning", (255, 149, 0), (201, 117, 0), "warning/surface", 2.20, 3.48),
        ("apple/Light", "warning", (255, 149, 0), (201, 117, 0), "warning/background", 1.97, 3.12),
        ("apple/Light", "info", (90, 200, 250), (6, 146, 210), "info/surface", 1.90, 3.47),
        ("apple/Light", "info", (90, 200, 250), (6, 146, 210), "info/background", 1.70, 3.11),
        ("apple/Light", "danger_hover", (214, 49, 40), (255, 105, 97), "best_ink/danger_hover", 4.85, 7.45),
        ("apple/Light", "danger_hover", (214, 49, 40), (255, 105, 97), "danger_hover/danger", 1.37, 1.26),
        ("apple/Light", "danger_pressed", (173, 40, 32), (255, 128, 121), "best_ink/danger_pressed", 6.77, 8.62),
        ("apple/Light", "danger_pressed", (173, 40, 32), (255, 128, 121), "danger_pressed/danger", 1.91, 1.46),
        ("apple/Dark", "primary", (10, 132, 255), (0, 114, 230), "primary_fg/primary", 3.65, 4.62),
        ("apple/Dark", "primary", (10, 132, 255), (0, 114, 230), "primary/background", 5.76, 4.55),
        ("apple/Dark", "accent_foreground", (10, 132, 255), (64, 156, 255), "accent_fg/accent", 4.06, 5.23),
        ("apple/Dark", "primary_hover", (64, 156, 255), (0, 102, 214), "primary_fg/primary_hover", 2.83, 5.42),
        ("apple/Dark", "primary_hover", (64, 156, 255), (0, 102, 214), "primary_hover/primary", 1.29, 1.18),
        ("apple/Dark", "primary_pressed", (0, 102, 214), (0, 82, 173), "primary_fg/primary_pressed", 5.42, 7.47),
        ("apple/Dark", "primary_pressed", (0, 102, 214), (0, 82, 173), "primary_pressed/primary", 1.49, 1.62),
        ("apple/Dark", "danger_pressed", (214, 49, 40), (255, 128, 121), "best_ink/danger_pressed", 4.85, 8.62),
        ("apple/Dark", "danger_pressed", (214, 49, 40), (255, 128, 121), "danger_pressed/danger", 1.42, 1.40),
        ("dracula/Light", "primary", (189, 147, 249), (168, 112, 247), "primary_fg/primary", 5.90, 4.63),
        ("dracula/Light", "primary", (189, 147, 249), (168, 112, 247), "primary/background", 2.26, 3.11),
        ("dracula/Light", "primary_foreground", (40, 42, 54), (35, 36, 47), "primary_fg/primary", 5.90, 4.63),
        ("dracula/Light", "accent_foreground", (120, 80, 200), (118, 78, 199), "accent_fg/accent", 4.49, 4.61),
        ("dracula/Light", "muted_foreground", (98, 114, 164), (88, 103, 152), "muted_fg/muted", 3.93, 4.61),
        ("dracula/Light", "ring", (189, 147, 249), (168, 112, 247), "ring/background", 2.26, 3.11),
        ("dracula/Light", "ring", (189, 147, 249), (168, 112, 247), "ring/surface", 2.41, 3.32),
        ("dracula/Light", "success", (80, 250, 123), (5, 164, 45), "success/surface", 1.37, 3.30),
        ("dracula/Light", "success", (80, 250, 123), (5, 164, 45), "success/background", 1.29, 3.10),
        ("dracula/Light", "warning", (241, 250, 140), (137, 148, 6), "warning/surface", 1.12, 3.33),
        ("dracula/Light", "warning", (241, 250, 140), (137, 148, 6), "warning/background", 1.05, 3.13),
        ("dracula/Light", "danger", (255, 85, 85), (255, 74, 74), "danger/surface", 3.14, 3.32),
        ("dracula/Light", "danger", (255, 85, 85), (255, 74, 74), "danger/background", 2.95, 3.11),
        ("dracula/Light", "danger", (255, 85, 85), (255, 74, 74), "best_ink/danger", 4.53, 4.64),
        ("dracula/Light", "info", (139, 233, 253), (3, 154, 186), "info/surface", 1.38, 3.31),
        ("dracula/Light", "info", (139, 233, 253), (3, 154, 186), "info/background", 1.30, 3.11),
        ("dracula/Light", "primary_pressed", (150, 110, 220), (210, 182, 251), "primary_fg/primary_pressed", 3.77, 8.66),
        ("dracula/Light", "primary_pressed", (150, 110, 220), (210, 182, 251), "primary_pressed/primary", 1.57, 1.87),
        ("dracula/Light", "danger_pressed", (220, 60, 60), (255, 132, 132), "best_ink/danger_pressed", 4.14, 6.51),
        ("dracula/Light", "danger_pressed", (220, 60, 60), (255, 132, 132), "danger_pressed/danger", 1.41, 1.40),
        ("dracula/Dark", "accent_foreground", (255, 121, 198), (255, 151, 211), "accent_fg/accent", 3.84, 4.63),
        ("dracula/Dark", "muted_foreground", (98, 114, 164), (175, 184, 209), "muted_fg/muted", 1.94, 4.62),
        ("dracula/Dark", "danger", (255, 85, 85), (255, 97, 97), "danger/surface", 2.91, 3.11),
        ("dracula/Dark", "danger", (255, 85, 85), (255, 97, 97), "danger/background", 4.53, 4.84),
        ("dracula/Dark", "danger", (255, 85, 85), (255, 97, 97), "best_ink/danger", 4.53, 4.84),
        ("dracula/Dark", "primary_pressed", (150, 110, 220), (210, 182, 251), "primary_fg/primary_pressed", 3.77, 8.02),
        ("dracula/Dark", "primary_pressed", (150, 110, 220), (210, 182, 251), "primary_pressed/primary", 1.57, 1.36),
        ("dracula/Dark", "danger_hover", (255, 110, 110), (255, 123, 123), "best_ink/danger_hover", 5.23, 5.68),
        ("dracula/Dark", "danger_hover", (255, 110, 110), (255, 123, 123), "danger_hover/danger", 1.15, 1.17),
        ("dracula/Dark", "danger_pressed", (220, 60, 60), (255, 144, 144), "best_ink/danger_pressed", 4.14, 6.54),
        ("dracula/Dark", "danger_pressed", (220, 60, 60), (255, 144, 144), "danger_pressed/danger", 1.41, 1.35),
        ("gruvbox/Light", "primary", (69, 133, 136), (61, 117, 120), "primary_fg/primary", 3.73, 4.61),
        ("gruvbox/Light", "primary", (69, 133, 136), (61, 117, 120), "primary/background", 3.73, 4.61),
        ("gruvbox/Light", "muted_foreground", (124, 111, 100), (102, 92, 84), "muted_fg/muted", 3.55, 4.75),
        ("gruvbox/Light", "success", (152, 151, 26), (121, 116, 14), "success/surface", 2.26, 3.55),
        ("gruvbox/Light", "success", (152, 151, 26), (121, 116, 14), "success/background", 2.73, 4.29),
        ("gruvbox/Light", "warning", (215, 153, 33), (161, 114, 25), "warning/surface", 1.81, 3.10),
        ("gruvbox/Light", "warning", (215, 153, 33), (161, 114, 25), "warning/background", 2.19, 3.75),
        ("gruvbox/Light", "primary_hover", (90, 150, 153), (55, 105, 108), "primary_fg/primary_hover", 2.96, 5.44),
        ("gruvbox/Light", "primary_hover", (90, 150, 153), (55, 105, 108), "primary_hover/primary", 1.26, 1.18),
        ("gruvbox/Light", "primary_pressed", (55, 105, 108), (50, 95, 98), "primary_fg/primary_pressed", 5.44, 6.28),
        ("gruvbox/Light", "primary_pressed", (55, 105, 108), (50, 95, 98), "primary_pressed/primary", 1.46, 1.36),
        ("gruvbox/Light", "danger_hover", (215, 60, 52), (184, 33, 26), "best_ink/danger_hover", 4.04, 5.66),
        ("gruvbox/Light", "danger_hover", (215, 60, 52), (184, 33, 26), "danger_hover/danger", 1.19, 1.17),
        ("gruvbox/Dark", "primary", (69, 133, 136), (81, 156, 159), "primary_fg/primary", 3.73, 4.64),
        ("gruvbox/Dark", "primary", (69, 133, 136), (81, 156, 159), "primary/background", 3.48, 4.64),
        ("gruvbox/Dark", "primary_foreground", (251, 241, 199), (40, 40, 40), "primary_fg/primary", 3.73, 4.64),
        ("gruvbox/Dark", "muted_foreground", (168, 153, 132), (189, 174, 147), "muted_fg/muted", 4.17, 5.32),
        ("gruvbox/Dark", "ring", (69, 133, 136), (81, 156, 159), "ring/background", 3.48, 4.64),
        ("gruvbox/Dark", "ring", (69, 133, 136), (81, 156, 159), "ring/surface", 2.74, 3.65),
        ("gruvbox/Dark", "danger", (204, 36, 29), (251, 88, 69), "danger/surface", 2.12, 3.64),
        ("gruvbox/Dark", "danger", (204, 36, 29), (251, 88, 69), "danger/background", 2.69, 4.62),
        ("gruvbox/Dark", "danger", (204, 36, 29), (251, 88, 69), "best_ink/danger", 4.82, 4.62),
        ("gruvbox/Dark", "info", (69, 133, 136), (81, 156, 159), "info/surface", 2.74, 3.65),
        ("gruvbox/Dark", "info", (69, 133, 136), (81, 156, 159), "info/background", 3.48, 4.64),
        ("gruvbox/Dark", "primary_hover", (90, 150, 153), (93, 169, 172), "primary_fg/primary_hover", 2.96, 5.43),
        ("gruvbox/Dark", "primary_hover", (90, 150, 153), (93, 169, 172), "primary_hover/primary", 1.26, 1.17),
        ("gruvbox/Dark", "primary_pressed", (55, 105, 108), (113, 180, 182), "primary_fg/primary_pressed", 5.44, 6.25),
        ("gruvbox/Dark", "primary_pressed", (55, 105, 108), (113, 180, 182), "primary_pressed/primary", 1.46, 1.35),
        ("gruvbox/Dark", "danger_hover", (215, 60, 52), (252, 114, 97), "best_ink/danger_hover", 4.04, 5.42),
        ("gruvbox/Dark", "danger_hover", (215, 60, 52), (252, 114, 97), "danger_hover/danger", 1.19, 1.17),
        ("gruvbox/Dark", "danger_pressed", (170, 30, 24), (252, 135, 121), "best_ink/danger_pressed", 6.36, 6.23),
        ("gruvbox/Dark", "danger_pressed", (170, 30, 24), (252, 135, 121), "danger_pressed/danger", 1.32, 1.35),
        ("material/Dark", "primary_pressed", (103, 80, 164), (160, 145, 201), "primary_fg/primary_pressed", 2.04, 4.63),
        ("material/Dark", "primary_pressed", (103, 80, 164), (160, 145, 201), "primary_pressed/primary", 3.78, 1.67),
        ("nord/Light", "primary", (94, 129, 172), (77, 109, 149), "primary_fg/primary", 3.50, 4.63),
        ("nord/Light", "primary", (94, 129, 172), (77, 109, 149), "primary/background", 3.50, 4.63),
        ("nord/Light", "accent_foreground", (94, 129, 172), (76, 86, 106), "accent_fg/accent", 2.98, 5.46),
        ("nord/Light", "success", (163, 190, 140), (107, 140, 79), "success/surface", 1.67, 3.14),
        ("nord/Light", "success", (163, 190, 140), (107, 140, 79), "success/background", 1.77, 3.32),
        ("nord/Light", "warning", (235, 203, 139), (170, 123, 29), "warning/surface", 1.28, 3.10),
        ("nord/Light", "warning", (235, 203, 139), (170, 123, 29), "warning/background", 1.35, 3.28),
        ("nord/Light", "info", (136, 192, 208), (64, 141, 163), "info/surface", 1.64, 3.10),
        ("nord/Light", "info", (136, 192, 208), (64, 141, 163), "info/background", 1.74, 3.28),
        ("nord/Light", "danger", (191, 97, 106), (178, 72, 82), "danger/surface", 3.36, 4.38),
        ("nord/Light", "danger", (191, 97, 106), (178, 72, 82), "danger/background", 3.55, 4.62),
        ("nord/Light", "danger", (191, 97, 106), (178, 72, 82), "best_ink/danger", 3.55, 4.62),
        ("nord/Light", "primary_hover", (129, 161, 193), (70, 98, 135), "primary_fg/primary_hover", 2.34, 5.43),
        ("nord/Light", "primary_hover", (129, 161, 193), (70, 98, 135), "primary_hover/primary", 1.50, 1.17),
        ("nord/Light", "danger_hover", (176, 85, 94), (160, 65, 74), "best_ink/danger_hover", 4.23, 5.44),
        ("nord/Light", "danger_hover", (176, 85, 94), (160, 65, 74), "danger_hover/danger", 1.19, 1.18),
        ("nord/Light", "danger_pressed", (155, 74, 83), (145, 59, 67), "best_ink/danger_pressed", 5.21, 6.26),
        ("nord/Light", "danger_pressed", (155, 74, 83), (145, 59, 67), "danger_pressed/danger", 1.47, 1.35),
        ("nord/Dark", "accent_foreground", (136, 192, 208), (147, 198, 212), "accent_fg/accent", 4.31, 4.63),
        ("nord/Dark", "danger", (191, 97, 106), (208, 139, 145), "danger/surface", 2.46, 3.73),
        ("nord/Dark", "danger", (191, 97, 106), (208, 139, 145), "danger/background", 3.05, 4.63),
        ("nord/Dark", "danger", (191, 97, 106), (208, 139, 145), "best_ink/danger", 3.55, 4.63),
        ("nord/Dark", "primary_hover", (143, 188, 187), (162, 206, 218), "primary_fg/primary_hover", 5.99, 7.36),
        ("nord/Dark", "primary_hover", (143, 188, 187), (162, 206, 218), "primary_hover/primary", 1.04, 1.18),
        ("nord/Dark", "primary_pressed", (94, 129, 172), (186, 219, 228), "primary_fg/primary_pressed", 3.10, 8.53),
        ("nord/Dark", "primary_pressed", (94, 129, 172), (186, 219, 228), "primary_pressed/primary", 2.01, 1.37),
        ("nord/Dark", "danger_hover", (208, 120, 128), (215, 156, 161), "best_ink/danger_hover", 3.97, 5.45),
        ("nord/Dark", "danger_hover", (208, 120, 128), (215, 156, 161), "danger_hover/danger", 1.30, 1.18),
        ("nord/Dark", "danger_pressed", (191, 97, 106), (221, 172, 176), "best_ink/danger_pressed", 3.55, 6.31),
        ("nord/Dark", "danger_pressed", (191, 97, 106), (221, 172, 176), "danger_pressed/danger", 1.00, 1.36),
        ("one-dark/Light", "primary", (64, 120, 242), (47, 108, 241), "primary_fg/primary", 4.05, 4.63),
        ("one-dark/Light", "primary", (64, 120, 242), (47, 108, 241), "primary/background", 3.88, 4.44),
        ("one-dark/Light", "muted_foreground", (160, 161, 167), (106, 108, 115), "muted_fg/muted", 2.26, 4.61),
        ("one-dark/Light", "danger", (228, 86, 73), (221, 49, 33), "danger/surface", 3.67, 4.63),
        ("one-dark/Light", "danger", (228, 86, 73), (221, 49, 33), "danger/background", 3.51, 4.44),
        ("one-dark/Light", "danger", (228, 86, 73), (221, 49, 33), "best_ink/danger", 3.67, 4.63),
        ("one-dark/Light", "primary_hover", (90, 140, 245), (26, 93, 240), "primary_fg/primary_hover", 3.23, 5.43),
        ("one-dark/Light", "primary_hover", (90, 140, 245), (26, 93, 240), "primary_hover/primary", 1.25, 1.17),
        ("one-dark/Light", "danger_hover", (234, 110, 98), (201, 44, 30), "best_ink/danger_hover", 3.73, 5.44),
        ("one-dark/Light", "danger_hover", (234, 110, 98), (201, 44, 30), "danger_hover/danger", 1.21, 1.17),
        ("one-dark/Light", "danger_pressed", (198, 68, 57), (184, 40, 27), "best_ink/danger_pressed", 4.90, 6.25),
        ("one-dark/Light", "danger_pressed", (198, 68, 57), (184, 40, 27), "danger_pressed/danger", 1.34, 1.35),
        ("one-dark/Dark", "muted_foreground", (92, 99, 112), (141, 149, 162), "muted_fg/muted", 2.55, 5.10),
        ("one-dark/Dark", "muted_foreground", (92, 99, 112), (141, 149, 162), "muted_fg/background", 2.32, 4.64),
        ("one-dark/Dark", "primary_pressed", (70, 145, 210), (146, 203, 247), "primary_fg/primary_pressed", 4.58, 8.88),
        ("one-dark/Dark", "primary_pressed", (70, 145, 210), (146, 203, 247), "primary_pressed/primary", 1.42, 1.36),
        ("one-dark/Dark", "danger_pressed", (195, 85, 94), (235, 148, 155), "best_ink/danger_pressed", 3.51, 6.77),
        ("one-dark/Dark", "danger_pressed", (195, 85, 94), (235, 148, 155), "danger_pressed/danger", 1.37, 1.41),
        ("solarized/Light", "foreground", (101, 123, 131), (80, 97, 104), "foreground/background", 4.13, 5.99),
        ("solarized/Light", "foreground", (101, 123, 131), (80, 97, 104), "foreground/surface", 3.64, 5.27),
        ("solarized/Light", "foreground", (101, 123, 131), (80, 97, 104), "foreground/muted_fg", 1.67, 1.15),
        ("solarized/Light", "foreground", (101, 123, 131), (80, 97, 104), "foreground/ghost_pressed", 3.19, 4.63),
        ("solarized/Light", "surface_foreground", (101, 123, 131), (80, 97, 104), "surface_fg/surface", 3.64, 5.27),
        ("solarized/Light", "primary", (38, 139, 210), (32, 116, 176), "primary_fg/primary", 3.41, 4.65),
        ("solarized/Light", "primary", (38, 139, 210), (32, 116, 176), "primary/background", 3.41, 4.65),
        ("solarized/Light", "secondary", (42, 161, 152), (32, 124, 117), "secondary_fg/secondary", 2.93, 4.63),
        ("solarized/Light", "muted_foreground", (147, 161, 161), (92, 106, 106), "muted_fg/muted", 2.18, 4.60),
        ("solarized/Light", "success", (133, 153, 0), (121, 139, 0), "success/surface", 2.62, 3.11),
        ("solarized/Light", "success", (133, 153, 0), (121, 139, 0), "success/background", 2.97, 3.54),
        ("solarized/Light", "warning", (181, 137, 0), (164, 124, 0), "warning/surface", 2.62, 3.14),
        ("solarized/Light", "warning", (181, 137, 0), (164, 124, 0), "warning/background", 2.98, 3.56),
        ("solarized/Light", "info", (42, 161, 152), (38, 145, 137), "info/surface", 2.58, 3.12),
        ("solarized/Light", "info", (42, 161, 152), (38, 145, 137), "info/background", 2.93, 3.54),
        ("solarized/Light", "danger", (220, 50, 47), (215, 39, 36), "danger/surface", 3.77, 4.08),
        ("solarized/Light", "danger", (220, 50, 47), (215, 39, 36), "danger/background", 4.29, 4.63),
        ("solarized/Light", "danger", (220, 50, 47), (215, 39, 36), "best_ink/danger", 4.29, 4.63),
        ("solarized/Light", "primary_hover", (60, 155, 220), (29, 105, 159), "primary_fg/primary_hover", 2.82, 5.45),
        ("solarized/Light", "primary_hover", (60, 155, 220), (29, 105, 159), "primary_hover/primary", 1.21, 1.17),
        ("solarized/Light", "primary_pressed", (28, 110, 170), (26, 95, 144), "primary_fg/primary_pressed", 5.04, 6.31),
        ("solarized/Light", "primary_pressed", (28, 110, 170), (26, 95, 144), "primary_pressed/primary", 1.48, 1.36),
        ("solarized/Light", "danger_hover", (228, 80, 77), (190, 40, 37), "best_ink/danger_hover", 3.51, 5.53),
        ("solarized/Light", "danger_hover", (228, 80, 77), (190, 40, 37), "danger_hover/danger", 1.22, 1.19),
        ("solarized/Light", "danger_pressed", (190, 40, 37), (173, 36, 34), "best_ink/danger_pressed", 5.53, 6.38),
        ("solarized/Light", "danger_pressed", (190, 40, 37), (173, 36, 34), "danger_pressed/danger", 1.29, 1.38),
        ("solarized/Dark", "foreground", (131, 148, 150), (157, 170, 170), "foreground/background", 4.75, 6.27),
        ("solarized/Dark", "foreground", (131, 148, 150), (157, 170, 170), "foreground/surface", 4.11, 5.43),
        ("solarized/Dark", "foreground", (131, 148, 150), (157, 170, 170), "foreground/muted_fg", 1.70, 1.12),
        ("solarized/Dark", "surface_foreground", (131, 148, 150), (157, 170, 170), "surface_fg/surface", 4.11, 5.43),
        ("solarized/Dark", "primary_foreground", (253, 246, 227), (0, 32, 41), "primary_fg/primary", 3.41, 4.61),
        ("solarized/Dark", "secondary_foreground", (131, 148, 150), (147, 161, 161), "secondary_fg/secondary", 4.11, 4.86),
        ("solarized/Dark", "accent_foreground", (42, 161, 152), (45, 171, 161), "accent_fg/accent", 4.12, 4.61),
        ("solarized/Dark", "muted_foreground", (88, 110, 117), (147, 161, 161), "muted_fg/muted", 2.42, 4.86),
        ("solarized/Dark", "danger", (220, 50, 47), (226, 87, 84), "danger/surface", 2.81, 3.54),
        ("solarized/Dark", "danger", (220, 50, 47), (226, 87, 84), "danger/background", 3.25, 4.09),
        ("solarized/Dark", "danger", (220, 50, 47), (226, 87, 84), "best_ink/danger", 4.29, 4.62),
        ("solarized/Dark", "primary_pressed", (28, 110, 170), (82, 167, 224), "primary_fg/primary_pressed", 5.04, 6.43),
        ("solarized/Dark", "primary_pressed", (28, 110, 170), (82, 167, 224), "primary_pressed/primary", 1.48, 1.40),
        ("solarized/Dark", "danger_hover", (228, 80, 77), (230, 108, 105), "best_ink/danger_hover", 3.97, 5.42),
        ("solarized/Dark", "danger_hover", (228, 80, 77), (230, 108, 105), "danger_hover/danger", 1.22, 1.17),
        ("solarized/Dark", "danger_pressed", (190, 40, 37), (233, 126, 123), "best_ink/danger_pressed", 5.53, 6.24),
        ("solarized/Dark", "danger_pressed", (190, 40, 37), (233, 126, 123), "danger_pressed/danger", 1.29, 1.35),
        ("ubuntu/Light", "primary", (233, 84, 32), (208, 69, 21), "primary_fg/primary", 3.65, 4.62),
        ("ubuntu/Light", "primary", (233, 84, 32), (208, 69, 21), "primary/background", 3.50, 4.43),
        ("ubuntu/Light", "warning", (249, 154, 0), (203, 125, 0), "warning/surface", 2.18, 3.24),
        ("ubuntu/Light", "warning", (249, 154, 0), (203, 125, 0), "warning/background", 2.08, 3.11),
        ("ubuntu/Light", "primary_hover", (205, 73, 27), (188, 62, 19), "primary_fg/primary_hover", 4.60, 5.47),
        ("ubuntu/Light", "primary_hover", (205, 73, 27), (188, 62, 19), "primary_hover/primary", 1.26, 1.18),
        ("ubuntu/Dark", "primary_foreground", (255, 255, 255), (17, 17, 17), "primary_fg/primary", 3.65, 5.17),
        ("ubuntu/Dark", "primary_pressed", (205, 73, 27), (242, 125, 84), "primary_fg/primary_pressed", 4.60, 7.05),
        ("ubuntu/Dark", "primary_pressed", (205, 73, 27), (242, 125, 84), "primary_pressed/primary", 1.26, 1.36),
        ("ubuntu/Dark", "danger_pressed", (218, 34, 34), (255, 150, 150), "best_ink/danger_pressed", 4.96, 9.04),
        ("ubuntu/Dark", "danger_pressed", (218, 34, 34), (255, 150, 150), "danger_pressed/danger", 1.70, 1.39),
        ("windows11/Dark", "primary_pressed", (0, 103, 192), (0, 120, 212), "primary_fg/primary_pressed", 3.70, 4.64),
        ("windows11/Dark", "primary_pressed", (0, 103, 192), (0, 120, 212), "primary_pressed/primary", 2.83, 2.26),
    ];

    fn color_token_mut<'a>(c: &'a mut ColorTokens, token: &str) -> &'a mut ThemeColor {
        match token {
            "foreground" => &mut c.foreground,
            "surface_foreground" => &mut c.surface_foreground,
            "primary" => &mut c.primary,
            "primary_foreground" => &mut c.primary_foreground,
            "secondary" => &mut c.secondary,
            "secondary_foreground" => &mut c.secondary_foreground,
            "accent" => &mut c.accent,
            "accent_foreground" => &mut c.accent_foreground,
            "muted_foreground" => &mut c.muted_foreground,
            "ring" => &mut c.ring,
            "success" => &mut c.success,
            "warning" => &mut c.warning,
            "danger" => &mut c.danger,
            "info" => &mut c.info,
            "primary_hover" => &mut c.primary_hover,
            "primary_pressed" => &mut c.primary_pressed,
            "danger_hover" => &mut c.danger_hover,
            "danger_pressed" => &mut c.danger_pressed,
            other => panic!("unexpected token {other}"),
        }
    }

    /// `(ratio, rule minimum)` for a 062 gate pair. `best_ink/<fill>` is the
    /// best of the danger ink candidates on that danger fill.
    fn gate_pair_ratio(c: &ColorTokens, pair: &str) -> (f32, f32) {
        if let Some(fill) = pair.strip_prefix("best_ink/") {
            let fill = match fill {
                "danger" => c.danger,
                "danger_hover" => c.danger_hover,
                "danger_pressed" => c.danger_pressed,
                other => panic!("unexpected fill {other}"),
            };
            let best = [c.primary_foreground, c.background, c.foreground]
                .into_iter()
                .map(|ink| contrast_ratio(ink, fill))
                .fold(0.0_f32, f32::max);
            return (best, 4.5);
        }
        let (fg, bg, min) = match pair {
            "foreground/background" => (c.foreground, c.background, 4.5),
            "surface_fg/surface" => (c.surface_foreground, c.surface, 4.5),
            "primary_fg/primary" => (c.primary_foreground, c.primary, 4.5),
            "secondary_fg/secondary" => (c.secondary_foreground, c.secondary, 4.5),
            "muted_fg/muted" => (c.muted_foreground, c.muted, 4.5),
            "accent_fg/accent" => (c.accent_foreground, c.accent, 4.5),
            "foreground/surface" => (c.foreground, c.surface, 4.5),
            "ring/background" => (c.ring, c.background, 3.0),
            "ring/surface" => (c.ring, c.surface, 3.0),
            "primary/background" => (c.primary, c.background, 3.0),
            "success/surface" => (c.success, c.surface, 3.0),
            "success/background" => (c.success, c.background, 3.0),
            "warning/surface" => (c.warning, c.surface, 3.0),
            "warning/background" => (c.warning, c.background, 3.0),
            "danger/surface" => (c.danger, c.surface, 3.0),
            "danger/background" => (c.danger, c.background, 3.0),
            "info/surface" => (c.info, c.surface, 3.0),
            "info/background" => (c.info, c.background, 3.0),
            "primary_fg/primary_hover" => (c.primary_foreground, c.primary_hover, 4.5),
            "primary_fg/primary_pressed" => (c.primary_foreground, c.primary_pressed, 4.5),
            "muted_fg/background" => (c.muted_foreground, c.background, 4.5),
            "foreground/ghost_pressed" => (c.foreground, c.ghost_pressed, 4.5),
            // State steps and text hierarchy (luminance distance).
            "primary_hover/primary" => (c.primary_hover, c.primary, 1.15),
            "primary_pressed/primary" => (c.primary_pressed, c.primary, 1.15),
            "danger_hover/danger" => (c.danger_hover, c.danger, 1.15),
            "danger_pressed/danger" => (c.danger_pressed, c.danger, 1.15),
            "foreground/muted_fg" => (c.foreground, c.muted_foreground, 1.1),
            other => panic!("unexpected pair {other}"),
        };
        (contrast_ratio(fg, bg), min)
    }

    /// Restore the pre-062 palette for `label` (e.g. `"solarized/Light"`).
    fn revert_roadmap_062(c: &mut ColorTokens, label: &str) {
        for (l, token, before, _, _, _, _) in ROADMAP_062_CHANGES {
            if *l == label {
                *color_token_mut(c, token) = ThemeColor::rgb(before.0, before.1, before.2);
            }
        }
    }

    /// ROADMAP 062: each changed token old -> new, with its pair ratio before
    /// and after. After ratios keep the brief's 0.1 margin (>= 4.6 / >= 3.1).
    #[test]
    fn roadmap_062_pack_retune_reaches_aa() {
        let themes = all_pack_themes();
        let mut tokens: Vec<(&str, &str)> = ROADMAP_062_CHANGES
            .iter()
            .map(|(label, token, ..)| (*label, *token))
            .collect();
        tokens.sort_unstable();
        tokens.dedup();
        assert_eq!(
            tokens.len(),
            104,
            "062 changed-token count (38 + 21 status + 45 review fixes)"
        );
        // Packs outside the brief's retune: only the review P1-2 text gate
        // (primary_fg on primary_pressed) moved one state token each.
        for untouched in [
            "default/dark",
            "windows10/",
            "windows11/",
            "material/",
            "high-contrast/",
        ] {
            let moved: Vec<_> = tokens
                .iter()
                .filter(|(l, _)| l.starts_with(untouched))
                .collect();
            assert!(
                moved.iter().all(|(_, t)| *t == "primary_pressed"),
                "{untouched} may only move primary_pressed: {moved:?}"
            );
        }
        let state_only: Vec<&str> = tokens
            .iter()
            .filter(|(l, _)| !ROADMAP_062_RETUNED.contains(l))
            .map(|(l, _)| *l)
            .collect();
        assert_eq!(
            state_only,
            ["default/dark", "material/Dark", "windows11/Dark"]
        );
        for (label, token, before, after, pair, ratio_before, ratio_after) in ROADMAP_062_CHANGES {
            let theme = &themes
                .iter()
                .find(|(l, _)| l == label)
                .unwrap_or_else(|| panic!("{label} resolves"))
                .1;
            let mut current = theme.colors;
            assert_eq!(
                *color_token_mut(&mut current, token),
                ThemeColor::rgb(after.0, after.1, after.2),
                "{label} {token} after value"
            );
            let mut old = theme.colors;
            revert_roadmap_062(&mut old, label);
            assert_eq!(
                *color_token_mut(&mut old, token),
                ThemeColor::rgb(before.0, before.1, before.2)
            );
            let (old_ratio, min) = gate_pair_ratio(&old, pair);
            let (new_ratio, _) = gate_pair_ratio(&theme.colors, pair);
            assert!(
                (old_ratio - ratio_before).abs() < 0.01,
                "{label} {pair} before {old_ratio:.2} != {ratio_before}"
            );
            assert!(
                (new_ratio - ratio_after).abs() < 0.01,
                "{label} {pair} after {new_ratio:.2} != {ratio_after}"
            );
            // 0.1 margin for 4.5 / 3 rules; ~1.5% for the 1.15 / 1.1 steps.
            let margin = if min >= 3.0 { 0.095 } else { 0.01 };
            assert!(
                new_ratio >= min + margin,
                "{label} {pair} after {new_ratio:.3} lacks the margin over {min}"
            );
        }
    }

    /// ROADMAP 060 retune record: D1 selected-item ratio before -> after.
    /// Only solarized light `accent_foreground`, gruvbox light `accent`, and
    /// gruvbox dark `accent` + `accent_foreground` changed; every other pack's
    /// D1 ratio is untouched.
    ///
    /// | pack/mode       | token(s)                  | before            | after             | D1 before | D1 after |
    /// |-----------------|---------------------------|-------------------|-------------------|-----------|----------|
    /// | solarized/Light | accent_foreground         | (108,113,196)     | (95,99,172)       | 3.78      | >= 4.6   |
    /// | gruvbox/Light   | accent                    | (214,93,14)       | (175,58,3)        | 3.41      | >= 5.3   |
    /// | gruvbox/Dark    | accent, accent_foreground | (214,93,14), (251,241,199) | (254,128,25), (40,40,40) | 3.41 | >= 5.8 |
    #[test]
    fn roadmap_060_pack_retune_reaches_aa() {
        type Rgb = (u8, u8, u8);
        // (token, before rgb, after rgb)
        type TokenChange = (&'static str, Rgb, Rgb);
        // (pack, mode, token changes, before D1 ratio, min after D1 ratio)
        type Retune = (&'static str, ThemeMode, &'static [TokenChange], f32, f32);
        let retunes: [Retune; 3] = [
            (
                "solarized",
                ThemeMode::Light,
                &[("accent_foreground", (108, 113, 196), (95, 99, 172))],
                3.78,
                4.6,
            ),
            (
                "gruvbox",
                ThemeMode::Light,
                &[("accent", (214, 93, 14), (175, 58, 3))],
                3.41,
                5.3,
            ),
            (
                "gruvbox",
                ThemeMode::Dark,
                &[
                    ("accent", (214, 93, 14), (254, 128, 25)),
                    ("accent_foreground", (251, 241, 199), (40, 40, 40)),
                ],
                3.41,
                5.8,
            ),
        ];
        fn token_mut<'a>(c: &'a mut ColorTokens, token: &str) -> &'a mut ThemeColor {
            match token {
                "accent" => &mut c.accent,
                "accent_foreground" => &mut c.accent_foreground,
                other => panic!("unexpected token {other}"),
            }
        }
        for (name, mode, changes, before_ratio, min_after) in retunes {
            let theme = packs::theme_by_name(name, mode).expect("pack resolves");

            // Reconstruct the pre-060 pair and confirm it failed. 062 changes
            // (e.g. solarized light foreground, a D1 fallback) are reverted first.
            let mut old = theme.clone();
            revert_roadmap_062(&mut old.colors, &format!("{name}/{mode:?}"));
            for (token, before, after) in changes {
                let mut current = theme.colors;
                assert_eq!(
                    *token_mut(&mut current, token),
                    ThemeColor::rgb(after.0, after.1, after.2),
                    "{name}/{mode:?} {token} after value"
                );
                *token_mut(&mut old.colors, token) = ThemeColor::rgb(before.0, before.1, before.2);
            }
            // The old selection algorithm used its best candidate if none met AA.
            let c = old.colors;
            let old_ratio = [c.accent_foreground, c.foreground, c.primary_foreground]
                .into_iter()
                .map(|fg| contrast_ratio(fg, c.accent))
                .fold(0.0, f32::max);
            assert!(
                (old_ratio - before_ratio).abs() < 0.01 && old_ratio < 4.5,
                "{name}/{mode:?} before ratio {old_ratio:.2}"
            );

            let (b, f) = theme.selected_item_colors();
            let new_ratio = contrast_ratio(f, b);
            assert!(
                new_ratio >= min_after && new_ratio >= 4.5,
                "{name}/{mode:?} after ratio {new_ratio:.2}"
            );
        }
    }

    /// Review P1 (060, option B): gruvbox accent graphics (chart dots, gauge
    /// hub, selected fill) must meet WCAG 1.4.11 non-text 3:1 against the
    /// background and surface, while the selected text pair stays AA.
    #[test]
    fn gruvbox_accent_meets_non_text_contrast() {
        let dark = packs::theme_by_name("gruvbox", ThemeMode::Dark).expect("gruvbox dark");
        let c = &dark.colors;
        let (bg, fg) = dark.selected_item_colors();
        let text = contrast_ratio(fg, bg);
        let vs_bg = contrast_ratio(c.accent, c.background);
        let vs_surface = contrast_ratio(c.accent, c.surface);
        assert!(text >= 4.5, "gruvbox/Dark selected text {text:.2}");
        assert!(vs_bg >= 3.0, "gruvbox/Dark accent vs background {vs_bg:.2}");
        assert!(
            vs_surface >= 3.0,
            "gruvbox/Dark accent vs surface {vs_surface:.2}"
        );

        let light = packs::theme_by_name("gruvbox", ThemeMode::Light).expect("gruvbox light");
        let c = &light.colors;
        let vs_bg = contrast_ratio(c.accent, c.background);
        assert!(
            vs_bg >= 3.0,
            "gruvbox/Light accent vs background {vs_bg:.2}"
        );
    }

    /// Accent is a soft selection tint, not a graphic token (ROADMAP 062).
    ///
    /// This guard documents that semantics instead of pinning a list of
    /// packs: `accent` is not required to reach 3:1 against the background,
    /// so non-text marks (charts, focus, graphics) must use `primary` or
    /// `ring`, which [`roadmap_062_every_pack_meets_aa_gate`] guarantees.
    /// The only accent graphic guarantee is gruvbox's (060 review P1), which
    /// stays enforced here and in [`gruvbox_accent_meets_non_text_contrast`].
    #[test]
    fn accent_vs_background_non_text_guard() {
        for (label, theme) in all_pack_themes() {
            let c = &theme.colors;
            let ratio = contrast_ratio(c.accent, c.background);
            if label.starts_with("gruvbox/") {
                assert!(ratio >= 3.0, "{label}: accent vs background {ratio:.2}");
            }
            // Everywhere, the graphic tokens charts must use instead of the
            // tint reach 3:1 against the background.
            for (token, color) in [("primary", c.primary), ("ring", c.ring)] {
                let r = contrast_ratio(color, c.background);
                assert!(r >= 3.0, "{label}: {token} vs background {r:.2}");
            }
        }
    }

    /// ROADMAP 062 gate: every pack and mode meets WCAG AA for the report
    /// pairs and WCAG 1.4.11 (3:1) for the focus ring, the primary graphic,
    /// and the status colours (`success`, `warning`, `danger`, `info`), which
    /// charts and icons paint as non-text marks on `surface` or `background`.
    #[test]
    fn roadmap_062_every_pack_meets_aa_gate() {
        let themes = all_pack_themes();
        assert_eq!(themes.len(), 26, "2 built-ins + 12 packs x 2 modes");
        let mut failures = Vec::new();
        for (label, theme) in &themes {
            let c = &theme.colors;
            if !theme.meets_wcag_aa() {
                for (pair, ratio, level) in theme.wcag_report() {
                    if !level.meets_aa() {
                        failures.push(format!("{label}: {pair} {ratio:.2} < 4.5"));
                    }
                }
            }
            let checks = [
                ("ring/background", c.ring, c.background, 3.0),
                ("ring/surface", c.ring, c.surface, 3.0),
                ("primary/background", c.primary, c.background, 3.0),
                ("foreground/surface", c.foreground, c.surface, 4.5),
                // Captions and helper text paint muted_foreground directly on
                // the window background, not only on the muted fill (review P2).
                ("muted_fg/background", c.muted_foreground, c.background, 4.5),
                // Secondary and ghost buttons paint foreground on their
                // hovered and pressed fills (review, button painted pairs).
                ("foreground/ghost_hover", c.foreground, c.ghost_hover, 4.5),
                (
                    "foreground/ghost_pressed",
                    c.foreground,
                    c.ghost_pressed,
                    4.5,
                ),
                ("success/surface", c.success, c.surface, 3.0),
                ("success/background", c.success, c.background, 3.0),
                ("warning/surface", c.warning, c.surface, 3.0),
                ("warning/background", c.warning, c.background, 3.0),
                ("danger/surface", c.danger, c.surface, 3.0),
                ("danger/background", c.danger, c.background, 3.0),
                ("info/surface", c.info, c.surface, 3.0),
                ("info/background", c.info, c.background, 3.0),
            ];
            for (pair, fg, bg, min) in checks {
                let ratio = contrast_ratio(fg, bg);
                if ratio < min {
                    failures.push(format!("{label}: {pair} {ratio:.2} < {min}"));
                }
            }
        }
        assert!(
            failures.is_empty(),
            "{} WCAG gate failure(s):\n{}",
            failures.len(),
            failures.join("\n")
        );
    }

    /// Pack/modes checked strictly by the state-feedback rule
    /// ([`roadmap_062_state_fills_give_feedback`]): packs retuned by ROADMAP
    /// 062 plus packs added after it. Untouched packs are listed instead.
    const ROADMAP_062_RETUNED: [&str; 17] = [
        "classic/Light",
        "classic/Dark",
        "default/light",
        "apple/Light",
        "apple/Dark",
        "ubuntu/Light",
        "ubuntu/Dark",
        "nord/Light",
        "nord/Dark",
        "dracula/Light",
        "dracula/Dark",
        "solarized/Light",
        "solarized/Dark",
        "gruvbox/Light",
        "gruvbox/Dark",
        "one-dark/Light",
        "one-dark/Dark",
    ];

    /// Untouched pack families that fail the state-feedback rule. 062 does
    /// not retune them for this rule (review P1-3 scope). The test checks
    /// that this list is exact, so it cannot go stale.
    const STATE_FEEDBACK_UNTOUCHED_EXCEPTIONS: &[(&str, &str)] = &[
        ("default/dark", "primary"),
        ("default/dark", "danger"),
        ("high-contrast/Dark", "primary"),
        ("high-contrast/Dark", "danger"),
        ("material/Dark", "primary"),
        ("material/Dark", "danger"),
        ("windows10/Dark", "primary"),
        ("windows10/Dark", "danger"),
        ("windows11/Light", "primary"),
        ("windows11/Light", "danger"),
        ("windows11/Dark", "primary"),
        ("windows11/Dark", "danger"),
    ];

    /// Ink a danger button paints on `fill`: the first of
    /// `primary_foreground`, `background`, `foreground` reaching 4.5:1.
    fn danger_ink(c: &ColorTokens, fill: ThemeColor) -> Option<ThemeColor> {
        [c.primary_foreground, c.background, c.foreground]
            .into_iter()
            .find(|ink| contrast_ratio(*ink, fill) >= 4.5)
    }

    /// `(rest, hover, pressed, ink)` for a button family. The danger ink is
    /// the one chosen for the rest fill.
    fn state_family(
        c: &ColorTokens,
        family: &str,
    ) -> (ThemeColor, ThemeColor, ThemeColor, ThemeColor) {
        match family {
            "primary" => (
                c.primary,
                c.primary_hover,
                c.primary_pressed,
                c.primary_foreground,
            ),
            "danger" => (
                c.danger,
                c.danger_hover,
                c.danger_pressed,
                danger_ink(c, c.danger).unwrap_or(c.primary_foreground),
            ),
            other => panic!("unexpected family {other}"),
        }
    }

    /// State-feedback rule for one button family, or why it fails.
    ///
    /// - Hover and pressed each differ from the rest fill by >= 1.15:1.
    /// - Pressed is at least as far from the rest fill as hover.
    /// - Direction: both states step away from the family's ink, so the ink
    ///   gains contrast on hover and on pressed. Under light ink the states
    ///   are darker; under dark ink they are lighter. Hover and pressed sit on
    ///   the same side of the rest fill, and one ink stays readable in every
    ///   state.
    fn state_feedback_failure(c: &ColorTokens, family: &str) -> Option<String> {
        let (rest, hover, pressed, ink) = state_family(c, family);
        let d_hover = contrast_ratio(rest, hover);
        let d_pressed = contrast_ratio(rest, pressed);
        let ink_rest = contrast_ratio(ink, rest);
        let ink_hover = contrast_ratio(ink, hover);
        let ink_pressed = contrast_ratio(ink, pressed);
        let mut why = Vec::new();
        if d_hover < 1.15 {
            why.push(format!("hover vs rest {d_hover:.3} < 1.15"));
        }
        if d_pressed < 1.15 {
            why.push(format!("pressed vs rest {d_pressed:.3} < 1.15"));
        }
        if d_pressed + 1e-4 < d_hover {
            why.push(format!(
                "pressed {d_pressed:.3} is nearer rest than hover {d_hover:.3}"
            ));
        }
        if ink_hover <= ink_rest || ink_pressed <= ink_rest {
            why.push(format!(
                "a state moves toward the ink (ink on rest {ink_rest:.2}, hover {ink_hover:.2}, pressed {ink_pressed:.2})"
            ));
        }
        (!why.is_empty()).then(|| why.join("; "))
    }

    /// Review P1-2: the button paints `primary_foreground` on every primary
    /// state fill, so the text must stay AA on hover and pressed in every pack.
    #[test]
    fn roadmap_062_primary_states_keep_text_aa() {
        let mut failures = Vec::new();
        for (label, theme) in all_pack_themes() {
            let c = &theme.colors;
            for (state, fill) in [
                ("primary_hover", c.primary_hover),
                ("primary_pressed", c.primary_pressed),
            ] {
                let ratio = contrast_ratio(c.primary_foreground, fill);
                if ratio < 4.5 {
                    failures.push(format!("{label}: primary_fg/{state} {ratio:.2} < 4.5"));
                }
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    /// Review: every danger fill (rest, hover, pressed) has an AA ink among
    /// `primary_foreground`, `background` and `foreground` in every pack.
    #[test]
    fn roadmap_062_danger_fills_have_aa_ink() {
        let mut failures = Vec::new();
        for (label, theme) in all_pack_themes() {
            let c = &theme.colors;
            for (state, fill) in [
                ("danger", c.danger),
                ("danger_hover", c.danger_hover),
                ("danger_pressed", c.danger_pressed),
            ] {
                if danger_ink(c, fill).is_none() {
                    let best = [c.primary_foreground, c.background, c.foreground]
                        .into_iter()
                        .map(|ink| contrast_ratio(ink, fill))
                        .fold(0.0_f32, f32::max);
                    failures.push(format!("{label}: {state} best ink {best:.2} < 4.5"));
                }
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    /// Review P1-3: primary and danger hover and pressed give visible,
    /// ordered feedback in the 062-retuned packs. Untouched-pack failures
    /// are listed exactly in [`STATE_FEEDBACK_UNTOUCHED_EXCEPTIONS`].
    #[test]
    fn roadmap_062_state_fills_give_feedback() {
        let mut failures = Vec::new();
        let mut untouched_failing = Vec::new();
        for (label, theme) in all_pack_themes() {
            for family in ["primary", "danger"] {
                let Some(why) = state_feedback_failure(&theme.colors, family) else {
                    continue;
                };
                if ROADMAP_062_RETUNED.contains(&label.as_str()) {
                    failures.push(format!("{label} {family}: {why}"));
                } else {
                    untouched_failing.push((label.clone(), family));
                }
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
        let mut expected: Vec<(String, &str)> = STATE_FEEDBACK_UNTOUCHED_EXCEPTIONS
            .iter()
            .map(|(l, f)| (l.to_string(), *f))
            .collect();
        expected.sort();
        untouched_failing.sort();
        assert_eq!(untouched_failing, expected, "untouched exception list");
    }

    /// Review P2: Solarized muted text stays AA but reads as secondary to
    /// body text (>= 1.1:1 apart) in both modes.
    #[test]
    fn solarized_muted_text_keeps_a_hierarchy() {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let c = packs::theme_by_name("solarized", mode)
                .expect("solarized")
                .colors;
            let apart = contrast_ratio(c.foreground, c.muted_foreground);
            assert!(
                apart >= 1.1,
                "solarized/{mode:?} fg vs muted_fg {apart:.3} < 1.1"
            );
            let body = contrast_ratio(c.foreground, c.muted);
            let muted = contrast_ratio(c.muted_foreground, c.muted);
            assert!(body > muted, "solarized/{mode:?} muted text must be dimmer");
            for (pair, bg) in [("muted", c.muted), ("background", c.background)] {
                let r = contrast_ratio(c.muted_foreground, bg);
                assert!(r >= 4.6, "solarized/{mode:?} muted_fg/{pair} {r:.2} < 4.6");
            }
        }
    }

    /// Review P2: gruvbox dark uses one teal for primary, ring and info, and
    /// hover is a visible step from it.
    #[test]
    fn gruvbox_dark_teals_are_unified() {
        let c = packs::theme_by_name("gruvbox", ThemeMode::Dark)
            .expect("gruvbox")
            .colors;
        assert_eq!(c.ring, c.primary, "ring = primary");
        assert_eq!(c.info, c.primary, "info = primary");
        for (pair, bg) in [("surface", c.surface), ("background", c.background)] {
            let r = contrast_ratio(c.primary, bg);
            assert!(r >= 3.0, "gruvbox/Dark primary/{pair} {r:.2} < 3");
        }
        let step = contrast_ratio(c.primary, c.primary_hover);
        assert!(step >= 1.15, "gruvbox/Dark hover step {step:.3} < 1.15");
    }
}
