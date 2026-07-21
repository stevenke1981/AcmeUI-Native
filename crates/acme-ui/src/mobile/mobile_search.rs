//! Mobile search — rounded search input with icon and clear button.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a mobile search bar.
pub struct MobileSearchBuilder<M> {
    pub id: WidgetKey,
    pub placeholder: String,
    pub on_submit: Option<M>,
    pub on_clear: Option<M>,
}

/// Create a mobile search bar builder.
pub fn mobile_search<M: Clone + 'static>(
    placeholder: impl Into<String>,
) -> MobileSearchBuilder<M> {
    MobileSearchBuilder {
        id: WidgetKey::from("mobile_search"),
        placeholder: placeholder.into(),
        on_submit: None,
        on_clear: None,
    }
}

impl<M: Clone + 'static> MobileSearchBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

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
        let icon = crate::icon::<M>(crate::IconName::Search).build();
        let mut row = crate::row::<M>()
            .key(b.id)
            .gap(8.0)
            .padding(10.0)
            .child(icon)
            .child(crate::label(b.placeholder));

        if let Some(msg) = b.on_clear {
            let clear = crate::button("search_clear", "✕").on_click(msg);
            row = row.child(clear);
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
}
