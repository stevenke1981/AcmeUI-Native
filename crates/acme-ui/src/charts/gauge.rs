//! Gauge — static gauge component using existing primitives.
//!
//! Renders as a rounded card-like representation with a value label in the
//! center. Uses block characters to approximate a semi-circular arc fill.

use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_widgets::{column, label_with_size};

/// Builder for a gauge widget.
pub struct GaugeBuilder<M> {
    pub id: WidgetKey,
    pub value: f32,
    pub size: f32,
    pub label: String,
    pub max: f32,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a gauge builder.
pub fn gauge<M: Clone + 'static>(id: impl Into<WidgetKey>) -> GaugeBuilder<M> {
    GaugeBuilder {
        id: id.into(),
        value: 0.0,
        size: 120.0,
        label: String::new(),
        max: 100.0,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> GaugeBuilder<M> {
    /// Set the current value.
    pub fn value(mut self, v: f32) -> Self {
        self.value = v;
        self
    }

    /// Set the gauge diameter.
    pub fn size(mut self, v: f32) -> Self {
        self.size = v;
        self
    }

    /// Set the descriptive label.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }

    /// Set the maximum value.
    pub fn max(mut self, m: f32) -> Self {
        self.max = m;
        self
    }

    /// Build the widget tree.
    ///
    /// Produces a Column with a label, a proportional "arc" of block
    /// characters forming a semi-circle, and the numeric value.
    pub fn build(self) -> WidgetNode<M> {
        let id_prefix = self.id.as_str().to_string();
        let ratio = (self.value / self.max.max(1.0)).clamp(0.0, 1.0);
        let pct = (ratio * 100.0).round() as u32;

        // Semi-circle arc using block characters: top half of a full-width block
        let arc_total = 16;
        let filled = (ratio * arc_total as f32).round() as usize;
        let filled_count = filled.min(arc_total);
        let empty_count = arc_total.saturating_sub(filled_count);

        let arc_filled = "\u{2588}".repeat(filled_count);
        let arc_empty = "\u{2591}".repeat(empty_count);
        let arc = format!("{}{}", arc_filled, arc_empty);

        let mut col = column::<M>()
            .key(format!("{}_gauge", &id_prefix).as_str())
            .gap(4.0);

        if !self.label.is_empty() {
            col = col.child(label_with_size::<M>(&self.label, 14.0));
        }

        // Arc representation
        col = col.child(label_with_size::<M>(arc, 24.0));

        // Value display
        col = col.child(label_with_size::<M>(
            format!("{} / {}  ({pct}%)", self.value, self.max),
            16.0,
        ));

        col.build()
    }
}
