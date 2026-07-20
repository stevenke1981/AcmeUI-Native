//! Checkbox input component.
//!
//! Renders as a Row with a check indicator (Card + "✓") and a label.
//! The parent app manages checked state via `on_click`.

use acme_core::WidgetKey;
use acme_widgets::*;

/// Builder for a Checkbox component.
pub struct CheckboxBuilder<M> {
    pub id: WidgetKey,
    pub label: String,
    pub checked: bool,
    pub disabled: bool,
    pub on_click: Option<M>,
}

/// Create a new Checkbox builder.
pub fn checkbox<M: Clone + 'static>(id: impl Into<WidgetKey>, checked: bool) -> CheckboxBuilder<M> {
    CheckboxBuilder {
        id: id.into(),
        label: String::new(),
        checked,
        disabled: false,
        on_click: None,
    }
}

impl<M: Clone + 'static> CheckboxBuilder<M> {
    /// Set the label text displayed next to the checkbox.
    pub fn label(mut self, value: impl Into<String>) -> Self {
        self.label = value.into();
        self
    }

    /// Set whether the checkbox is disabled.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    /// Set the message dispatched when the checkbox is toggled.
    pub fn on_click(mut self, msg: M) -> Self {
        self.on_click = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<CheckboxBuilder<M>> for WidgetNode<M> {
    fn from(b: CheckboxBuilder<M>) -> Self {
        let check_symbol = if b.checked { "✓" } else { "" };
        let indicator = card::<M>()
            .padding(2.0)
            .variant(if b.checked {
                CardVariant::Interactive
            } else {
                CardVariant::Outlined
            })
            .child(label::<M>(check_symbol));

        row::<M>()
            .key(b.id)
            .gap(8.0)
            .child(indicator)
            .child(label::<M>(b.label))
            .build()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;
    use acme_layout::{LayoutKind, Length};

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn checkbox_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = checkbox("test-cb", true).label("Enable feature").into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Row);
        // The Card child (index 0) holds the check indicator
        assert!(!layout.children.is_empty());
        // The Label child (index 1) has a non-zero min_height
        let label_leaf = &layout.children[1];
        assert!(label_leaf.children.is_empty());
        assert_ne!(label_leaf.style.min_height, Length::px(0.0));
    }

    #[test]
    fn checkbox_builder_defaults() {
        let cb = checkbox::<TestMsg>("cb", false);
        assert!(!cb.checked);
        assert!(!cb.disabled);
        assert!(cb.on_click.is_none());
        assert!(cb.label.is_empty());
    }

    #[test]
    fn checkbox_checked_shows_mark() {
        let node: WidgetNode<TestMsg> = checkbox("cb", true).label("Opt").into();
        let WidgetNode::Row(container) = &node else {
            panic!("expected Row");
        };
        assert_eq!(container.children.len(), 2);
        // First child is the Card indicator
        assert!(matches!(&container.children[0], WidgetNode::Card(_)));
        // Second child is the Label
        let WidgetNode::Label(lbl) = &container.children[1] else {
            panic!("expected Label");
        };
        assert_eq!(lbl.text, "Opt");
    }
}
