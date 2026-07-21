//! Thumbnail placeholder.
use crate::WidgetNode;
pub fn thumbnail<M: Clone + 'static>(label: impl Into<String>) -> WidgetNode<M> {
    crate::card::<M>().child(crate::label(label)).build()
}
