//! DataList — key-value pair data display.
//! Aligns with Radix Themes Data List.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Data list orientation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DataListOrientation {
    #[default]
    Vertical,
    Horizontal,
}

/// A single data list item (key-value pair).
#[derive(Clone, Debug)]
pub struct DataListItem {
    pub label: String,
    pub value: String,
}

impl DataListItem {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

/// Builder for a data list.
pub struct DataListBuilder<M> {
    pub id: WidgetKey,
    pub items: Vec<DataListItem>,
    pub orientation: DataListOrientation,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a data list builder.
pub fn data_list<M: Clone + 'static>() -> DataListBuilder<M> {
    DataListBuilder {
        id: WidgetKey::from("data_list"),
        items: Vec::new(),
        orientation: DataListOrientation::default(),
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> DataListBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn item(mut self, item: DataListItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn orientation(mut self, value: DataListOrientation) -> Self {
        self.orientation = value;
        self
    }
}

impl<M: Clone + 'static> From<DataListBuilder<M>> for WidgetNode<M> {
    fn from(b: DataListBuilder<M>) -> Self {
        let mut col = crate::column::<M>().key(b.id).gap(8.0).padding(4.0);
        for item in &b.items {
            let row = match b.orientation {
                DataListOrientation::Horizontal => crate::row::<M>()
                    .gap(12.0)
                    .child(crate::label(item.label.clone()))
                    .child(crate::label(item.value.clone()))
                    .build(),
                DataListOrientation::Vertical => crate::column::<M>()
                    .gap(2.0)
                    .child(crate::label(item.label.clone()))
                    .child(crate::label(item.value.clone()))
                    .build(),
            };
            col = col.child(row);
        }
        col.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn data_list_produces_column() {
        let node: WidgetNode<Msg> = data_list()
            .item(DataListItem::new("Name", "Alice"))
            .into();
        assert!(matches!(node, WidgetNode::Column(_)));
    }

    #[test]
    fn data_list_child_count() {
        let node: WidgetNode<Msg> = data_list()
            .item(DataListItem::new("Name", "Alice"))
            .item(DataListItem::new("Age", "30"))
            .into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column");
        };
        assert_eq!(c.children.len(), 2);
    }

    #[test]
    fn data_list_horizontal_orientation() {
        let b = data_list::<Msg>().orientation(DataListOrientation::Horizontal);
        assert_eq!(b.orientation, DataListOrientation::Horizontal);
    }
}
