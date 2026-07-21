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

/// Return `value` if finite, otherwise `fallback`.
fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

/// Sanitise slider numeric parameters so all downstream arithmetic is safe.
///
/// * NaN / infinity → finite defaults
/// * `min > max` → swapped
/// * zero‑range → widened so `max = min + 1`
/// * `step <= 0` or non‑finite → `1.0`
/// * `value` is clamped, then snap‑rounded to the nearest step
fn normalize_slider(min: &mut f32, max: &mut f32, value: &mut f32, step: &mut f32) {
    *min = finite_or(*min, 0.0);
    *max = finite_or(*max, 100.0);
    if *min > *max {
        std::mem::swap(min, max);
    }
    if (*max - *min).abs() < f32::EPSILON {
        *max = *min + 1.0;
    }
    // NaN → min; infinity → saturate to max/min via clamp
    *value = if value.is_nan() {
        *min
    } else {
        value.clamp(*min, *max)
    };
    *step = if step.is_finite() && *step > 0.0 {
        *step
    } else {
        1.0
    };
    *value = (((*value - *min) / *step).round() * *step + *min).clamp(*min, *max);
}

impl<M: Clone + 'static> From<SliderBuilder<M>> for WidgetNode<M> {
    fn from(b: SliderBuilder<M>) -> Self {
        // ── 1. Sanitise all numeric inputs ─────────────────────────
        let mut value = b.value;
        let mut min = b.min;
        let mut max = b.max;
        let mut step = b.step;
        normalize_slider(&mut min, &mut max, &mut value, &mut step);

        // ── 2. Fill ratio (guaranteed finite, non‑zero range) ──────
        let ratio = ((value - min) / (max - min)).clamp(0.0, 1.0);

        // ── 3. Track height from ControlSize ────────────────────────
        let track_h = match b.size {
            crate::ControlSize::Sm => 4.0,
            crate::ControlSize::Md => 6.0,
            crate::ControlSize::Lg => 8.0,
            _ => 6.0, // Xs, Xl fall back to Md
        };

        // ── 4. Build fill bar (accent-coloured left portion) ────────
        let fill = row::<M>()
            .h(Length::Px(track_h))
            .w(Length::Percent(ratio))
            .child(label_with_size::<M>("", 12.0));

        // ── 5. Build track remainder (right portion) ───────────────
        let track_remainder = row::<M>()
            .h(Length::Px(track_h))
            .w(Length::Percent(1.0 - ratio))
            .child(label_with_size::<M>("", 12.0));

        // ── 7. Assemble track row ──────────────────────────────────
        let mut track_row = row::<M>()
            .child(fill)
            .child(track_remainder);

        // ── 7b. Wire on_change to the track row ────────────────────
        if let Some(msg) = b.on_change {
            track_row = track_row.on_click(msg);
        }

        // ── 8. Wrap with value label if show_value ──────────────────
        if b.show_value {
            let display = format_numeric(value);
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

    // ── Percentage-unit fix (P0-A01) ─────────────────────────────────

    /// Verify Percent values are in 0.0–1.0 range (not 0–100).
    #[test]
    fn slider_percent_is_zero_to_one() {
        let node: WidgetNode<TestMsg> = slider("s")
            .min(0.0)
            .max(100.0)
            .value(50.0)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.children[0].style.width, Length::Percent(0.5));
        assert_eq!(layout.children[1].style.width, Length::Percent(0.5));
    }

    #[test]
    fn slider_percent_full_range() {
        let node: WidgetNode<TestMsg> = slider("s")
            .min(0.0)
            .max(100.0)
            .value(100.0)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.children[0].style.width, Length::Percent(1.0));
        assert_eq!(layout.children[1].style.width, Length::Percent(0.0));
    }

    #[test]
    fn slider_percent_zero() {
        let node: WidgetNode<TestMsg> = slider("s")
            .min(0.0)
            .max(100.0)
            .value(0.0)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.children[0].style.width, Length::Percent(0.0));
        assert_eq!(layout.children[1].style.width, Length::Percent(1.0));
    }

    // ── on_change message wiring ─────────────────────────────────────

    #[test]
    fn slider_on_change_wired_to_track_row() {
        let node: WidgetNode<TestMsg> = slider("s")
            .value(50.0)
            .min(0.0)
            .max(100.0)
            .on_change(TestMsg::ValueChanged)
            .into();
        // Without show_value the outer node is the track Row itself.
        let WidgetNode::Row(row) = &node else {
            panic!("expected Row variant for track row");
        };
        assert_eq!(row.message, Some(TestMsg::ValueChanged));
    }

    #[test]
    fn slider_no_on_change_has_no_message() {
        let node: WidgetNode<TestMsg> = slider("s")
            .value(50.0)
            .min(0.0)
            .max(100.0)
            .into();
        let WidgetNode::Row(row) = &node else {
            panic!("expected Row variant");
        };
        assert_eq!(row.message, None);
    }

    // ── Range-safety fix (P0-A02) ────────────────────────────────────

    #[test]
    fn slider_normalize_nan_value() {
        let node: WidgetNode<TestMsg> = slider("s")
            .min(0.0)
            .max(100.0)
            .value(f32::NAN)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // NaN → clamped to min → ratio 0.0
        assert_eq!(layout.children[0].style.width, Length::Percent(0.0));
    }

    #[test]
    fn slider_normalize_inf_value() {
        let node: WidgetNode<TestMsg> = slider("s")
            .min(0.0)
            .max(100.0)
            .value(f32::INFINITY)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Infinity → clamped to max → ratio 1.0
        assert_eq!(layout.children[0].style.width, Length::Percent(1.0));
    }

    #[test]
    fn slider_normalize_neg_inf_value() {
        let node: WidgetNode<TestMsg> = slider("s")
            .min(0.0)
            .max(100.0)
            .value(f32::NEG_INFINITY)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // -Infinity → clamped to min → ratio 0.0
        assert_eq!(layout.children[0].style.width, Length::Percent(0.0));
    }

    #[test]
    fn slider_normalize_min_greater_than_max() {
        let node: WidgetNode<TestMsg> = slider("s")
            .min(100.0)
            .max(0.0)
            .value(50.0)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // min/max swapped → 50 in [0,100] → ratio 0.5
        assert_eq!(layout.children[0].style.width, Length::Percent(0.5));
    }

    #[test]
    fn slider_normalize_zero_range() {
        let node: WidgetNode<TestMsg> = slider("s")
            .min(50.0)
            .max(50.0)
            .value(50.0)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Zero-range widened → value at min → ratio 0.0
        assert_eq!(layout.children[0].style.width, Length::Percent(0.0));
    }

    #[test]
    fn slider_normalize_negative_step() {
        let node: WidgetNode<TestMsg> = slider("s")
            .min(0.0)
            .max(100.0)
            .value(25.0)
            .step(-1.0)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Negative step → reset to 1.0 → 25 stays 25 → ratio 0.25
        assert_eq!(layout.children[0].style.width, Length::Percent(0.25));
    }

    #[test]
    fn slider_normalize_nan_min() {
        let node: WidgetNode<TestMsg> = slider("s")
            .min(f32::NAN)
            .max(100.0)
            .value(50.0)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // NaN min → fallback 0.0
        assert_eq!(layout.children[0].style.width, Length::Percent(0.5));
    }

    #[test]
    fn slider_normalize_nan_max() {
        let node: WidgetNode<TestMsg> = slider("s")
            .min(0.0)
            .max(f32::NAN)
            .value(50.0)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // NaN max → fallback 100.0
        assert_eq!(layout.children[0].style.width, Length::Percent(0.5));
    }

    #[test]
    fn slider_normalize_zero_step() {
        let node: WidgetNode<TestMsg> = slider("s")
            .min(0.0)
            .max(100.0)
            .value(25.5)
            .step(0.0)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Zero step → reset to 1.0 → 25.5 rounds to 26 → ratio 0.26
        assert_eq!(layout.children[0].style.width, Length::Percent(0.26));
    }
}
