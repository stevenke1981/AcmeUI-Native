//! Inset — content inset within a surface (flush-edge content).
//! Aligns with Radix Themes Inset.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Inset side.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum InsetSide {
    #[default]
    All,
    Top,
    Bottom,
    Left,
    Right,
    X,
    Y,
}

/// Builder for an inset container.
pub struct InsetBuilder<M> {
    pub id: WidgetKey,
    pub side: InsetSide,
    pub child: Option<WidgetNode<M>>,
}

/// Create an inset builder.
pub fn inset<M: Clone + 'static>() -> InsetBuilder<M> {
    InsetBuilder {
        id: WidgetKey::from("inset"),
        side: InsetSide::default(),
        child: None,
    }
}

impl<M: Clone + 'static> InsetBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn side(mut self, value: InsetSide) -> Self {
        self.side = value;
        self
    }

    pub fn child(mut self, node: WidgetNode<M>) -> Self {
        self.child = Some(node);
        self
    }
}

impl<M: Clone + 'static> From<InsetBuilder<M>> for WidgetNode<M> {
    fn from(b: InsetBuilder<M>) -> Self {
        let mut container = crate::column::<M>().key(b.id).padding(0.0);
        if let Some(child) = b.child {
            container = container.child(child);
        }
        container.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn inset_produces_column() {
        let node: WidgetNode<Msg> = inset().child(crate::label("content")).into();
        assert!(matches!(node, WidgetNode::Column(_)));
    }

    #[test]
    fn inset_with_child() {
        let node: WidgetNode<Msg> = inset().child(crate::label("img")).into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column");
        };
        assert_eq!(c.children.len(), 1);
    }

    #[test]
    fn inset_default_side_all() {
        let b = inset::<Msg>();
        assert_eq!(b.side, InsetSide::All);
    }
}
