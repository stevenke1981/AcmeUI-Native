//! SegmentedControl component.
//!
//! Renders as a Row of tab-like Buttons. The selected item uses an accent
//! (Primary) variant; unselected items use Ghost variant.

use acme_core::WidgetKey;
use acme_widgets::*;

/// Builder for a SegmentedControl component.
pub struct SegmentedControlBuilder<M> {
    pub id: WidgetKey,
    pub items: Vec<String>,
    pub selected: usize,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a new SegmentedControl builder.
pub fn segmented_control<M: Clone + 'static>(
    id: impl Into<WidgetKey>,
) -> SegmentedControlBuilder<M> {
    SegmentedControlBuilder {
        id: id.into(),
        items: vec![],
        selected: 0,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> SegmentedControlBuilder<M> {
    /// Add an item to the segmented control.
    pub fn item(mut self, label: impl Into<String>) -> Self {
        self.items.push(label.into());
        self
    }

    /// Set the index of the currently selected item (default 0).
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }
}

impl<M: Clone + 'static> From<SegmentedControlBuilder<M>> for WidgetNode<M> {
    fn from(b: SegmentedControlBuilder<M>) -> Self {
        let id_str = b.id.as_str().to_string();
        let mut seg_row = row::<M>().key(b.id).gap(0.0);
        for (i, item) in b.items.iter().enumerate() {
            let is_selected = i == b.selected;
            let btn_key = format!("{id_str}_{i}");
            seg_row = seg_row.child(button::<M>(btn_key.as_str(), item.as_str()).variant(
                if is_selected {
                    ButtonVariant::Primary
                } else {
                    ButtonVariant::Ghost
                },
            ));
        }
        seg_row.build()
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
    fn segmented_control_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = segmented_control("seg1")
            .item("Day")
            .item("Week")
            .item("Month")
            .selected(1)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Row);
        assert_eq!(layout.children.len(), 3);
        // Each child is a Button leaf with non-zero height
        for child in &layout.children {
            assert!(child.children.is_empty());
            assert_ne!(child.style.height, Length::px(0.0));
        }
    }

    #[test]
    fn segmented_control_builder_defaults() {
        let s = segmented_control::<TestMsg>("s");
        assert!(s.items.is_empty());
        assert_eq!(s.selected, 0);
    }

    #[test]
    fn segmented_control_selected_index() {
        let s: SegmentedControlBuilder<TestMsg> = segmented_control("seg")
            .item("A")
            .item("B")
            .item("C")
            .selected(2);
        assert_eq!(s.selected, 2);
        assert_eq!(s.items.len(), 3);
    }

    #[test]
    fn segmented_control_buttons_have_variants() {
        let node: WidgetNode<TestMsg> = segmented_control("seg")
            .item("One")
            .item("Two")
            .selected(0)
            .into();
        let WidgetNode::Row(container) = &node else {
            panic!("expected Row");
        };
        assert_eq!(container.children.len(), 2);
        // First button should be Primary (selected)
        let WidgetNode::Button(btn0) = &container.children[0] else {
            panic!("expected Button");
        };
        assert_eq!(btn0.variant, ButtonVariant::Primary);
        // Second button should be Ghost (unselected)
        let WidgetNode::Button(btn1) = &container.children[1] else {
            panic!("expected Button");
        };
        assert_eq!(btn1.variant, ButtonVariant::Ghost);
    }
}
