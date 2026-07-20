//! Tag component — a compact Row with Label and optional close Button.

use crate::WidgetNode;

/// Builder for a tag widget.
pub struct TagBuilder<M> {
    pub label: String,
    pub removable: bool,
    pub size: crate::ControlSize,
    pub tone: crate::Tone,
    pub color: Option<acme_theme::ThemeColor>,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a tag builder.
pub fn tag<M>(label: impl Into<String>) -> TagBuilder<M> {
    TagBuilder {
        label: label.into(),
        removable: false,
        size: crate::ControlSize::Md,
        tone: crate::Tone::Neutral,
        color: None,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> TagBuilder<M> {
    /// Make the tag removable (shows a close button).
    pub fn removable(mut self) -> Self {
        self.removable = true;
        self
    }

    /// Set size to Small.
    pub fn small(mut self) -> Self {
        self.size = crate::ControlSize::Sm;
        self
    }

    /// Set size to Medium.
    pub fn medium(mut self) -> Self {
        self.size = crate::ControlSize::Md;
        self
    }

    /// Set size to Large.
    pub fn large(mut self) -> Self {
        self.size = crate::ControlSize::Lg;
        self
    }

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

    /// Set the tag color (overrides tone-based color).
    pub fn color(mut self, color: acme_theme::ThemeColor) -> Self {
        self.color = Some(color);
        self
    }

    /// Build the tag widget.
    pub fn build(self) -> WidgetNode<M> {
        let gap = match self.size {
            crate::ControlSize::Xs | crate::ControlSize::Sm => 2.0,
            crate::ControlSize::Md => 4.0,
            crate::ControlSize::Lg => 6.0,
            crate::ControlSize::Xl => 8.0,
        };
        let mut r = crate::row().gap(gap).child(crate::label(&self.label));
        if self.removable {
            r = r.child(crate::button("tag-close", "✕"));
        }
        r.build()
    }
}

impl<M: Clone + 'static> From<TagBuilder<M>> for WidgetNode<M> {
    fn from(b: TagBuilder<M>) -> Self {
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
    fn tag_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = tag("rust").build();
        let ctx = test_context();
        let layout = node.to_layout_with_context(NodeId::new(1), &ctx);
        let mut fonts = acme_text::FontSystem::new();
        let snapshot = LayoutEngine::new()
            .compute_with_text(&layout, (800.0, 600.0), &mut fonts, 1.0)
            .unwrap();
        let rect = snapshot.get(NodeId::new(1)).unwrap();
        assert!(rect.width > 0.0, "tag width should be > 0");
        assert!(rect.height > 0.0, "tag height should be > 0");
    }

    #[test]
    fn tag_displays_label_text() {
        let node: WidgetNode<TestMsg> = tag("rust").build();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row variant");
        };
        assert_eq!(r.children.len(), 1);
        let WidgetNode::Label(l) = &r.children[0] else {
            panic!("expected Label child");
        };
        assert_eq!(l.text, "rust");
    }

    #[test]
    fn tag_removable_has_close_button() {
        let node: WidgetNode<TestMsg> = tag("rust").removable().build();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row variant");
        };
        assert_eq!(r.children.len(), 2);
        let WidgetNode::Button(_) = &r.children[1] else {
            panic!("expected Button as second child");
        };
    }
}
