//! Badge component — a compact Card containing a Label with tone-based coloring.

use crate::WidgetNode;

/// Builder for a badge widget.
pub struct BadgeBuilder<M> {
    pub label: String,
    pub tone: crate::Tone,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a badge builder.
pub fn badge<M>(label: impl Into<String>) -> BadgeBuilder<M> {
    BadgeBuilder {
        label: label.into(),
        tone: crate::Tone::Neutral,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> BadgeBuilder<M> {
    /// Set the tone to Primary.
    pub fn primary(mut self) -> Self {
        self.tone = crate::Tone::Primary;
        self
    }

    /// Set the tone to Success.
    pub fn success(mut self) -> Self {
        self.tone = crate::Tone::Success;
        self
    }

    /// Set the tone to Warning.
    pub fn warning(mut self) -> Self {
        self.tone = crate::Tone::Warning;
        self
    }

    /// Set the tone to Danger.
    pub fn danger(mut self) -> Self {
        self.tone = crate::Tone::Danger;
        self
    }

    /// Set the tone explicitly.
    pub fn tone(mut self, tone: crate::Tone) -> Self {
        self.tone = tone;
        self
    }

    /// Build the badge widget.
    pub fn build(self) -> WidgetNode<M> {
        crate::card()
            .child(crate::label(&self.label))
            .padding(4.0)
            .gap(0.0)
            .build()
    }
}

impl<M: Clone + 'static> From<BadgeBuilder<M>> for WidgetNode<M> {
    fn from(b: BadgeBuilder<M>) -> Self {
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
    fn badge_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = badge("New").primary().build();
        let ctx = test_context();
        let layout = node.to_layout_with_context(NodeId::new(1), &ctx);
        let snapshot = LayoutEngine::new()
            .compute(&layout, (800.0, 600.0))
            .unwrap();
        let rect = snapshot.get(NodeId::new(1)).unwrap();
        assert!(rect.width > 0.0, "badge width should be > 0");
        assert!(rect.height > 0.0, "badge height should be > 0");
    }

    #[test]
    fn badge_displays_label_text() {
        let node: WidgetNode<TestMsg> = badge("New").primary().build();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        assert_eq!(c.children.len(), 1);
        let WidgetNode::Label(l) = &c.children[0] else {
            panic!("expected Label child");
        };
        assert_eq!(l.text, "New");
    }
}
