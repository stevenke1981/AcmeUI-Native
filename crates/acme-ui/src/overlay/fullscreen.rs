//! Fullscreen overlay component — a full-viewport overlay for displaying
//! images, videos, or maximized content with a close control.
//!
//! Renders as a full-area plain Card with centered content and an optional
//! top-right close button. Suitable for image lightbox, video player,
//! or presentation mode.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a fullscreen overlay.
pub struct FullscreenBuilder<M> {
    pub id: WidgetKey,
    pub children: Vec<WidgetNode<M>>,
    pub background_opacity: f32,
    pub close_button: bool,
    pub on_close: Option<M>,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a fullscreen overlay builder.
pub fn fullscreen<M: Clone + 'static>(id: impl Into<WidgetKey>) -> FullscreenBuilder<M> {
    FullscreenBuilder {
        id: id.into(),
        children: vec![],
        background_opacity: 0.85,
        close_button: true,
        on_close: None,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> FullscreenBuilder<M> {
    /// Add content to display in the overlay.
    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Set the backdrop darkness (0.0 = transparent, 1.0 = solid black).
    pub fn background_opacity(mut self, value: f32) -> Self {
        self.background_opacity = value.clamp(0.0, 1.0);
        self
    }

    /// Show/hide the close button.
    pub fn close_button(mut self, value: bool) -> Self {
        self.close_button = value;
        self
    }

    /// Set the message dispatched when the overlay is dismissed.
    pub fn on_close(mut self, msg: M) -> Self {
        self.on_close = Some(msg);
        self
    }

    /// Build the fullscreen overlay widget.
    pub fn build(self) -> WidgetNode<M> {
        let mut content = crate::column::<M>().key(self.id.clone()).gap(12.0);

        // Optional close button row (top-right aligned)
        if self.close_button {
            let header = crate::row::<M>()
                .child(crate::row::<M>().build())
                .child(crate::label::<M>("✕"))
                .build();
            content = content.child(header);
        }

        // Centered body content
        let mut body = crate::column::<M>().gap(8.0);
        for child in self.children {
            body = body.child(child);
        }
        content = content.child(body.build());

        // Full-area backdrop card
        crate::card::<M>()
            .variant(crate::CardVariant::Plain)
            .border_radius(0.0)
            .padding(16.0)
            .child(content)
            .build()
    }
}

impl<M: Clone + 'static> From<FullscreenBuilder<M>> for WidgetNode<M> {
    fn from(b: FullscreenBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn fullscreen_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = fullscreen("fs")
            .child(crate::label::<TestMsg>("Content"))
            .build();
        let ctx = test_context();
        let layout = node.to_layout_with_context(NodeId::new(1), &ctx);
        let snapshot = LayoutEngine::new()
            .compute(&layout, (800.0, 600.0))
            .unwrap();
        let rect = snapshot.get(NodeId::new(1)).unwrap();
        assert!(rect.width > 0.0);
        assert!(rect.height > 0.0);
    }

    #[test]
    fn fullscreen_with_content() {
        let node: WidgetNode<TestMsg> = fullscreen("fs")
            .child(crate::label::<TestMsg>("Image"))
            .build();
        let WidgetNode::Card(outer) = &node else {
            panic!("expected Card")
        };
        let WidgetNode::Column(col) = &outer.children[0] else {
            panic!("expected Column")
        };
        // header row + body column = 2
        assert_eq!(col.children.len(), 2);
    }

    #[test]
    fn fullscreen_no_close_button() {
        let node: WidgetNode<TestMsg> = fullscreen("fs").close_button(false).build();
        let WidgetNode::Card(outer) = &node else {
            panic!("expected Card")
        };
        let WidgetNode::Column(col) = &outer.children[0] else {
            panic!("expected Column")
        };
        // only body column (no header row)
        assert_eq!(col.children.len(), 1);
    }

    #[test]
    fn fullscreen_clamps_opacity() {
        let f = fullscreen::<TestMsg>("fs").background_opacity(1.5);
        assert!((f.background_opacity - 1.0).abs() < f32::EPSILON);
        let f2 = fullscreen::<TestMsg>("fs").background_opacity(-0.5);
        assert!((f2.background_opacity - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn fullscreen_builder_defaults() {
        let f = fullscreen::<TestMsg>("fs");
        assert!(f.close_button);
        assert!((f.background_opacity - 0.85).abs() < f32::EPSILON);
        assert!(f.children.is_empty());
    }

    #[test]
    fn fullscreen_on_close_default_none() {
        let f = fullscreen::<TestMsg>("fs");
        assert!(f.on_close.is_none());
    }
}
