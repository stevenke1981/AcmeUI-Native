//! IconButton component — a compact square button showing only an icon.
//!
//! Renders as a [`Button`] widget with the icon character as the label.
//! The parent app manages selected state via `on_click`.

use acme_core::WidgetKey;
use acme_widgets::*;
use crate::{ControlSize, IconName};

/// Builder for an IconButton component.
pub struct IconButtonBuilder<M> {
    pub id: WidgetKey,
    pub icon: IconName,
    pub variant: ButtonVariant,
    pub size: ControlSize,
    pub disabled: bool,
    pub selected: bool,
    pub on_click: Option<M>,
}

/// Create a new IconButton builder.
pub fn icon_button<M: Clone + 'static>(
    id: impl Into<WidgetKey>,
    icon: IconName,
) -> IconButtonBuilder<M> {
    IconButtonBuilder {
        id: id.into(),
        icon,
        variant: ButtonVariant::Secondary,
        size: ControlSize::Md,
        disabled: false,
        selected: false,
        on_click: None,
    }
}

impl<M: Clone + 'static> IconButtonBuilder<M> {
    /// Set the button style variant.
    pub fn variant(mut self, value: ButtonVariant) -> Self {
        self.variant = value;
        self
    }

    /// Set the control size.
    pub fn size(mut self, value: ControlSize) -> Self {
        self.size = value;
        self
    }

    /// Set whether the icon button is disabled.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    /// Set whether the icon button is selected (toggled on).
    pub fn selected(mut self, value: bool) -> Self {
        self.selected = value;
        self
    }

    /// Set the message dispatched when the icon button is clicked.
    pub fn on_click(mut self, msg: M) -> Self {
        self.on_click = Some(msg);
        self
    }
}

// ---------------------------------------------------------------------------
// Conversion helpers
// ---------------------------------------------------------------------------

fn control_size_to_button_size(s: ControlSize) -> ButtonSize {
    match s {
        ControlSize::Xs => ButtonSize::XS,
        ControlSize::Sm => ButtonSize::Small,
        ControlSize::Md => ButtonSize::Medium,
        ControlSize::Lg => ButtonSize::Large,
        ControlSize::Xl => ButtonSize::Large,
    }
}

// ---------------------------------------------------------------------------
// From impl
// ---------------------------------------------------------------------------

impl<M: Clone + 'static> From<IconButtonBuilder<M>> for WidgetNode<M> {
    fn from(b: IconButtonBuilder<M>) -> Self {
        let label = b.icon.char();
        let btn_size = control_size_to_button_size(b.size);
        let btn = button::<M>(b.id, label)
            .variant(b.variant)
            .size(btn_size)
            .disabled(b.disabled);
        if let Some(msg) = b.on_click {
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
    fn icon_button_builder_defaults() {
        let btn = icon_button::<TestMsg>("icon1", IconName::Search);
        assert!(!btn.disabled);
        assert!(!btn.selected);
        assert_eq!(btn.variant, ButtonVariant::Secondary);
        assert_eq!(btn.size, ControlSize::Md);
        assert!(btn.on_click.is_none());
    }

    #[test]
    fn icon_button_builds_into_button_widget() {
        let node: WidgetNode<TestMsg> = icon_button("icon2", IconName::Close).into();
        let WidgetNode::Button(b) = &node else {
            panic!("expected Button");
        };
        assert_eq!(b.label, "✕");
    }

    #[test]
    fn icon_button_variant_is_set() {
        let node: WidgetNode<TestMsg> =
            icon_button("icon3", IconName::Settings)
                .variant(ButtonVariant::Primary)
                .into();
        let WidgetNode::Button(b) = &node else {
            panic!("expected Button");
        };
        assert_eq!(b.variant, ButtonVariant::Primary);
    }

    #[test]
    fn icon_button_disabled_state() {
        let node: WidgetNode<TestMsg> =
            icon_button("icon4", IconName::Plus).disabled(true).into();
        let WidgetNode::Button(b) = &node else {
            panic!("expected Button");
        };
        assert!(b.disabled);
    }
}
