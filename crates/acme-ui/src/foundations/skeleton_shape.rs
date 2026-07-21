//! SkeletonShape — loading placeholder with shape variants (circle/text/rect).
//! Absorbs gpui-component's skeleton variant strength.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Skeleton shape variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SkeletonShape {
    #[default]
    Text,
    Circle,
    Rect,
    Rounded,
}

/// Builder for a skeleton shape.
pub struct SkeletonShapeBuilder<M> {
    pub id: WidgetKey,
    pub shape: SkeletonShape,
    pub width: f32,
    pub height: f32,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a skeleton shape builder.
pub fn skeleton_shape<M: Clone + 'static>() -> SkeletonShapeBuilder<M> {
    SkeletonShapeBuilder {
        id: WidgetKey::from("skeleton_shape"),
        shape: SkeletonShape::default(),
        width: 100.0,
        height: 16.0,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> SkeletonShapeBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn shape(mut self, value: SkeletonShape) -> Self {
        self.shape = value;
        self
    }

    pub fn width(mut self, value: f32) -> Self {
        self.width = value;
        self
    }

    pub fn height(mut self, value: f32) -> Self {
        self.height = value;
        self
    }

    /// Convenience for a circle of the given diameter.
    pub fn circle(mut self, diameter: f32) -> Self {
        self.shape = SkeletonShape::Circle;
        self.width = diameter;
        self.height = diameter;
        self
    }
}

impl<M: Clone + 'static> From<SkeletonShapeBuilder<M>> for WidgetNode<M> {
    fn from(b: SkeletonShapeBuilder<M>) -> Self {
        let variant = match b.shape {
            SkeletonShape::Rounded | SkeletonShape::Circle => {
                acme_widgets::CardVariant::Elevated
            }
            _ => acme_widgets::CardVariant::Muted,
        };
        crate::card::<M>()
            .key(b.id)
            .variant(variant)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn skeleton_shape_produces_card() {
        let node: WidgetNode<Msg> = skeleton_shape().into();
        assert!(matches!(node, WidgetNode::Card(_)));
    }

    #[test]
    fn skeleton_shape_circle() {
        let b = skeleton_shape::<Msg>().circle(40.0);
        assert_eq!(b.shape, SkeletonShape::Circle);
        assert_eq!(b.width, 40.0);
        assert_eq!(b.height, 40.0);
    }

    #[test]
    fn skeleton_shape_default_text() {
        let b = skeleton_shape::<Msg>();
        assert_eq!(b.shape, SkeletonShape::Text);
    }
}
