//! VisuallyHidden — content hidden visually but available to screen readers.
//! Aligns with Radix UI Visually Hidden primitive.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for visually hidden content.
pub struct VisuallyHiddenBuilder<M> {
    pub id: WidgetKey,
    pub text: String,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a visually hidden builder.
pub fn visually_hidden<M: Clone + 'static>(text: impl Into<String>) -> VisuallyHiddenBuilder<M> {
    VisuallyHiddenBuilder {
        id: WidgetKey::from("visually_hidden"),
        text: text.into(),
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> VisuallyHiddenBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }
}

impl<M: Clone + 'static> From<VisuallyHiddenBuilder<M>> for WidgetNode<M> {
    fn from(b: VisuallyHiddenBuilder<M>) -> Self {
        // Rendered with zero size but retains text for accessibility tree.
        let mut node = crate::label(b.text);
        if let WidgetNode::Label(ref mut l) = node {
            l.font_size = Some(0.0);
        }
        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn visually_hidden_produces_label() {
        let node: WidgetNode<Msg> = visually_hidden("Close menu").into();
        assert!(matches!(node, WidgetNode::Label(_)));
    }

    #[test]
    fn visually_hidden_retains_text() {
        let node: WidgetNode<Msg> = visually_hidden("Skip to content").into();
        let WidgetNode::Label(l) = &node else {
            panic!("expected Label");
        };
        assert_eq!(l.text, "Skip to content");
    }

    #[test]
    fn visually_hidden_zero_font_size() {
        let node: WidgetNode<Msg> = visually_hidden("hidden").into();
        let WidgetNode::Label(l) = &node else {
            panic!("expected Label");
        };
        assert_eq!(l.font_size, Some(0.0));
    }
}
