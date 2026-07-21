//! AcmeUI-Native Design System V2
//!
//! Synthesized from shadcn/ui, Material UI, and Ant Design best practices.
//! All colors, spacing, typography via semantic design tokens — no hardcoded values.
//!
//! Architecture: Seed Tokens → Derived Tokens → Component Tokens

#![forbid(unsafe_op_in_unsafe_fn)]

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
    pub disabled_text: ThemeColor,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeValidationError {
    InvalidColor,
    InvalidSpacing,
    InvalidRadius,
    InvalidTypography,
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
        ];
        if !color_fields.into_iter().all(ThemeColor::is_valid) {
            return Err(ThemeValidationError::InvalidColor);
        }
        Ok(())
    }

    fn builtin(mode: ThemeMode) -> Self {
        let (colors, shadows) = match mode {
            ThemeMode::Light => Self::light_colors(),
            ThemeMode::Dark => Self::dark_colors(),
        };
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
            muted: ThemeColor::rgb(244, 245, 247), // #F4F5F7
            muted_foreground: ThemeColor::rgb(107, 114, 128), // #6B7280

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
            muted: ThemeColor::rgb(22, 27, 34),     // #161B22
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
            primary_pressed: ThemeColor::rgb(64, 114, 230), // #4072E6
            danger_hover: ThemeColor::rgb(252, 129, 129),  // #FC8181
            danger_pressed: ThemeColor::rgb(235, 80, 80),  // #EB5050
            ghost_hover: ThemeColor::rgb(28, 33, 40),      // #1C2128
            ghost_pressed: ThemeColor::rgb(45, 51, 59),    // #2D333B

            // Disabled
            disabled_bg: ThemeColor::rgb(28, 33, 40), // #1C2128
            disabled_text: ThemeColor::rgb(75, 85, 99), // #4B5563
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
    }
}
