//! Spinner component — a rotating indicator rendered as a Label.

use crate::WidgetNode;

/// Builder for a spinner widget.
pub struct SpinnerBuilder<M> {
    pub size: crate::ControlSize,
    pub tone: crate::Tone,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a spinner builder.
pub fn spinner<M>() -> SpinnerBuilder<M> {
    SpinnerBuilder {
        size: crate::ControlSize::Md,
        tone: crate::Tone::Neutral,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> SpinnerBuilder<M> {
    /// Set the spinner size using a standard ControlSize.
    pub fn size(mut self, value: crate::ControlSize) -> Self {
        self.size = value;
        self
    }

    /// Set the spinner tone.
    pub fn tone(mut self, value: crate::Tone) -> Self {
        self.tone = value;
        self
    }

    /// Build the spinner widget.
    pub fn build(self) -> WidgetNode<M> {
        let font_size = match self.size {
            crate::ControlSize::Xs => 22.0,
            crate::ControlSize::Sm => 28.0,
            crate::ControlSize::Md => 34.0,
            crate::ControlSize::Lg => 40.0,
            crate::ControlSize::Xl => 48.0,
        } * 0.75;
        let mut lbl = crate::label("⟳");
        if let WidgetNode::Label(ref mut l) = lbl {
            l.font_size = Some(font_size);
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
        let node: WidgetNode<TestMsg> = spinner().size(crate::ControlSize::Xs).build();
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
