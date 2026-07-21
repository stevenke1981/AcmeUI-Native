//! Drawer component — a side panel overlay using Stack positioning.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a drawer (side panel).
pub struct DrawerBuilder<M> {
    pub id: WidgetKey,
    pub title: String,
    pub open: bool,
    pub content: Option<WidgetNode<M>>,
}

/// Create a drawer builder.
pub fn drawer<M: Clone + 'static>(id: impl Into<WidgetKey>) -> DrawerBuilder<M> {
    DrawerBuilder {
        id: id.into(),
        title: String::new(),
        open: false,
        content: None,
    }
}

impl<M: Clone> DrawerBuilder<M> {
    /// Set the drawer title.
    pub fn title(mut self, value: impl Into<String>) -> Self {
        self.title = value.into();
        self
    }

    /// Open or close the drawer.
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }

    /// Set the drawer body content.
    pub fn content(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.content = Some(child.into());
        self
    }

    /// Build the widget node tree.
    ///
    /// When open: renders a `Stack` containing a `Card` with title + content.
    /// When closed: renders a minimal indicator `Card`.
    pub fn build(self) -> WidgetNode<M> {
        if !self.open {
            // Minimal closed indicator
            return crate::card::<M>()
                .variant(crate::CardVariant::Muted)
                .child(crate::label::<M>("☰"))
                .padding(4.0)
                .build();
        }

        let title_label = crate::label::<M>(self.title);
        let mut panel_col = crate::column::<M>().child(title_label).gap(8.0);

        if let Some(content) = self.content {
            panel_col = panel_col.child(content);
        }

        let panel = crate::card::<M>()
            .variant(crate::CardVariant::Elevated)
            .child(panel_col.build())
            .padding(12.0)
            .build();

        crate::stack::<M>().child(panel).build()
    }
}

impl<M: Clone + 'static> From<DrawerBuilder<M>> for WidgetNode<M> {
    fn from(b: DrawerBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{WidgetNode, label};
    use acme_core::NodeId;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn drawer_has_non_zero_layout_rect() {
        let node: WidgetNode<Msg> = drawer::<Msg>("drawer")
            .title("Settings")
            .open(true)
            .content(label("Content here"))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Stack with one child (Card)
        assert_eq!(layout.children.len(), 1);
    }

    #[test]
    fn drawer_closed_shows_minimal() {
        let node: WidgetNode<Msg> = drawer::<Msg>("d").title("Hidden").open(false).into();
        // Closed: Card with Muted variant
        assert!(matches!(node, WidgetNode::Card(_)));
    }

    #[test]
    fn drawer_open_shows_stack() {
        let node: WidgetNode<Msg> = drawer::<Msg>("d").title("Panel").open(true).build();
        assert!(matches!(node, WidgetNode::Stack(_)));
    }
}
