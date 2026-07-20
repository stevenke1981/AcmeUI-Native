//! Kbd component — a keyboard shortcut indicator rendered as a styled Label.

use crate::WidgetNode;

/// Builder for a keyboard shortcut (kbd) widget.
pub struct KbdBuilder<M> {
    pub text: String,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a kbd builder.
pub fn kbd<M>(text: impl Into<String>) -> KbdBuilder<M> {
    KbdBuilder {
        text: text.into(),
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> KbdBuilder<M> {
    /// Build the kbd widget.
    pub fn build(self) -> WidgetNode<M> {
        let mut lbl = crate::label(self.text);
        if let WidgetNode::Label(ref mut l) = lbl {
            l.font_size = Some(12.0);
        }
        lbl
    }
}

impl<M: Clone + 'static> From<KbdBuilder<M>> for WidgetNode<M> {
    fn from(b: KbdBuilder<M>) -> Self {
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
    fn kbd_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = kbd("Ctrl+S").build();
        let ctx = test_context();
        let layout = node.to_layout_with_context(NodeId::new(1), &ctx);
        let mut fonts = acme_text::FontSystem::new();
        let snapshot = LayoutEngine::new()
            .compute_with_text(&layout, (800.0, 600.0), &mut fonts, 1.0)
            .unwrap();
        let rect = snapshot.get(NodeId::new(1)).unwrap();
        assert!(rect.width > 0.0, "kbd width should be > 0");
        assert!(rect.height > 0.0, "kbd height should be > 0");
    }

    #[test]
    fn kbd_displays_label_text() {
        let node: WidgetNode<TestMsg> = kbd("Ctrl+S").build();
        let WidgetNode::Label(l) = &node else {
            panic!("expected Label variant");
        };
        assert_eq!(l.text, "Ctrl+S");
    }
}
