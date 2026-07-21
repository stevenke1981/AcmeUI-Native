//! TagInput component — an input box where users can type text and press Enter
//! to create tags/chips. Each tag can be removed via a close button.
//!
//! Common in email "To" fields and category inputs.
//!
//! Renders as a [`Card`] (Outlined variant) containing a [`Column`] with:
//! - A [`Row`] of tag badges (using the [`tag()`](crate::tag) builder style)
//! - A [`TextInput`] at the end for typing new tags

use acme_core::WidgetKey;
use acme_widgets::*;

/// Builder for a [`TagInput`] component.
pub struct TagInputBuilder<M> {
    pub id: WidgetKey,
    pub placeholder: String,
    pub tags: Vec<String>,
    pub max_tags: usize,
    pub on_add: Option<M>,
    pub on_remove: Option<M>,
}

/// Create a new [`TagInputBuilder`].
pub fn tag_input<M: Clone + 'static>(id: impl Into<WidgetKey>) -> TagInputBuilder<M> {
    TagInputBuilder {
        id: id.into(),
        placeholder: String::new(),
        tags: vec![],
        max_tags: 0,
        on_add: None,
        on_remove: None,
    }
}

impl<M: Clone + 'static> TagInputBuilder<M> {
    /// Set the placeholder text shown when no tags exist.
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Set the initial tags.
    pub fn tags<T: Into<String>>(mut self, tags: Vec<T>) -> Self {
        self.tags = tags.into_iter().map(|t| t.into()).collect();
        self
    }

    /// Set the maximum number of tags (0 = unlimited).
    pub fn max_tags(mut self, n: usize) -> Self {
        self.max_tags = n;
        self
    }

    /// Set the message dispatched when a tag is added (e.g. on Enter).
    pub fn on_add(mut self, message: M) -> Self {
        self.on_add = Some(message);
        self
    }

    /// Set the message dispatched when a tag is removed via the close button.
    pub fn on_remove(mut self, message: M) -> Self {
        self.on_remove = Some(message);
        self
    }
}

impl<M: Clone + 'static> From<TagInputBuilder<M>> for WidgetNode<M> {
    fn from(b: TagInputBuilder<M>) -> Self {
        // --- Tag row ---
        let mut tag_row = row::<M>().gap(4.0);

        if b.tags.is_empty() {
            // Show the placeholder label when no tags exist
            tag_row = tag_row.child(label::<M>(&b.placeholder));
        } else {
            for (i, tag_text) in b.tags.iter().enumerate() {
                let close_key = format!("{}_remove_{}", b.id.as_str(), i);
                let close_btn: WidgetNode<M> = if let Some(ref msg) = b.on_remove {
                    button(close_key, "✕").on_click(msg.clone())
                } else {
                    button(close_key, "✕").into()
                };

                let tag_badge = row::<M>()
                    .gap(2.0)
                    .child(label::<M>(tag_text))
                    .child(close_btn);
                tag_row = tag_row.child(tag_badge);
            }
        }

        // --- Text input ---
        let input_key = format!("{}_input", b.id.as_str());
        let input = text_input(input_key).placeholder("Type and press Enter...");
        let input: WidgetNode<M> = if let Some(ref msg) = b.on_add {
            input.on_change(msg.clone())
        } else {
            input.build()
        };

        // --- Assemble: Card (Outlined) → Column → [tag_row, input] ---
        card::<M>()
            .variant(CardVariant::Outlined)
            .padding(8.0)
            .gap(4.0)
            .child(column::<M>().gap(4.0).child(tag_row).child(input).build())
            .build()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {
        TagAdded,
        TagRemoved,
    }

    /// Verify that an empty TagInput renders the placeholder text.
    #[test]
    fn tag_input_empty() {
        let node: WidgetNode<TestMsg> = tag_input("t1").placeholder("Add tags...").into();

        // The outermost widget is a Card
        let WidgetNode::Card(card) = &node else {
            panic!("expected Card variant");
        };
        assert!(!card.children.is_empty());

        // The Card's first (and only) child is the Column
        let WidgetNode::Column(col) = &card.children[0] else {
            panic!("expected Column child inside Card");
        };

        // Column's first child is the tag Row
        let WidgetNode::Row(tag_row) = &col.children[0] else {
            panic!("expected Row as first Column child");
        };

        // When no tags, the tag row shows the placeholder as a Label
        let WidgetNode::Label(lbl) = &tag_row.children[0] else {
            panic!("expected Label with placeholder");
        };
        assert_eq!(lbl.text, "Add tags...");
    }

    /// Verify that initial tags produce tag badges in the tag row.
    #[test]
    fn tag_input_with_tags() {
        let node: WidgetNode<TestMsg> =
            tag_input("t2").tags(vec!["rust", "ui", "component"]).into();

        let WidgetNode::Card(card) = &node else {
            panic!("expected Card variant");
        };
        let WidgetNode::Column(col) = &card.children[0] else {
            panic!("expected Column child inside Card");
        };
        let WidgetNode::Row(tag_row) = &col.children[0] else {
            panic!("expected Row as first Column child");
        };

        // Each tag is a Row itself with label + close button = 3 tag rows
        assert_eq!(tag_row.children.len(), 3, "should have 3 tag badges");

        // Verify the first tag badge
        let WidgetNode::Row(first_tag) = &tag_row.children[0] else {
            panic!("expected Row per tag");
        };
        assert_eq!(
            first_tag.children.len(),
            2,
            "tag badge should have label + close button"
        );

        let WidgetNode::Label(lbl) = &first_tag.children[0] else {
            panic!("expected Label as first child of tag badge");
        };
        assert_eq!(lbl.text, "rust");
    }

    /// Verify that max_tags is stored correctly on the builder.
    #[test]
    fn tag_input_max_tags() {
        let b = tag_input::<TestMsg>("t3").tags(vec!["a", "b"]).max_tags(5);
        assert_eq!(b.max_tags, 5);
        assert_eq!(b.tags.len(), 2);
    }

    /// Verify that the From trait produces a Card variant.
    #[test]
    fn tag_input_from_trait() {
        let builder = tag_input::<TestMsg>("t4").placeholder("Enter tags...");
        let node: WidgetNode<TestMsg> = builder.into();
        assert!(
            matches!(node, WidgetNode::Card(_)),
            "TagInput should produce a Card node"
        );

        // Card wraps a Column with children
        let WidgetNode::Card(card) = &node else {
            panic!("expected Card");
        };
        assert_eq!(card.children.len(), 1, "Card should have one Column child");
        let WidgetNode::Column(col) = &card.children[0] else {
            panic!("expected Column child");
        };
        // Column should have at least a Row (tag area) and a TextInput
        assert!(col.children.len() >= 2, "should have at least 2 children");
    }

    /// Verify that on_add / on_remove messages are stored.
    #[test]
    fn tag_input_messages() {
        let b = tag_input::<TestMsg>("t5")
            .tags(vec!["hello"])
            .on_add(TestMsg::TagAdded)
            .on_remove(TestMsg::TagRemoved);
        assert!(b.on_add.is_some());
        assert!(b.on_remove.is_some());
    }
}
