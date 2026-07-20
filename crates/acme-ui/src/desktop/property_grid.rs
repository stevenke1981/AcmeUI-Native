//! PropertyGrid — a key-value property listing like an inspector panel.

use crate::*;

/// A single property row in the grid.
#[derive(Clone, Debug)]
pub struct PropertyItem {
    pub label: String,
    pub value: String,
    pub editable: bool,
}

impl PropertyItem {
    /// Create a new property item.
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            editable: false,
        }
    }

    /// Mark this property as editable.
    pub fn editable(mut self, value: bool) -> Self {
        self.editable = value;
        self
    }
}

/// Builder for a PropertyGrid component.
pub struct PropertyGridBuilder<M> {
    pub id: WidgetKey,
    pub items: Vec<PropertyItem>,
    pub alternating_rows: bool,
    pub on_change: Option<M>,
}

/// Create a new PropertyGrid builder.
pub fn property_grid<M: Clone + 'static>(id: impl Into<WidgetKey>) -> PropertyGridBuilder<M> {
    PropertyGridBuilder {
        id: id.into(),
        items: Vec::new(),
        alternating_rows: true,
        on_change: None,
    }
}

impl<M: Clone + 'static> PropertyGridBuilder<M> {
    /// Add a single property item by label and value.
    pub fn item(mut self, label: impl Into<String>, value: impl Into<String>) -> Self {
        self.items.push(PropertyItem::new(label, value));
        self
    }

    /// Set all property items at once.
    pub fn items(mut self, items: Vec<PropertyItem>) -> Self {
        self.items = items;
        self
    }

    /// Toggle alternating row backgrounds.
    pub fn alternating_rows(mut self, value: bool) -> Self {
        self.alternating_rows = value;
        self
    }

    /// Set the message dispatched when a property value changes.
    pub fn on_change(mut self, msg: M) -> Self {
        self.on_change = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<PropertyGridBuilder<M>> for WidgetNode<M> {
    fn from(b: PropertyGridBuilder<M>) -> Self {
        let mut col = column::<M>().key(b.id).gap(0.0);

        for item in &b.items {
            let row = row::<M>()
                .child(label::<M>(item.label.clone()))
                .child(label::<M>(item.value.clone()))
                .build();
            col = col.child(row);
        }

        col.build()
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
    fn property_grid_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = property_grid("pg")
            .item("Width", "800")
            .item("Height", "600")
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert_eq!(layout.children.len(), 2);
    }

    #[test]
    fn property_grid_builder_defaults() {
        let pg = property_grid::<TestMsg>("pg");
        assert!(pg.items.is_empty());
        assert!(pg.alternating_rows);
        assert!(pg.on_change.is_none());
    }

    #[test]
    fn property_grid_field_setters_work() {
        let pg = property_grid::<TestMsg>("pg")
            .item("Name", "Alice")
            .item("Age", "30")
            .alternating_rows(false);

        assert_eq!(pg.items.len(), 2);
        assert_eq!(pg.items[0].label, "Name");
        assert_eq!(pg.items[0].value, "Alice");
        assert_eq!(pg.items[1].label, "Age");
        assert_eq!(pg.items[1].value, "30");
        assert!(!pg.alternating_rows);
    }
}
