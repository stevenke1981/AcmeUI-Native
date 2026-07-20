use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_layout::{LayoutKind, LayoutStyle, Length, Overflow};

/// A scrollable viewport.
#[derive(Clone, Debug, PartialEq)]
pub struct ScrollView<M> {
    pub key: WidgetKey,
    pub children: Vec<WidgetNode<M>>,
    pub viewport_height: Length,
}
impl<M> ScrollView<M> {
    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.children.push(child.into());
        self
    }
    pub fn viewport_height(mut self, value: f32) -> Self {
        self.viewport_height = Length::px(value);
        self
    }
    pub fn build(self) -> WidgetNode<M> {
        WidgetNode::ScrollView(self)
    }
    pub(crate) fn layout(&self) -> LayoutStyle {
        LayoutStyle {
            kind: LayoutKind::Column,
            height: self.viewport_height,
            overflow: Overflow::Scroll,
            ..Default::default()
        }
    }
}

/// Create a scroll view builder.
pub fn scroll_view<M>(key: impl Into<WidgetKey>) -> ScrollView<M> {
    ScrollView {
        key: key.into(),
        children: vec![],
        viewport_height: Length::Auto,
    }
}
