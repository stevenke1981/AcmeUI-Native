//! ToggleButton component — a button that toggles between selected/unselected state.
//!
//! Renders as a [`Button`] widget with Primary variant when selected,
//! Secondary variant when not selected.

use acme_core::WidgetKey;
use acme_widgets::*;
use crate::{ControlSize, IconName};

/// Builder for a ToggleButton component.
pub struct ToggleButtonBuilder<M> {
    pub id: WidgetKey,
    pub label: String,
    pub icon: Option<IconName>,
    pub selected: bool,
    pub disabled: bool,
    pub size: ControlSize,
    pub on_toggle: Option<M>,
}

/// Create a new ToggleButton builder.
pub fn toggle_button<M: Clone + 'static>(
    id: impl Into<WidgetKey>,
) -> ToggleButtonBuilder<M> {
    ToggleButtonBuilder {
        id: id.into(),
        label: String::new(),
        icon: None,
        selected: false,
        disabled: false,
        size: ControlSize::Md,
        on_toggle: None,
    }
}

impl<M: Clone + 'static> ToggleButtonBuilder<M> {
    /// Set the label text displayed on the toggle button.
    pub fn label(mut self, value: impl Into<String>) -> Self {
        self.label = value.into();
        self
    }

    /// Set an optional icon displayed alongside the label.
    pub fn icon(mut self, value: IconName) -> Self {
        self.icon = Some(value);
        self
    }

    /// Set whether the toggle button is currently selected.
    pub fn selected(mut self, value: bool) -> Self {
        self.selected = value;
        self
    }

    /// Set whether the toggle button is disabled.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    /// Set the control size.
    pub fn size(mut self, value: ControlSize) -> Self {
        self.size = value;
        self
    }

    /// Set the message dispatched when the toggle button is toggled.
    pub fn on_toggle(mut self, msg: M) -> Self {
        self.on_toggle = Some(msg);
        self
    }
}

// ---------------------------------------------------------------------------
// From impl
// ---------------------------------------------------------------------------

impl<M: Clone + 'static> From<ToggleButtonBuilder<M>> for WidgetNode<M> {
    fn from(b: ToggleButtonBuilder<M>) -> Self {
        let label = b.label.clone();
        let btn_size = match b.size {
            ControlSize::Xs => ButtonSize::XS,
            ControlSize::Sm => ButtonSize::Small,
            ControlSize::Md => ButtonSize::Medium,
            ControlSize::Lg => ButtonSize::Large,
            ControlSize::Xl => ButtonSize::Large,
        };
        let variant = if b.selected {
            ButtonVariant::Primary
        } else {
            ButtonVariant::Secondary
        };
        let btn = button::<M>(b.id, label)
            .variant(variant)
            .size(btn_size)
            .disabled(b.disabled);
        if let Some(msg) = b.on_toggle {
            btn.on_click(msg)
        } else {
            btn.into()
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn toggle_button_builder_defaults() {
        let tb = toggle_button::<TestMsg>("tb1");
        assert!(tb.label.is_empty());
        assert!(!tb.selected);
        assert!(!tb.disabled);
        assert!(tb.icon.is_none());
        assert_eq!(tb.size, ControlSize::Md);
        assert!(tb.on_toggle.is_none());
    }

    #[test]
    fn toggle_button_builds_into_button_widget() {
        let node: WidgetNode<TestMsg> = toggle_button("tb2").label("Bold").into();
        let WidgetNode::Button(b) = &node else {
            panic!("expected Button");
        };
        assert_eq!(b.label, "Bold");
    }

    #[test]
    fn toggle_button_selected_uses_primary_variant() {
        let node: WidgetNode<TestMsg> =
            toggle_button("tb3").label("Italic").selected(true).into();
        let WidgetNode::Button(b) = &node else {
            panic!("expected Button");
        };
        assert_eq!(b.variant, ButtonVariant::Primary);
    }

    #[test]
    fn toggle_button_unselected_uses_secondary_variant() {
        let node: WidgetNode<TestMsg> =
            toggle_button("tb4").label("Underline").into();
        let WidgetNode::Button(b) = &node else {
            panic!("expected Button");
        };
        assert_eq!(b.variant, ButtonVariant::Secondary);
    }

    #[test]
    fn toggle_button_disabled_state() {
        let node: WidgetNode<TestMsg> =
            toggle_button("tb5").label("Strike").disabled(true).into();
        let WidgetNode::Button(b) = &node else {
            panic!("expected Button");
        };
        assert!(b.disabled);
    }
}
