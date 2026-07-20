//! Textarea (multi-line text) component.
//!
//! Renders as a Card containing a Label with the text content.
//! The parent app manages the actual text content and updates.

use acme_core::WidgetKey;
use acme_widgets::*;

/// Builder for a Textarea component.
pub struct TextareaBuilder<M> {
    pub id: WidgetKey,
    pub placeholder: String,
    pub rows: usize,
    pub value: String,
    pub disabled: bool,
    pub on_input: Option<M>,
}

/// Create a new Textarea builder.
pub fn textarea<M: Clone + 'static>(id: impl Into<WidgetKey>) -> TextareaBuilder<M> {
    TextareaBuilder {
        id: id.into(),
        placeholder: String::new(),
        rows: 3,
        value: String::new(),
        disabled: false,
        on_input: None,
    }
}

impl<M: Clone + 'static> TextareaBuilder<M> {
    /// Set the placeholder text shown when empty.
    pub fn placeholder(mut self, value: impl Into<String>) -> Self {
        self.placeholder = value.into();
        self
    }

    /// Set the number of visible rows (default 3).
    pub fn rows(mut self, value: usize) -> Self {
        self.rows = value;
        self
    }

    /// Set the current text value.
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }

    /// Set whether the textarea is disabled.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    /// Set the message dispatched when the text changes.
    pub fn on_input(mut self, msg: M) -> Self {
        self.on_input = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<TextareaBuilder<M>> for WidgetNode<M> {
    fn from(b: TextareaBuilder<M>) -> Self {
        let display_text = if b.value.is_empty() {
            b.placeholder.as_str()
        } else {
            b.value.as_str()
        };

        card::<M>()
            .key(b.id)
            .variant(CardVariant::Outlined)
            .padding(8.0)
            .child(label::<M>(display_text))
            .build()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;
    use acme_layout::{LayoutKind, Length};

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn textarea_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = textarea("ta1").value("Hello\nWorld").rows(4).into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert!(!layout.children.is_empty());
        // Inner label child has non-zero sizing
        let label_leaf = &layout.children[0];
        assert!(label_leaf.children.is_empty());
        assert_ne!(label_leaf.style.min_height, Length::px(0.0));
    }

    #[test]
    fn textarea_builder_defaults() {
        let t = textarea::<TestMsg>("t");
        assert!(t.value.is_empty());
        assert_eq!(t.rows, 3);
        assert!(!t.disabled);
        assert!(t.on_input.is_none());
    }

    #[test]
    fn textarea_shows_placeholder_when_empty() {
        let node: WidgetNode<TestMsg> = textarea("ta").placeholder("Enter text...").into();
        let WidgetNode::Card(card) = &node else {
            panic!("expected Card");
        };
        let WidgetNode::Label(lbl) = &card.children[0] else {
            panic!("expected Label");
        };
        assert_eq!(lbl.text, "Enter text...");
    }

    #[test]
    fn textarea_shows_value_when_not_empty() {
        let node: WidgetNode<TestMsg> = textarea("ta").value("Actual content").into();
        let WidgetNode::Card(card) = &node else {
            panic!("expected Card");
        };
        let WidgetNode::Label(lbl) = &card.children[0] else {
            panic!("expected Label");
        };
        assert_eq!(lbl.text, "Actual content");
    }
}
