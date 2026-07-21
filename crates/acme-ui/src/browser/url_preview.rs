use crate::WidgetNode;
pub fn url_preview<M: Clone + 'static>(url: impl Into<String>) -> WidgetNode<M> {
    crate::card::<M>().child(crate::label(url)).build()
}
