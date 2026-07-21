use crate::WidgetNode;
pub fn rich_text<M: Clone + 'static>(text: impl Into<String>) -> WidgetNode<M> {
    crate::label(text)
}
