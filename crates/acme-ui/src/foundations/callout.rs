//! Callout content block.
use crate::WidgetNode;
pub fn callout<M: Clone + 'static>(
    title: impl Into<String>,
    body: impl Into<String>,
) -> WidgetNode<M> {
    crate::card::<M>()
        .child(
            crate::column::<M>()
                .gap(4.0)
                .child(crate::label(title))
                .child(crate::label(body))
                .build(),
        )
        .build()
}
