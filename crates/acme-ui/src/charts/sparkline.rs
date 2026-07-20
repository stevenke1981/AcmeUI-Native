//! Sparkline — minimal inline chart using existing primitives.
//!
//! Renders as a Row of tiny vertical bars (block characters) proportional
//! to each data value. Compact and lightweight.

use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_widgets::{label_with_size, row};

/// Builder for a sparkline widget.
pub struct SparklineBuilder<M> {
    pub id: WidgetKey,
    pub data: Vec<f32>,
    pub height: f32,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a sparkline builder.
pub fn sparkline<M: Clone + 'static>(id: impl Into<WidgetKey>) -> SparklineBuilder<M> {
    SparklineBuilder {
        id: id.into(),
        data: vec![],
        height: 24.0,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> SparklineBuilder<M> {
    /// Add a data value.
    pub fn value(mut self, v: f32) -> Self {
        self.data.push(v);
        self
    }

    /// Set the sparkline height.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }

    /// Build the widget tree.
    ///
    /// Produces a Row where each data point is shown as a vertical bar
    /// using full-block characters at different heights, represented
    /// by █ ░ ▒ ▓ characters for different quartile ranges.
    pub fn build(self) -> WidgetNode<M> {
        let id_prefix = self.id.as_str().to_string();
        let max_v = self.data.iter().cloned().fold(0.0f32, f32::max).max(1.0);

        let mut spark_row = row::<M>()
            .key(format!("{}_sparkline", &id_prefix).as_str())
            .gap(1.0);

        // For each data value, pick a block character representing its
        // relative height in 4 quartiles.
        for val in self.data.iter() {
            let ratio = (val / max_v).clamp(0.0, 1.0);
            let block = if ratio > 0.75 {
                "\u{2588}" // Full block
            } else if ratio > 0.5 {
                "\u{2593}" // Dark shade
            } else if ratio > 0.25 {
                "\u{2592}" // Medium shade
            } else {
                "\u{2591}" // Light shade
            };
            let text = block.to_string();

            spark_row = spark_row.child(label_with_size::<M>(text, self.height.max(8.0)));
        }

        spark_row.build()
    }
}
