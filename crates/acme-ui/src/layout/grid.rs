//! Grid component — a column of rows, each row up to `cols` children.

use crate::WidgetNode;

/// Builder for a grid layout.
pub struct GridBuilder<M> {
    pub cols: usize,
    pub gap: f32,
    pub children: Vec<WidgetNode<M>>,
}

/// Create a grid builder.
pub fn grid<M: Clone + 'static>() -> GridBuilder<M> {
    GridBuilder {
        cols: 2,
        gap: 0.0,
        children: vec![],
    }
}

impl<M: Clone> GridBuilder<M> {
    /// Set the number of columns.
    pub fn cols(mut self, n: usize) -> Self {
        self.cols = n.max(1);
        self
    }

    /// Set the gap between cells.
    pub fn gap(mut self, value: f32) -> Self {
        self.gap = value;
        self
    }

    /// Add a child widget.
    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Build the widget node tree.
    pub fn build(self) -> WidgetNode<M> {
        if self.cols == 0 || self.children.is_empty() {
            return crate::column::<M>().build();
        }

        let mut rows: Vec<WidgetNode<M>> = Vec::new();
        for chunk in self.children.chunks(self.cols) {
            let mut row_builder = crate::row::<M>().gap(self.gap);
            for child in chunk {
                row_builder = row_builder.child(child.clone());
            }
            rows.push(row_builder.build());
        }

        let mut col = crate::column::<M>().gap(self.gap);
        for row_node in rows {
            col = col.child(row_node);
        }
        col.build()
    }
}

impl<M: Clone + 'static> From<GridBuilder<M>> for WidgetNode<M> {
    fn from(b: GridBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::label;
    use acme_core::NodeId;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn grid_has_non_zero_layout_rect() {
        let node: WidgetNode<Msg> = grid::<Msg>()
            .cols(3)
            .gap(8.0)
            .child(label("A"))
            .child(label("B"))
            .child(label("C"))
            .child(label("D"))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Column with 2 rows (4 items ÷ 3 cols = 2 rows)
        assert_eq!(layout.children.len(), 2);
        // First row has 3 items
        assert_eq!(layout.children[0].children.len(), 3);
        // Second row has 1 item
        assert_eq!(layout.children[1].children.len(), 1);
    }

    #[test]
    fn grid_single_row() {
        let node: WidgetNode<Msg> = grid::<Msg>().cols(5).child(label("only")).into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.children.len(), 1);
        assert_eq!(layout.children[0].children.len(), 1);
    }

    #[test]
    fn grid_empty() {
        let node: WidgetNode<Msg> = grid::<Msg>().cols(3).build();
        let layout = node.to_layout(NodeId::new(1));
        // Empty column
        assert!(layout.children.is_empty());
    }
}
