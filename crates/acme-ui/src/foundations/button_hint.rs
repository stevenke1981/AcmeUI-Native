//! Button hint row for keyboard-oriented interfaces.
use crate::WidgetNode;
pub fn button_hint<M: Clone + 'static>(
    label: impl Into<String>,
    hint: impl Into<String>,
) -> WidgetNode<M> {
    crate::row::<M>()
        .gap(8.0)
        .child(crate::label(label))
        .child(crate::label(hint))
        .build()
}
