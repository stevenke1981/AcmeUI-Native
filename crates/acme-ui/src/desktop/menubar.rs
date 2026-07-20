//! Menubar — a horizontal menu bar for desktop applications.
//!
//! Each menu has a label and items shown as a dropdown when open.
//! Follows the AcmeUIKit alignment builder pattern.

use crate::*;
use acme_core::WidgetKey;

/// A single menu entry with a label and its items.
#[derive(Clone, Debug, PartialEq)]
pub struct MenuEntry<M> {
    pub label: String,
    pub items: Vec<MenuItem<M>>,
}

/// Builder for a Menubar component.
pub struct MenubarBuilder<M> {
    pub id: WidgetKey,
    pub menus: Vec<MenuEntry<M>>,
    pub open_index: Option<usize>,
}

/// Create a new Menubar builder.
pub fn menubar<M: Clone + 'static>(id: impl Into<WidgetKey>) -> MenubarBuilder<M> {
    MenubarBuilder {
        id: id.into(),
        menus: vec![],
        open_index: None,
    }
}

impl<M: Clone + 'static> MenubarBuilder<M> {
    /// Add a new menu with the given label.
    ///
    /// Subsequent `.item()` and `.separator()` calls add items to this menu
    /// until the next `.menu()` call.
    pub fn menu(mut self, label: impl Into<String>) -> Self {
        self.menus.push(MenuEntry {
            label: label.into(),
            items: vec![],
        });
        self
    }

    /// Add a menu item to the most recently added menu.
    pub fn item(mut self, item: MenuItem<M>) -> Self {
        if let Some(menu) = self.menus.last_mut() {
            menu.items.push(item);
        }
        self
    }

    /// Add a separator line to the most recently added menu.
    pub fn separator(mut self) -> Self {
        if let Some(menu) = self.menus.last_mut() {
            menu.items.push(MenuItem {
                key: WidgetKey::from(""),
                label: String::new(),
                disabled: true,
                message: None,
                separator: true,
                children: vec![],
            });
        }
        self
    }

    /// Set which menu is currently open by index.
    ///
    /// When `Some(index)`, that menu's items are rendered as a dropdown overlay.
    pub fn open_index(mut self, index: Option<usize>) -> Self {
        self.open_index = index;
        self
    }
}

impl<M: Clone + 'static> From<MenubarBuilder<M>> for WidgetNode<M> {
    fn from(b: MenubarBuilder<M>) -> Self {
        // Build a row of label widgets (one per menu entry)
        let mut label_row = row::<M>().key(b.id.clone()).gap(8.0);

        for entry in &b.menus {
            label_row = label_row.child(label::<M>(entry.label.as_str()));
        }

        // If a menu is open, wrap in a Stack with the menu dropdown
        if let Some(idx) = b.open_index {
            if let Some(entry) = b.menus.get(idx) {
                let menu_key = format!("{}-dropdown", b.id.as_str());
                let mut menu_dropdown = menu::<M>(menu_key.as_str()).open(true);
                for item in &entry.items {
                    menu_dropdown = menu_dropdown.item(item.clone());
                }

                return stack::<M>()
                    .key(b.id)
                    .child(label_row.build())
                    .child(menu_dropdown.build())
                    .build();
            }
        }

        // No open menu: just the row of labels
        label_row.build()
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

    /// Empty message enum for builder-only tests.
    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    /// Message enum for tests that verify item messages and layout.
    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        NewFile,
        ExitApp,
        Undo,
        Redo,
    }

    #[test]
    fn menubar_creates_menu_entries() {
        let mb = menubar::<TestMsg>("main")
            .menu("File")
            .menu("Edit")
            .menu("Help");
        assert_eq!(mb.menus.len(), 3);
        assert_eq!(mb.menus[0].label, "File");
        assert_eq!(mb.menus[1].label, "Edit");
        assert_eq!(mb.menus[2].label, "Help");
    }

    #[test]
    fn menubar_builder_defaults() {
        let mb = menubar::<TestMsg>("main");
        assert!(mb.menus.is_empty());
        assert!(mb.open_index.is_none());
        assert_eq!(mb.id.as_str(), "main");
    }

    #[test]
    fn menubar_closed_renders_row() {
        let node: WidgetNode<Msg> = menubar("main").menu("File").menu("Edit").into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Row);
        assert_eq!(layout.children.len(), 2);
    }

    #[test]
    fn menubar_open_renders_stack() {
        let node: WidgetNode<Msg> = menubar("main")
            .menu("File")
            .item(menu_item("new", "New"))
            .menu("Edit")
            .open_index(Some(0))
            .into();
        assert!(matches!(node, WidgetNode::Stack(_)));
    }

    #[test]
    fn menubar_open_shows_menu_items() {
        let node: WidgetNode<Msg> = menubar("main")
            .menu("File")
            .item(menu_item("new", "New"))
            .item(menu_item("open", "Open..."))
            .open_index(Some(0))
            .into();
        let WidgetNode::Stack(s) = &node else {
            panic!("expected Stack variant");
        };
        assert_eq!(s.children.len(), 2);
        // First child is the label Row
        assert!(matches!(s.children[0], WidgetNode::Row(_)));
        // Second child is the Menu dropdown
        let WidgetNode::Menu(m) = &s.children[1] else {
            panic!("expected Menu as second child");
        };
        assert!(m.open);
        assert_eq!(m.items.len(), 2);
        assert_eq!(m.items[0].label, "New");
        assert_eq!(m.items[1].label, "Open...");
    }

    #[test]
    fn menubar_separator_and_messages() {
        let mb = menubar::<Msg>("main")
            .menu("File")
            .item(menu_item("new", "New").on_click(Msg::NewFile))
            .separator()
            .item(menu_item("exit", "Exit").on_click(Msg::ExitApp));
        assert_eq!(mb.menus.len(), 1);
        let items = &mb.menus[0].items;
        assert_eq!(items.len(), 3);
        assert!(!items[0].separator);
        assert_eq!(items[0].message, Some(Msg::NewFile));
        assert_eq!(items[0].label, "New");
        assert!(items[1].separator);
        assert!(items[1].disabled);
        assert!(!items[2].separator);
        assert_eq!(items[2].label, "Exit");
        assert_eq!(items[2].message, Some(Msg::ExitApp));
    }

    #[test]
    fn menubar_none_open_index_returns_row() {
        let node: WidgetNode<Msg> = menubar("main")
            .menu("File")
            .menu("Edit")
            .open_index(None)
            .into();
        assert!(matches!(node, WidgetNode::Row(_)));
    }

    #[test]
    fn menubar_multiple_menus_with_second_open() {
        let node: WidgetNode<Msg> = menubar("main")
            .menu("File")
            .item(menu_item("new", "New"))
            .menu("Edit")
            .item(menu_item("undo", "Undo").on_click(Msg::Undo))
            .item(menu_item("redo", "Redo").on_click(Msg::Redo))
            .open_index(Some(1))
            .into();
        let WidgetNode::Stack(s) = &node else {
            panic!("expected Stack variant");
        };
        let WidgetNode::Menu(m) = &s.children[1] else {
            panic!("expected Menu as second child");
        };
        assert_eq!(m.items.len(), 2);
        assert_eq!(m.items[0].label, "Undo");
        assert_eq!(m.items[1].label, "Redo");
        assert_eq!(m.items[0].message, Some(Msg::Undo));
        assert_eq!(m.items[1].message, Some(Msg::Redo));
    }
}
