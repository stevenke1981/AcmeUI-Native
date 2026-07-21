//! Skeleton component — a placeholder Card for loading states.
//!
//! Renders as a `Column` with explicit `width` / `height` (so the skeleton
//! actually occupies layout space) wrapping a muted `Card`.

use crate::WidgetNode;

/// Builder for a skeleton (loading placeholder) widget.
pub struct SkeletonBuilder<M> {
    pub width: f32,
    pub height: f32,
    pub line: bool,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a skeleton builder.
pub fn skeleton<M>(width: f32, height: f32) -> SkeletonBuilder<M> {
    SkeletonBuilder {
        width,
        height,
        line: false,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> SkeletonBuilder<M> {
    /// Make the skeleton line-shaped (reduced height to ~1em).
    pub fn line(mut self) -> Self {
        self.line = true;
        self
    }

    /// Build the skeleton widget.
    ///
    /// Returns a `Column` with the configured `width` / `height` wrapping a
    /// muted `Card`, so the skeleton occupies measurable layout space.
    pub fn build(self) -> WidgetNode<M> {
        let h = if self.line { 16.0 } else { self.height };
        let card = crate::card()
            .padding(0.0)
            .variant(crate::CardVariant::Muted)
            .build();
        crate::column()
            .width(self.width)
            .height(h)
            .child(card)
            .build()
    }
}

impl<M: Clone + 'static> From<SkeletonBuilder<M>> for WidgetNode<M> {
    fn from(b: SkeletonBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn skeleton_occupies_layout_space() {
        let node: WidgetNode<TestMsg> = skeleton(100.0, 20.0).build();
        // Skeleton now returns a Column with explicit width/height
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column variant (wrapper for sized skeleton)");
        };
        assert_eq!(c.width, Some(100.0));
        assert_eq!(c.height, Some(20.0));
        // First child should be the muted Card
        assert!(!c.children.is_empty(), "skeleton should have a card child");
        let WidgetNode::Card(card) = &c.children[0] else {
            panic!("expected Card as first child of skeleton Column");
        };
        assert_eq!(card.variant, crate::CardVariant::Muted);
    }

    #[test]
    fn skeleton_line_mode() {
        let node: WidgetNode<TestMsg> = skeleton(200.0, 20.0).line().build();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column variant");
        };
        assert_eq!(c.width, Some(200.0));
        // line mode overrides height to 16px
        assert_eq!(c.height, Some(16.0));
    }

    #[test]
    fn skeleton_from_trait() {
        let node: WidgetNode<TestMsg> = skeleton(100.0, 20.0).into();
        let WidgetNode::Column(_) = &node else {
            panic!("expected Column variant");
        };
    }
}
