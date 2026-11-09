//! Mobile search — rounded search input with icon and clear button.

use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_layout::Length;
use acme_widgets::style::prelude::*;

/// Builder for a mobile search bar.
pub struct MobileSearchBuilder<M> {
    pub id: WidgetKey,
    pub placeholder: String,
    /// Controlled query value; update it from the host text-input state.
    pub value: String,
    /// Marker message for input changes, matching the foundation TextInput API.
    pub on_input: Option<M>,
    pub on_submit: Option<M>,
    pub on_clear: Option<M>,
}

/// Create a mobile search bar builder.
pub fn mobile_search<M: Clone + 'static>(placeholder: impl Into<String>) -> MobileSearchBuilder<M> {
    MobileSearchBuilder {
        id: WidgetKey::from("mobile_search"),
        placeholder: placeholder.into(),
        value: String::new(),
        on_input: None,
        on_submit: None,
        on_clear: None,
    }
}

impl<M: Clone + 'static> MobileSearchBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    /// Set the current query. The host owns editing state for `{key}__input`.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    pub fn on_input(mut self, msg: M) -> Self {
        self.on_input = Some(msg);
        self
    }

    /// Attach the search action. Hosts may also dispatch the root Row message
    /// on Enter while its input is focused and no composition is active.
    pub fn on_submit(mut self, msg: M) -> Self {
        self.on_submit = Some(msg);
        self
    }

    pub fn on_clear(mut self, msg: M) -> Self {
        self.on_clear = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<MobileSearchBuilder<M>> for WidgetNode<M> {
    fn from(b: MobileSearchBuilder<M>) -> Self {
        let id = b.id.as_str();
        let icon = if let Some(msg) = b.on_submit.as_ref() {
            crate::button(format!("{id}__submit"), "⌕ Search")
                .min_h(Length::px(44.0))
                .on_click(msg.clone())
        } else {
            crate::icon::<M>(crate::IconName::Search).build()
        };
        let mut input = crate::text_input(format!("{id}__input"))
            .placeholder(b.placeholder)
            .value(b.value)
            .w(Length::px(0.0))
            .min_w(Length::px(0.0))
            .min_h(Length::px(44.0))
            .flex_1();
        // This is an input-change marker, never the submit action.
        input.message = b.on_input;
        let mut row = crate::row::<M>()
            .key(b.id.clone())
            .w(Length::Percent(1.0))
            .bg_muted()
            .rounded(16.0)
            .gap(8.0)
            .padding(10.0)
            .child(icon)
            .child(input);

        if let Some(msg) = b.on_clear {
            row = row.child(
                crate::button(format!("{id}__clear"), "Clear")
                    .min_h(Length::px(44.0))
                    .on_click(msg),
            );
        }
        if let Some(msg) = b.on_submit {
            row = row.on_click(msg);
        }
        row.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Submit,
        Clear,
    }

    #[test]
    fn mobile_search_produces_row() {
        let node: WidgetNode<Msg> = mobile_search("Search…").into();
        assert!(matches!(node, WidgetNode::Row(_)));
    }

    #[test]
    fn mobile_search_has_icon_and_placeholder() {
        let node: WidgetNode<Msg> = mobile_search("Find").into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        assert_eq!(r.children.len(), 2);
    }

    #[test]
    fn mobile_search_with_clear_adds_button() {
        let node: WidgetNode<Msg> = mobile_search("Q").on_clear(Msg::Clear).into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        assert_eq!(r.children.len(), 3);
    }

    #[test]
    fn mobile_search_stores_submit_message() {
        let b = mobile_search::<Msg>("Q").on_submit(Msg::Submit);
        assert_eq!(b.on_submit, Some(Msg::Submit));
    }
}

#[cfg(test)]
#[path = "mobile_search_tests.rs"]
mod acceptance_tests;
