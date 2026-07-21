//! DragRegion component — a draggable area for window moving or panel dragging.
//!
//! Renders as a column with explicit height wrapping a muted Card that shows
//! a subtle drag indicator "⋯" and optional child content.

use crate::*;

/// Builder for a DragRegion component.
pub struct DragRegionBuilder<M> {
    pub id: WidgetKey,
    pub height: f32,
    pub child: Option<WidgetNode<M>>,
    pub on_drag_start: Option<M>,
    pub on_drag_end: Option<M>,
}

/// Create a new DragRegion builder.
pub fn drag_region<M: Clone + 'static>(id: impl Into<WidgetKey>) -> DragRegionBuilder<M> {
    DragRegionBuilder {
        id: id.into(),
        height: 24.0,
        child: None,
        on_drag_start: None,
        on_drag_end: None,
    }
}

impl<M: Clone + 'static> DragRegionBuilder<M> {
    /// Set the height of the drag region in pixels.
    pub fn height(mut self, value: f32) -> Self {
        self.height = value;
        self
    }

    /// Set optional child content displayed inside the drag region.
    pub fn child(mut self, node: impl Into<WidgetNode<M>>) -> Self {
        self.child = Some(node.into());
        self
    }

    /// Set the message dispatched when dragging starts.
    pub fn on_drag_start(mut self, msg: M) -> Self {
        self.on_drag_start = Some(msg);
        self
    }

    /// Set the message dispatched when dragging ends.
    pub fn on_drag_end(mut self, msg: M) -> Self {
        self.on_drag_end = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<DragRegionBuilder<M>> for WidgetNode<M> {
    fn from(b: DragRegionBuilder<M>) -> Self {
        let mut inner = card::<M>().variant(CardVariant::Muted).child(
            label_builder::<M>("⋯")
                .font_size(14.0)
                .color(acme_theme::Theme::light().colors.muted_foreground)
                .build(),
        );

        if let Some(child) = b.child {
            inner = inner.child(child);
        }

        column::<M>()
            .key(b.id)
            .height(b.height)
            .child(inner)
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
    use acme_layout::LayoutKind;
    use acme_layout::Length;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn drag_region_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = drag_region("dr").into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert_eq!(layout.style.height, Length::px(24.0));
    }

    #[test]
    fn drag_region_builder_defaults() {
        let dr = drag_region::<TestMsg>("dr");
        assert!((dr.height - 24.0).abs() < f32::EPSILON);
        assert!(dr.child.is_none());
        assert!(dr.on_drag_start.is_none());
        assert!(dr.on_drag_end.is_none());
    }

    #[test]
    fn drag_region_with_height() {
        let node: WidgetNode<TestMsg> = drag_region("dr").height(48.0).into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.height, Length::px(48.0));
    }

    #[test]
    fn drag_region_with_child() {
        let node: WidgetNode<TestMsg> = drag_region("dr").child(label::<TestMsg>("Content")).into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant");
        };
        assert_eq!(col.children.len(), 1);
        let WidgetNode::Card(c) = &col.children[0] else {
            panic!("expected Card child");
        };
        // Muted indicator + child label
        assert!(!c.children.is_empty());
    }

    #[test]
    fn drag_region_messages() {
        #[derive(Clone, Debug, PartialEq)]
        enum Msg {
            Start,
            End,
        }
        let dr = drag_region::<Msg>("dr")
            .on_drag_start(Msg::Start)
            .on_drag_end(Msg::End);
        assert_eq!(dr.on_drag_start, Some(Msg::Start));
        assert_eq!(dr.on_drag_end, Some(Msg::End));
    }
}
