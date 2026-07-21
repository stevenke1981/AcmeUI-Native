//! Notice banner shorthand.
use crate::WidgetNode;
pub fn notice<M: Clone + 'static>(text: impl Into<String>) -> WidgetNode<M> {
    crate::card::<M>().child(crate::label(text)).build()
}
