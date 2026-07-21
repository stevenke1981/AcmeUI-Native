//! Compact metric card.
use crate::WidgetNode;
pub fn metric_card<M: Clone + 'static>(
    label: impl Into<String>,
    value: impl Into<String>,
) -> WidgetNode<M> {
    crate::card::<M>()
        .child(
            crate::column::<M>()
                .gap(4.0)
                .child(crate::label(label))
                .child(crate::label(value))
                .build(),
        )
        .build()
}
