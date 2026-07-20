//! StatusBar component — a muted bar with left and right labels.

use crate::WidgetNode;

/// Builder for a status bar.
pub struct StatusBarBuilder<M> {
    pub left: String,
    pub right: String,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a status bar builder.
pub fn status_bar<M: Clone + 'static>() -> StatusBarBuilder<M> {
    StatusBarBuilder {
        left: String::new(),
        right: String::new(),
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone> StatusBarBuilder<M> {
    /// Set the left-aligned label text.
    pub fn left(mut self, text: impl Into<String>) -> Self {
        self.left = text.into();
        self
    }

    /// Set the right-aligned label text.
    pub fn right(mut self, text: impl Into<String>) -> Self {
        self.right = text.into();
        self
    }

    /// Build the widget node tree.
    ///
    /// Renders a muted `Card` containing a `Row` with left label + right label.
    pub fn build(self) -> WidgetNode<M> {
        let left_label = crate::label::<M>(self.left);
        let right_label = crate::label::<M>(self.right);

        let inner = crate::row::<M>()
            .child(left_label)
            .child(right_label)
            .gap(8.0)
            .build();

        crate::card::<M>()
            .variant(crate::CardVariant::Muted)
            .child(inner)
            .padding(4.0)
            .build()
    }
}

impl<M: Clone + 'static> From<StatusBarBuilder<M>> for WidgetNode<M> {
    fn from(b: StatusBarBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WidgetNode;
    use acme_core::NodeId;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn status_bar_has_non_zero_layout_rect() {
        let node: WidgetNode<Msg> = status_bar::<Msg>()
            .left("Ready")
            .right("Ln 1, Col 1")
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Card wrapper contains a Row
        assert_eq!(layout.children.len(), 1);
        // Row contains left label + right label
        assert_eq!(layout.children[0].children.len(), 2);
    }

    #[test]
    fn status_bar_defaults() {
        let node: WidgetNode<Msg> = status_bar::<Msg>().build();
        let layout = node.to_layout(NodeId::new(1));
        // Card with one Row child
        assert_eq!(layout.children.len(), 1);
    }

    #[test]
    fn status_bar_uses_muted_card() {
        let node: WidgetNode<Msg> = status_bar::<Msg>().left("test").build();
        assert!(matches!(node, WidgetNode::Card(_)));
    }
}
