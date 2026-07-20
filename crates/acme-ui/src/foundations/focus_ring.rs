//! FocusRing component — a focus indicator wrapper.
//!
//! Wraps a child widget in a Card:
//! - When focused: adds 2px padding with the theme's ring color as background
//! - When not focused: returns the child or empty Card with no visible effect

use crate::*;

/// Builder for a FocusRing component.
pub struct FocusRingBuilder<M> {
    pub id: WidgetKey,
    pub focused: bool,
    pub child: Option<WidgetNode<M>>,
}

/// Create a new FocusRing builder.
pub fn focus_ring<M: Clone + 'static>(id: impl Into<WidgetKey>) -> FocusRingBuilder<M> {
    FocusRingBuilder {
        id: id.into(),
        focused: false,
        child: None,
    }
}

impl<M: Clone + 'static> FocusRingBuilder<M> {
    /// Set whether the focus ring is visible.
    pub fn focused(mut self, value: bool) -> Self {
        self.focused = value;
        self
    }

    /// Set the child widget wrapped by the focus ring.
    pub fn child(mut self, node: impl Into<WidgetNode<M>>) -> Self {
        self.child = Some(node.into());
        self
    }
}

impl<M: Clone + 'static> From<FocusRingBuilder<M>> for WidgetNode<M> {
    fn from(b: FocusRingBuilder<M>) -> Self {
        let mut wrapper = card::<M>().key(b.id);

        if b.focused {
            let theme = acme_theme::Theme::light();
            wrapper = wrapper
                .padding(2.0)
                .background_color(theme.colors.ring);
        }

        if let Some(child) = b.child {
            wrapper = wrapper.child(child);
        }

        wrapper.build()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;
    use acme_layout::Length;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn focus_ring_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = focus_ring("fr")
            .focused(true)
            .child(label::<TestMsg>("Input"))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Card wraps as a container with padding
        assert!(!layout.children.is_empty());
        assert_ne!(layout.style.min_height, Length::px(0.0));
    }

    #[test]
    fn focus_ring_builder_defaults() {
        let fr = focus_ring::<TestMsg>("fr");
        assert!(!fr.focused);
        assert!(fr.child.is_none());
    }

    #[test]
    fn focus_ring_focused_sets_background_and_padding() {
        let node: WidgetNode<TestMsg> = focus_ring("fr")
            .focused(true)
            .child(label::<TestMsg>("Focused"))
            .into();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        assert!(c.background_color.is_some(), "focused ring must set background_color");
        assert_eq!(c.padding.top, 2.0);
        assert_eq!(c.children.len(), 1);
    }

    #[test]
    fn focus_ring_not_focused_no_background() {
        let node: WidgetNode<TestMsg> = focus_ring("fr")
            .focused(false)
            .child(label::<TestMsg>("Normal"))
            .into();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        assert!(c.background_color.is_none(), "unfocused ring must not set background_color");
        assert_eq!(c.padding.top, 0.0);
    }

    #[test]
    fn focus_ring_without_child() {
        let node: WidgetNode<TestMsg> = focus_ring("fr")
            .focused(true)
            .into();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        assert!(c.children.is_empty());
    }

    #[test]
    fn focus_ring_toggle_focused() {
        let fr = focus_ring::<TestMsg>("fr").focused(true);
        assert!(fr.focused);
        let fr = fr.focused(false);
        assert!(!fr.focused);
    }
}
