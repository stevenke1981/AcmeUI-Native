//! Bottom navigation bar with icon + label items.
//!
//! Renders as a [`Row`] with evenly distributed [`Column`] items.
//! Each item displays an icon and text label.

use acme_core::WidgetKey;
use acme_widgets::*;

/// Builder for a single bottom navigation item.
pub struct BottomNavItemBuilder<M> {
    pub id: WidgetKey,
    pub label: String,
    pub icon: Option<crate::IconName>,
    pub selected: bool,
    pub on_select: Option<M>,
}

/// Builder for a bottom navigation bar.
pub struct BottomNavBuilder<M> {
    pub id: WidgetKey,
    pub items: Vec<BottomNavItemBuilder<M>>,
    pub selected_index: usize,
}

/// Create a new bottom navigation bar builder.
pub fn bottom_nav<M: Clone + 'static>(id: impl Into<WidgetKey>) -> BottomNavBuilder<M> {
    BottomNavBuilder {
        id: id.into(),
        items: Vec::new(),
        selected_index: 0,
    }
}

/// Create a new bottom navigation item builder.
pub fn bottom_nav_item<M: Clone + 'static>(
    id: impl Into<WidgetKey>,
    label: impl Into<String>,
) -> BottomNavItemBuilder<M> {
    BottomNavItemBuilder {
        id: id.into(),
        label: label.into(),
        icon: None,
        selected: false,
        on_select: None,
    }
}

impl<M: Clone + 'static> BottomNavBuilder<M> {
    /// Add an item to the navigation bar.
    pub fn item(mut self, item: BottomNavItemBuilder<M>) -> Self {
        self.items.push(item);
        self
    }

    /// Set the currently selected index.
    pub fn selected_index(mut self, index: usize) -> Self {
        self.selected_index = index;
        self
    }
}

impl<M: Clone + 'static> BottomNavItemBuilder<M> {
    /// Set the icon for this item.
    pub fn icon(mut self, value: crate::IconName) -> Self {
        self.icon = Some(value);
        self
    }

    /// Set whether this item is selected.
    pub fn selected(mut self, value: bool) -> Self {
        self.selected = value;
        self
    }

    /// Set the message dispatched when this item is selected.
    pub fn on_select(mut self, msg: M) -> Self {
        self.on_select = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<BottomNavBuilder<M>> for WidgetNode<M> {
    fn from(b: BottomNavBuilder<M>) -> Self {
        let any_explicit = b.items.iter().any(|i| i.selected);
        let selected_index = b.selected_index;
        let item_nodes: Vec<WidgetNode<M>> = b
            .items
            .into_iter()
            .enumerate()
            .map(|(idx, item)| {
                let is_selected = if any_explicit {
                    item.selected
                } else {
                    idx == selected_index
                };
                let item_key = item.id.clone();
                let btn_key = format!("{}_btn", item_key.as_str());
                let mut col = column::<M>().key(item_key).gap(4.0).padding(8.0);
                if let Some(ref msg) = item.on_select {
                    col = col.on_click(msg.clone());
                }
                if let Some(icon_name) = &item.icon {
                    col = col.child(crate::icon::<M>(*icon_name).build());
                }
                let tab_btn = button::<M>(btn_key.as_str(), item.label).variant(if is_selected {
                    ButtonVariant::Primary
                } else {
                    ButtonVariant::Ghost
                });
                let btn_node: WidgetNode<M> = match item.on_select {
                    Some(msg) => tab_btn.on_click(msg),
                    None => tab_btn.into(),
                };
                col = col.child(btn_node);
                col.build()
            })
            .collect();

        let mut row_builder = row::<M>().key(b.id).gap(8.0);
        for node in item_nodes {
            row_builder = row_builder.child(node);
        }
        row_builder.build()
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
    fn bottom_nav_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = bottom_nav("nav")
            .item(bottom_nav_item("home", "Home").icon(crate::IconName::User))
            .item(bottom_nav_item("settings", "Settings").icon(crate::IconName::Settings))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Row);
        assert!(!layout.children.is_empty());
        assert_eq!(layout.children.len(), 2);
        let first_item = &layout.children[0];
        assert_eq!(first_item.style.kind, LayoutKind::Column);
        assert_ne!(first_item.style.min_height, Length::px(0.0));
    }

    #[test]
    fn bottom_nav_builder_defaults() {
        let nav = bottom_nav::<TestMsg>("nav");
        assert!(nav.items.is_empty());
        assert_eq!(nav.selected_index, 0);
    }

    #[test]
    fn bottom_nav_contains_items() {
        let node: WidgetNode<TestMsg> = bottom_nav("nav")
            .item(bottom_nav_item("a", "Alpha"))
            .item(bottom_nav_item("b", "Beta"))
            .item(bottom_nav_item("c", "Gamma"))
            .into();
        let WidgetNode::Row(container) = &node else {
            panic!("expected Row");
        };
        assert_eq!(container.children.len(), 3);
        // Each child is a Column
        for child in &container.children {
            assert!(matches!(child, WidgetNode::Column(_)));
        }
    }

    #[test]
    fn bottom_nav_renders_keyed_clickable_tab_buttons() {
        #[derive(Clone, Debug, PartialEq)]
        enum NavMsg {
            Home,
            Search,
        }

        let node: WidgetNode<NavMsg> = bottom_nav("nav")
            .selected_index(1)
            .item(bottom_nav_item("home", "Home").on_select(NavMsg::Home))
            .item(bottom_nav_item("search", "Search").on_select(NavMsg::Search))
            .into();

        let WidgetNode::Row(row) = &node else {
            panic!("expected Row");
        };
        let WidgetNode::Column(first_col) = &row.children[0] else {
            panic!("expected first Column");
        };
        assert_eq!(first_col.key.as_ref().map(|k| k.as_str()), Some("home"));
        let WidgetNode::Button(first_btn) = &first_col.children[0] else {
            panic!("expected tab Button");
        };
        assert_eq!(first_btn.key.as_str(), "home_btn");
        assert_eq!(first_btn.activate(), Some(&NavMsg::Home));
        assert_eq!(first_btn.variant, ButtonVariant::Ghost);

        let WidgetNode::Column(second_col) = &row.children[1] else {
            panic!("expected second Column");
        };
        let WidgetNode::Button(second_btn) = &second_col.children[0] else {
            panic!("expected second tab Button");
        };
        assert_eq!(second_btn.variant, ButtonVariant::Primary);
        assert_eq!(second_btn.activate(), Some(&NavMsg::Search));
    }
}
