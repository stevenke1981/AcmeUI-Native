//! Spinner component — a rotating indicator rendered as a Label.

use crate::WidgetNode;

/// Builder for a spinner widget.
pub struct SpinnerBuilder<M> {
    pub size: f32,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a spinner builder.
pub fn spinner<M>() -> SpinnerBuilder<M> {
    SpinnerBuilder {
        size: 16.0,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> SpinnerBuilder<M> {
    /// Set the spinner size in pixels.
    pub fn size(mut self, px: f32) -> Self {
        self.size = px;
        self
    }

    /// Build the spinner widget.
    pub fn build(self) -> WidgetNode<M> {
        let mut lbl = crate::label("⟳");
        if let WidgetNode::Label(ref mut l) = lbl {
            l.font_size = Some(self.size);
        }
        lbl
    }
}

impl<M: Clone + 'static> From<SpinnerBuilder<M>> for WidgetNode<M> {
    fn from(b: SpinnerBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
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
    fn spinner_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = spinner().size(24.0).build();
        let ctx = test_context();
        let layout = node.to_layout_with_context(NodeId::new(1), &ctx);
        let mut fonts = acme_text::FontSystem::new();
        let snapshot = LayoutEngine::new()
            .compute_with_text(&layout, (800.0, 600.0), &mut fonts, 1.0)
            .unwrap();
        let rect = snapshot.get(NodeId::new(1)).unwrap();
        assert!(rect.width > 0.0, "spinner width should be > 0");
        assert!(rect.height > 0.0, "spinner height should be > 0");
    }

    #[test]
    fn spinner_displays_label_text() {
        let node: WidgetNode<TestMsg> = spinner().build();
        let WidgetNode::Label(l) = &node else {
            panic!("expected Label variant");
        };
        assert_eq!(l.text, "⟳");
    }
}
