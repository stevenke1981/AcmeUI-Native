//! MultiSelect component.
//!
//! Renders as a Column with a Row of selected tags (each a small label with "✕")
//! and, when open, a Card dropdown with checkable option labels.

use acme_core::WidgetKey;
use acme_widgets::*;

/// A single multi-select option.
#[derive(Clone, Debug)]
pub struct MultiSelectOption {
    pub label: String,
    pub value: String,
}

/// Create a new multi-select option.
pub fn multi_select_option(
    label: impl Into<String>,
    value: impl Into<String>,
) -> MultiSelectOption {
    MultiSelectOption {
        label: label.into(),
        value: value.into(),
    }
}

/// Builder for a MultiSelect component.
pub struct MultiSelectBuilder<M> {
    pub id: WidgetKey,
    pub placeholder: String,
    pub options: Vec<MultiSelectOption>,
    pub selected_values: Vec<String>,
    pub open: bool,
    pub disabled: bool,
    pub on_change: Option<M>,
}

/// Create a new MultiSelect builder.
pub fn multi_select<M: Clone + 'static>(id: impl Into<WidgetKey>) -> MultiSelectBuilder<M> {
    MultiSelectBuilder {
        id: id.into(),
        placeholder: String::new(),
        options: vec![],
        selected_values: vec![],
        open: false,
        disabled: false,
        on_change: None,
    }
}

impl<M: Clone + 'static> MultiSelectBuilder<M> {
    /// Set the placeholder text shown when no items are selected.
    pub fn placeholder(mut self, value: impl Into<String>) -> Self {
        self.placeholder = value.into();
        self
    }

    /// Add an option to the dropdown list.
    pub fn option(mut self, opt: MultiSelectOption) -> Self {
        self.options.push(opt);
        self
    }

    /// Set the list of currently selected values.
    pub fn selected_values(mut self, values: Vec<String>) -> Self {
        self.selected_values = values;
        self
    }

    /// Set whether the dropdown is open.
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }

    /// Set whether the multi-select is disabled.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    /// Set the message dispatched when the selection changes.
    pub fn on_change(mut self, msg: M) -> Self {
        self.on_change = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<MultiSelectBuilder<M>> for WidgetNode<M> {
    fn from(b: MultiSelectBuilder<M>) -> Self {
        // Build the tag row: selected items as small labels with "✕"
        let mut tag_row = row::<M>().gap(4.0);

        // Find selected options to build tags
        let selected_labels: Vec<&str> = b
            .options
            .iter()
            .filter(|opt| b.selected_values.contains(&opt.value))
            .map(|opt| opt.label.as_str())
            .collect();

        if selected_labels.is_empty() {
            tag_row = tag_row.child(label::<M>(&b.placeholder));
        } else {
            for label_text in &selected_labels {
                let tag = row::<M>()
                    .gap(2.0)
                    .child(label::<M>(*label_text))
                    .child(label::<M>("✕"));
                tag_row = tag_row.child(tag);
            }
        }

        // Build the dropdown with all options
        if b.open {
            let mut dropdown = card::<M>().variant(CardVariant::Outlined);
            for opt in &b.options {
                let checked = if b.selected_values.contains(&opt.value) {
                    "✓ "
                } else {
                    "  "
                };
                dropdown = dropdown.child(label::<M>(format!("{}{}", checked, opt.label)));
            }

            column::<M>()
                .key(b.id)
                .gap(2.0)
                .child(tag_row)
                .child(dropdown)
                .build()
        } else {
            column::<M>()
                .key(b.id)
                .gap(2.0)
                .child(tag_row)
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
    fn multi_select_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = multi_select("ms1")
            .placeholder("Select items...")
            .option(multi_select_option("A", "a"))
            .option(multi_select_option("B", "b"))
            .selected_values(vec!["a".to_string()])
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
    fn multi_select_builder_defaults() {
        let m = multi_select::<TestMsg>("m");
        assert!(m.placeholder.is_empty());
        assert!(m.options.is_empty());
        assert!(m.selected_values.is_empty());
        assert!(!m.open);
        assert!(!m.disabled);
        assert!(m.on_change.is_none());
    }

    #[test]
    fn multi_select_shows_placeholder_when_empty() {
        let node: WidgetNode<TestMsg> = multi_select("ms")
            .placeholder("Pick something")
            .option(multi_select_option("A", "a"))
            .into();
        let WidgetNode::Column(container) = &node else {
            panic!("expected Column");
        };
        // First child = tag Row
        let WidgetNode::Row(tag_row) = &container.children[0] else {
            panic!("expected Row");
        };
        // When no selection, shows placeholder label
        let WidgetNode::Label(lbl) = &tag_row.children[0] else {
            panic!("expected Label");
        };
        assert_eq!(lbl.text, "Pick something");
    }

    #[test]
    fn multi_select_shows_selected_tags() {
        let node: WidgetNode<TestMsg> = multi_select("ms")
            .option(multi_select_option("Apple", "apple"))
            .option(multi_select_option("Banana", "banana"))
            .selected_values(vec!["apple".to_string()])
            .into();
        let WidgetNode::Column(container) = &node else {
            panic!("expected Column");
        };
        let WidgetNode::Row(tag_row) = &container.children[0] else {
            panic!("expected Row");
        };
        // The tag row has one tag (each tag is a Row of label + "✕")
        assert_eq!(tag_row.children.len(), 1);
    }

    #[test]
    fn multi_select_open_shows_dropdown() {
        let node: WidgetNode<TestMsg> = multi_select("ms")
            .option(multi_select_option("Red", "red"))
            .open(true)
            .into();
        let WidgetNode::Column(container) = &node else {
            panic!("expected Column");
        };
        assert_eq!(container.children.len(), 2);
        let WidgetNode::Card(_) = &container.children[1] else {
            panic!("expected Card dropdown");
        };
    }
}
