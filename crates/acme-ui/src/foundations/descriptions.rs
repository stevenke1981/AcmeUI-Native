//! Descriptions component — a description list with title and key-value rows.
//!
//! Renders as a Column with optional title and a list of label-value item Rows.

use std::marker::PhantomData;
use acme_core::WidgetKey;
use acme_widgets::*;

/// A single description item with label and value.
#[derive(Clone, Debug)]
pub struct DescriptionItem {
    pub label: String,
    pub value: String,
}

/// Builder for a Descriptions component.
pub struct DescriptionsBuilder<M> {
    pub id: WidgetKey,
    pub title: Option<String>,
    pub items: Vec<DescriptionItem>,
    pub column: usize,
    pub phantom: PhantomData<M>,
}

/// Create a new Descriptions builder.
pub fn descriptions<M: Clone + 'static>(id: impl Into<WidgetKey>) -> DescriptionsBuilder<M> {
    DescriptionsBuilder {
        id: id.into(),
        title: None,
        items: vec![],
        column: 1,
        phantom: PhantomData,
    }
}

/// Create a description list item.
pub fn description_item(label: impl Into<String>, value: impl Into<String>) -> DescriptionItem {
    DescriptionItem {
        label: label.into(),
        value: value.into(),
    }
}

impl<M: Clone + 'static> DescriptionsBuilder<M> {
    /// Set the optional title text.
    pub fn title(mut self, value: impl Into<String>) -> Self {
        self.title = Some(value.into());
        self
    }

    /// Add a single description item.
    pub fn item(mut self, item: DescriptionItem) -> Self {
        self.items.push(item);
        self
    }

    /// Set all items at once.
    pub fn items(mut self, items: Vec<DescriptionItem>) -> Self {
        self.items = items;
        self
    }

    /// Set the number of columns (default 1).
    pub fn column(mut self, value: usize) -> Self {
        self.column = value;
        self
    }
}

impl<M: Clone + 'static> From<DescriptionsBuilder<M>> for WidgetNode<M> {
    fn from(b: DescriptionsBuilder<M>) -> Self {
        let mut col = column::<M>().gap(4.0);

        if let Some(t) = &b.title {
            col = col.child(label::<M>(t.clone()));
        }

        for item in &b.items {
            let row = row::<M>()
                .gap(8.0)
                .child(
                    card::<M>()
                        .variant(CardVariant::Muted)
                        .padding(6.0)
                        .child(label::<M>(item.label.clone())),
                )
                .child(
                    card::<M>()
                        .variant(CardVariant::Plain)
                        .padding(6.0)
                        .child(label::<M>(item.value.clone())),
                );
            col = col.child(row);
        }

        col.key(b.id).build()
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
    fn descriptions_builder_defaults() {
        let d = descriptions::<TestMsg>("d");
        assert!(d.title.is_none());
        assert!(d.items.is_empty());
        assert_eq!(d.column, 1);
    }

    #[test]
    fn descriptions_renders_column() {
        let node: WidgetNode<TestMsg> = descriptions("d")
            .item(description_item("Name", "Alice"))
            .item(description_item("Role", "Developer"))
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        // 2 items = 2 children (no title)
        assert_eq!(col.children.len(), 2);
    }

    #[test]
    fn descriptions_with_title() {
        let node: WidgetNode<TestMsg> = descriptions("d")
            .title("User Info")
            .item(description_item("Email", "a@b.com"))
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        // title + 1 item = 2 children
        assert_eq!(col.children.len(), 2);
    }

    #[test]
    fn description_item_has_correct_fields() {
        let item = description_item("Key", "Value");
        assert_eq!(item.label, "Key");
        assert_eq!(item.value, "Value");
    }

    #[test]
    fn descriptions_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = descriptions("d")
            .item(description_item("A", "1"))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert!(!layout.children.is_empty());
    }
}
