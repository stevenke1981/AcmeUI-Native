use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_theme::{Theme, ThemeColor};

/// Button style variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
    Danger,
}

/// Button size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ButtonSize {
    XS,
    Small,
    #[default]
    Medium,
    Large,
}

/// Interactive state for button hit-testing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ButtonState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
}

/// Resolved button colors computed from theme and state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedButtonStyle {
    pub background: ThemeColor,
    pub foreground: ThemeColor,
    pub border: ThemeColor,
    pub focus: ThemeColor,
}

/// A button widget.
#[derive(Clone, Debug, PartialEq)]
pub struct Button<M> {
    pub key: WidgetKey,
    pub label: String,
    pub variant: ButtonVariant,
    pub size: ButtonSize,
    pub disabled: bool,
    pub loading: bool,
    pub full_width: bool,
    pub leading_icon: Option<String>,
    pub trailing_icon: Option<String>,
    message: Option<M>,
}

/// Create a button builder.
pub fn button<M>(key: impl Into<WidgetKey>, label: impl Into<String>) -> Button<M> {
    Button {
        key: key.into(),
        label: label.into(),
        variant: ButtonVariant::Secondary,
        size: ButtonSize::Medium,
        disabled: false,
        loading: false,
        full_width: false,
        leading_icon: None,
        trailing_icon: None,
        message: None,
    }
}

impl<M> Button<M> {
    pub fn primary(mut self) -> Self {
        self.variant = ButtonVariant::Primary;
        self
    }
    pub fn variant(mut self, value: ButtonVariant) -> Self {
        self.variant = value;
        self
    }
    pub fn size(mut self, value: ButtonSize) -> Self {
        self.size = value;
        self
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    pub fn loading(mut self, value: bool) -> Self {
        self.loading = value;
        self
    }
    pub fn full_width(mut self, value: bool) -> Self {
        self.full_width = value;
        self
    }
    pub fn leading_icon(mut self, icon: impl Into<String>) -> Self {
        self.leading_icon = Some(icon.into());
        self
    }
    pub fn trailing_icon(mut self, icon: impl Into<String>) -> Self {
        self.trailing_icon = Some(icon.into());
        self
    }
    pub fn on_click(mut self, message: M) -> WidgetNode<M> {
        self.message = Some(message);
        WidgetNode::Button(self)
    }
    pub fn activate(&self) -> Option<&M> {
        if self.disabled {
            None
        } else {
            self.message.as_ref()
        }
    }
    /// Resolve visual style from theme and state.
    /// Uses separate `button_pressed` token for pressed state (not `button_hover`).
    pub fn resolve_style(&self, theme: &Theme, state: ButtonState) -> ResolvedButtonStyle {
        let c = theme.colors;
        let background = if self.disabled {
            c.disabled_bg
        } else if state.pressed {
            // Pressed uses a separate token from hover
            match self.variant {
                ButtonVariant::Primary => c.button_pressed,
                ButtonVariant::Danger => c.button_pressed,
                ButtonVariant::Secondary | ButtonVariant::Ghost => c.button_pressed,
            }
        } else if state.hovered {
            match self.variant {
                ButtonVariant::Primary => c.accent_hover,
                ButtonVariant::Danger => c.danger,
                ButtonVariant::Secondary => c.surface_hover,
                ButtonVariant::Ghost => c.background,
            }
        } else {
            match self.variant {
                ButtonVariant::Primary => c.button_primary_bg,
                ButtonVariant::Danger => c.button_danger_bg,
                ButtonVariant::Secondary => c.button_secondary_bg,
                ButtonVariant::Ghost => c.button_ghost_bg,
            }
        };
        ResolvedButtonStyle {
            background,
            foreground: if self.disabled {
                c.disabled_text
            } else if self.variant == ButtonVariant::Primary {
                c.on_accent
            } else if self.variant == ButtonVariant::Danger {
                c.on_danger
            } else {
                c.text
            },
            border: c.border,
            focus: c.focus,
        }
    }
}

impl<M> From<Button<M>> for WidgetNode<M> {
    fn from(value: Button<M>) -> Self {
        WidgetNode::Button(value)
    }
}
