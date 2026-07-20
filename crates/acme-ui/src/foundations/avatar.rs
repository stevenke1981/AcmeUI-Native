//! Avatar component — extracts initials and renders as a circular background + Label.

use crate::WidgetNode;

/// Builder for an avatar widget.
pub struct AvatarBuilder<M> {
    pub name: String,
    pub size: f32,
    _phantom: std::marker::PhantomData<M>,
}

/// Create an avatar builder.
pub fn avatar<M>(name: impl Into<String>) -> AvatarBuilder<M> {
    AvatarBuilder {
        name: name.into(),
        size: 32.0,
        _phantom: std::marker::PhantomData,
    }
}

/// Extract initials from a name (first char of each word, max 2).
fn extract_initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect()
}

impl<M: Clone + 'static> AvatarBuilder<M> {
    /// Set the avatar size in pixels.
    pub fn size(mut self, px: f32) -> Self {
        self.size = px;
        self
    }

    /// Build the avatar widget.
    pub fn build(self) -> WidgetNode<M> {
        let initials = extract_initials(&self.name);
        let sz = self.size;
        crate::stack()
            .child(
                crate::card()
                    .padding(0.0)
                    .variant(crate::CardVariant::Plain)
                    .build(),
            )
            .child(crate::label(initials))
            .size(sz, sz)
            .build()
    }
}

impl<M: Clone + 'static> From<AvatarBuilder<M>> for WidgetNode<M> {
    fn from(b: AvatarBuilder<M>) -> Self {
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
    fn avatar_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = avatar("John Doe").size(40.0).build();
        let ctx = test_context();
        let layout = node.to_layout_with_context(NodeId::new(1), &ctx);
        let mut fonts = acme_text::FontSystem::new();
        let snapshot = LayoutEngine::new()
            .compute_with_text(&layout, (800.0, 600.0), &mut fonts, 1.0)
            .unwrap();
        let rect = snapshot.get(NodeId::new(1)).unwrap();
        assert!(rect.width > 0.0, "avatar width should be > 0");
        assert!(rect.height > 0.0, "avatar height should be > 0");
    }

    #[test]
    fn avatar_displays_label_text() {
        let node: WidgetNode<TestMsg> = avatar("John Doe").build();
        // Avatar is a Stack containing a Card and a Label
        let WidgetNode::Stack(s) = &node else {
            panic!("expected Stack variant");
        };
        assert_eq!(s.children.len(), 2);
        // Second child should be the initials label
        let WidgetNode::Label(l) = &s.children[1] else {
            panic!("expected Label as second child");
        };
        assert_eq!(l.text, "JD");
    }

    #[test]
    fn avatar_single_word_initials() {
        let node: WidgetNode<TestMsg> = avatar("Alice").build();
        let WidgetNode::Stack(s) = &node else {
            panic!("expected Stack variant");
        };
        let WidgetNode::Label(l) = &s.children[1] else {
            panic!("expected Label as second child");
        };
        assert_eq!(l.text, "A");
    }
}
