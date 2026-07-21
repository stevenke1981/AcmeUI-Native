//! Masonry-style responsive content layout.

use crate::WidgetNode;

/// Builder for a simple balanced masonry layout.
pub struct MasonryBuilder<M> {
    columns: usize,
    gap: f32,
    children: Vec<WidgetNode<M>>,
}

/// Create a masonry layout with the requested column count.
pub fn masonry<M: Clone + 'static>(columns: usize) -> MasonryBuilder<M> {
    MasonryBuilder {
        columns: columns.max(1),
        gap: 12.0,
        children: Vec::new(),
    }
}

impl<M: Clone + 'static> MasonryBuilder<M> {
    /// Set spacing between columns and items.
    pub fn gap(mut self, value: f32) -> Self {
        self.gap = value.max(0.0);
        self
    }

    /// Add an item; items are assigned to columns in order.
    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Build the masonry row.
    pub fn build(self) -> WidgetNode<M> {
        let mut items: Vec<Vec<WidgetNode<M>>> = (0..self.columns).map(|_| Vec::new()).collect();
        for (index, child) in self.children.into_iter().enumerate() {
            items[index % self.columns].push(child);
        }
        let mut row = crate::row::<M>().gap(self.gap);
        for children in items {
            let mut column = crate::column::<M>().gap(self.gap);
            for child in children {
                column = column.child(child);
            }
            row = row.child(column.build());
        }
        row.build()
    }
}

impl<M: Clone + 'static> From<MasonryBuilder<M>> for WidgetNode<M> {
    fn from(value: MasonryBuilder<M>) -> Self {
        value.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masonry_creates_requested_columns() {
        let node = masonry::<()>(3)
            .child(crate::label("A"))
            .child(crate::label("B"))
            .build();
        assert_eq!(node.children().len(), 3);
    }
}
