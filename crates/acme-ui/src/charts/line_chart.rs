//! LineChart — static line chart using existing primitives.
//!
//! Renders as a Column with dots/points shown as text markers at each data
//! position. Height sets the chart area visual height.

use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_widgets::{column, label_with_size, separator};

/// A single data point on the line chart.
#[derive(Clone, Debug)]
pub struct ChartPoint {
    pub label: String,
    pub value: f32,
}

impl ChartPoint {
    pub fn new(label: impl Into<String>, value: f32) -> Self {
        Self {
            label: label.into(),
            value,
        }
    }
}

/// Builder for a line chart widget.
pub struct LineChartBuilder<M> {
    pub id: WidgetKey,
    pub height: f32,
    pub data: Vec<ChartPoint>,
    pub show_dots: bool,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a line chart builder.
pub fn line_chart<M: Clone + 'static>(id: impl Into<WidgetKey>) -> LineChartBuilder<M> {
    LineChartBuilder {
        id: id.into(),
        height: 200.0,
        data: vec![],
        show_dots: true,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> LineChartBuilder<M> {
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

    /// Toggle dot rendering.
    pub fn show_dots(mut self, show: bool) -> Self {
        self.show_dots = show;
        self
    }

    /// Build the widget tree.
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
            .gap(4.0);

        col = col.child(label_with_size::<M>(
            format!("LineChart — {} points", self.data.len()),
            14.0,
        ));

        // Render a small inline representation using dots/ticks
        for pt in self.data.iter() {
            let ratio = (pt.value / max_v).clamp(0.0, 1.0);
            let tick = if self.show_dots {
                "\u{25CF}"
            } else {
                "\u{2502}"
            };
            let bar_len = (ratio * 20.0).round() as usize;
            let spacer = "\u{00A0}".repeat(20usize.saturating_sub(bar_len));
            let visual = format!(
                "{}{}{}  {}  ({})",
                tick,
                "\u{2500}".repeat(bar_len),
                spacer,
                pt.label,
                pt.value
            );

            col = col.child(label_with_size::<M>(visual, 13.0));
        }

        // Show Y-axis legend
        col = col.child(separator());
        col = col.child(label_with_size::<M>(
            format!("Range: 0 — {:.1}", max_v),
            12.0,
        ));

        col.build()
    }
}
