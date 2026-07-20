//! SearchInput component.
//!
//! Renders as a Row with a "⌕" search icon Label and a TextInput.

use acme_core::WidgetKey;
use acme_widgets::*;

/// Builder for a SearchInput component.
pub struct SearchInputBuilder<M> {
    pub id: WidgetKey,
    pub placeholder: String,
    pub value: String,
    pub disabled: bool,
    pub on_input: Option<M>,
}

/// Create a new SearchInput builder.
pub fn search_input<M: Clone + 'static>(id: impl Into<WidgetKey>) -> SearchInputBuilder<M> {
    SearchInputBuilder {
        id: id.into(),
        placeholder: String::new(),
        value: String::new(),
        disabled: false,
        on_input: None,
    }
}

impl<M: Clone + 'static> SearchInputBuilder<M> {
    /// Set the placeholder text.
    pub fn placeholder(mut self, value: impl Into<String>) -> Self {
        self.placeholder = value.into();
        self
    }

    /// Set the current input value.
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }

    /// Set whether the input is disabled.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    /// Set the message dispatched when the input text changes.
    pub fn on_input(mut self, msg: M) -> Self {
        self.on_input = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<SearchInputBuilder<M>> for WidgetNode<M> {
    fn from(b: SearchInputBuilder<M>) -> Self {
        let input_key = format!("{}_input", b.id.as_str());
        row::<M>()
            .key(b.id)
            .gap(4.0)
            .child(label::<M>("⌕"))
            .child(
                text_input::<M>(input_key.as_str())
                    .value(b.value.as_str())
                    .placeholder(b.placeholder.as_str()),
            )
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
    fn search_input_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = search_input("si1")
            .placeholder("Search...")
            .value("hello")
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Row);
        assert!(!layout.children.is_empty());
        // second child = TextInput with non-zero height
        let textinput_leaf = &layout.children[1];
        assert!(textinput_leaf.children.is_empty());
        assert_ne!(textinput_leaf.style.min_height, Length::px(0.0));
    }

    #[test]
    fn search_input_builder_defaults() {
        let s = search_input::<TestMsg>("s");
        assert!(s.placeholder.is_empty());
        assert!(s.value.is_empty());
        assert!(!s.disabled);
        assert!(s.on_input.is_none());
    }

    #[test]
    fn search_input_shows_icon_and_input() {
        let node: WidgetNode<TestMsg> = search_input("si").placeholder("Find...").into();
        let WidgetNode::Row(container) = &node else {
            panic!("expected Row");
        };
        assert_eq!(container.children.len(), 2);
        // First child is the search icon label
        let WidgetNode::Label(icon) = &container.children[0] else {
            panic!("expected Label icon");
        };
        assert_eq!(icon.text, "⌕");
        // Second child is the TextInput
        assert!(matches!(&container.children[1], WidgetNode::TextInput(_)));
    }
}
