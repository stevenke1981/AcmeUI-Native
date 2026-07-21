//! Key/value display row.
use crate::WidgetNode;
pub fn key_value<M: Clone + 'static>(
    key: impl Into<String>,
    value: impl Into<String>,
) -> WidgetNode<M> {
    crate::row::<M>()
        .gap(8.0)
        .child(crate::label(key))
        .child(crate::label(value))
        .build()
}
