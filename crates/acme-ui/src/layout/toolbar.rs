//! Toolbar component — horizontal row of groups separated by vertical dividers.

use crate::WidgetNode;

/// Builder for a toolbar.
pub struct ToolbarBuilder<M> {
    pub children: Vec<WidgetNode<M>>,
}

/// Create a toolbar builder.
pub fn toolbar<M: Clone + 'static>() -> ToolbarBuilder<M> {
    ToolbarBuilder { children: vec![] }
}

impl<M: Clone> ToolbarBuilder<M> {
    /// Add a child widget group.
    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Build the widget node tree.
    /// Each child is separated by a vertical separator.
    pub fn build(self) -> WidgetNode<M> {
        let mut row = crate::row::<M>().gap(4.0);
        for (i, child) in self.children.into_iter().enumerate() {
            if i > 0 {
                row = row.child(crate::separator::<M>());
            }
            row = row.child(child);
        }
        row.build()
    }
}

impl<M: Clone + 'static> From<ToolbarBuilder<M>> for WidgetNode<M> {
    fn from(b: ToolbarBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{label, WidgetNode};
    use acme_core::NodeId;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn toolbar_has_non_zero_layout_rect() {
        let node: WidgetNode<Msg> = toolbar::<Msg>()
            .child(label("File"))
            .child(label("Edit"))
            .child(label("View"))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Row: group1 + sep + group2 + sep + group3 = 5 children
        assert_eq!(layout.children.len(), 5);
    }

    #[test]
    fn toolbar_single_child_no_separator() {
        let node: WidgetNode<Msg> = toolbar::<Msg>().child(label("Only")).into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.children.len(), 1);
    }

    #[test]
    fn toolbar_empty() {
        let node: WidgetNode<Msg> = toolbar::<Msg>().build();
        let layout = node.to_layout(NodeId::new(1));
        assert!(layout.children.is_empty());
    }
}
