//! Quote block.
use crate::WidgetNode;
pub fn quote<M: Clone + 'static>(
    text: impl Into<String>,
    author: Option<impl Into<String>>,
) -> WidgetNode<M> {
    let mut c = crate::column::<M>().gap(4.0).child(crate::label(text));
    if let Some(a) = author {
        c = c.child(crate::label(a));
    }
    c.build()
}
