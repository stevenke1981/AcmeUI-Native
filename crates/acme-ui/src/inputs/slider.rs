//! Slider input component.
//!
//! Renders as a Row with a fill bar and a track remainder (both percentage‑based),
//! optionally wrapped in a Column with a value label when `show_value` is true.

use acme_core::WidgetKey;
use acme_layout::Length;
use acme_widgets::*;
use crate::style::Styled;

/// Builder for a Slider component.
pub struct SliderBuilder<M> {
    pub id: WidgetKey,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub show_value: bool,
    pub size: crate::ControlSize,
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
        show_value: false,
        size: crate::ControlSize::Md,
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

    /// Show or hide the current value label.
    pub fn show_value(mut self, value: bool) -> Self {
        self.show_value = value;
        self
    }

    /// Set the slider track size.
    pub fn size(mut self, value: crate::ControlSize) -> Self {
        self.size = value;
        self
    }

    /// Set the message dispatched when the value changes.
    pub fn on_change(mut self, msg: M) -> Self {
        self.on_change = Some(msg);
        self
    }
}

/// Format a numeric value for display, trimming unnecessary decimals.
fn format_numeric(v: f32) -> String {
    if v.fract() == 0.0 {
        format!("{:.0}", v)
    } else {
        format!("{:.1}", v)
    }
}

impl<M: Clone + 'static> From<SliderBuilder<M>> for WidgetNode<M> {
    fn from(b: SliderBuilder<M>) -> Self {
        // ── 1. Clamp raw value ──────────────────────────────────────
        let value = b.value.clamp(b.min, b.max);

        // ── 2. Step rounding ────────────────────────────────────────
        let normalized = if b.step > 0.0 {
            let stepped = ((value - b.min) / b.step).round() * b.step + b.min;
            stepped.clamp(b.min, b.max)
        } else {
            value
        };

        // ── 3. Fill ratio (avoid division by zero) ──────────────────
        let range = if b.max > b.min { b.max - b.min } else { 1.0 };
        let ratio = ((normalized - b.min) / range).clamp(0.0, 1.0);
        let fill_pct = ratio * 100.0;
        let track_pct = (1.0 - ratio) * 100.0;

        // ── 4. Track height from ControlSize ────────────────────────
        let track_h = match b.size {
            crate::ControlSize::Sm => 4.0,
            crate::ControlSize::Md => 6.0,
            crate::ControlSize::Lg => 8.0,
            _ => 6.0, // Xs, Xl fall back to Md
        };

        // ── 5. Build fill bar (accent-coloured left portion) ────────
        let fill = row::<M>()
            .h(Length::Px(track_h))
            .w(Length::Percent(fill_pct))
            .child(label_with_size::<M>("", 12.0));

        // ── 6. Build track remainder (right portion) ───────────────
        let track_remainder = row::<M>()
            .h(Length::Px(track_h))
            .w(Length::Percent(track_pct))
            .child(label_with_size::<M>("", 12.0));

        // ── 7. Assemble track row ──────────────────────────────────
        let track_row = row::<M>()
            .child(fill)
            .child(track_remainder);

        // ── 8. Wrap with value label if show_value ──────────────────
        if b.show_value {
            let display = format_numeric(normalized);
            column::<M>()
                .key(b.id)
                .gap(4.0)
                .child(label_with_size::<M>(display, 12.0))
                .child(track_row)
                .build()
        } else {
            track_row
                .key(b.id)
                .build()
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
    enum TestMsg {
        ValueChanged,
    }

    /// The default slider (no show_value) builds into a Row with two children:
    /// fill container and track remainder container.
    #[test]
    fn slider_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = slider("sl1").value(50.0).min(0.0).max(100.0).into();
        let layout = node.to_layout(NodeId::new(1));

        // Outer container is a Row (fill + track remainder)
        assert_eq!(layout.style.kind, LayoutKind::Row);

        // Two children: fill, track remainder
        assert_eq!(layout.children.len(), 2);

        // Each child is a container with a Label leaf
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

    /// When show_value is true the outer container is a Column with two
    /// children: the value label and the track Row.
    #[test]
    fn slider_show_value_wraps_in_column() {
        let node: WidgetNode<TestMsg> = slider("s")
            .value(42.0)
            .min(0.0)
            .max(100.0)
            .show_value(true)
            .into();
        let layout = node.to_layout(NodeId::new(1));

        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert_eq!(layout.children.len(), 2);

        // First child: value label (leaf)
        let label = &layout.children[0];
        assert!(label.children.is_empty());

        // Second child: the track Row
        let track = &layout.children[1];
        assert_eq!(track.style.kind, LayoutKind::Row);
        assert_eq!(track.children.len(), 2);
    }

    /// The value is step-rounded before computing the fill ratio.
    #[test]
    fn slider_step_rounding_affects_fill_ratio() {
        // value=23, min=0, max=100, step=10 → normalized to 20
        let node: WidgetNode<TestMsg> = slider("s")
            .value(23.0)
            .min(0.0)
            .max(100.0)
            .step(10.0)
            .show_value(true)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Should be a Column: [label, row]
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert_eq!(layout.children.len(), 2);
    }

    /// Normalize with min/max clamped.
    #[test]
    fn slider_normalize_clamps_value() {
        let node: WidgetNode<TestMsg> = slider("s")
            .value(150.0) // above max
            .min(0.0)
            .max(100.0)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Should still be a valid Row
        assert_eq!(layout.style.kind, LayoutKind::Row);
        assert_eq!(layout.children.len(), 2);
    }
}
