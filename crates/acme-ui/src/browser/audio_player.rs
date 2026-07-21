use crate::WidgetNode;
pub fn audio_player<M: Clone + 'static>(title: impl Into<String>) -> WidgetNode<M> {
    crate::card::<M>().child(crate::label(title)).build()
}
