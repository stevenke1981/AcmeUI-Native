//! ScatterChart — a static scatter/bubble chart using existing primitives.
//!
//! Renders as a Column with a header and a grid of data point markers.
//! Each point is represented as a label with a dot character and label.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// A single data point in the scatter chart.
#[derive(Clone, Debug)]
pub struct ScatterPoint {
    pub label: String,
    pub x: f32,
    pub y: f32,
    pub size: f32,
}

impl ScatterPoint {
    /// Create a new scatter data point.
    pub fn new(label: impl Into<String>, x: f32, y: f32) -> Self {
        Self {
            label: label.into(),
            x,
            y,
            size: 1.0,
        }
    }

    /// Set the visual size (marker scale).
    pub fn size(mut self, value: f32) -> Self {
        self.size = value.max(0.5);
        self
    }

    /// Return a marker character based on size.
    fn marker_char(&self) -> &str {
        if self.size >= 2.0 {
            "⬤"
        } else if self.size >= 1.5 {
            "◉"
        } else {
            "●"
        }
    }
}

/// Builder for a scatter chart widget.
pub struct ScatterChartBuilder<M> {
    pub id: WidgetKey,
    pub points: Vec<ScatterPoint>,
    pub width: f32,
    pub height: f32,
    pub show_grid: bool,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a scatter chart builder.
pub fn scatter_chart<M: Clone + 'static>(id: impl Into<WidgetKey>) -> ScatterChartBuilder<M> {
    ScatterChartBuilder {
        id: id.into(),
        points: vec![],
        width: 300.0,
        height: 200.0,
        show_grid: true,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> ScatterChartBuilder<M> {
    /// Add a data point.
    pub fn point(mut self, entry: ScatterPoint) -> Self {
        self.points.push(entry);
        self
    }

    /// Set the chart width.
    pub fn width(mut self, value: f32) -> Self {
        self.width = value;
        self
    }

    /// Set the chart height.
    pub fn height(mut self, value: f32) -> Self {
        self.height = value;
        self
    }

    /// Show/hide grid lines.
    pub fn show_grid(mut self, value: bool) -> Self {
        self.show_grid = value;
        self
    }

    /// Build the scatter chart widget.
    pub fn build(self) -> WidgetNode<M> {
        let id_prefix = self.id.as_str().to_string();

        let mut col = crate::column::<M>()
            .key(WidgetKey::new(format!("{}_scatter", &id_prefix)))
            .gap(4.0);

        // Header summary
        let summary = format!("Scatter — {} points", self.points.len());
        col = col.child(crate::label_with_size::<M>(summary, 14.0));

        if self.points.is_empty() {
            col = col.child(crate::label_with_size::<M>("(no data)", 13.0));
            return col.build();
        }

        // Build point rows
        let mut point_rows = crate::column::<M>().gap(2.0);

        for (i, p) in self.points.iter().enumerate() {
            let marker = format!("{} ({:.1}, {:.1})", p.marker_char(), p.x, p.y,);

            let row = crate::row::<M>()
                .key(format!("{}_point_{}", id_prefix, i).as_str())
                .gap(6.0)
                .child(crate::label_with_size::<M>(format!("{}:", p.label), 12.0))
                .child(crate::label_with_size::<M>(marker, 12.0))
                .build();

            point_rows = point_rows.child(row);
        }

        col = col.child(point_rows.build());

        col.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;
    use acme_layout::{LayoutEngine, WidgetLayoutContext};

    fn test_context() -> WidgetLayoutContext {
        WidgetLayoutContext {
            body_font_size: 16.0,
            body_line_height: 22.0,
            label_font_size: 14.0,
            control_height: 32.0,
            scale_factor: 1.0,
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn scatter_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = scatter_chart("sc").build();
        let ctx = test_context();
        let layout = node.to_layout_with_context(NodeId::new(1), &ctx);
        let snapshot = LayoutEngine::new()
            .compute(&layout, (800.0, 600.0))
            .unwrap();
        let rect = snapshot.get(NodeId::new(1)).unwrap();
        assert!(rect.height > 0.0, "scatter height should be > 0");
    }

    #[test]
    fn scatter_with_points() {
        let node: WidgetNode<TestMsg> = scatter_chart("sc")
            .point(ScatterPoint::new("A", 10.0, 20.0))
            .point(ScatterPoint::new("B", 30.0, 15.0).size(2.0))
            .build();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column")
        };
        // header + point_rows = 2
        assert_eq!(col.children.len(), 2);
    }

    #[test]
    fn scatter_empty_shows_placeholder() {
        let node: WidgetNode<TestMsg> = scatter_chart("sc").build();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column")
        };
        // header + placeholder = 2
        assert_eq!(col.children.len(), 2);
    }

    #[test]
    fn scatter_builder_defaults() {
        let s = scatter_chart::<TestMsg>("sc");
        assert!(s.points.is_empty());
        assert!(s.show_grid);
        assert!((s.width - 300.0).abs() < f32::EPSILON);
        assert!((s.height - 200.0).abs() < f32::EPSILON);
    }

    #[test]
    fn scatter_point_size_capped() {
        let p = ScatterPoint::new("x", 1.0, 2.0).size(0.0);
        assert!((p.size - 0.5).abs() < f32::EPSILON);
    }
}
