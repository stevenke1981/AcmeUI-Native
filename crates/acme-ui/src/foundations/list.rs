//! List component — vertical list of items.
//!
//! Each item can have an optional leading icon, label text, description, and
//! selected indicator.

use acme_core::WidgetKey;
use acme_widgets::*;
use crate::{icon, IconName};

// ---------------------------------------------------------------------------
// ListItem
// ---------------------------------------------------------------------------

/// Builder for a single list item.
pub struct ListItemBuilder<M> {
    pub id: WidgetKey,
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<IconName>,
    pub selected: bool,
    pub disabled: bool,
    pub on_click: Option<M>,
}

/// Create a new list item builder.
pub fn list_item<M: Clone + 'static>(
    id: impl Into<WidgetKey>,
    label: impl Into<String>,
) -> ListItemBuilder<M> {
    ListItemBuilder {
        id: id.into(),
        label: label.into(),
        description: None,
        icon: None,
        selected: false,
        disabled: false,
        on_click: None,
    }
}

impl<M: Clone + 'static> ListItemBuilder<M> {
    /// Set the optional description text.
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Set the leading icon.
    pub fn icon(mut self, value: IconName) -> Self {
        self.icon = Some(value);
        self
    }

    /// Set the selected state.
    pub fn selected(mut self, value: bool) -> Self {
        self.selected = value;
        self
    }

    /// Set whether the item is disabled.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    /// Set the message dispatched when the item is activated.
    pub fn on_click(mut self, msg: M) -> Self {
        self.on_click = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<ListItemBuilder<M>> for WidgetNode<M> {
    fn from(b: ListItemBuilder<M>) -> Self {
        let mut item_row = row::<M>().key(b.id).gap(8.0).padding(4.0);

        // Optional leading icon
        if let Some(name) = b.icon {
            item_row = item_row.child(icon(name).size(16.0));
        }

        // Label
        item_row = item_row.child(label::<M>(b.label));

        // Optional description
        if let Some(desc) = b.description {
            item_row = item_row.child(label::<M>(desc));
        }

        // Selected indicator
        if b.selected {
            item_row = item_row.child(icon(IconName::Check).size(14.0));
        }

        item_row.build()
    }
}

// ---------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------

/// List display variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListVariant {
    #[default]
    Plain,
    Bordered,
}

/// Builder for a List component.
pub struct ListBuilder<M> {
    pub id: WidgetKey,
    pub items: Vec<ListItemBuilder<M>>,
    pub variant: ListVariant,
    pub size: crate::ControlSize,
}

/// Create a new List builder.
pub fn list<M: Clone + 'static>(id: impl Into<WidgetKey>) -> ListBuilder<M> {
    ListBuilder {
        id: id.into(),
        items: vec![],
        variant: ListVariant::Plain,
        size: crate::ControlSize::Md,
    }
}

impl<M: Clone + 'static> ListBuilder<M> {
    /// Add an item to the list.
    pub fn item(mut self, item: ListItemBuilder<M>) -> Self {
        self.items.push(item);
        self
    }

    /// Set the list variant (Plain or Bordered).
    pub fn variant(mut self, value: ListVariant) -> Self {
        self.variant = value;
        self
    }

    /// Set the list item size.
    pub fn size(mut self, value: crate::ControlSize) -> Self {
        self.size = value;
        self
    }
}

impl<M: Clone + 'static> From<ListBuilder<M>> for WidgetNode<M> {
    fn from(b: ListBuilder<M>) -> Self {
        match b.variant {
            ListVariant::Plain => {
                let mut col = column::<M>().key(b.id).gap(0.0);
                for item in b.items {
                    col = col.child(item);
                }
                col.build()
            }
            ListVariant::Bordered => {
                let mut card = card::<M>()
                    .key(b.id)
                    .variant(CardVariant::Outlined)
                    .gap(0.0);
                for item in b.items {
                    card = card.child(item);
                }
                card.build()
            }
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
    #[allow(dead_code)]
    enum TestMsg {
        ItemClicked,
    }

    #[test]
    fn list_item_builder_defaults() {
        let item = list_item::<TestMsg>("i1", "Home");
        assert_eq!(item.label, "Home");
        assert!(item.description.is_none());
        assert!(item.icon.is_none());
        assert!(!item.selected);
        assert!(!item.disabled);
        assert!(item.on_click.is_none());
    }

    #[test]
    fn list_item_builds_row_with_label() {
        let node: WidgetNode<TestMsg> = list_item("i1", "Settings").into();
        let WidgetNode::Row(container) = &node else {
            panic!("expected Row variant");
        };
        assert_eq!(container.key.as_ref().unwrap().as_str(), "i1");
        // At minimum one child: the label
        assert!(!container.children.is_empty());
    }

    #[test]
    fn list_item_icon_and_description() {
        let node: WidgetNode<TestMsg> = list_item("i1", "Files")
            .icon(IconName::Folder)
            .description("12 items")
            .into();
        let WidgetNode::Row(container) = &node else {
            panic!("expected Row variant");
        };
        // icon + label + description = 3 children
        assert_eq!(container.children.len(), 3);
    }

    #[test]
    fn list_item_selected_shows_check() {
        let node: WidgetNode<TestMsg> = list_item("i1", "Selected").selected(true).into();
        let WidgetNode::Row(container) = &node else {
            panic!("expected Row variant");
        };
        // label + check icon = 2 children (no icon, no description)
        assert_eq!(container.children.len(), 2);
        // Last child should be the Check icon (rendered as Label)
        let last = &container.children[1];
        let WidgetNode::Label(l) = last else {
            panic!("expected Label for check icon");
        };
        assert_eq!(l.text, "✓");
    }

    #[test]
    fn list_plain_builds_column() {
        let node: WidgetNode<TestMsg> = list("mylist")
            .item(list_item("a", "Alpha"))
            .item(list_item("b", "Beta"))
            .into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column variant");
        };
        assert_eq!(c.key.as_ref().unwrap().as_str(), "mylist");
        assert_eq!(c.children.len(), 2);
    }

    #[test]
    fn list_bordered_builds_card() {
        let node: WidgetNode<TestMsg> = list("blist")
            .variant(ListVariant::Bordered)
            .item(list_item("a", "Item A"))
            .into();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant for bordered list");
        };
        assert_eq!(c.children.len(), 1);
    }

    #[test]
    fn list_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = list("l")
            .item(list_item("a", "Alpha"))
            .item(list_item("b", "Beta"))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert_eq!(layout.children.len(), 2);
        for child in &layout.children {
            assert!(!child.children.is_empty());
        }
    }
}
