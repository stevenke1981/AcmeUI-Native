//! Affix — sticky/fixed positioning wrapper.
//! Aligns with Ant Design Affix component.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Affix position.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AffixPosition {
    #[default]
    Top,
    Bottom,
}

/// Builder for an affix wrapper.
pub struct AffixBuilder<M> {
    pub id: WidgetKey,
    pub position: AffixPosition,
    pub offset: f32,
    pub child: Option<WidgetNode<M>>,
}

/// Create an affix builder.
pub fn affix<M: Clone + 'static>() -> AffixBuilder<M> {
    AffixBuilder {
        id: WidgetKey::from("affix"),
        position: AffixPosition::default(),
        offset: 0.0,
        child: None,
    }
}

impl<M: Clone + 'static> AffixBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn position(mut self, value: AffixPosition) -> Self {
        self.position = value;
        self
    }

    pub fn offset(mut self, value: f32) -> Self {
        self.offset = value;
        self
    }

    pub fn child(mut self, node: WidgetNode<M>) -> Self {
        self.child = Some(node);
        self
    }
}

impl<M: Clone + 'static> From<AffixBuilder<M>> for WidgetNode<M> {
    fn from(b: AffixBuilder<M>) -> Self {
        let mut col = crate::column::<M>().key(b.id).padding(b.offset);
        if let Some(child) = b.child {
            col = col.child(child);
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
    fn affix_produces_column() {
        let node: WidgetNode<Msg> = affix().child(crate::label("sticky")).into();
        assert!(matches!(node, WidgetNode::Column(_)));
    }

    #[test]
    fn affix_default_position_top() {
        let b = affix::<Msg>();
        assert_eq!(b.position, AffixPosition::Top);
    }

    #[test]
    fn affix_with_offset() {
        let b = affix::<Msg>().offset(64.0);
        assert_eq!(b.offset, 64.0);
    }
}
