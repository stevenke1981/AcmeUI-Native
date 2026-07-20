//! Autocomplete input component.
//!
//! Renders as a Column with a TextInput showing the current value and,
//! when open, a Card dropdown of filterable option labels.

use acme_core::WidgetKey;
use acme_widgets::*;

/// A single autocomplete option.
#[derive(Clone, Debug)]
pub struct AutoCompleteOption {
    pub label: String,
    pub value: String,
}

/// Create a new autocomplete option.
pub fn autocomplete_option(
    label: impl Into<String>,
    value: impl Into<String>,
) -> AutoCompleteOption {
    AutoCompleteOption {
        label: label.into(),
        value: value.into(),
    }
}

/// Builder for an Autocomplete component.
pub struct AutoCompleteBuilder<M> {
    pub id: WidgetKey,
    pub placeholder: String,
    pub options: Vec<AutoCompleteOption>,
    pub value: String,
    pub open: bool,
    pub disabled: bool,
    pub on_select: Option<M>,
    pub on_change: Option<M>,
}

/// Create a new Autocomplete builder.
pub fn autocomplete<M: Clone + 'static>(id: impl Into<WidgetKey>) -> AutoCompleteBuilder<M> {
    AutoCompleteBuilder {
        id: id.into(),
        placeholder: String::new(),
        options: vec![],
        value: String::new(),
        open: false,
        disabled: false,
        on_select: None,
        on_change: None,
    }
}

impl<M: Clone + 'static> AutoCompleteBuilder<M> {
    /// Set the placeholder text shown when the input is empty.
    pub fn placeholder(mut self, value: impl Into<String>) -> Self {
        self.placeholder = value.into();
        self
    }

    /// Add an option to the dropdown list.
    pub fn option(mut self, opt: AutoCompleteOption) -> Self {
        self.options.push(opt);
        self
    }

    /// Set the current text value.
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }

    /// Set whether the dropdown is open.
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }

    /// Set whether the input is disabled.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    /// Set the message dispatched when an option is selected.
    pub fn on_select(mut self, msg: M) -> Self {
        self.on_select = Some(msg);
        self
    }

    /// Set the message dispatched when the text changes.
    pub fn on_change(mut self, msg: M) -> Self {
        self.on_change = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<AutoCompleteBuilder<M>> for WidgetNode<M> {
    fn from(b: AutoCompleteBuilder<M>) -> Self {
        let input_key = format!("{}_input", b.id.as_str());

        // Text input showing the current value
        let input = text_input::<M>(input_key.as_str())
            .value(b.value.as_str())
            .placeholder(b.placeholder.as_str())
            .disabled(b.disabled);

        // Build the dropdown card if open
        if b.open {
            let mut dropdown = card::<M>().variant(CardVariant::Outlined);
            for opt in &b.options {
                dropdown = dropdown.child(label::<M>(&opt.label));
            }

            column::<M>()
                .key(b.id)
                .gap(2.0)
                .child(input)
                .child(dropdown)
                .build()
        } else {
            column::<M>()
                .key(b.id)
                .gap(2.0)
                .child(input)
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
    fn autocomplete_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = autocomplete("ac1")
            .placeholder("Search...")
            .option(autocomplete_option("Apple", "apple"))
            .option(autocomplete_option("Banana", "banana"))
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
    fn autocomplete_builder_defaults() {
        let a = autocomplete::<TestMsg>("a");
        assert!(a.placeholder.is_empty());
        assert!(a.options.is_empty());
        assert!(a.value.is_empty());
        assert!(!a.open);
        assert!(!a.disabled);
        assert!(a.on_select.is_none());
        assert!(a.on_change.is_none());
    }

    #[test]
    fn autocomplete_shows_text_input() {
        let node: WidgetNode<TestMsg> = autocomplete("ac").value("hello").into();
        let WidgetNode::Column(container) = &node else {
            panic!("expected Column");
        };
        // First child = TextInput
        let WidgetNode::TextInput(input) = &container.children[0] else {
            panic!("expected TextInput");
        };
        assert_eq!(input.value, "hello");
    }

    #[test]
    fn autocomplete_open_shows_dropdown() {
        let node: WidgetNode<TestMsg> = autocomplete("ac")
            .option(autocomplete_option("Red", "red"))
            .option(autocomplete_option("Blue", "blue"))
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
    fn autocomplete_closed_hides_dropdown() {
        let node: WidgetNode<TestMsg> = autocomplete("ac")
            .option(autocomplete_option("Red", "red"))
            .option(autocomplete_option("Blue", "blue"))
            .open(false)
            .into();
        let WidgetNode::Column(container) = &node else {
            panic!("expected Column");
        };
        // Only one child (input) when closed
        assert_eq!(container.children.len(), 1);
    }
}
