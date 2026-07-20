//! RangeSlider input component — dual-handle slider for selecting a range.
//!
//! Renders as a Column with:
//! - a track (Stack: background + fill range)
//! - optional labels showing "low_value - high_value"

use acme_core::WidgetKey;
use acme_widgets::*;

/// Builder for a RangeSlider component.
pub struct RangeSliderBuilder<M> {
    pub id: WidgetKey,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub low_value: f64,
    pub high_value: f64,
    pub disabled: bool,
    pub size: crate::ControlSize,
    pub show_labels: bool,
    pub on_change: Option<M>,
}

/// Create a new RangeSlider builder.
///
/// Defaults: min=0, max=100, step=1, low_value=25, high_value=75.
pub fn range_slider<M: Clone + 'static>(id: impl Into<WidgetKey>) -> RangeSliderBuilder<M> {
    RangeSliderBuilder {
        id: id.into(),
        min: 0.0,
        max: 100.0,
        step: 1.0,
        low_value: 25.0,
        high_value: 75.0,
        disabled: false,
        size: crate::ControlSize::Md,
        show_labels: false,
        on_change: None,
    }
}

impl<M: Clone + 'static> RangeSliderBuilder<M> {
    /// Set the minimum value (default 0).
    pub fn min(mut self, v: f64) -> Self {
        self.min = v;
        self
    }

    /// Set the maximum value (default 100).
    pub fn max(mut self, v: f64) -> Self {
        self.max = v;
        self
    }

    /// Set the step increment (default 1).
    pub fn step(mut self, v: f64) -> Self {
        self.step = v;
        self
    }

    /// Set the low (left) handle value.
    pub fn low_value(mut self, v: f64) -> Self {
        self.low_value = v;
        self
    }

    /// Set the high (right) handle value.
    pub fn high_value(mut self, v: f64) -> Self {
        self.high_value = v;
        self
    }

    /// Set whether the slider is disabled.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    /// Set the slider track size.
    pub fn size(mut self, value: crate::ControlSize) -> Self {
        self.size = value;
        self
    }

    /// Show or hide the min/max/current value labels.
    pub fn show_labels(mut self, value: bool) -> Self {
        self.show_labels = value;
        self
    }

    /// Set the message dispatched when the range changes.
    pub fn on_change(mut self, msg: M) -> Self {
        self.on_change = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<RangeSliderBuilder<M>> for WidgetNode<M> {
    fn from(b: RangeSliderBuilder<M>) -> Self {
        // Track stack: background + fill range
        let track = stack::<M>()
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
            .build();

        let mut col = column::<M>().key(b.id).gap(4.0).child(track);

        // Optional labels showing "low - high"
        if b.show_labels {
            let label_text = format!("{} - {}", b.low_value, b.high_value);
            col = col.child(label::<M>(label_text));
        }

        col.build()
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
        RangeChanged,
    }

    #[test]
    fn range_slider_builder_defaults() {
        let rs = range_slider::<TestMsg>("rs");
        assert_eq!(rs.min, 0.0);
        assert_eq!(rs.max, 100.0);
        assert_eq!(rs.step, 1.0);
        assert_eq!(rs.low_value, 25.0);
        assert_eq!(rs.high_value, 75.0);
        assert!(!rs.disabled);
        assert!(!rs.show_labels);
        assert!(rs.on_change.is_none());
    }

    #[test]
    fn range_slider_custom_values() {
        let rs = range_slider::<TestMsg>("rs")
            .min(0.0)
            .max(50.0)
            .step(0.5)
            .low_value(10.0)
            .high_value(40.0)
            .show_labels(true)
            .on_change(TestMsg::RangeChanged);
        assert_eq!(rs.min, 0.0);
        assert_eq!(rs.max, 50.0);
        assert_eq!(rs.step, 0.5);
        assert_eq!(rs.low_value, 10.0);
        assert_eq!(rs.high_value, 40.0);
        assert!(rs.show_labels);
        assert!(rs.on_change.is_some());
    }

    #[test]
    fn range_slider_produces_column_with_track_and_labels() {
        let node: WidgetNode<TestMsg> = range_slider("rs")
            .low_value(20.0)
            .high_value(80.0)
            .show_labels(true)
            .into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column variant");
        };
        // Track stack + label row = 2 children
        assert_eq!(c.children.len(), 2);
        // First child is a Stack (the track)
        assert!(matches!(&c.children[0], WidgetNode::Stack(_)));
        // Second child is a Label (the value text)
        let WidgetNode::Label(l) = &c.children[1] else {
            panic!("expected Label for value display");
        };
        assert_eq!(l.text, "20 - 80");
    }

    #[test]
    fn range_slider_hides_labels_when_disabled() {
        let node: WidgetNode<TestMsg> = range_slider("rs")
            .low_value(10.0)
            .high_value(90.0)
            .show_labels(false)
            .into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column variant");
        };
        // Only the track stack, no label
        assert_eq!(c.children.len(), 1);
    }

    #[test]
    fn range_slider_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = range_slider("rs")
            .low_value(10.0)
            .high_value(90.0)
            .show_labels(true)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        // Stack child (track) + label leaf = 2 children
        assert_eq!(layout.children.len(), 2);
        // Track is a Stack with 2 Card children
        assert_eq!(layout.children[0].style.kind, LayoutKind::Stack);
        assert_eq!(layout.children[0].children.len(), 2);
        // Label has non-zero min_height
        assert_ne!(layout.children[1].style.min_height, Length::px(0.0));
    }
}
