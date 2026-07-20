use crate::WidgetNode;

/// A horizontal line separator.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Separator {
    pub thickness: f32,
}

/// Create a separator widget.
pub fn separator<M>() -> WidgetNode<M> {
    WidgetNode::Separator(Separator { thickness: 1.0 })
}
