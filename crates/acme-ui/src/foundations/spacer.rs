//! Flexible spacer.
use crate::WidgetNode;
pub fn spacer<M: Clone + 'static>(height: f32) -> WidgetNode<M> {
    crate::column::<M>().height(height.max(0.0)).build()
}
