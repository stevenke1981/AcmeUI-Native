//! Divider component for separating content with an optional label.

use crate::WidgetNode;

/// Build a horizontal divider, optionally with a centered label.
pub fn divider<M: Clone + 'static>() -> WidgetNode<M> {
    crate::row::<M>().key("acmeui-divider").height(1.0).build()
}

/// Build a labeled divider.
pub fn labeled_divider<M: Clone + 'static>(label: impl Into<String>) -> WidgetNode<M> {
    crate::row::<M>()
        .key("acmeui-labeled-divider")
        .gap(8.0)
        .child(crate::label(label))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use acme_widgets::WidgetNode;

    #[test]
    fn divider_builds_stable_row() {
        let node = divider::<()>();
        assert!(matches!(node, WidgetNode::Row(_)));
    }

    #[test]
    fn labeled_divider_contains_label() {
        let node = labeled_divider::<()>("Advanced");
        assert_eq!(node.children().len(), 1);
    }
}
