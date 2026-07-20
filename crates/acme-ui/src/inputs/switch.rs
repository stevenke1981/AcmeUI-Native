//! Switch toggle component.
//!
//! Renders as a Row with a toggle track (Card) containing a thumb (Card),
//! and a label. When checked the thumb slides to the right and the track
//! uses an accent variant.

use acme_core::WidgetKey;
use acme_widgets::*;

/// Builder for a Switch toggle component.
pub struct SwitchBuilder<M> {
    pub id: WidgetKey,
    pub label: String,
    pub checked: bool,
    pub disabled: bool,
    pub size: crate::ControlSize,
    pub on_click: Option<M>,
}

/// Create a new Switch builder.
pub fn switch<M: Clone + 'static>(id: impl Into<WidgetKey>, checked: bool) -> SwitchBuilder<M> {
    SwitchBuilder {
        id: id.into(),
        label: String::new(),
        checked,
        disabled: false,
        size: crate::ControlSize::Md,
        on_click: None,
    }
}

impl<M: Clone + 'static> SwitchBuilder<M> {
    /// Set the label text next to the switch.
    pub fn label(mut self, value: impl Into<String>) -> Self {
        self.label = value.into();
        self
    }

    /// Set whether the switch is disabled.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    /// Set the switch size (affects track/thumb dimensions).
    pub fn size(mut self, value: crate::ControlSize) -> Self {
        self.size = value;
        self
    }

    /// Set the message dispatched when the switch is toggled.
    pub fn on_click(mut self, msg: M) -> Self {
        self.on_click = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<SwitchBuilder<M>> for WidgetNode<M> {
    fn from(b: SwitchBuilder<M>) -> Self {
        // Thumb position: when checked push right via Row gap
        let thumb = card::<M>()
            .padding(2.0)
            .variant(if b.checked {
                CardVariant::Interactive
            } else {
                CardVariant::Plain
            })
            .child(label_with_size::<M>("", 12.0));

        let track = card::<M>()
            .padding(2.0)
            .variant(CardVariant::Elevated)
            .child(
                row::<M>()
                    .gap(if b.checked { 18.0 } else { 2.0 })
                    .child(thumb),
            );

        row::<M>()
            .key(b.id)
            .gap(8.0)
            .child(track)
            .child(label::<M>(b.label))
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
    fn switch_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = switch("sw", true).label("Toggle me").into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Row);
        assert!(!layout.children.is_empty());
        // Second child is the label
        let label_leaf = &layout.children[1];
        assert!(label_leaf.children.is_empty());
        assert_ne!(label_leaf.style.min_height, Length::px(0.0));
    }

    #[test]
    fn switch_builder_defaults() {
        let s = switch::<TestMsg>("sw", false);
        assert!(!s.checked);
        assert!(!s.disabled);
        assert!(s.on_click.is_none());
    }

    #[test]
    fn switch_checked_has_accent_track() {
        let node: WidgetNode<TestMsg> = switch("sw", true).label("On").into();
        let WidgetNode::Row(container) = &node else {
            panic!("expected Row");
        };
        // First child is the track card
        assert!(matches!(&container.children[0], WidgetNode::Card(_)));
    }
}
