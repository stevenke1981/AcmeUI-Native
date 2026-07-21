//! Status dot with accessible text label.
use crate::WidgetNode;
pub fn status_dot<M: Clone + 'static>(status: impl Into<String>) -> WidgetNode<M> {
    crate::label(status)
}
