use crate::WidgetNode;
use acme_core::WidgetKey;

/// A tooltip that wraps a child and shows text on hover.
#[derive(Clone, Debug, PartialEq)]
pub struct Tooltip<M> {
    pub key: WidgetKey,
    pub child: Box<WidgetNode<M>>,
    pub text: String,
    pub delay_ms: u64,
}

/// Create a tooltip builder.
pub fn tooltip<M>(
    key: impl Into<WidgetKey>,
    child: impl Into<WidgetNode<M>>,
    text: impl Into<String>,
) -> Tooltip<M> {
    Tooltip {
        key: key.into(),
        child: Box::new(child.into()),
        text: text.into(),
        delay_ms: 500,
    }
}

impl<M> From<Tooltip<M>> for WidgetNode<M> {
    fn from(value: Tooltip<M>) -> Self {
        WidgetNode::Tooltip(value)
    }
}
