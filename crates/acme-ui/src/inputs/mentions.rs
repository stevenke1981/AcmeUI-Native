//! Mentions input component.
//!
//! Renders as a Column with a Row containing an "@" badge label and a TextInput,
//! and when open, a dropdown Card of filterable option labels.

use crate::inputs::select::SelectOption;
use acme_core::WidgetKey;
use acme_widgets::*;

/// Builder for a Mentions component.
pub struct MentionsBuilder<M> {
    pub id: WidgetKey,
    pub value: String,
    pub placeholder: String,
    pub options: Vec<SelectOption>,
    pub open: bool,
    pub on_select: Option<M>,
}

/// Create a new Mentions builder.
pub fn mentions<M: Clone + 'static>(id: impl Into<WidgetKey>) -> MentionsBuilder<M> {
    MentionsBuilder {
        id: id.into(),
        value: String::new(),
        placeholder: String::new(),
        options: vec![],
        open: false,
        on_select: None,
    }
}

impl<M: Clone + 'static> MentionsBuilder<M> {
    /// Set the current text value.
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }

    /// Set the placeholder text shown when empty.
    pub fn placeholder(mut self, v: impl Into<String>) -> Self {
        self.placeholder = v.into();
        self
    }

    /// Add an option to the dropdown list.
    pub fn option(mut self, opt: SelectOption) -> Self {
        self.options.push(opt);
        self
    }

    /// Set whether the dropdown is open.
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }

    /// Set the message dispatched when an option is selected.
    pub fn on_select(mut self, msg: M) -> Self {
        self.on_select = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<MentionsBuilder<M>> for WidgetNode<M> {
    fn from(b: MentionsBuilder<M>) -> Self {
        let input_key = format!("{}_input", b.id.as_str());

        // Row: "@" badge + text input
        let input_row = row::<M>()
            .gap(4.0)
            .child(label::<M>("@"))
            .child(
                text_input::<M>(input_key.as_str())
                    .value(b.value.as_str())
                    .placeholder(b.placeholder.as_str()),
            );

        // Dropdown list when open
        if b.open {
            let mut dropdown = card::<M>().variant(CardVariant::Outlined);
            for opt in &b.options {
                dropdown = dropdown.child(label::<M>(&opt.label));
            }

            column::<M>()
                .key(b.id)
                .gap(2.0)
                .child(input_row)
                .child(dropdown)
                .build()
        } else {
            column::<M>()
                .key(b.id)
                .gap(2.0)
                .child(input_row)
                .build()
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;
    use acme_layout::LayoutKind;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn mentions_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = mentions("m1")
            .placeholder("Type @ to mention...")
            .option(SelectOption::new("Alice", "alice"))
            .option(SelectOption::new("Bob", "bob"))
            .option(SelectOption::new("Charlie", "charlie"))
            .open(true)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert!(!layout.children.is_empty());
        // Second child = dropdown with options
        let dropdown = &layout.children[1];
        assert!(!dropdown.children.is_empty());
    }

    #[test]
    fn mentions_builder_defaults() {
        let m = mentions::<TestMsg>("m");
        assert!(m.value.is_empty());
        assert!(m.placeholder.is_empty());
        assert!(m.options.is_empty());
        assert!(!m.open);
        assert!(m.on_select.is_none());
    }

    #[test]
    fn mentions_shows_mention_badge_and_input() {
        let node: WidgetNode<TestMsg> = mentions("m").value("hello").into();
        let WidgetNode::Column(container) = &node else {
            panic!("expected Column");
        };
        // First child = input Row
        let WidgetNode::Row(input_row) = &container.children[0] else {
            panic!("expected Row");
        };
        // First child of row = "@" badge label
        let WidgetNode::Label(badge) = &input_row.children[0] else {
            panic!("expected Label badge");
        };
        assert_eq!(badge.text, "@");
        // Second child of row = TextInput
        let WidgetNode::TextInput(input) = &input_row.children[1] else {
            panic!("expected TextInput");
        };
        assert_eq!(input.value, "hello");
    }

    #[test]
    fn mentions_open_shows_dropdown() {
        let node: WidgetNode<TestMsg> = mentions("m")
            .option(SelectOption::new("Red", "red"))
            .option(SelectOption::new("Blue", "blue"))
            .open(true)
            .into();
        let WidgetNode::Column(container) = &node else {
            panic!("expected Column");
        };
        assert_eq!(container.children.len(), 2);
        // Second child = Card dropdown
        let WidgetNode::Card(_) = &container.children[1] else {
            panic!("expected Card dropdown");
        };
    }

    #[test]
    fn mentions_closed_hides_dropdown() {
        let node: WidgetNode<TestMsg> = mentions("m")
            .option(SelectOption::new("Red", "red"))
            .option(SelectOption::new("Blue", "blue"))
            .open(false)
            .into();
        let WidgetNode::Column(container) = &node else {
            panic!("expected Column");
        };
        // Only one child (input row) when closed
        assert_eq!(container.children.len(), 1);
    }

    #[test]
    fn mentions_with_multiple_options() {
        let m = mentions::<TestMsg>("m")
            .option(SelectOption::new("Alice", "alice"))
            .option(SelectOption::new("Bob", "bob"))
            .option(SelectOption::new("Charlie", "charlie"));
        assert_eq!(m.options.len(), 3);
        assert_eq!(m.options[0].label, "Alice");
        assert_eq!(m.options[1].label, "Bob");
        assert_eq!(m.options[2].label, "Charlie");
    }
}
