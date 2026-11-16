//! BarChart — immediate-mode bar chart rendered via [`WidgetNode::Canvas`].

use crate::WidgetNode;
use acme_core::{Point, Rect, RectStyle, WidgetKey};
use acme_widgets::canvas;
use acme_widgets::painter_bridge::theme_color;

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
    pub width: f32,
    pub height: f32,
    pub bars: Vec<BarEntry>,
    pub max_value: f32,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a bar chart builder.
pub fn bar_chart<M: Clone + 'static>(id: impl Into<WidgetKey>) -> BarChartBuilder<M> {
    BarChartBuilder {
        id: id.into(),
        width: 320.0,
        height: 200.0,
        bars: vec![],
        max_value: 100.0,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> BarChartBuilder<M> {
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

    /// Build the widget tree — returns a [`WidgetNode::Canvas`] drawing proportional bars.
    pub fn build(self) -> WidgetNode<M> {
        let max_v = self.max_value.max(1.0);
        let id_prefix = self.id.as_str().to_string();
        let bars = self.bars;
        let width = self.width.max(40.0);
        let height = self.height.max(40.0);
        let accessible = format!("BarChart — {} bars", bars.len());

        canvas::<M>()
            .key(format!("{id_prefix}_chart"))
            .size(width, height)
            .accessible_label(accessible)
            .paint(move |painter, rect, theme| {
                let bg = theme_color(theme.colors.surface);
                let border = theme_color(theme.colors.border);
                painter.rect(
                    rect,
                    RectStyle::solid(bg)
                        .radius(theme.radii.md)
                        .border(1.0, border),
                );

                let pad = theme.spacing.px3;
                let inner_w = (rect.size.width.get() - pad * 2.0).max(1.0);
                let inner_h = (rect.size.height.get() - pad * 2.0).max(1.0);
                let left = rect.origin.x.get() + pad;
                let top = rect.origin.y.get() + pad;
                let bottom = top + inner_h;

                // Baseline axis
                painter.line(
                    Point::new(left, bottom),
                    Point::new(left + inner_w, bottom),
                    1.0,
                    border,
                );

                if bars.is_empty() {
                    return;
                }

                let palette = [
                    theme.colors.primary,
                    theme.colors.success,
                    theme.colors.info,
                    theme.colors.warning,
                    theme.colors.danger,
                ];
                let slot_w = inner_w / bars.len() as f32;
                let gap = (slot_w * 0.25).min(theme.spacing.px2);
                let bar_w = (slot_w - gap).max(2.0);

                for (i, entry) in bars.iter().enumerate() {
                    let ratio = (entry.value / max_v).clamp(0.0, 1.0);
                    let bar_h = (inner_h * ratio).max(if ratio > 0.0 { 2.0 } else { 0.0 });
                    if bar_h <= 0.0 {
                        continue;
                    }
                    let bx = left + i as f32 * slot_w + (slot_w - bar_w) * 0.5;
                    let by = bottom - bar_h;
                    let fill = theme_color(palette[i % palette.len()]);
                    painter.rounded_rect(
                        Rect::new(bx, by, bar_w, bar_h),
                        theme.radii.sm.min(bar_w * 0.5),
                        fill,
                    );
                }
            })
            .build()
    }
}

impl<M: Clone + 'static> From<BarChartBuilder<M>> for WidgetNode<M> {
    fn from(b: BarChartBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::{DrawCommand, Painter};
    use acme_theme::Theme;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn bar_chart_emits_canvas_and_draws_bars() {
        let node: WidgetNode<Msg> = bar_chart("sales")
            .height(160.0)
            .max_value(100.0)
            .bar(BarEntry::new("Q1", 40.0))
            .bar(BarEntry::new("Q2", 80.0))
            .bar(BarEntry::new("Q3", 60.0))
            .into();

        let WidgetNode::Canvas(c) = &node else {
            panic!("expected Canvas");
        };
        assert_eq!(c.id.as_ref().map(WidgetKey::as_str), Some("sales_chart"));

        let theme = Theme::dark();
        let mut painter = Painter::new();
        c.paint_into(&mut painter, Rect::new(0.0, 0.0, 320.0, 160.0), &theme);
        let scene = painter.finish();
        let quads = scene
            .commands()
            .iter()
            .filter(|cmd| matches!(cmd, DrawCommand::Quad(_)))
            .count();
        // Surface bg + baseline + 3 bars = >= 5 quads
        assert!(quads >= 5, "expected at least 5 quads, got {quads}");
    }
}
