//! ContextMenu component — right-click context menu popup using Stack positioning.
//!
//! Renders as the anchor widget when closed, and as a Stack with anchor + dropdown
//! Card of menu items when open.

use acme_core::WidgetKey;
use acme_widgets::*;

/// Builder for a ContextMenu (right-click context menu).
pub struct ContextMenuBuilder<M> {
    pub id: WidgetKey,
    pub anchor: Option<WidgetNode<M>>,
    pub items: Vec<MenuItem<M>>,
    pub open: bool,
}

/// Create a context menu builder.
pub fn context_menu<M: Clone + 'static>(id: impl Into<WidgetKey>) -> ContextMenuBuilder<M> {
    ContextMenuBuilder {
        id: id.into(),
        anchor: None,
        items: vec![],
        open: false,
    }
}

impl<M: Clone + 'static> ContextMenuBuilder<M> {
    /// Set the anchor widget that triggers the context menu.
    pub fn anchor(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.anchor = Some(child.into());
        self
    }

    /// Add a menu item.
    pub fn item(mut self, item: MenuItem<M>) -> Self {
        self.items.push(item);
        self
    }

    /// Add a separator line between items.
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

    /// Open or close the context menu.
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }
}

impl<M: Clone + 'static> From<ContextMenuBuilder<M>> for WidgetNode<M> {
    fn from(b: ContextMenuBuilder<M>) -> Self {
        if !b.open {
            // Closed: render just the anchor widget, or a minimal label if none
            return b.anchor.unwrap_or_else(|| label::<M>(""));
        }

        // Open: render a Stack with anchor + dropdown Card of items
        let anchor = b.anchor.unwrap_or_else(|| label::<M>(""));

        let mut dropdown = card::<M>().variant(CardVariant::Outlined);
        for item in &b.items {
            if item.separator {
                dropdown = dropdown.child(separator::<M>());
            } else {
                dropdown = dropdown.child(label::<M>(&item.label));
            }
        }

        stack::<M>()
            .key(b.id)
            .child(anchor)
            .child(dropdown.build())
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
    fn context_menu_builder_defaults() {
        let cm = context_menu::<TestMsg>("ctx1");
        assert_eq!(cm.id.as_str(), "ctx1");
        assert!(cm.anchor.is_none());
        assert!(cm.items.is_empty());
        assert!(!cm.open);
    }

    #[test]
    fn context_menu_closed_renders_anchor() {
        let node: WidgetNode<TestMsg> = context_menu("ctx1")
            .anchor(label("Right-click me"))
            .item(menu_item("cut", "Cut"))
            .open(false)
            .into();
        // When closed, the result should be the anchor label
        assert!(matches!(node, WidgetNode::Label(_)));
    }

    #[test]
    fn context_menu_closed_no_anchor_renders_empty_label() {
        let node: WidgetNode<TestMsg> =
            context_menu::<TestMsg>("ctx1").open(false).into();
        assert!(matches!(node, WidgetNode::Label(_)));
    }

    #[test]
    fn context_menu_open_renders_stack() {
        let node: WidgetNode<TestMsg> = context_menu("ctx1")
            .anchor(label("Right-click me"))
            .item(menu_item("cut", "Cut"))
            .item(menu_item("copy", "Copy"))
            .open(true)
            .into();
        assert!(matches!(node, WidgetNode::Stack(_)));
    }

    #[test]
    fn context_menu_open_has_anchor_and_dropdown() {
        let node: WidgetNode<TestMsg> = context_menu("ctx1")
            .anchor(label("Right-click me"))
            .item(menu_item("cut", "Cut"))
            .item(menu_item("copy", "Copy"))
            .open(true)
            .into();
        let WidgetNode::Stack(stack) = &node else {
            panic!("expected Stack");
        };
        assert_eq!(stack.children.len(), 2);
        // First child is the anchor
        assert!(matches!(&stack.children[0], WidgetNode::Label(_)));
        // Second child is the dropdown Card
        assert!(matches!(&stack.children[1], WidgetNode::Card(_)));
    }

    #[test]
    fn context_menu_open_with_separator() {
        let node: WidgetNode<TestMsg> = context_menu("ctx1")
            .anchor(label("Right-click me"))
            .item(menu_item("cut", "Cut"))
            .separator()
            .item(menu_item("paste", "Paste"))
            .open(true)
            .into();
        let WidgetNode::Stack(stack) = &node else {
            panic!("expected Stack");
        };
        let WidgetNode::Card(card) = &stack.children[1] else {
            panic!("expected Card dropdown");
        };
        // Three children in the card: label, separator, label
        assert_eq!(card.children.len(), 3);
        assert!(matches!(&card.children[0], WidgetNode::Label(_)));
        assert!(matches!(&card.children[1], WidgetNode::Separator(_)));
        assert!(matches!(&card.children[2], WidgetNode::Label(_)));
    }

    #[test]
    fn context_menu_open_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = context_menu("ctx1")
            .anchor(label("Right-click me"))
            .item(menu_item("cut", "Cut"))
            .item(menu_item("copy", "Copy"))
            .open(true)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Stack);
        assert_eq!(layout.children.len(), 2);
    }
}
