use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_style::prelude::*;
use acme_style::{Style, TextAlign};
use acme_theme::{Theme, ThemeColor};

/// Horizontal label inset (logical px) used by [`Button::label_x`] for
/// `TextAlign::Left` / `TextAlign::Right` when `style.padding` is unset.
pub(crate) const DEFAULT_LABEL_PAD: f32 = 12.0;

/// Button style variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
    /// Minimal chrome with a transparent rest fill and no default border.
    Subtle,
    Danger,
}

/// Button size matching AcmeUIKit design tokens.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ButtonSize {
    XS,    // 24px
    Small, // 30px
    #[default]
    Medium, // 36px
    Large, // 42px
}

impl ButtonSize {
    pub fn px(&self) -> f32 {
        match self {
            Self::XS => 24.0,
            Self::Small => 30.0,
            Self::Medium => 36.0,
            Self::Large => 42.0,
        }
    }
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
    accessibility_toggled: Option<bool>,
    selected: Option<bool>,
    /// Accumulated GPUI‑inspired style (background, border, rounding, etc.).
    pub style: Style,
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
        accessibility_toggled: None,
        selected: None,
        style: Style::new(),
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
    /// Expose the current state of a toggle button to assistive technology.
    /// This does not change the button's appearance or activation behavior.
    pub fn with_accessibility_toggled(mut self, toggled: bool) -> Self {
        self.accessibility_toggled = Some(toggled);
        self
    }

    /// Current toggle state, or `None` for an ordinary button.
    pub fn accessibility_toggled(&self) -> Option<bool> {
        self.accessibility_toggled.or(self.selected)
    }

    /// Set the visual selection state and provide a default toggle state to accessibility.
    pub fn selected(mut self, value: bool) -> Self {
        self.selected = Some(value);
        self
    }

    /// Whether the button uses the theme's selected colors.
    pub fn is_selected(&self) -> bool {
        self.selected.unwrap_or(false)
    }

    /// Resolve the resting border width, excluding the focus ring.
    pub fn resolved_border_width(&self) -> f32 {
        self.style.border_width.unwrap_or(match self.variant {
            ButtonVariant::Subtle => 0.0,
            _ => 1.0,
        })
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
    /// Left x of the label for a button painted in `rect_x..rect_x + rect_width`
    /// with a shaped label `text_width` wide.
    ///
    /// - `style.text_align` unset or `Center`: centred, overflow pinned at the
    ///   left edge (`rect_x + max(0, rect_width - text_width) / 2`).
    /// - `Left`: `rect_x + pad`, where `pad` is `style.padding.left` when padding
    ///   is set, else 12.0 logical px.
    /// - `Right`: mirrors `Left` (`rect_x + rect_width - pad - text_width`, `pad`
    ///   from `style.padding.right`); text too wide for the rect clamps to
    ///   `rect_x + pad.left` so it starts where `Left` would.
    ///
    /// Padding is read as a whole: a partial setter such as `.pr(3.0)` sets
    /// every other side to 0. Left/Right alignment is intended for full-width
    /// or explicitly padded buttons; on an auto-width button (laid out with
    /// 10px side padding) the 12.0 default leaves `Right` clamped to `Left`.
    pub fn label_x(&self, rect_x: f32, rect_width: f32, text_width: f32) -> f32 {
        let pad = |side: fn(&acme_layout::Edges) -> f32| {
            self.style.padding.as_ref().map_or(DEFAULT_LABEL_PAD, side)
        };
        match self.style.text_align {
            None | Some(TextAlign::Center) => {
                rect_x + crate::label_align_offset(TextAlign::Center, rect_width, text_width)
            }
            Some(TextAlign::Left) => rect_x + pad(|e| e.left),
            Some(TextAlign::Right) => {
                let start = rect_x + pad(|e| e.left);
                (rect_x + rect_width - pad(|e| e.right) - text_width).max(start)
            }
        }
    }
    /// Resolve visual style from theme and state.
    /// Uses V2 semantic color tokens from `acme-theme`.
    pub fn resolve_style(&self, theme: &Theme, state: ButtonState) -> ResolvedButtonStyle {
        let c = theme.colors;
        let selected = (!self.disabled && self.is_selected()).then(|| theme.selected_item_colors());
        let background = if let Some(bg) = self.style.background {
            bg.resolve(theme)
        } else if self.disabled {
            c.disabled_bg
        } else if let Some((background, _)) = selected {
            background
        } else if state.pressed {
            match self.variant {
                ButtonVariant::Primary => c.primary_pressed,
                ButtonVariant::Danger => c.danger_pressed,
                ButtonVariant::Secondary | ButtonVariant::Ghost | ButtonVariant::Subtle => {
                    c.ghost_pressed
                }
            }
        } else if state.hovered {
            match self.variant {
                ButtonVariant::Primary => c.primary_hover,
                ButtonVariant::Danger => c.danger_hover,
                ButtonVariant::Secondary | ButtonVariant::Ghost | ButtonVariant::Subtle => {
                    c.ghost_hover
                }
            }
        } else {
            match self.variant {
                ButtonVariant::Primary => c.primary,
                ButtonVariant::Danger => c.danger,
                ButtonVariant::Secondary => c.secondary,
                ButtonVariant::Ghost | ButtonVariant::Subtle => {
                    ThemeColor::rgba(0.0, 0.0, 0.0, 0.0)
                }
            }
        };
        ResolvedButtonStyle {
            background,
            foreground: if let Some(fg) = self.style.text_color {
                fg.resolve(theme)
            } else if self.disabled {
                c.disabled_text
            } else if let Some((_, foreground)) =
                selected.filter(|_| self.style.background.is_none())
            {
                foreground
            } else {
                let candidates = match self.variant {
                    ButtonVariant::Primary | ButtonVariant::Danger => {
                        [c.primary_foreground, c.background, c.foreground]
                    }
                    ButtonVariant::Secondary if !(state.pressed || state.hovered) => {
                        [c.secondary_foreground, c.foreground, c.background]
                    }
                    ButtonVariant::Secondary | ButtonVariant::Ghost | ButtonVariant::Subtle => {
                        [c.foreground, c.background, c.secondary_foreground]
                    }
                };
                let painted = if background.alpha == 0.0 {
                    c.background
                } else {
                    background
                };
                readable_foreground(theme, painted, candidates)
            },
            border: self
                .style
                .border_color
                .map_or(c.border, |color| color.resolve(theme)),
            focus: c.ring,
        }
    }
}

/// First candidate token reaching WCAG AA (4.5:1) on `bg`, else the highest
/// ratio. Mirrors `Theme::selected_item_colors`; only existing tokens are used.
fn readable_foreground(theme: &Theme, bg: ThemeColor, candidates: [ThemeColor; 3]) -> ThemeColor {
    candidates
        .iter()
        .copied()
        .find(|fg| theme.contrast(*fg, bg) >= 4.5)
        .unwrap_or_else(|| {
            candidates.iter().copied().fold(candidates[0], |best, fg| {
                if theme.contrast(fg, bg) > theme.contrast(best, bg) {
                    fg
                } else {
                    best
                }
            })
        })
}

impl<M> From<Button<M>> for WidgetNode<M> {
    fn from(value: Button<M>) -> Self {
        WidgetNode::Button(value)
    }
}

/// Implement the `Styled` trait so button builders can use tailwind‑style
/// utility methods directly on the button value.
impl<M> Styled for Button<M> {
    fn style(&self) -> &Style {
        &self.style
    }
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}
