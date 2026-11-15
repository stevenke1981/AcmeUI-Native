//! DropdownMenu component — a trigger button that shows a popup menu.
//!
//! Built on the Popover and Menu primitives from acme-widgets.

use crate::{MenuItem, WidgetNode};
use acme_core::WidgetKey;

/// Builder for a dropdown menu.
pub struct DropdownMenuBuilder<M> {
    pub id: WidgetKey,
    pub trigger_label: String,
    pub items: Vec<MenuItem<M>>,
    pub open: bool,
    pub on_toggle: Option<M>,
}

/// Create a dropdown menu builder.
pub fn dropdown_menu<M: Clone + 'static>(id: impl Into<WidgetKey>) -> DropdownMenuBuilder<M> {
    DropdownMenuBuilder {
        id: id.into(),
        trigger_label: String::new(),
        items: vec![],
        open: false,
        on_toggle: None,
    }
}

impl<M: Clone> DropdownMenuBuilder<M> {
    /// Set the trigger button label.
    pub fn trigger_label(mut self, value: impl Into<String>) -> Self {
        self.trigger_label = value.into();
        self
    }

    /// Add a menu item.
    pub fn item(mut self, item: MenuItem<M>) -> Self {
        self.items.push(item);
        self
    }

    /// Add a visual separator between items.
    pub fn separator(mut self) -> Self {
        self.items.push(MenuItem {
            key: WidgetKey::from(""),
            label: String::new(),
            disabled: true,
            message: None,
            separator: true,
            children: vec![],
        });
        self
    }

    /// Open or close the dropdown menu.
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }

    /// Set the message dispatched when the trigger button is clicked to toggle the menu.
    pub fn on_toggle(mut self, msg: M) -> Self {
        self.on_toggle = Some(msg);
        self
    }

    /// Build the widget node tree.
    ///
    /// When open with items: renders a `Popover` with the trigger button as
    /// the anchor and an `Outlined` `Card` containing the menu items as content.
    /// When closed or empty: renders just the trigger button.
    pub fn build(self) -> WidgetNode<M> {
        let id = self.id;
        let trigger_btn = crate::button(
            format!("{}_trigger", id.as_str()).as_str(),
            &self.trigger_label,
        );
        let trigger: WidgetNode<M> = match self.on_toggle {
            Some(msg) => trigger_btn.on_click(msg),
            None => trigger_btn.into(),
        };

        if !self.open || self.items.is_empty() {
            return trigger;
        }

        // Build menu content as a column of items inside an Outlined Card.
        let mut content_col = crate::column::<M>().gap(2.0);

        for (idx, item) in self.items.iter().enumerate() {
            if item.separator {
                content_col = content_col.child(crate::separator::<M>());
            } else {
                let item_key = if item.key.as_str().is_empty() {
                    format!("{}_item_{idx}", id.as_str())
                } else {
                    format!("{}_{}", id.as_str(), item.key.as_str())
                };
                let btn = crate::button::<M>(item_key.as_str(), &item.label)
                    .variant(crate::ButtonVariant::Ghost)
                    .disabled(item.disabled);
                let node: WidgetNode<M> = match item.activate() {
                    Some(msg) => btn.on_click(msg.clone()),
                    None => btn.into(),
                };
                content_col = content_col.child(node);
            }
        }

        let content = crate::card::<M>()
            .variant(crate::CardVariant::Outlined)
            .child(content_col.build())
            .padding(4.0)
            .build();

        crate::popover::<M>(id.as_str(), trigger)
            .content(content)
            .open(true)
            .build()
    }
}

impl<M: Clone + 'static> From<DropdownMenuBuilder<M>> for WidgetNode<M> {
    fn from(b: DropdownMenuBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WidgetNode;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn dropdown_menu_defaults() {
        let d = dropdown_menu::<Msg>("menu");
        assert!(d.trigger_label.is_empty());
        assert!(!d.open);
        assert!(d.items.is_empty());
    }

    #[test]
    fn dropdown_menu_closed_returns_button() {
        let node: WidgetNode<Msg> = dropdown_menu::<Msg>("menu")
            .trigger_label("File")
            .item(crate::menu_item("new", "New"))
            .open(false)
            .into();
        assert!(matches!(node, WidgetNode::Button(_)));
    }

    #[test]
    fn dropdown_menu_open_returns_popover() {
        let node: WidgetNode<Msg> = dropdown_menu::<Msg>("menu")
            .trigger_label("File")
            .item(crate::menu_item("new", "New"))
            .open(true)
            .into();
        assert!(matches!(node, WidgetNode::Popover(_)));
    }

    #[test]
    fn dropdown_menu_empty_items_closed() {
        let node: WidgetNode<Msg> = dropdown_menu::<Msg>("menu")
            .trigger_label("File")
            .open(true) // no items -> just trigger
            .into();
        assert!(matches!(node, WidgetNode::Button(_)));
    }

    #[test]
    fn dropdown_menu_items_and_separator() {
        let d = dropdown_menu::<Msg>("menu")
            .trigger_label("Edit")
            .item(crate::menu_item("undo", "Undo"))
            .separator()
            .item(crate::menu_item("redo", "Redo"));
        assert_eq!(d.items.len(), 3);
        assert!(!d.items[0].separator);
        assert!(d.items[1].separator);
        assert!(!d.items[2].separator);
    }

    #[test]
    fn dropdown_menu_open_popover_has_anchor_and_content() {
        let node: WidgetNode<Msg> = dropdown_menu::<Msg>("menu")
            .trigger_label("File")
            .item(crate::menu_item("new", "New"))
            .item(crate::menu_item("exit", "Exit"))
            .open(true)
            .into();
        let WidgetNode::Popover(p) = &node else {
            panic!("expected Popover variant");
        };
        assert_eq!(p.children.len(), 2);
        assert!(matches!(p.children[0], WidgetNode::Button(_)));
        assert!(matches!(p.children[1], WidgetNode::Card(_)));
    }

    #[test]
    fn dropdown_menu_from_conversion() {
        let d = dropdown_menu::<Msg>("m")
            .trigger_label("Edit")
            .item(crate::menu_item("undo", "Undo"))
            .open(true);
        let node: WidgetNode<Msg> = d.into();
        assert!(matches!(node, WidgetNode::Popover(_)));
    }

    #[test]
    fn dropdown_menu_renders_keyed_buttons_and_handles_disabled_and_toggle() {
        #[derive(Clone, Debug, PartialEq)]
        enum ActionMsg {
            Toggle,
            NewFile,
            Secret,
        }

        let node: WidgetNode<ActionMsg> = dropdown_menu("file_menu")
            .trigger_label("File")
            .on_toggle(ActionMsg::Toggle)
            .item(crate::menu_item("new", "New").on_click(ActionMsg::NewFile))
            .item(
                crate::menu_item("secret", "Secret")
                    .on_click(ActionMsg::Secret)
                    .disabled(true),
            )
            .open(true)
            .into();

        let WidgetNode::Popover(popover) = &node else {
            panic!("expected Popover");
        };
        let WidgetNode::Button(trigger) = &popover.children[0] else {
            panic!("expected trigger Button");
        };
        assert_eq!(trigger.activate(), Some(&ActionMsg::Toggle));

        let WidgetNode::Card(card) = &popover.children[1] else {
            panic!("expected Card");
        };
        let WidgetNode::Column(col) = &card.children[0] else {
            panic!("expected Column inside Card");
        };
        assert_eq!(col.children.len(), 2);

        let WidgetNode::Button(new_btn) = &col.children[0] else {
            panic!("expected Button for menu item");
        };
        assert_eq!(new_btn.key.as_str(), "file_menu_new");
        assert!(!new_btn.disabled);
        assert_eq!(new_btn.activate(), Some(&ActionMsg::NewFile));

        let WidgetNode::Button(secret_btn) = &col.children[1] else {
            panic!("expected Button for disabled menu item");
        };
        assert_eq!(secret_btn.key.as_str(), "file_menu_secret");
        assert!(secret_btn.disabled);
        assert_eq!(secret_btn.activate(), None);
    }
}
