//! LineChart — immediate-mode line chart rendered via [`WidgetNode::Canvas`].

use crate::WidgetNode;
use acme_core::{Point, RectStyle, WidgetKey};
use acme_widgets::canvas;
use acme_widgets::painter_bridge::theme_color;

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
    pub width: f32,
    pub height: f32,
    pub data: Vec<ChartPoint>,
    pub show_dots: bool,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a line chart builder.
pub fn line_chart<M: Clone + 'static>(id: impl Into<WidgetKey>) -> LineChartBuilder<M> {
    LineChartBuilder {
        id: id.into(),
        width: 320.0,
        height: 200.0,
        data: vec![],
        show_dots: true,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> LineChartBuilder<M> {
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

    /// Build the widget tree as a [`WidgetNode::Canvas`].
    pub fn build(self) -> WidgetNode<M> {
        let id_prefix = self.id.as_str().to_string();
        let data = self.data;
        let show_dots = self.show_dots;
        let width = self.width.max(40.0);
        let height = self.height.max(40.0);
        let accessible = format!("LineChart — {} points", data.len());

        canvas::<M>()
            .key(format!("{id_prefix}_chart"))
            .size(width, height)
            .accessible_label(accessible)
            .paint(move |painter, rect, theme| {
                let surface = theme_color(theme.colors.surface);
                let border = theme_color(theme.colors.border);
                painter.rect(
                    rect,
                    RectStyle::solid(surface)
                        .radius(theme.radii.md)
                        .border(1.0, border),
                );

                let pad = theme.spacing.px3;
                let inner_w = (rect.size.width.get() - pad * 2.0).max(1.0);
                let inner_h = (rect.size.height.get() - pad * 2.0).max(1.0);
                let left = rect.origin.x.get() + pad;
                let top = rect.origin.y.get() + pad;
                let bottom = top + inner_h;

                // Y and X axis lines + mid gridline
                painter.line(Point::new(left, top), Point::new(left, bottom), 1.0, border);
                painter.line(
                    Point::new(left, bottom),
                    Point::new(left + inner_w, bottom),
                    1.0,
                    border,
                );
                let mid_y = top + inner_h * 0.5;
                painter.line(
                    Point::new(left, mid_y),
                    Point::new(left + inner_w, mid_y),
                    1.0,
                    theme_color(theme.colors.border.with_alpha(0.45)),
                );

                if data.is_empty() {
                    return;
                }

                let max_v = data.iter().map(|p| p.value).fold(0.0f32, f32::max).max(1.0);
                let denom = (data.len().saturating_sub(1)).max(1) as f32;
                let line_color = theme_color(theme.colors.primary);
                let dot_color = theme_color(theme.colors.accent);

                let points: Vec<Point<acme_core::Logical>> = data
                    .iter()
                    .enumerate()
                    .map(|(i, pt)| {
                        let ratio = (pt.value / max_v).clamp(0.0, 1.0);
                        let x = if data.len() == 1 {
                            left + inner_w * 0.5
                        } else {
                            left + (i as f32 / denom) * inner_w
                        };
                        let y = bottom - ratio * inner_h;
                        Point::new(x, y)
                    })
                    .collect();

                if points.len() >= 2 {
                    painter.polyline(&points, 2.0, line_color);
                }

                if show_dots {
                    for &pt in &points {
                        painter.circle(pt, 4.0, dot_color);
                        painter.circle(pt, 2.0, surface);
                    }
                }
            })
            .build()
    }
}

impl<M: Clone + 'static> From<LineChartBuilder<M>> for WidgetNode<M> {
    fn from(b: LineChartBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::{DrawCommand, Painter, Rect};
    use acme_theme::Theme;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn line_chart_emits_canvas_and_draws_axes_polyline_and_dots() {
        let node: WidgetNode<Msg> = line_chart("latency")
            .height(180.0)
            .point(ChartPoint::new("Mon", 20.0))
            .point(ChartPoint::new("Tue", 60.0))
            .point(ChartPoint::new("Wed", 45.0))
            .show_dots(true)
            .into();

        let WidgetNode::Canvas(c) = &node else {
            panic!("expected Canvas");
        };
        assert_eq!(c.id.as_ref().map(WidgetKey::as_str), Some("latency_chart"));

        let theme = Theme::dark();
        let mut painter = Painter::new();
        c.paint_into(&mut painter, Rect::new(0.0, 0.0, 320.0, 180.0), &theme);
        let scene = painter.finish();
        let quads = scene
            .commands()
            .iter()
            .filter(|cmd| matches!(cmd, DrawCommand::Quad(_)))
            .count();
        // bg + 3 grid/axes + segments + 6 dot circles = >= 10 quads
        assert!(quads >= 10, "expected >= 10 quads, got {quads}");
    }
}
