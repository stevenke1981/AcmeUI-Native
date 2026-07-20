//! BarChart — static bar chart using existing primitives.
//!
//! Renders as a Column of rows. Each bar row shows a label and a
//! proportional text-based bar made of block characters.

use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_widgets::{column, label_with_size, row};

/// A single bar entry in the chart.
#[derive(Clone, Debug)]
pub struct BarEntry {
    pub label: String,
    pub value: f32,
}

impl BarEntry {
    pub fn new(label: impl Into<String>, value: f32) -> Self {
        Self {
            label: label.into(),
            value,
        }
    }
}

/// Builder for a bar chart widget.
pub struct BarChartBuilder<M> {
    pub id: WidgetKey,
    pub height: f32,
    pub bars: Vec<BarEntry>,
    pub max_value: f32,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a bar chart builder.
pub fn bar_chart<M: Clone + 'static>(id: impl Into<WidgetKey>) -> BarChartBuilder<M> {
    BarChartBuilder {
        id: id.into(),
        height: 200.0,
        bars: vec![],
        max_value: 100.0,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> BarChartBuilder<M> {
    /// Set the chart height.
    pub fn height(mut self, value: f32) -> Self {
        self.height = value;
        self
    }

    /// Add a bar entry.
    pub fn bar(mut self, entry: BarEntry) -> Self {
        self.bars.push(entry);
        self
    }

    /// Set the maximum value for scaling.
    pub fn max_value(mut self, value: f32) -> Self {
        self.max_value = value;
        self
    }

    /// Build the widget tree — returns a Column containing bar rows.
    pub fn build(self) -> WidgetNode<M> {
        let max_v = self.max_value.max(1.0);
        let id_prefix = self.id.as_str().to_string();

        let mut col = column::<M>()
            .key(WidgetKey::new(format!("{}_chart", &id_prefix)))
            .gap(4.0);

        // Title showing chart name
        col = col.child(label_with_size::<M>(
            format!("BarChart — {} bars", self.bars.len()),
            14.0,
        ));

        for (i, entry) in self.bars.iter().enumerate() {
            let ratio = (entry.value / max_v).clamp(0.0, 1.0);
            // Build a proportional text bar using block characters
            let bar_len = (ratio * 30.0).round() as usize;
            let bar_chars = "\u{2588}".repeat(bar_len.max(1));
            let bar_text = if ratio > 0.0 {
                format!("{}  {:.0}%", bar_chars, ratio * 100.0)
            } else {
                "  (0%)".to_string()
            };

            let label_text = format!("{}:", entry.label);
            let bar_row = row::<M>()
                .key(format!("{}_row_{}", id_prefix, i).as_str())
                .gap(6.0)
                .child(label_with_size::<M>(label_text, 13.0))
                .child(label_with_size::<M>(bar_text, 13.0))
                .build();

            col = col.child(bar_row);
        }

        col.build()
    }
}
