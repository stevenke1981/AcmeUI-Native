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
    pub on_select_option: Option<fn(String) -> M>,
    pub on_toggle: Option<M>,
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
        on_select_option: None,
        on_toggle: None,
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

    /// Set a callback producing a message from the chosen option's value when clicked.
    pub fn on_select_option(mut self, f: fn(String) -> M) -> Self {
        self.on_select_option = Some(f);
        self
    }

    /// Set the message dispatched when the open/close toggle button is clicked.
    pub fn on_toggle(mut self, msg: M) -> Self {
        self.on_toggle = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<ComboboxBuilder<M>> for WidgetNode<M> {
    fn from(b: ComboboxBuilder<M>) -> Self {
        let id_str = b.id.as_str().to_string();
        let input_key = format!("{id_str}_input");
        let toggle_key = format!("{id_str}_toggle");
        let toggle_glyph = if b.open { "▴" } else { "▾" };
        let toggle_btn =
            button::<M>(toggle_key.as_str(), toggle_glyph).variant(ButtonVariant::Ghost);
        let toggle_node: WidgetNode<M> = match b.on_toggle {
            Some(msg) => toggle_btn.on_click(msg),
            None => toggle_btn.into(),
        };

        let input_row = row::<M>()
            .gap(4.0)
            .child(
                text_input::<M>(input_key.as_str())
                    .value(b.value.as_str())
                    .placeholder(b.placeholder.as_str()),
            )
            .child(toggle_node);

        let query = b.value.trim().to_ascii_lowercase();
        let filtered: Vec<&SelectOption> = b
            .options
            .iter()
            .filter(|opt| {
                query.is_empty()
                    || opt.label.to_ascii_lowercase().contains(&query)
                    || opt.value.to_ascii_lowercase().contains(&query)
            })
            .collect();

        let mut dropdown = card::<M>().variant(CardVariant::Outlined);
        if filtered.is_empty() && !b.options.is_empty() {
            dropdown = dropdown.child(label::<M>("No matching options"));
        } else {
            for (idx, opt) in filtered.iter().enumerate() {
                let opt_btn = button::<M>(format!("{id_str}_opt_{idx}").as_str(), &opt.label)
                    .variant(ButtonVariant::Ghost);
                let opt_node: WidgetNode<M> = if let Some(cb) = b.on_select_option {
                    opt_btn.on_click(cb(opt.value.clone()))
                } else if let Some(ref msg) = b.on_select {
                    opt_btn.on_click(msg.clone())
                } else {
                    opt_btn.into()
                };
                dropdown = dropdown.child(opt_node);
            }
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

    #[test]
    fn combobox_filters_options_and_wires_toggle_and_option_buttons() {
        #[derive(Clone, Debug, PartialEq)]
        enum ComboMsg {
            Toggle,
            Select(String),
        }

        let node: WidgetNode<ComboMsg> = combobox("colors")
            .value("bl")
            .open(true)
            .option(SelectOption::new("Red", "red"))
            .option(SelectOption::new("Blue", "blue"))
            .option(SelectOption::new("Black", "black"))
            .on_toggle(ComboMsg::Toggle)
            .on_select_option(ComboMsg::Select)
            .into();

        let WidgetNode::Column(container) = &node else {
            panic!("expected Column");
        };
        let WidgetNode::Row(input_row) = &container.children[0] else {
            panic!("expected input Row");
        };
        let WidgetNode::Button(toggle) = &input_row.children[1] else {
            panic!("expected toggle Button");
        };
        assert_eq!(toggle.label, "▴");
        assert_eq!(toggle.activate(), Some(&ComboMsg::Toggle));

        let WidgetNode::Card(dropdown) = &container.children[1] else {
            panic!("expected dropdown Card");
        };
        assert_eq!(dropdown.children.len(), 2);

        let WidgetNode::Button(opt0) = &dropdown.children[0] else {
            panic!("expected option Button");
        };
        assert_eq!(opt0.label, "Blue");
        assert_eq!(opt0.activate(), Some(&ComboMsg::Select("blue".into())));

        let WidgetNode::Button(opt1) = &dropdown.children[1] else {
            panic!("expected second option Button");
        };
        assert_eq!(opt1.label, "Black");
        assert_eq!(opt1.activate(), Some(&ComboMsg::Select("black".into())));
    }
}
