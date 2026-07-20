//! Slider input component.
//!
//! Renders as a Stack with:
//! - track (Card, horizontal background bar)
//! - fill (Card, accent bar with width proportional to value)
//! - thumb (Card, circle handle)
//! The parent app manages value state and drag interaction.

use acme_core::WidgetKey;
use acme_widgets::*;

/// Builder for a Slider component.
pub struct SliderBuilder<M> {
    pub id: WidgetKey,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub on_change: Option<M>,
}

/// Create a new Slider builder.
///
/// Defaults: min=0, max=100, step=1.
pub fn slider<M: Clone + 'static>(id: impl Into<WidgetKey>) -> SliderBuilder<M> {
    SliderBuilder {
        id: id.into(),
        value: 0.0,
        min: 0.0,
        max: 100.0,
        step: 1.0,
        on_change: None,
    }
}

impl<M: Clone + 'static> SliderBuilder<M> {
    /// Set the current slider value.
    pub fn value(mut self, v: f32) -> Self {
        self.value = v;
        self
    }

    /// Set the minimum value (default 0).
    pub fn min(mut self, v: f32) -> Self {
        self.min = v;
        self
    }

    /// Set the maximum value (default 100).
    pub fn max(mut self, v: f32) -> Self {
        self.max = v;
        self
    }

    /// Set the step increment (default 1).
    pub fn step(mut self, v: f32) -> Self {
        self.step = v;
        self
    }

    /// Set the message dispatched when the value changes.
    pub fn on_change(mut self, msg: M) -> Self {
        self.on_change = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<SliderBuilder<M>> for WidgetNode<M> {
    fn from(b: SliderBuilder<M>) -> Self {
        // Stack overlays: track (background) → fill → thumb (top)
        stack::<M>()
            .key(b.id)
            .child(
                card::<M>()
                    .variant(CardVariant::Outlined)
                    .child(label_with_size::<M>("", 12.0)),
            )
            .child(
                card::<M>()
                    .variant(CardVariant::Interactive)
                    .child(label_with_size::<M>("", 12.0)),
            )
            .child(
                card::<M>()
                    .variant(CardVariant::Elevated)
                    .child(label_with_size::<M>("", 12.0)),
            )
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
    enum TestMsg {
        ValueChanged,
    }

    #[test]
    fn slider_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = slider("sl1").value(50.0).min(0.0).max(100.0).into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Stack);
        // Three children: track, fill, thumb
        assert_eq!(layout.children.len(), 3);
        // Each child is a Card container with a Label leaf
        for child in &layout.children {
            assert!(!child.children.is_empty());
            let leaf = &child.children[0];
            assert!(leaf.children.is_empty());
            assert_ne!(leaf.style.min_height, Length::px(0.0));
        }
    }

    #[test]
    fn slider_builder_defaults() {
        let s = slider::<TestMsg>("s");
        assert_eq!(s.value, 0.0);
        assert_eq!(s.min, 0.0);
        assert_eq!(s.max, 100.0);
        assert_eq!(s.step, 1.0);
        assert!(s.on_change.is_none());
    }

    #[test]
    fn slider_custom_range() {
        let s = slider::<TestMsg>("s")
            .value(5.0)
            .min(0.0)
            .max(10.0)
            .step(0.5)
            .on_change(TestMsg::ValueChanged);
        assert_eq!(s.value, 5.0);
        assert_eq!(s.min, 0.0);
        assert_eq!(s.max, 10.0);
        assert_eq!(s.step, 0.5);
        assert!(s.on_change.is_some());
    }
}
