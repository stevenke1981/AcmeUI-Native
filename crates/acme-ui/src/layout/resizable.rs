//! Resizable — draggable panel resizing with configurable direction.
//! Aligns with shadcn/ui Resizable.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Resize direction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ResizeDirection {
    #[default]
    Horizontal,
    Vertical,
}

/// Builder for a resizable panel group.
pub struct ResizableBuilder<M> {
    pub id: WidgetKey,
    pub direction: ResizeDirection,
    pub panels: Vec<WidgetNode<M>>,
    pub handle_width: f32,
}

/// Create a resizable panel group builder.
pub fn resizable<M: Clone + 'static>() -> ResizableBuilder<M> {
    ResizableBuilder {
        id: WidgetKey::from("resizable"),
        direction: ResizeDirection::default(),
        panels: Vec::new(),
        handle_width: 4.0,
    }
}

impl<M: Clone + 'static> ResizableBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn direction(mut self, value: ResizeDirection) -> Self {
        self.direction = value;
        self
    }

    pub fn panel(mut self, node: WidgetNode<M>) -> Self {
        self.panels.push(node);
        self
    }

    pub fn handle_width(mut self, value: f32) -> Self {
        self.handle_width = value;
        self
    }
}

impl<M: Clone + 'static> From<ResizableBuilder<M>> for WidgetNode<M> {
    fn from(b: ResizableBuilder<M>) -> Self {
        let is_horizontal = b.direction == ResizeDirection::Horizontal;
        let mut container = if is_horizontal {
            crate::row::<M>().key(b.id).gap(0.0)
        } else {
            crate::column::<M>().key(b.id).gap(0.0)
        };

        for (i, panel) in b.panels.into_iter().enumerate() {
            if i > 0 {
                // Insert drag handle between panels
                let handle = if is_horizontal {
                    crate::row::<M>()
                        .width(b.handle_width)
                        .child(crate::label("⋮"))
                        .build()
                } else {
                    crate::row::<M>()
                        .height(b.handle_width)
                        .child(crate::label("⋯"))
                        .build()
                };
                container = container.child(handle);
            }
            container = container.child(panel);
        }
        container.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn resizable_horizontal_produces_row() {
        let node: WidgetNode<Msg> = resizable()
            .panel(crate::label("A"))
            .panel(crate::label("B"))
            .into();
        assert!(matches!(node, WidgetNode::Row(_)));
    }

    #[test]
    fn resizable_vertical_produces_column() {
        let node: WidgetNode<Msg> = resizable()
            .direction(ResizeDirection::Vertical)
            .panel(crate::label("A"))
            .panel(crate::label("B"))
            .into();
        assert!(matches!(node, WidgetNode::Column(_)));
    }

    #[test]
    fn resizable_inserts_handles_between_panels() {
        let node: WidgetNode<Msg> = resizable()
            .panel(crate::label("A"))
            .panel(crate::label("B"))
            .panel(crate::label("C"))
            .into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        // 3 panels + 2 handles = 5
        assert_eq!(r.children.len(), 5);
    }
}
