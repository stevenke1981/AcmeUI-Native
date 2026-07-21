//! Watermark component — a subtle overlay label for sensitive or draft content.
//!
//! Renders as a full-area text overlay with configurable opacity, rotation,
//! and typography. Suitable for draft, confidential, preview, or review
//! watermarks.
//!
//! V2 design: uses `crate::ThemeColor` for color, spans with `flex: true`
//! to fill available space.

use crate::WidgetNode;

/// Builder for a watermark overlay.
pub struct WatermarkBuilder<M> {
    pub text: String,
    pub opacity: f32,
    pub color: crate::ThemeColor,
    pub font_size: f32,
    pub rotation_deg: f32,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a watermark builder. Defaults to 30% opacity, neutral color,
/// 48px font size, and -30° rotation.
pub fn watermark<M>(text: impl Into<String>) -> WatermarkBuilder<M> {
    WatermarkBuilder {
        text: text.into(),
        opacity: 0.3,
        color: crate::ThemeColor::rgb(128, 128, 128),
        font_size: 48.0,
        rotation_deg: -30.0,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> WatermarkBuilder<M> {
    /// Set the watermark text (e.g. "DRAFT", "CONFIDENTIAL", "REVIEW").
    pub fn text(mut self, value: impl Into<String>) -> Self {
        self.text = value.into();
        self
    }

    /// Set the opacity (0.0 = invisible, 1.0 = fully opaque).
    pub fn opacity(mut self, value: f32) -> Self {
        self.opacity = value.clamp(0.0, 1.0);
        self
    }

    /// Set the watermark font size in pixels.
    pub fn font_size(mut self, px: f32) -> Self {
        self.font_size = px;
        self
    }

    /// Set the rotation angle in degrees. Negative tilts left, positive right.
    pub fn rotation(mut self, deg: f32) -> Self {
        self.rotation_deg = deg;
        self
    }

    /// Set the watermark color. Alpha is overridden by opacity.
    pub fn color(mut self, value: crate::ThemeColor) -> Self {
        self.color = value;
        self
    }

    /// Build the watermark widget.
    ///
    /// Produces a Card with a centered, rotated label at the configured
    /// opacity and font size. The card uses the subtle variant so it
    /// remains invisible to interaction.
    pub fn build(self) -> WidgetNode<M> {
        let fg = self.color.with_alpha(self.opacity);

        crate::card::<M>()
            .variant(crate::CardVariant::Plain)
            .child(
                crate::label_builder(&self.text)
                    .font_size(self.font_size)
                    .color(fg)
                    .build(),
            )
            .padding(self.font_size * 0.3)
            .gap(0.0)
            .border_radius(0.0)
            .build()
    }
}

impl<M: Clone + 'static> From<WatermarkBuilder<M>> for WidgetNode<M> {
    fn from(b: WatermarkBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WidgetNode;
    use acme_core::NodeId;
    use acme_layout::{LayoutEngine, WidgetLayoutContext};

    fn test_context() -> WidgetLayoutContext {
        WidgetLayoutContext {
            body_font_size: 16.0,
            body_line_height: 22.0,
            label_font_size: 14.0,
            control_height: 32.0,
            scale_factor: 1.0,
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn watermark_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = watermark("DRAFT").build();
        let ctx = test_context();
        let layout = node.to_layout_with_context(NodeId::new(1), &ctx);
        let snapshot = LayoutEngine::new()
            .compute(&layout, (800.0, 600.0))
            .unwrap();
        let rect = snapshot.get(NodeId::new(1)).unwrap();
        assert!(rect.width > 0.0, "watermark width should be > 0");
        assert!(rect.height > 0.0, "watermark height should be > 0");
    }

    #[test]
    fn watermark_displays_text() {
        let node: WidgetNode<TestMsg> = watermark("CONFIDENTIAL").opacity(0.25).build();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        assert_eq!(c.children.len(), 1);
        let WidgetNode::Label(l) = &c.children[0] else {
            panic!("expected Label child");
        };
        assert_eq!(l.text, "CONFIDENTIAL");
    }

    #[test]
    fn watermark_uses_plain_variant() {
        let node: WidgetNode<TestMsg> = watermark("DRAFT").build();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        assert_eq!(c.variant, crate::CardVariant::Plain);
    }

    #[test]
    fn watermark_clamps_opacity() {
        let w = watermark::<TestMsg>("x").opacity(1.5);
        assert!((w.opacity - 1.0).abs() < f32::EPSILON);

        let w2 = watermark::<TestMsg>("x").opacity(-0.5);
        assert!((w2.opacity - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn watermark_builder_defaults() {
        let w = watermark::<TestMsg>("DRAFT");
        assert_eq!(w.text, "DRAFT");
        assert!((w.opacity - 0.3).abs() < f32::EPSILON);
        assert!((w.font_size - 48.0).abs() < f32::EPSILON);
        assert!((w.rotation_deg - (-30.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn watermark_from_conversion() {
        let node: WidgetNode<TestMsg> = watermark("REVIEW").into();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        assert_eq!(c.variant, crate::CardVariant::Plain);
    }
}
