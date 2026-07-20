//! PieChart — static pie chart using existing primitives.
//!
//! Renders as a Row: a stacked bar (pie approximation) + legend Column.
//! The stacked bar uses block characters to show proportional segments.

use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_theme::ThemeColor;
use acme_widgets::{column, label_with_size};

/// A single pie slice descriptor.
#[derive(Clone, Debug)]
pub struct PieSlice {
    pub label: String,
    pub value: f32,
    pub color: ThemeColor,
}

impl PieSlice {
    pub fn new(label: impl Into<String>, value: f32, color: ThemeColor) -> Self {
        Self {
            label: label.into(),
            value,
            color,
        }
    }
}

/// Builder for a pie chart widget.
pub struct PieChartBuilder<M> {
    pub id: WidgetKey,
    pub size: f32,
    pub slices: Vec<PieSlice>,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a pie chart builder.
pub fn pie_chart<M: Clone + 'static>(id: impl Into<WidgetKey>) -> PieChartBuilder<M> {
    PieChartBuilder {
        id: id.into(),
        size: 180.0,
        slices: vec![],
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> PieChartBuilder<M> {
    /// Set the chart diameter.
    pub fn size(mut self, value: f32) -> Self {
        self.size = value;
        self
    }

    /// Add a pie slice.
    pub fn slice(mut self, s: PieSlice) -> Self {
        self.slices.push(s);
        self
    }

    /// Build the widget tree.
    pub fn build(self) -> WidgetNode<M> {
        let id_prefix = self.id.as_str().to_string();
        let total: f32 = self.slices.iter().map(|s| s.value).sum();
        let total = total.max(1.0);

        let mut col = column::<M>()
            .key(format!("{}_chart", &id_prefix).as_str())
            .gap(4.0);

        // Title
        col = col.child(label_with_size::<M>(
            format!("PieChart — {} slices", self.slices.len()),
            14.0,
        ));

        // Pie representation: stacked horizontal bar using block chars
        let mut pie_bar = String::from(" ");
        for slice in &self.slices {
            let ratio = (slice.value / total).clamp(0.0, 1.0);
            let count = (ratio * 20.0).round() as usize;
            for _ in 0..count {
                pie_bar.push('\u{2588}');
            }
        }
        if pie_bar.trim().is_empty() {
            pie_bar.push('\u{2588}');
        }
        col = col.child(label_with_size::<M>(pie_bar, 18.0));

        // Legend
        let mut legend = column::<M>()
            .key(format!("{}_legend", &id_prefix).as_str())
            .gap(2.0);
        for slice in &self.slices {
            let pct = (slice.value / total * 100.0).round() as u32;
            let legend_text = format!("\u{25A0} {} — {} ({pct}%)", slice.label, slice.value);
            legend = legend.child(label_with_size::<M>(legend_text, 12.0));
        }
        col = col.child(legend.build());

        col.build()
    }
}
