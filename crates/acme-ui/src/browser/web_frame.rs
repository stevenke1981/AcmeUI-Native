use crate::WidgetNode;
pub fn web_frame<M: Clone + 'static>(url: impl Into<String>) -> WidgetNode<M> {
    crate::card::<M>().child(crate::label(url)).build()
}
