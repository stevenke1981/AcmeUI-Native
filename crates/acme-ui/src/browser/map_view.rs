use crate::WidgetNode;
pub fn map_view<M: Clone + 'static>(location: impl Into<String>) -> WidgetNode<M> {
    crate::card::<M>().child(crate::label(location)).build()
}
