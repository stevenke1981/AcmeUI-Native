//! AspectRatio component — a container that constrains its child to a fixed
//! aspect ratio (width / height).
//!
//! Renders as a `Column` with explicit width and height computed from the
//! given ratio and a known width (or height).

use crate::WidgetNode;

/// Builder for an aspect-ratio container.
pub struct AspectRatioBuilder<M> {
    pub ratio: f32, // width / height
    pub width: f32,
    pub child: Option<WidgetNode<M>>,
}

/// Create an aspect-ratio container builder.
///
/// `ratio` is width / height (e.g. 16.0 / 9.0 for 16:9).
pub fn aspect_ratio<M>(ratio: f32) -> AspectRatioBuilder<M> {
    AspectRatioBuilder {
        ratio,
        width: 200.0,
        child: None,
    }
}

impl<M: Clone + 'static> AspectRatioBuilder<M> {
    /// Set the container width in pixels (height = width / ratio).
    pub fn width(mut self, value: f32) -> Self {
        self.width = value;
        self
    }

    /// Set the child widget to place inside the ratio container.
    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.child = Some(child.into());
        self
    }

    /// Build the aspect-ratio container.
    ///
    /// Returns a `Column` with explicit `width` and `height` where
    /// height = `width / ratio`, wrapping the child content.
    pub fn build(self) -> WidgetNode<M> {
        let height = if self.ratio > 0.0 {
            self.width / self.ratio
        } else {
            self.width
        };

        let mut col = crate::column()
            .width(self.width)
            .height(height)
            .padding(0.0);

        if let Some(child) = self.child {
            col = col.child(child);
        }

        col.build()
    }
}

impl<M: Clone + 'static> From<AspectRatioBuilder<M>> for WidgetNode<M> {
    fn from(b: AspectRatioBuilder<M>) -> Self {
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
    fn aspect_ratio_computes_height() {
        let node: WidgetNode<TestMsg> = aspect_ratio(16.0 / 9.0).width(320.0).build();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant");
        };
        assert_eq!(col.width, Some(320.0));
        assert_eq!(col.height, Some(180.0)); // 320 / (16/9) = 180
    }

    #[test]
    fn aspect_ratio_square() {
        let node: WidgetNode<TestMsg> = aspect_ratio(1.0).width(100.0).build();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant");
        };
        assert_eq!(col.width, Some(100.0));
        assert_eq!(col.height, Some(100.0));
    }

    #[test]
    fn aspect_ratio_ultrawide() {
        let node: WidgetNode<TestMsg> = aspect_ratio(21.0 / 9.0).width(420.0).build();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant");
        };
        assert_eq!(col.width, Some(420.0));
        assert_eq!(col.height, Some(180.0)); // 420 / (21/9) = 180
    }

    #[test]
    fn aspect_ratio_zero_ratio_fallback() {
        let node: WidgetNode<TestMsg> = aspect_ratio(0.0).width(100.0).build();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant");
        };
        assert_eq!(col.width, Some(100.0));
        assert_eq!(col.height, Some(100.0)); // fallback: height = width
    }

    #[test]
    fn aspect_ratio_with_child() {
        let node: WidgetNode<TestMsg> = aspect_ratio(4.0 / 3.0)
            .width(120.0)
            .child(crate::label("content"))
            .build();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant");
        };
        assert_eq!(col.width, Some(120.0));
        assert_eq!(col.height, Some(90.0)); // 120 / (4/3) = 90
        assert_eq!(col.children.len(), 1);
    }

    #[test]
    fn aspect_ratio_from_trait() {
        let node: WidgetNode<TestMsg> = aspect_ratio(1.0).width(50.0).into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant");
        };
        assert_eq!(col.width, Some(50.0));
    }
}
