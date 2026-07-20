//! Combobox input component.
//!
//! Renders as a Column with a Row containing a TextInput and "▾" button,
//! and a dropdown Card of option Labels.

use crate::inputs::select::SelectOption;
use acme_core::WidgetKey;
use acme_widgets::*;

/// Builder for a Combobox component.
pub struct ComboboxBuilder<M> {
    pub id: WidgetKey,
    pub value: String,
    pub placeholder: String,
    pub options: Vec<SelectOption>,
    pub open: bool,
    pub on_select: Option<M>,
}

/// Create a new Combobox builder.
pub fn combobox<M: Clone + 'static>(id: impl Into<WidgetKey>) -> ComboboxBuilder<M> {
    ComboboxBuilder {
        id: id.into(),
        value: String::new(),
        placeholder: String::new(),
        options: vec![],
        open: false,
        on_select: None,
    }
}

impl<M: Clone + 'static> ComboboxBuilder<M> {
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

impl<M: Clone + 'static> From<ComboboxBuilder<M>> for WidgetNode<M> {
    fn from(b: ComboboxBuilder<M>) -> Self {
        // Input row: TextInput + dropdown arrow
        let input_key = format!("{}_input", b.id.as_str());
        let input_row = row::<M>()
            .gap(4.0)
            .child(
                text_input::<M>(input_key.as_str())
                    .value(b.value.as_str())
                    .placeholder(b.placeholder.as_str()),
            )
            .child(label::<M>("▾"));

        // Dropdown list
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
    fn combobox_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = combobox("cb1")
            .placeholder("Type or select...")
            .option(SelectOption::new("Red", "red"))
            .option(SelectOption::new("Blue", "blue"))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert!(!layout.children.is_empty());
        // Second child = dropdown with options
        let dropdown = &layout.children[1];
        assert!(!dropdown.children.is_empty());
    }

    #[test]
    fn combobox_builder_defaults() {
        let c = combobox::<TestMsg>("c");
        assert!(c.value.is_empty());
        assert!(c.placeholder.is_empty());
        assert!(c.options.is_empty());
        assert!(!c.open);
        assert!(c.on_select.is_none());
    }

    #[test]
    fn combobox_shows_text_input() {
        let node: WidgetNode<TestMsg> = combobox("cb").value("hello").into();
        let WidgetNode::Column(container) = &node else {
            panic!("expected Column");
        };
        // First child = input Row
        let WidgetNode::Row(input_row) = &container.children[0] else {
            panic!("expected Row");
        };
        // First child of row = TextInput
        assert!(matches!(&input_row.children[0], WidgetNode::TextInput(_)));
    }
}
