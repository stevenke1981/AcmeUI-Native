//! Media card placeholder with title and caption.
use crate::WidgetNode;
pub fn media_card<M: Clone + 'static>(
    title: impl Into<String>,
    caption: impl Into<String>,
) -> WidgetNode<M> {
    crate::card::<M>()
        .child(
            crate::column::<M>()
                .gap(4.0)
                .child(crate::label(title))
                .child(crate::label(caption))
                .build(),
        )
        .build()
}
