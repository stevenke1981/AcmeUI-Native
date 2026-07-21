//! Mobile list item — title + detail + optional trailing icon, tappable.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a mobile list item.
pub struct MobileListItemBuilder<M> {
    pub id: WidgetKey,
    pub title: String,
    pub detail: Option<String>,
    pub trailing_icon: Option<crate::IconName>,
    pub on_tap: Option<M>,
}

/// Create a mobile list item builder.
pub fn mobile_list_item<M: Clone + 'static>(
    title: impl Into<String>,
    detail: Option<impl Into<String>>,
) -> MobileListItemBuilder<M> {
    MobileListItemBuilder {
        id: WidgetKey::from("mobile_list_item"),
        title: title.into(),
        detail: detail.map(Into::into),
        trailing_icon: None,
        on_tap: None,
    }
}

impl<M: Clone + 'static> MobileListItemBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn trailing_icon(mut self, icon: crate::IconName) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    pub fn on_tap(mut self, msg: M) -> Self {
        self.on_tap = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<MobileListItemBuilder<M>> for WidgetNode<M> {
    fn from(b: MobileListItemBuilder<M>) -> Self {
        let mut text_col = crate::column::<M>().gap(2.0).child(crate::label(b.title));
        if let Some(d) = b.detail {
            text_col = text_col.child(crate::label(d));
        }

        let mut row = crate::row::<M>()
            .key(b.id)
            .gap(12.0)
            .padding(14.0)
            .child(text_col.build());

        if let Some(icon_name) = b.trailing_icon {
            row = row.child(crate::icon::<M>(icon_name).build());
        }
        if let Some(msg) = b.on_tap {
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
        Tapped,
    }

    #[test]
    fn mobile_list_item_produces_row() {
        let node: WidgetNode<Msg> = mobile_list_item("Title", None::<&str>).into();
        assert!(matches!(node, WidgetNode::Row(_)));
    }

    #[test]
    fn mobile_list_item_with_detail() {
        let node: WidgetNode<Msg> = mobile_list_item("Title", Some("Detail")).into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        // First child is the text column with 2 labels
        let WidgetNode::Column(col) = &r.children[0] else {
            panic!("expected Column");
        };
        assert_eq!(col.children.len(), 2);
    }

    #[test]
    fn mobile_list_item_with_trailing_icon() {
        let node: WidgetNode<Msg> = mobile_list_item("Settings", None::<&str>)
            .trailing_icon(crate::IconName::ChevronRight)
            .into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        assert_eq!(r.children.len(), 2);
    }

    #[test]
    fn mobile_list_item_on_tap() {
        let node: WidgetNode<Msg> = mobile_list_item("X", None::<&str>)
            .on_tap(Msg::Tapped)
            .into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        assert_eq!(r.message, Some(Msg::Tapped));
    }
}
