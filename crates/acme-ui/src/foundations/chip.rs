//! Small semantic chip.
use crate::WidgetNode;
pub fn chip<M: Clone + 'static>(text: impl Into<String>) -> WidgetNode<M> {
    crate::label(text)
}
