use crate::WidgetNode;
pub fn code_viewer<M: Clone + 'static>(code: impl Into<String>) -> WidgetNode<M> {
    crate::card::<M>().child(crate::label(code)).build()
}
