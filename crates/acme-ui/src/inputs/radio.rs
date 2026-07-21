//! Radio button and RadioGroup input components.
//!
//! Radio renders as a Row with a circle indicator (○/●) and a label.
//! RadioGroup wraps multiple Radio items in a Column.

use acme_core::WidgetKey;
use acme_layout::Edges;
use acme_widgets::*;

/// Builder for a single Radio button.
pub struct RadioBuilder<M> {
    pub id: WidgetKey,
    pub value: String,
    pub label: String,
    pub selected: bool,
    pub disabled: bool,
    pub on_click: Option<M>,
}

/// Create a new Radio builder.
pub fn radio<M: Clone + 'static>(
    id: impl Into<WidgetKey>,
    value: impl Into<String>,
) -> RadioBuilder<M> {
    RadioBuilder {
        id: id.into(),
        value: value.into(),
        label: String::new(),
        selected: false,
        disabled: false,
        on_click: None,
    }
}

impl<M: Clone + 'static> RadioBuilder<M> {
    /// Set the label text displayed next to the radio indicator.
    pub fn label(mut self, value: impl Into<String>) -> Self {
        self.label = value.into();
        self
    }

    /// Mark this radio as selected (shows filled circle).
    pub fn selected(mut self, value: bool) -> Self {
        self.selected = value;
        self
    }

    /// Set whether the radio is disabled.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    /// Set the message dispatched when this radio is selected.
    pub fn on_click(mut self, msg: M) -> Self {
        self.on_click = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<RadioBuilder<M>> for WidgetNode<M> {
    fn from(b: RadioBuilder<M>) -> Self {
        let indicator = if b.selected { "●" } else { "○" };
        row::<M>()
            .key(b.id)
            .gap(8.0)
            .child(label_with_size::<M>(indicator, 16.0))
            .child(label::<M>(b.label))
            .build()
    }
}

/// Builder for a RadioGroup — a Column of Radio items.
pub struct RadioGroupBuilder<M> {
    pub selected: Option<String>,
    pub children: Vec<WidgetNode<M>>,
}

/// Create a new RadioGroup builder.
pub fn radio_group<M: Clone + 'static>() -> RadioGroupBuilder<M> {
    RadioGroupBuilder {
        selected: None,
        children: vec![],
    }
}

impl<M: Clone + 'static> RadioGroupBuilder<M> {
    /// Add a child widget (typically a RadioBuilder or WidgetNode).
    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Set the currently selected value.
    pub fn selected(mut self, value: Option<String>) -> Self {
        self.selected = value;
        self
    }
}

impl<M: Clone + 'static> From<RadioGroupBuilder<M>> for WidgetNode<M> {
    fn from(b: RadioGroupBuilder<M>) -> Self {
        WidgetNode::Column(Container {
            key: None,
            children: b.children,
            gap: 4.0,
            padding: Edges::default(),
            width: None,
            height: None,
            style: crate::Style::new(),
            message: None,
        })
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
    fn radio_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = radio("r1", "opt1").label("Option 1").selected(true).into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Row);
        assert!(!layout.children.is_empty());
        let label_leaf = &layout.children[1];
        assert!(label_leaf.children.is_empty());
        assert_ne!(label_leaf.style.min_height, Length::px(0.0));
    }

    #[test]
    fn radio_builder_defaults() {
        let r = radio::<TestMsg>("r", "val");
        assert_eq!(r.value, "val");
        assert!(!r.selected);
        assert!(!r.disabled);
    }

    #[test]
    fn radio_selected_shows_filled() {
        let node: WidgetNode<TestMsg> = radio("r", "a").label("A").selected(true).into();
        let WidgetNode::Row(container) = &node else {
            panic!("expected Row");
        };
        assert_eq!(container.children.len(), 2);
        // First child is the indicator label
        let WidgetNode::Label(indicator) = &container.children[0] else {
            panic!("expected Label indicator");
        };
        assert_eq!(indicator.text, "●");
    }

    #[test]
    fn radio_unselected_shows_empty() {
        let node: WidgetNode<TestMsg> = radio("r", "a").label("A").into();
        let WidgetNode::Row(container) = &node else {
            panic!("expected Row");
        };
        let WidgetNode::Label(indicator) = &container.children[0] else {
            panic!("expected Label indicator");
        };
        assert_eq!(indicator.text, "○");
    }

    #[test]
    fn radio_group_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = radio_group()
            .child(radio("r1", "a").label("A"))
            .child(radio("r2", "b").label("B").selected(true))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert_eq!(layout.children.len(), 2);
        assert!(
            layout.children[0].style.min_height != Length::px(0.0)
                || layout.children[0].style.height != Length::px(0.0)
        );
    }

    #[test]
    fn radio_group_column_structure() {
        let node: WidgetNode<TestMsg> = radio_group()
            .child(radio("r1", "a").label("A"))
            .child(radio("r2", "b").label("B"))
            .into();
        let WidgetNode::Column(container) = &node else {
            panic!("expected Column");
        };
        assert_eq!(container.children.len(), 2);
    }
}
