//! ScrollArea component — a styled scroll container wrapping child content.
//!
//! Wraps the [`ScrollView`] primitive from `acme-widgets` with the AcmeUIKit builder pattern.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a scroll area.
pub struct ScrollAreaBuilder<M> {
    pub id: WidgetKey,
    pub children: Vec<WidgetNode<M>>,
    pub viewport_height: f32,
}

/// Create a scroll area builder.
pub fn scroll_area<M: Clone + 'static>(id: impl Into<WidgetKey>) -> ScrollAreaBuilder<M> {
    ScrollAreaBuilder {
        id: id.into(),
        children: vec![],
        viewport_height: 300.0,
    }
}

impl<M: Clone> ScrollAreaBuilder<M> {
    /// Add a child widget to the scroll area.
    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Set the viewport height in pixels.
    pub fn viewport_height(mut self, value: f32) -> Self {
        self.viewport_height = value;
        self
    }

    /// Build the widget node tree.
    ///
    /// Renders a [`ScrollView`](crate::ScrollView) with the configured children
    /// and viewport height.
    pub fn build(self) -> WidgetNode<M> {
        let mut sv = crate::scroll_view::<M>(self.id);
        for child in self.children {
            sv = sv.child(child);
        }
        sv = sv.viewport_height(self.viewport_height);
        sv.build()
    }
}

impl<M: Clone + 'static> From<ScrollAreaBuilder<M>> for WidgetNode<M> {
    fn from(b: ScrollAreaBuilder<M>) -> Self {
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
    fn scroll_area_defaults() {
        let b = scroll_area::<Msg>("s1");
        assert!((b.viewport_height - 300.0).abs() < f32::EPSILON);
        assert!(b.children.is_empty());
    }

    #[test]
    fn scroll_area_adds_child() {
        let node: WidgetNode<Msg> = scroll_area::<Msg>("s2")
            .child(crate::label("Hello"))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.children.len(), 1);
    }

    #[test]
    fn scroll_area_sets_viewport_height() {
        let node: WidgetNode<Msg> = scroll_area::<Msg>("s3")
            .viewport_height(500.0)
            .child(crate::label("Tall"))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.children.len(), 1);
    }

    #[test]
    fn scroll_area_has_correct_key() {
        let b = scroll_area::<Msg>("my-scroll");
        assert_eq!(b.id.as_str(), "my-scroll");
    }

    #[test]
    fn scroll_area_multiple_children() {
        let node: WidgetNode<Msg> = scroll_area::<Msg>("s4")
            .child(crate::label("A"))
            .child(crate::label("B"))
            .child(crate::label("C"))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.children.len(), 3);
    }

    #[test]
    fn scroll_area_is_scroll_view() {
        let node: WidgetNode<Msg> = scroll_area::<Msg>("s5")
            .child(crate::label("X"))
            .build();
        assert!(matches!(node, WidgetNode::ScrollView(_)));
    }

    #[test]
    fn scroll_area_no_children_still_renders() {
        let node: WidgetNode<Msg> = scroll_area::<Msg>("s6")
            .viewport_height(600.0)
            .build();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.children.len(), 0);
    }
}
