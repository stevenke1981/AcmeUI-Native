//! AppBar — top application bar with title and actions.
//! Aligns with MUI AppBar component.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// AppBar position.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AppBarPosition {
    #[default]
    Static,
    Fixed,
    Sticky,
}

/// Builder for an app bar.
pub struct AppBarBuilder<M> {
    pub id: WidgetKey,
    pub title: String,
    pub position: AppBarPosition,
    pub leading: Option<WidgetNode<M>>,
    pub actions: Vec<WidgetNode<M>>,
}

/// Create an app bar builder.
pub fn app_bar<M: Clone + 'static>(title: impl Into<String>) -> AppBarBuilder<M> {
    AppBarBuilder {
        id: WidgetKey::from("app_bar"),
        title: title.into(),
        position: AppBarPosition::default(),
        leading: None,
        actions: Vec::new(),
    }
}

impl<M: Clone + 'static> AppBarBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn position(mut self, value: AppBarPosition) -> Self {
        self.position = value;
        self
    }

    pub fn leading(mut self, node: WidgetNode<M>) -> Self {
        self.leading = Some(node);
        self
    }

    pub fn action(mut self, node: WidgetNode<M>) -> Self {
        self.actions.push(node);
        self
    }
}

impl<M: Clone + 'static> From<AppBarBuilder<M>> for WidgetNode<M> {
    fn from(b: AppBarBuilder<M>) -> Self {
        let mut row = crate::row::<M>()
            .key(b.id)
            .gap(12.0)
            .padding(16.0)
            .height(64.0);

        if let Some(leading) = b.leading {
            row = row.child(leading);
        }
        row = row.child(crate::label(b.title));

        for action in b.actions {
            row = row.child(action);
        }
        row.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn app_bar_produces_row() {
        let node: WidgetNode<Msg> = app_bar("My App").into();
        assert!(matches!(node, WidgetNode::Row(_)));
    }

    #[test]
    fn app_bar_with_leading_and_actions() {
        let node: WidgetNode<Msg> = app_bar("Title")
            .leading(crate::label("☰"))
            .action(crate::label("⌕"))
            .action(crate::label("⋮"))
            .into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        // leading + title + 2 actions = 4
        assert_eq!(r.children.len(), 4);
    }

    #[test]
    fn app_bar_default_position_static() {
        let b = app_bar::<Msg>("X");
        assert_eq!(b.position, AppBarPosition::Static);
    }
}
