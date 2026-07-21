use crate::WidgetNode;
pub fn pdf_viewer<M: Clone + 'static>(title: impl Into<String>) -> WidgetNode<M> {
    crate::card::<M>().child(crate::label(title)).build()
}
