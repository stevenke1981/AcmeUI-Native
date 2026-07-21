//! Breadcrumb component — a navigation path showing page hierarchy.
//!
//! Renders as a Row of clickable items separated by a configurable
//! separator character. The last item is rendered as plain text
//! (the current page) while earlier items are rendered as buttons.

use acme_core::WidgetKey;

/// A single breadcrumb item descriptor.
#[derive(Clone, Debug)]
pub struct BreadcrumbItem<M> {
    /// Display label for this breadcrumb segment.
    pub label: String,
    /// Optional key and message for navigation.
    pub key: Option<WidgetKey>,
    /// Message dispatched when this item is clicked. None = plain label.
    pub on_click: Option<M>,
}

impl<M> BreadcrumbItem<M> {
    /// Create a non-interactive (current page) breadcrumb item.
    pub fn current(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            key: None,
            on_click: None,
        }
    }

    /// Create a clickable breadcrumb item.
    pub fn link(label: impl Into<String>, key: impl Into<WidgetKey>, msg: M) -> Self {
        Self {
            label: label.into(),
            key: Some(key.into()),
            on_click: Some(msg),
        }
    }
}

/// Builder for a Breadcrumb component.
pub struct BreadcrumbBuilder<M> {
    pub id: WidgetKey,
    pub items: Vec<BreadcrumbItem<M>>,
    pub separator: String,
    pub size: crate::ControlSize,
}

/// Create a new Breadcrumb builder.
pub fn breadcrumb<M: Clone + 'static>(id: impl Into<WidgetKey>) -> BreadcrumbBuilder<M> {
    BreadcrumbBuilder {
        id: id.into(),
        items: vec![],
        separator: "/".to_string(),
        size: crate::ControlSize::Sm,
    }
}

impl<M: Clone + 'static> BreadcrumbBuilder<M> {
    /// Set the separator character(s) between items.
    pub fn separator(mut self, value: impl Into<String>) -> Self {
        self.separator = value.into();
        self
    }

    /// Set the breadcrumb item size.
    pub fn size(mut self, value: crate::ControlSize) -> Self {
        self.size = value;
        self
    }

    /// Add a breadcrumb item.
    pub fn item(mut self, item: BreadcrumbItem<M>) -> Self {
        self.items.push(item);
        self
    }
}

impl<M: Clone + 'static> From<BreadcrumbBuilder<M>> for crate::WidgetNode<M> {
    fn from(b: BreadcrumbBuilder<M>) -> Self {
        let mut row = crate::row::<M>().key(b.id).gap(4.0);

        let total = b.items.len();
        for (i, item) in b.items.into_iter().enumerate() {
            // Add the label or clickable link
            if let (Some(key), Some(msg)) = (item.key, item.on_click) {
                let is_last = i == total - 1;
                let label = if is_last {
                    format!("[{}]", item.label)
                } else {
                    item.label
                };
                row = row.child(crate::button(key, label).on_click(msg));
            } else {
                row = row.child(crate::label::<M>(item.label));
            }

            // Add separator between items (not after last)
            if i < total - 1 {
                row = row.child(crate::label::<M>(&b.separator));
            }
        }

        row.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WidgetNode;
    use acme_core::NodeId;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Navigate,
    }

    #[test]
    fn breadcrumb_has_non_zero_layout_rect() {
        let node: WidgetNode<Msg> = breadcrumb::<Msg>("bc")
            .item(BreadcrumbItem::link("Home", "home", Msg::Navigate))
            .item(BreadcrumbItem::link("Products", "prods", Msg::Navigate))
            .item(BreadcrumbItem::current("Details"))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Row: [Home] [/] [Products] [/] [Details] = 5 children
        assert_eq!(layout.children.len(), 5);
    }

    #[test]
    fn breadcrumb_single_item() {
        let node: WidgetNode<Msg> = breadcrumb::<Msg>("bc")
            .item(BreadcrumbItem::current("Dashboard"))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Row: [Dashboard] = 1 child
        assert_eq!(layout.children.len(), 1);
    }

    #[test]
    fn breadcrumb_builder_defaults() {
        let b = breadcrumb::<Msg>("bc");
        assert!(b.items.is_empty());
        assert_eq!(b.separator, "/");
        assert_eq!(b.size, crate::ControlSize::Sm);
    }

    #[test]
    fn breadcrumb_custom_separator() {
        let node: WidgetNode<Msg> = breadcrumb::<Msg>("bc")
            .separator("›")
            .item(BreadcrumbItem::link("A", "a", Msg::Navigate))
            .item(BreadcrumbItem::current("B"))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Row: [A] [›] [B] = 3 children
        assert_eq!(layout.children.len(), 3);
    }
}
