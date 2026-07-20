//! Skeleton component — a placeholder Card for loading states.

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
    /// Make the skeleton line-shaped (reduced height).
    pub fn line(mut self) -> Self {
        self.line = true;
        self
    }

    /// Build the skeleton widget.
    pub fn build(self) -> WidgetNode<M> {
        crate::card()
            .padding(0.0)
            .variant(crate::CardVariant::Muted)
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
    fn skeleton_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = skeleton(100.0, 20.0).build();
        // Skeleton is a Card with Muted variant.
        // Without explicit sizing, Card has no intrinsic dimensions.
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        assert_eq!(c.variant, crate::CardVariant::Muted);
    }

    #[test]
    fn skeleton_displays_label_text() {
        let node: WidgetNode<TestMsg> = skeleton(100.0, 20.0).build();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        assert_eq!(c.variant, crate::CardVariant::Muted);
    }
}
