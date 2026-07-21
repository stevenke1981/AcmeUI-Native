//! Multi-line text block.
use crate::WidgetNode;
pub fn text_block<M: Clone + 'static>(text: impl Into<String>) -> WidgetNode<M> {
    crate::label(text)
}
