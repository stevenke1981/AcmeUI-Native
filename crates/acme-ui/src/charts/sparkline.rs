//! Sparkline — immediate-mode inline trend chart rendered via [`WidgetNode::Canvas`].

use crate::WidgetNode;
use acme_core::{Point, WidgetKey};
use acme_widgets::canvas;
use acme_widgets::painter_bridge::theme_color;

/// Builder for a sparkline widget.
pub struct SparklineBuilder<M> {
    pub id: WidgetKey,
    pub data: Vec<f32>,
    pub width: f32,
    pub height: f32,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a sparkline builder.
pub fn sparkline<M: Clone + 'static>(id: impl Into<WidgetKey>) -> SparklineBuilder<M> {
    SparklineBuilder {
        id: id.into(),
        data: vec![],
        width: 120.0,
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

    /// Set the sparkline width.
    pub fn width(mut self, w: f32) -> Self {
        self.width = w;
        self
    }

    /// Set the sparkline height.
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }

    /// Build the widget tree as a [`WidgetNode::Canvas`].
    pub fn build(self) -> WidgetNode<M> {
        let id_prefix = self.id.as_str().to_string();
        let data = self.data;
        let height = self.height.max(8.0);
        let width = self.width.max((data.len() as f32 * 8.0).max(48.0));
        let accessible = format!("Sparkline — {} points", data.len());

        canvas::<M>()
            .key(format!("{id_prefix}_sparkline"))
            .size(width, height)
            .accessible_label(accessible)
            .paint(move |painter, rect, theme| {
                let pad = 2.0;
                let inner_w = (rect.size.width.get() - pad * 2.0).max(1.0);
                let inner_h = (rect.size.height.get() - pad * 2.0).max(1.0);
                let left = rect.origin.x.get() + pad;
                let top = rect.origin.y.get() + pad;
                let bottom = top + inner_h;

                painter.line(
                    Point::new(left, bottom),
                    Point::new(left + inner_w, bottom),
                    1.0,
                    theme_color(theme.colors.border),
                );

                if data.is_empty() {
                    return;
                }

                let min_v = data.iter().copied().fold(f32::INFINITY, f32::min);
                let max_v = data.iter().copied().fold(f32::NEG_INFINITY, f32::max);
                let span = (max_v - min_v).abs().max(1e-5);
                let stroke = theme_color(theme.colors.primary);

                if data.len() == 1 {
                    let pt = Point::new(left + inner_w * 0.5, top + inner_h * 0.5);
                    painter.circle(pt, 2.5, stroke);
                    return;
                }

                let denom = (data.len() - 1) as f32;
                let points: Vec<Point<acme_core::Logical>> = data
                    .iter()
                    .enumerate()
                    .map(|(i, &val)| {
                        let t = (val - min_v) / span;
                        let x = left + (i as f32 / denom) * inner_w;
                        let y = bottom - t * inner_h;
                        Point::new(x, y)
                    })
                    .collect();

                painter.polyline(&points, 2.0, stroke);
                if let Some(&last) = points.last() {
                    painter.circle(last, 2.5, theme_color(theme.colors.accent));
                }
            })
            .build()
    }
}

impl<M: Clone + 'static> From<SparklineBuilder<M>> for WidgetNode<M> {
    fn from(b: SparklineBuilder<M>) -> Self {
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
    fn sparkline_emits_canvas_and_draws_polyline() {
        let node: WidgetNode<Msg> = sparkline("cpu")
            .height(32.0)
            .value(10.0)
            .value(55.0)
            .value(35.0)
            .value(90.0)
            .into();

        let WidgetNode::Canvas(c) = &node else {
            panic!("expected Canvas");
        };
        assert_eq!(c.id.as_ref().map(WidgetKey::as_str), Some("cpu_sparkline"));

        let theme = Theme::light();
        let mut painter = Painter::new();
        c.paint_into(&mut painter, Rect::new(0.0, 0.0, 120.0, 32.0), &theme);
        let scene = painter.finish();
        let quads = scene
            .commands()
            .iter()
            .filter(|cmd| matches!(cmd, DrawCommand::Quad(_)))
            .count();
        // Baseline + polyline segments + terminal dot
        assert!(quads >= 5, "expected at least 5 quads, got {quads}");
    }
}
