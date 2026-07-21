use crate::WidgetNode;
pub fn bubble_chart<M: Clone + 'static>(title: impl Into<String>) -> WidgetNode<M> {
    crate::card::<M>().child(crate::label(title)).build()
}
