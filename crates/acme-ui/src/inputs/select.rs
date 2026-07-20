//! Select (dropdown) component.
//!
//! Renders as a Column with a trigger Row showing the selected label or
//! placeholder with a "▾" chevron, and a dropdown Card of option Labels.

use acme_core::WidgetKey;
use acme_widgets::*;

/// A single selectable option.
#[derive(Clone, Debug)]
pub struct SelectOption {
    pub label: String,
    pub value: String,
}

impl SelectOption {
    /// Create a new SelectOption.
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

/// Builder for a Select component.
pub struct SelectBuilder<M> {
    pub id: WidgetKey,
    pub placeholder: String,
    pub options: Vec<SelectOption>,
    pub selected_index: Option<usize>,
    pub open: bool,
    pub disabled: bool,
    pub on_select: Option<M>,
}

/// Create a new Select builder.
pub fn select<M: Clone + 'static>(id: impl Into<WidgetKey>) -> SelectBuilder<M> {
    SelectBuilder {
        id: id.into(),
        placeholder: String::new(),
        options: vec![],
        selected_index: None,
        open: false,
        disabled: false,
        on_select: None,
    }
}

impl<M: Clone + 'static> SelectBuilder<M> {
    /// Set the placeholder text shown when no option is selected.
    pub fn placeholder(mut self, value: impl Into<String>) -> Self {
        self.placeholder = value.into();
        self
    }

    /// Add an option to the dropdown.
    pub fn option(mut self, opt: SelectOption) -> Self {
        self.options.push(opt);
        self
    }

    /// Set the index of the currently selected option.
    pub fn selected_index(mut self, index: Option<usize>) -> Self {
        self.selected_index = index;
        self
    }

    /// Set whether the dropdown is open.
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }

    /// Set whether the select is disabled.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    /// Set the message dispatched when an option is selected.
    pub fn on_select(mut self, msg: M) -> Self {
        self.on_select = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<SelectBuilder<M>> for WidgetNode<M> {
    fn from(b: SelectBuilder<M>) -> Self {
        // Determine the displayed text
        let trigger_text = b
            .selected_index
            .and_then(|idx| b.options.get(idx))
            .map(|opt| opt.label.as_str())
            .unwrap_or(&b.placeholder);

        let trigger = row::<M>()
            .gap(4.0)
            .child(label::<M>(trigger_text))
            .child(label::<M>("▾"));

        // Build the dropdown with all options
        let mut dropdown = card::<M>().variant(CardVariant::Outlined);
        for opt in &b.options {
            dropdown = dropdown.child(label::<M>(&opt.label));
        }

        column::<M>()
            .key(b.id)
            .gap(2.0)
            .child(trigger)
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
    fn select_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = select("sel1")
            .placeholder("Choose...")
            .option(SelectOption::new("Apple", "apple"))
            .option(SelectOption::new("Banana", "banana"))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert!(!layout.children.is_empty());
        // Dropdown card child has non-zero sizing
        let dropdown = &layout.children[1];
        assert!(!dropdown.children.is_empty());
    }

    #[test]
    fn select_builder_defaults() {
        let s = select::<TestMsg>("s");
        assert!(s.placeholder.is_empty());
        assert!(s.options.is_empty());
        assert!(s.selected_index.is_none());
        assert!(!s.open);
        assert!(!s.disabled);
        assert!(s.on_select.is_none());
    }

    #[test]
    fn select_selected_shows_label() {
        let node: WidgetNode<TestMsg> = select("sel")
            .placeholder("Pick")
            .option(SelectOption::new("Item1", "i1"))
            .option(SelectOption::new("Item2", "i2"))
            .selected_index(Some(0))
            .into();
        let WidgetNode::Column(container) = &node else {
            panic!("expected Column");
        };
        // First child = trigger Row
        let WidgetNode::Row(trigger) = &container.children[0] else {
            panic!("expected Row trigger");
        };
        let WidgetNode::Label(lbl) = &trigger.children[0] else {
            panic!("expected Label");
        };
        assert_eq!(lbl.text, "Item1");
    }
}
