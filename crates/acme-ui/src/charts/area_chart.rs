//! AreaChart — static area chart using existing primitives.
//!
//! Similar to LineChart but with a filled area below the line,
//! represented by a Column of stacked proportional blocks.

use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_widgets::{column, label_with_size, separator};

use super::line_chart::ChartPoint;

/// Builder for an area chart widget.
pub struct AreaChartBuilder<M> {
    pub id: WidgetKey,
    pub height: f32,
    pub data: Vec<ChartPoint>,
    _phantom: std::marker::PhantomData<M>,
}

/// Create an area chart builder.
pub fn area_chart<M: Clone + 'static>(id: impl Into<WidgetKey>) -> AreaChartBuilder<M> {
    AreaChartBuilder {
        id: id.into(),
        height: 200.0,
        data: vec![],
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> AreaChartBuilder<M> {
    /// Set the chart height.
    pub fn height(mut self, value: f32) -> Self {
        self.height = value;
        self
    }

    /// Add a data point.
    pub fn point(mut self, pt: ChartPoint) -> Self {
        self.data.push(pt);
        self
    }

    /// Build the widget tree.
    ///
    /// Produces a Column of text-encoded area blocks. Each data point is
    /// shown as a filled horizontal bar (████) proportional to its value,
    /// stacked below a label row. This gives a visual "area" effect.
    pub fn build(self) -> WidgetNode<M> {
        let id_prefix = self.id.as_str().to_string();
        let max_v = self
            .data
            .iter()
            .map(|p| p.value)
            .fold(0.0f32, f32::max)
            .max(1.0);

        let mut col = column::<M>()
            .key(format!("{}_chart", &id_prefix).as_str())
            .gap(3.0);

        // Title
        col = col.child(label_with_size::<M>(
            format!("AreaChart — {} points", self.data.len()),
            14.0,
        ));

        // Stack the area blocks: for each data point show a filled bar
        // proportional to value, then the label beneath.
        for (_i, pt) in self.data.iter().enumerate() {
            let ratio = (pt.value / max_v).clamp(0.0, 1.0);
            let block_count = (ratio * 24.0).round() as usize;
            let area_line = if block_count > 0 {
                "\u{2588}".repeat(block_count)
            } else {
                "\u{2591}".to_string()
            };
            let line_text = format!("{}  {}: {}", area_line, pt.label, pt.value);

            col = col.child(label_with_size::<M>(line_text, 12.0));
        }

        col = col.child(separator());
        col = col.child(label_with_size::<M>(
            format!("Range: 0 — {:.1}", max_v),
            12.0,
        ));

        col.build()
    }
}
