//! Semantic theme tokens used by AcmeUI widgets.
#![forbid(unsafe_op_in_unsafe_fn)]

/// The built-in appearance variants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
}

/// Framework-owned color value. Components must consume semantic fields from
/// [`ColorTokens`] rather than embedding visual colors themselves.
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
    pub fn is_valid(self) -> bool {
        [self.red, self.green, self.blue, self.alpha]
            .into_iter()
            .all(|v| v.is_finite() && (0.0..=1.0).contains(&v))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorTokens {
    pub background: ThemeColor,
    pub surface: ThemeColor,
    pub surface_hover: ThemeColor,
    pub text: ThemeColor,
    pub text_muted: ThemeColor,
    pub border: ThemeColor,
    pub accent: ThemeColor,
    pub on_accent: ThemeColor,
    pub accent_hover: ThemeColor,
    pub focus: ThemeColor,
    pub danger: ThemeColor,
    pub on_danger: ThemeColor,
    pub disabled_text: ThemeColor,
    pub disabled_bg: ThemeColor,
    pub button_primary_bg: ThemeColor,
    pub button_secondary_bg: ThemeColor,
    pub button_ghost_bg: ThemeColor,
    pub button_danger_bg: ThemeColor,
    pub button_hover: ThemeColor,
    pub button_pressed: ThemeColor,
    pub card_elevated_shadow: ThemeColor,
    pub card_interactive_hover: ThemeColor,
    pub input_label: ThemeColor,
    pub input_description: ThemeColor,
    pub input_validation_error: ThemeColor,
    pub input_validation_success: ThemeColor,
    // Data widgets
    pub table_header_bg: ThemeColor,
    pub table_row_even_bg: ThemeColor,
    pub table_row_odd_bg: ThemeColor,
    pub table_row_hover_bg: ThemeColor,
    pub table_row_selected_bg: ThemeColor,
    pub table_sticky_header_shadow: ThemeColor,
    pub tree_indent_guide: ThemeColor,
    pub tree_expand_chevron: ThemeColor,
    pub datagrid_gridline: ThemeColor,
    pub datagrid_frozen_shadow: ThemeColor,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpacingTokens {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RadiusTokens {
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypographyTokens {
    pub body_size: f32,
    pub label_size: f32,
    pub line_height: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub mode: ThemeMode,
    pub colors: ColorTokens,
    pub spacing: SpacingTokens,
    pub radii: RadiusTokens,
    pub typography: TypographyTokens,
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
    pub fn validate(&self) -> Result<(), ThemeValidationError> {
        let c = self.colors;
        if ![
            c.background,
            c.surface,
            c.surface_hover,
            c.text,
            c.text_muted,
            c.border,
            c.accent,
            c.on_accent,
            c.accent_hover,
            c.focus,
            c.danger,
            c.on_danger,
            c.disabled_text,
            c.disabled_bg,
            c.button_primary_bg,
            c.button_secondary_bg,
            c.button_ghost_bg,
            c.button_danger_bg,
            c.button_hover,
            c.button_pressed,
            c.card_elevated_shadow,
            c.card_interactive_hover,
            c.input_label,
            c.input_description,
            c.input_validation_error,
            c.input_validation_success,
            c.table_header_bg,
            c.table_row_even_bg,
            c.table_row_odd_bg,
            c.table_row_hover_bg,
            c.table_row_selected_bg,
            c.table_sticky_header_shadow,
            c.tree_indent_guide,
            c.tree_expand_chevron,
            c.datagrid_gridline,
            c.datagrid_frozen_shadow,
        ]
        .into_iter()
        .all(ThemeColor::is_valid)
        {
            return Err(ThemeValidationError::InvalidColor);
        }
        if ![
            self.spacing.xs,
            self.spacing.sm,
            self.spacing.md,
            self.spacing.lg,
            self.spacing.xl,
        ]
        .into_iter()
        .all(valid_metric)
        {
            return Err(ThemeValidationError::InvalidSpacing);
        }
        if ![self.radii.sm, self.radii.md, self.radii.lg]
            .into_iter()
            .all(valid_metric)
        {
            return Err(ThemeValidationError::InvalidRadius);
        }
        if ![
            self.typography.body_size,
            self.typography.label_size,
            self.typography.line_height,
        ]
        .into_iter()
        .all(|v| valid_metric(v) && v > 0.0)
        {
            return Err(ThemeValidationError::InvalidTypography);
        }
        Ok(())
    }
    fn builtin(mode: ThemeMode) -> Self {
        let colors = match mode {
            ThemeMode::Light => ColorTokens {
                background: rgb(248, 250, 252),
                surface: rgb(255, 255, 255),
                surface_hover: rgb(241, 245, 249),
                text: rgb(15, 23, 42),
                text_muted: rgb(71, 85, 105),
                border: rgb(203, 213, 225),
                accent: rgb(37, 99, 235),
                on_accent: rgb(255, 255, 255),
                accent_hover: rgb(29, 78, 216),
                focus: rgb(59, 130, 246),
                danger: rgb(220, 38, 38),
                on_danger: rgb(255, 255, 255),
                disabled_text: rgb(100, 116, 139),
                disabled_bg: rgb(226, 232, 240),
                button_primary_bg: rgb(37, 99, 235),
                button_secondary_bg: rgb(255, 255, 255),
                button_ghost_bg: rgb(248, 250, 252),
                button_danger_bg: rgb(220, 38, 38),
                button_hover: rgb(241, 245, 249),
                button_pressed: rgb(226, 232, 240),
                card_elevated_shadow: ThemeColor::rgba(0.0, 0.0, 0.0, 0.1),
                card_interactive_hover: rgb(241, 245, 249),
                input_label: rgb(71, 85, 105),
                input_description: rgb(100, 116, 139),
                input_validation_error: rgb(220, 38, 38),
                input_validation_success: rgb(22, 163, 74),
                // Data widgets — Light
                table_header_bg: rgb(241, 245, 249),
                table_row_even_bg: rgb(255, 255, 255),
                table_row_odd_bg: rgb(248, 250, 252),
                table_row_hover_bg: rgb(241, 245, 249),
                table_row_selected_bg: rgb(219, 234, 254),
                table_sticky_header_shadow: ThemeColor::rgba(0.0, 0.0, 0.0, 0.08),
                tree_indent_guide: rgb(203, 213, 225),
                tree_expand_chevron: rgb(100, 116, 139),
                datagrid_gridline: rgb(226, 232, 240),
                datagrid_frozen_shadow: ThemeColor::rgba(0.0, 0.0, 0.0, 0.12),
            },
            ThemeMode::Dark => ColorTokens {
                background: rgb(15, 23, 42),
                surface: rgb(30, 41, 59),
                surface_hover: rgb(51, 65, 85),
                text: rgb(248, 250, 252),
                text_muted: rgb(148, 163, 184),
                border: rgb(71, 85, 105),
                accent: rgb(96, 165, 250),
                on_accent: rgb(15, 23, 42),
                accent_hover: rgb(147, 197, 253),
                focus: rgb(96, 165, 250),
                danger: rgb(248, 113, 113),
                on_danger: rgb(15, 23, 42),
                disabled_text: rgb(100, 116, 139),
                disabled_bg: rgb(51, 65, 85),
                button_primary_bg: rgb(96, 165, 250),
                button_secondary_bg: rgb(30, 41, 59),
                button_ghost_bg: rgb(15, 23, 42),
                button_danger_bg: rgb(248, 113, 113),
                button_hover: rgb(51, 65, 85),
                button_pressed: rgb(71, 85, 105),
                card_elevated_shadow: ThemeColor::rgba(0.0, 0.0, 0.0, 0.3),
                card_interactive_hover: rgb(51, 65, 85),
                input_label: rgb(148, 163, 184),
                input_description: rgb(100, 116, 139),
                input_validation_error: rgb(248, 113, 113),
                input_validation_success: rgb(74, 222, 128),
                // Data widgets — Dark
                table_header_bg: rgb(51, 65, 85),
                table_row_even_bg: rgb(30, 41, 59),
                table_row_odd_bg: rgb(15, 23, 42),
                table_row_hover_bg: rgb(51, 65, 85),
                table_row_selected_bg: rgb(30, 58, 138),
                table_sticky_header_shadow: ThemeColor::rgba(0.0, 0.0, 0.0, 0.3),
                tree_indent_guide: rgb(71, 85, 105),
                tree_expand_chevron: rgb(148, 163, 184),
                datagrid_gridline: rgb(71, 85, 105),
                datagrid_frozen_shadow: ThemeColor::rgba(0.0, 0.0, 0.0, 0.4),
            },
        };
        Self {
            mode,
            colors,
            spacing: SpacingTokens {
                xs: 4.0,
                sm: 8.0,
                md: 12.0,
                lg: 16.0,
                xl: 24.0,
            },
            radii: RadiusTokens {
                sm: 4.0,
                md: 8.0,
                lg: 12.0,
            },
            typography: TypographyTokens {
                body_size: 16.0,
                label_size: 14.0,
                line_height: 1.4,
            },
        }
    }
}

const fn rgb(r: u8, g: u8, b: u8) -> ThemeColor {
    ThemeColor::rgba(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0)
}
fn valid_metric(value: f32) -> bool {
    value.is_finite() && value >= 0.0
}

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
    fn rejects_invalid_custom_values() {
        let mut t = Theme::light();
        t.spacing.md = f32::NAN;
        assert_eq!(t.validate(), Err(ThemeValidationError::InvalidSpacing));
        let mut t = Theme::dark();
        t.colors.text.alpha = 1.1;
        assert_eq!(t.validate(), Err(ThemeValidationError::InvalidColor));
    }
}
