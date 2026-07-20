use crate::WidgetNode;
use acme_core::WidgetKey;

/// Popover placement relative to anchor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PopoverPlacement {
    Bottom,
    Top,
    Left,
    Right,
}

/// A popover attached to an anchor element.
#[derive(Clone, Debug, PartialEq)]
pub struct Popover<M> {
    pub key: WidgetKey,
    /// First element is anchor, second is content.
    pub children: Vec<WidgetNode<M>>,
    pub open: bool,
    pub placement: PopoverPlacement,
}

/// Create a popover builder.
pub fn popover<M>(key: impl Into<WidgetKey>, anchor: impl Into<WidgetNode<M>>) -> Popover<M> {
    Popover {
        key: key.into(),
        children: vec![anchor.into()],
        open: false,
        placement: PopoverPlacement::Bottom,
    }
}

impl<M> Popover<M> {
    pub fn content(mut self, content: impl Into<WidgetNode<M>>) -> Self {
        self.children.push(content.into());
        self
    }
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }
    pub fn placement(mut self, value: PopoverPlacement) -> Self {
        self.placement = value;
        self
    }
    pub fn build(self) -> WidgetNode<M> {
        WidgetNode::Popover(self)
    }
}

impl<M> From<Popover<M>> for WidgetNode<M> {
    fn from(value: Popover<M>) -> Self {
        WidgetNode::Popover(value)
    }
}
