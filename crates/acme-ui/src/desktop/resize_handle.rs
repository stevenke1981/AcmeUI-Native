//! ResizeHandle — a draggable resize handle for split panes.

use crate::*;

/// Orientation of the resize handle.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Orientation {
    /// Drags left-right (vertical divider).
    #[default]
    Horizontal,
    /// Drags up-down (horizontal divider).
    Vertical,
}

/// Builder for a ResizeHandle component.
pub struct ResizeHandleBuilder<M> {
    pub id: WidgetKey,
    pub orientation: Orientation,
    pub thickness: f32,
    pub on_drag: Option<M>,
}

/// Create a new ResizeHandle builder.
pub fn resize_handle<M: Clone + 'static>(
    id: impl Into<WidgetKey>,
    orientation: Orientation,
) -> ResizeHandleBuilder<M> {
    ResizeHandleBuilder {
        id: id.into(),
        orientation,
        thickness: 6.0,
        on_drag: None,
    }
}

impl<M: Clone + 'static> ResizeHandleBuilder<M> {
    /// Set the handle thickness in pixels.
    pub fn thickness(mut self, value: f32) -> Self {
        self.thickness = value;
        self
    }

    /// Set the message dispatched while dragging.
    pub fn on_drag(mut self, msg: M) -> Self {
        self.on_drag = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<ResizeHandleBuilder<M>> for WidgetNode<M> {
    fn from(b: ResizeHandleBuilder<M>) -> Self {
        match b.orientation {
            Orientation::Horizontal => {
                // Vertical divider: width = thickness, height stretches
                column::<M>()
                    .key(b.id)
                    .width(b.thickness)
                    .child(
                        card::<M>()
                            .variant(CardVariant::Muted)
                            .child(label::<M>("║"))
                            .build(),
                    )
                    .build()
            }
            Orientation::Vertical => {
                // Horizontal divider: height = thickness, width stretches
                column::<M>()
                    .key(b.id)
                    .height(b.thickness)
                    .child(
                        card::<M>()
                            .variant(CardVariant::Muted)
                            .child(label::<M>("═"))
                            .build(),
                    )
                    .build()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;
    use acme_layout::{LayoutKind, Length};

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn resize_handle_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = resize_handle("rh", Orientation::Horizontal).into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert_eq!(layout.style.width, Length::px(6.0));
    }

    #[test]
    fn resize_handle_builder_defaults() {
        let rh = resize_handle::<TestMsg>("rh", Orientation::Vertical);
        assert_eq!(rh.orientation, Orientation::Vertical);
        assert!((rh.thickness - 6.0).abs() < f32::EPSILON);
        assert!(rh.on_drag.is_none());
    }

    #[test]
    fn resize_handle_field_setters_work() {
        #[derive(Clone, Debug, PartialEq)]
        enum Msg {
            Drag(f32),
        }

        let rh = resize_handle::<Msg>("rh", Orientation::Horizontal)
            .thickness(10.0)
            .on_drag(Msg::Drag(0.0));

        assert!((rh.thickness - 10.0).abs() < f32::EPSILON);
        assert_eq!(rh.on_drag, Some(Msg::Drag(0.0)));
    }
}
