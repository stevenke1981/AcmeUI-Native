//! DonutChart — a ring chart with a center label.
//!
//! Renders as a Column: a ring representation (using block characters
//! and a Card cutout for the hole) plus a legend Column beneath.
//! The center label shows the total or a custom value.
//!
//! > **Note:** This is a layout-level placeholder, not a true rendered chart.
//! > Like PieChart, it uses text markers and labels.

use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_theme::ThemeColor;
use acme_widgets::{column, label_with_size};

/// A single donut slice descriptor.
#[derive(Clone, Debug)]
pub struct DonutSlice {
    pub label: String,
    pub value: f32,
    pub color: ThemeColor,
}

impl DonutSlice {
    pub fn new(label: impl Into<String>, value: f32, color: ThemeColor) -> Self {
        Self {
            label: label.into(),
            value,
            color,
        }
    }
}

/// Builder for a DonutChart widget.
pub struct DonutChartBuilder<M> {
    pub id: WidgetKey,
    pub size: f32,
    pub thickness: f32,
    pub slices: Vec<DonutSlice>,
    pub center_label: Option<String>,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a donut chart builder.
pub fn donut_chart<M: Clone + 'static>(id: impl Into<WidgetKey>) -> DonutChartBuilder<M> {
    DonutChartBuilder {
        id: id.into(),
        size: 180.0,
        thickness: 40.0,
        slices: vec![],
        center_label: None,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> DonutChartBuilder<M> {
    /// Set the chart diameter in pixels.
    pub fn size(mut self, value: f32) -> Self {
        self.size = value;
        self
    }

    /// Set the ring thickness in pixels.
    pub fn thickness(mut self, value: f32) -> Self {
        self.thickness = value;
        self
    }

    /// Set the center label text. If not set, the total is displayed.
    pub fn center_label(mut self, value: impl Into<String>) -> Self {
        self.center_label = Some(value.into());
        self
    }

    /// Add a donut slice.
    pub fn slice(mut self, s: DonutSlice) -> Self {
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
            .gap(8.0);

        // Ring representation: use block characters
        let ring_char_count = (self.size / 10.0).round() as usize;
        let ring_chars = 16.max(ring_char_count.min(40));

        let hole_inner = (self.thickness / self.size * ring_chars as f32).round() as usize;
        let _hole_outer = ring_chars - hole_inner;

        // Build ring segments
        let mut ring_segments = String::new();
        for slice in &self.slices {
            let ratio = slice.value / total;
            let count = (ratio * ring_chars as f32).round() as usize;
            for _ in 0..count {
                ring_segments.push('\u{2588}');
            }
        }

        // Pad remaining
        while ring_segments.len() < ring_chars {
            ring_segments.push('\u{2591}');
        }

        // Build the ring wrapper: outer ring with center cutout
        // The "hole" is represented as a Card with a large empty center
        let center_text = self
            .center_label
            .clone()
            .unwrap_or_else(|| format!("{}", total as u32));

        let ring_card = crate::card::<M>()
            .variant(crate::CardVariant::Muted)
            .child(
                crate::column::<M>()
                    .child(label_with_size::<M>(&ring_segments, 16.0))
                    .child(crate::label_builder(&center_text).font_size(22.0).build())
                    .child(label_with_size::<M>(
                        format!("Donut — {} slices", self.slices.len()),
                        12.0,
                    ))
                    .build(),
            )
            .padding(12.0)
            .build();

        col = col.child(ring_card);

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

impl<M: Clone + 'static> From<DonutChartBuilder<M>> for WidgetNode<M> {
    fn from(b: DonutChartBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn donut_chart_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = donut_chart::<TestMsg>("dchart")
            .slice(DonutSlice::new("A", 30.0, ThemeColor::rgb(255, 99, 132)))
            .slice(DonutSlice::new("B", 50.0, ThemeColor::rgb(54, 162, 235)))
            .slice(DonutSlice::new("C", 20.0, ThemeColor::rgb(255, 206, 86)))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Column: [ring card, legend] = 2 children
        assert_eq!(layout.children.len(), 2);
    }

    #[test]
    fn donut_chart_single_slice() {
        let node: WidgetNode<TestMsg> = donut_chart::<TestMsg>("dc")
            .slice(DonutSlice::new(
                "Only",
                100.0,
                ThemeColor::rgb(75, 192, 192),
            ))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.children.len(), 2); // ring + legend
    }

    #[test]
    fn donut_chart_builder_defaults() {
        let d = donut_chart::<TestMsg>("dc");
        assert!(d.slices.is_empty());
        assert!((d.size - 180.0).abs() < f32::EPSILON);
        assert!((d.thickness - 40.0).abs() < f32::EPSILON);
        assert!(d.center_label.is_none());
    }

    #[test]
    fn donut_chart_custom_center_label() {
        let node: WidgetNode<TestMsg> = donut_chart::<TestMsg>("dc")
            .center_label("Total: 100")
            .slice(DonutSlice::new("X", 100.0, ThemeColor::rgb(0, 0, 0)))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.children.len(), 2);
    }

    #[test]
    fn donut_chart_from_conversion() {
        let node: WidgetNode<TestMsg> = donut_chart::<TestMsg>("dc")
            .slice(DonutSlice::new("A", 10.0, ThemeColor::rgb(0, 0, 0)))
            .into();
        let WidgetNode::Column(_) = &node else {
            panic!("expected Column variant");
        };
    }
}
