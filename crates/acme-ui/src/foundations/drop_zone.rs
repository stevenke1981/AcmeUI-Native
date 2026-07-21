//! DropZone component — a drop target area with visual feedback state.
//!
//! Renders as a column with explicit height wrapping a Card:
//! - Normal state: Outlined card with icon + label
//! - Active state: Interactive card with different label text

use crate::*;

/// Builder for a DropZone component.
pub struct DropZoneBuilder<M> {
    pub id: WidgetKey,
    pub label: String,
    pub active: bool,
    pub height: f32,
    pub on_drop: Option<M>,
    pub on_drag_over: Option<M>,
    pub on_drag_leave: Option<M>,
}

/// Create a new DropZone builder.
pub fn drop_zone<M: Clone + 'static>(
    id: impl Into<WidgetKey>,
    label: impl Into<String>,
) -> DropZoneBuilder<M> {
    DropZoneBuilder {
        id: id.into(),
        label: label.into(),
        active: false,
        height: 120.0,
        on_drop: None,
        on_drag_over: None,
        on_drag_leave: None,
    }
}

impl<M: Clone + 'static> DropZoneBuilder<M> {
    /// Set whether a drag is currently hovering over the zone.
    pub fn active(mut self, value: bool) -> Self {
        self.active = value;
        self
    }

    /// Set the drop zone height in pixels.
    pub fn height(mut self, value: f32) -> Self {
        self.height = value;
        self
    }

    /// Set the message dispatched when content is dropped.
    pub fn on_drop(mut self, msg: M) -> Self {
        self.on_drop = Some(msg);
        self
    }

    /// Set the message dispatched when a drag enters / moves over.
    pub fn on_drag_over(mut self, msg: M) -> Self {
        self.on_drag_over = Some(msg);
        self
    }

    /// Set the message dispatched when a drag leaves the zone.
    pub fn on_drag_leave(mut self, msg: M) -> Self {
        self.on_drag_leave = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<DropZoneBuilder<M>> for WidgetNode<M> {
    fn from(b: DropZoneBuilder<M>) -> Self {
        let variant = if b.active {
            CardVariant::Interactive
        } else {
            CardVariant::Outlined
        };

        let display_text = if b.active {
            "Release to drop..."
        } else {
            &b.label
        };

        let inner = card::<M>()
            .variant(variant)
            .gap(8.0)
            .child(icon::<M>(IconName::Plus).size(24.0))
            .child(label::<M>(display_text))
            .build();

        column::<M>()
            .key(b.id)
            .height(b.height)
            .child(inner)
            .build()
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
    fn drop_zone_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = drop_zone("dz", "Drop files here").into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert_eq!(layout.style.height, Length::px(120.0));
    }

    #[test]
    fn drop_zone_builder_defaults() {
        let dz = drop_zone::<TestMsg>("dz", "Upload");
        assert_eq!(dz.label, "Upload");
        assert!(!dz.active);
        assert!((dz.height - 120.0).abs() < f32::EPSILON);
        assert!(dz.on_drop.is_none());
        assert!(dz.on_drag_over.is_none());
        assert!(dz.on_drag_leave.is_none());
    }

    #[test]
    fn drop_zone_active_uses_interactive_variant() {
        let node: WidgetNode<TestMsg> = drop_zone("dz", "Drop").active(true).into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant");
        };
        assert_eq!(col.children.len(), 1);
        let WidgetNode::Card(c) = &col.children[0] else {
            panic!("expected Card child");
        };
        assert_eq!(c.variant, CardVariant::Interactive);
    }

    #[test]
    fn drop_zone_inactive_uses_outlined_variant() {
        let node: WidgetNode<TestMsg> = drop_zone("dz", "Area").active(false).into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant");
        };
        let WidgetNode::Card(c) = &col.children[0] else {
            panic!("expected Card child");
        };
        assert_eq!(c.variant, CardVariant::Outlined);
    }

    #[test]
    fn drop_zone_has_icon_and_label() {
        let node: WidgetNode<TestMsg> = drop_zone("dz", "Upload files").into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant");
        };
        let WidgetNode::Card(c) = &col.children[0] else {
            panic!("expected Card child");
        };
        // Icon (Label) + text label = 2 children
        assert_eq!(c.children.len(), 2);
        // First child is the icon
        let WidgetNode::Label(icon_label) = &c.children[0] else {
            panic!("expected Label for icon");
        };
        assert_eq!(icon_label.text, "+");
        // Second child is the text label
        let WidgetNode::Label(text_label) = &c.children[1] else {
            panic!("expected Label for text");
        };
        assert_eq!(text_label.text, "Upload files");
    }

    #[test]
    fn drop_zone_messages() {
        #[derive(Clone, Debug, PartialEq)]
        enum Msg {
            Drop,
            Over,
            Leave,
        }
        let dz = drop_zone::<Msg>("dz", "Target")
            .on_drop(Msg::Drop)
            .on_drag_over(Msg::Over)
            .on_drag_leave(Msg::Leave);
        assert_eq!(dz.on_drop, Some(Msg::Drop));
        assert_eq!(dz.on_drag_over, Some(Msg::Over));
        assert_eq!(dz.on_drag_leave, Some(Msg::Leave));
    }
}
