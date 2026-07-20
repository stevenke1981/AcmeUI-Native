//! NavigationMenu component — a horizontal navigation bar with items that can
//! have dropdown sub-items (like a website nav bar with dropdowns).

use crate::WidgetNode;

/// A single navigation menu item, optionally with child items for a dropdown.
#[derive(Clone, Debug, PartialEq)]
pub struct NavMenuItem {
    pub label: String,
    pub value: String,
    pub children: Vec<NavMenuItem>,
    pub disabled: bool,
}

/// Create a navigation menu item.
pub fn nav_menu_item(label: impl Into<String>, value: impl Into<String>) -> NavMenuItem {
    NavMenuItem {
        label: label.into(),
        value: value.into(),
        children: vec![],
        disabled: false,
    }
}

impl NavMenuItem {
    /// Add a child item (for dropdown sub-menus).
    pub fn child(mut self, item: NavMenuItem) -> Self {
        self.children.push(item);
        self
    }

    /// Set the disabled state.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
}

/// Builder for a horizontal navigation menu.
pub struct NavigationMenuBuilder<M> {
    id: crate::WidgetKey,
    items: Vec<NavMenuItem>,
    selected_value: String,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a navigation menu builder.
pub fn navigation_menu<M: Clone + 'static>(
    id: impl Into<crate::WidgetKey>,
) -> NavigationMenuBuilder<M> {
    NavigationMenuBuilder {
        id: id.into(),
        items: vec![],
        selected_value: String::new(),
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone> NavigationMenuBuilder<M> {
    /// Add a navigation menu item.
    pub fn item(mut self, item: NavMenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Set the selected item's value. The item with this value will be
    /// visually highlighted; if it has children, a dropdown card is shown.
    pub fn selected(mut self, value: impl Into<String>) -> Self {
        self.selected_value = value.into();
        self
    }

    /// Build the widget node tree.
    ///
    /// Produces a horizontal [`Row`] where each menu item is a [`Column`]
    /// containing a [`Button`]. If the item is selected and has children, a
    /// [`Card`] with child buttons is appended inside that column.
    pub fn build(self) -> WidgetNode<M> {
        let mut row = crate::row::<M>().gap(0.0);

        for item in &self.items {
            let is_selected = item.value == self.selected_value;
            let item_key = format!("{}_{}", self.id.as_str(), item.value);
            let btn = crate::button::<M>(item_key.as_str(), &item.label)
                .disabled(item.disabled);
            let mut col = crate::column::<M>()
                .child(btn)
                .gap(0.0);

            if is_selected && !item.children.is_empty() {
                let mut dropdown = crate::card::<M>()
                    .variant(crate::CardVariant::Outlined)
                    .padding(4.0)
                    .gap(2.0);
                for child in &item.children {
                    let child_key = format!("{}_{}_{}", self.id.as_str(), item.value, child.value);
                    let child_btn = crate::button::<M>(child_key.as_str(), &child.label)
                        .disabled(child.disabled);
                    dropdown = dropdown.child(child_btn);
                }
                col = col.child(dropdown.build());
            }

            row = row.child(col.build());
        }

        row.build()
    }
}

impl<M: Clone + 'static> From<NavigationMenuBuilder<M>> for WidgetNode<M> {
    fn from(b: NavigationMenuBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
use crate::WidgetNode;
    use acme_core::NodeId;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn navigation_menu_builds_row() {
        let node: WidgetNode<Msg> = navigation_menu::<Msg>("nav")
            .item(nav_menu_item("Home", "home"))
            .item(nav_menu_item("About", "about"))
            .build();
        assert!(matches!(node, WidgetNode::Row(_)));
    }

    #[test]
    fn navigation_menu_has_correct_item_count() {
        let node: WidgetNode<Msg> = navigation_menu::<Msg>("nav")
            .item(nav_menu_item("Home", "home"))
            .item(nav_menu_item("Products", "products"))
            .item(nav_menu_item("About", "about"))
            .selected("home")
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Row contains 3 columns
        assert_eq!(layout.children.len(), 3);
    }

    #[test]
    fn navigation_menu_selected_shows_dropdown() {
        let node: WidgetNode<Msg> = navigation_menu::<Msg>("nav")
            .item(nav_menu_item("Home", "home"))
            .item(
                nav_menu_item("Products", "products")
                    .child(nav_menu_item("Sub A", "sub-a"))
                    .child(nav_menu_item("Sub B", "sub-b")),
            )
            .item(nav_menu_item("About", "about"))
            .selected("products")
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Row contains 3 columns
        assert_eq!(layout.children.len(), 3);
        // First item (not selected, no children): 1 child (button only)
        assert_eq!(layout.children[0].children.len(), 1);
        // Second item (selected, has children): button + card dropdown = 2
        assert_eq!(layout.children[1].children.len(), 2);
        // Third item (not selected, no children): 1 child (button only)
        assert_eq!(layout.children[2].children.len(), 1);
    }

    #[test]
    fn navigation_menu_non_selected_no_dropdown() {
        let node: WidgetNode<Msg> = navigation_menu::<Msg>("nav")
            .item(nav_menu_item("Products", "products"))
            .item(nav_menu_item("About", "about"))
            .selected("about")
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // First item (not selected, no children): 1 child
        assert_eq!(layout.children[0].children.len(), 1);
        // Second item (selected, no children): 1 child (button only)
        assert_eq!(layout.children[1].children.len(), 1);
    }

    #[test]
    fn navigation_menu_no_children_no_card() {
        let node: WidgetNode<Msg> = navigation_menu::<Msg>("nav")
            .item(nav_menu_item("Alone", "alone"))
            .selected("alone")
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Selected but no children -> still only the button
        assert_eq!(layout.children[0].children.len(), 1);
    }

    #[test]
    fn navigation_menu_empty() {
        let node: WidgetNode<Msg> = navigation_menu::<Msg>("nav").build();
        let layout = node.to_layout(NodeId::new(1));
        assert!(layout.children.is_empty());
    }

    #[test]
    fn navigation_menu_from_conversion() {
        let menu = navigation_menu::<Msg>("nav")
            .item(nav_menu_item("Home", "home"));
        let node: WidgetNode<Msg> = menu.into();
        assert!(matches!(node, WidgetNode::Row(_)));
    }
}
