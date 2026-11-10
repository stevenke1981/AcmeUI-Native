//! FloatButton — floating action button (FAB).
//! Mini, normal and extended elevated primary actions. The host places the
//! resulting widget in its overlay; `placement` is host-owned metadata.

use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_layout::Length;
use acme_widgets::style::Styled;

/// FAB size and content presentation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FloatButtonSize {
    /// Compact 40 px control.
    Mini,
    /// Standard 56 px control.
    #[default]
    Normal,
    /// A 56 px high action with an intrinsic-width icon and label.
    Extended,
}

/// Float button shape.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FloatButtonShape {
    #[default]
    Circle,
    Square,
}

/// Float button position on screen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FloatButtonPlacement {
    #[default]
    BottomRight,
    BottomLeft,
    TopRight,
    TopLeft,
}

/// Builder for a floating action button.
pub struct FloatButtonBuilder<M> {
    pub id: WidgetKey,
    pub icon: Option<crate::IconName>,
    pub label: Option<String>,
    pub shape: FloatButtonShape,
    pub size: FloatButtonSize,
    pub placement: FloatButtonPlacement,
    pub on_click: Option<M>,
}

/// Create a float button builder.
pub fn float_button<M: Clone + 'static>() -> FloatButtonBuilder<M> {
    FloatButtonBuilder {
        id: WidgetKey::from("float_button"),
        icon: None,
        label: None,
        shape: FloatButtonShape::default(),
        size: FloatButtonSize::default(),
        placement: FloatButtonPlacement::default(),
        on_click: None,
    }
}

impl<M: Clone + 'static> FloatButtonBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn icon(mut self, value: crate::IconName) -> Self {
        self.icon = Some(value);
        self
    }

    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }

    pub fn shape(mut self, value: FloatButtonShape) -> Self {
        self.shape = value;
        self
    }

    pub fn size(mut self, value: FloatButtonSize) -> Self {
        self.size = value;
        self
    }

    /// Preferred viewport corner for the host's overlay layout.
    /// Conversion to a widget does not install a global overlay or position it.
    pub fn placement(mut self, value: FloatButtonPlacement) -> Self {
        self.placement = value;
        self
    }

    pub fn on_click(mut self, msg: M) -> Self {
        self.on_click = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<FloatButtonBuilder<M>> for WidgetNode<M> {
    fn from(b: FloatButtonBuilder<M>) -> Self {
        let height = if b.size == FloatButtonSize::Mini {
            40.0
        } else {
            56.0
        };
        let extended = b.size == FloatButtonSize::Extended;
        let radius = if b.shape == FloatButtonShape::Circle {
            height / 2.0
        } else {
            0.0
        };
        let label = b.label.filter(|text| !text.is_empty());
        let text = match (b.icon, label) {
            (Some(icon), Some(label)) if extended => format!("{} {label}", icon.char()),
            (Some(icon), _) => icon.char().to_owned(),
            (None, Some(label)) => label,
            (None, None) => "+".into(),
        };
        // A typed Button gives hosts one actionable/focusable descendant.
        // Keep the outer instance key stable when size/content changes.
        let mut button = crate::button(format!("{}__action", b.id.as_str()), text)
            .primary()
            .h(Length::px(height))
            .min_w(Length::px(height))
            .max_w(Length::Percent(1.0))
            .flex_shrink(1.0)
            .rounded(radius)
            .border_width(0.0);
        if extended {
            button = button.px(16.0);
        } else {
            button = button.w(Length::px(height)).p(0.0);
        }
        let action = if let Some(msg) = b.on_click {
            button.on_click(msg)
        } else {
            button.into()
        };
        let mut surface = crate::card()
            .key(b.id)
            .variant(crate::CardVariant::Elevated)
            .padding(0.0)
            .border_radius(radius)
            .border_width(0.0)
            .h(Length::px(height))
            .min_w(Length::px(height))
            .max_w(Length::Percent(1.0))
            .flex_shrink(if extended { 1.0 } else { 0.0 });
        if !extended {
            surface = surface.w(Length::px(height));
        }
        surface.child(action).build()
    }
}

#[cfg(test)]
#[path = "float_button_tests.rs"]
mod tests;
