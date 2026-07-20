//! Pagination component — prev / page labels / next controls.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for pagination controls.
pub struct PaginationBuilder<M> {
    pub id: WidgetKey,
    pub current: usize,
    pub total: usize,
    pub on_page_change: Option<M>,
}

/// Create a pagination builder.
pub fn pagination<M: Clone + 'static>(id: impl Into<WidgetKey>) -> PaginationBuilder<M> {
    PaginationBuilder {
        id: id.into(),
        current: 1,
        total: 1,
        on_page_change: None,
    }
}

impl<M: Clone> PaginationBuilder<M> {
    /// Set the current page (1-indexed).
    pub fn current(mut self, page: usize) -> Self {
        self.current = page;
        self
    }

    /// Set the total number of pages.
    pub fn total(mut self, n: usize) -> Self {
        self.total = n.max(1);
        self
    }

    /// Set the message fired on page change.
    pub fn on_page_change(mut self, msg: M) -> Self {
        self.on_page_change = Some(msg);
        self
    }

    /// Build the widget node tree.
    pub fn build(self) -> WidgetNode<M> {
        let id_prefix = self.id.as_str().to_string();
        let mut row = crate::row::<M>().gap(4.0);

        // Previous button
        let prev_key = format!("{id_prefix}_prev");
        let prev_disabled = self.current <= 1;
        if let Some(ref msg) = self.on_page_change {
            let mut b = crate::button(prev_key.as_str(), "◂");
            if prev_disabled {
                b = b.disabled(true);
            }
            row = row.child(b.on_click(msg.clone()));
        } else {
            row = row.child(crate::button(prev_key.as_str(), "◂").disabled(prev_disabled));
        }

        // Page labels
        for p in 1..=self.total {
            let label = if p == self.current {
                format!("[{p}]")
            } else {
                p.to_string()
            };
            row = row.child(crate::label::<M>(label));
        }

        // Next button
        let next_key = format!("{id_prefix}_next");
        let next_disabled = self.current >= self.total;
        if let Some(ref msg) = self.on_page_change {
            let mut b = crate::button(next_key.as_str(), "▸");
            if next_disabled {
                b = b.disabled(true);
            }
            row = row.child(b.on_click(msg.clone()));
        } else {
            row = row.child(crate::button(next_key.as_str(), "▸").disabled(next_disabled));
        }

        row.build()
    }
}

impl<M: Clone + 'static> From<PaginationBuilder<M>> for WidgetNode<M> {
    fn from(b: PaginationBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WidgetNode;
    use acme_core::NodeId;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        ChangePage,
    }

    #[test]
    fn pagination_has_non_zero_layout_rect() {
        let node: WidgetNode<Msg> = pagination::<Msg>("pages")
            .current(2)
            .total(5)
            .on_page_change(Msg::ChangePage)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Row: prev + 5 page labels + next = 7 children
        assert_eq!(layout.children.len(), 7);
    }

    #[test]
    fn pagination_single_page() {
        let node: WidgetNode<Msg> = pagination::<Msg>("p").current(1).total(1).into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.children.len(), 3); // prev + 1 label + next
    }

    #[test]
    fn pagination_key_is_stored() {
        let b = pagination::<Msg>("my-paginator");
        assert_eq!(b.id.as_str(), "my-paginator");
    }
}
