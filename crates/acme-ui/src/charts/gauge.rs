//! Gauge — immediate-mode semi-circular gauge rendered via [`WidgetNode::Canvas`].

use crate::WidgetNode;
use acme_core::{Point, WidgetKey};
use acme_widgets::canvas;
use acme_widgets::painter_bridge::theme_color;

/// Builder for a gauge widget.
pub struct GaugeBuilder<M> {
    pub id: WidgetKey,
    pub value: f32,
    pub size: f32,
    pub label: String,
    pub max: f32,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a gauge builder.
pub fn gauge<M: Clone + 'static>(id: impl Into<WidgetKey>) -> GaugeBuilder<M> {
    GaugeBuilder {
        id: id.into(),
        value: 0.0,
        size: 120.0,
        label: String::new(),
        max: 100.0,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> GaugeBuilder<M> {
    /// Set the current value.
    pub fn value(mut self, v: f32) -> Self {
        self.value = v;
        self
    }

    /// Set the gauge diameter.
    pub fn size(mut self, v: f32) -> Self {
        self.size = v;
        self
    }

    /// Set the descriptive label.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }

    /// Set the maximum value.
    pub fn max(mut self, m: f32) -> Self {
        self.max = m;
        self
    }

    /// Build the widget tree as a [`WidgetNode::Canvas`].
    pub fn build(self) -> WidgetNode<M> {
        let id_prefix = self.id.as_str().to_string();
        let ratio = (self.value / self.max.max(1.0)).clamp(0.0, 1.0);
        let pct = (ratio * 100.0).round() as u32;
        let size = self.size.max(32.0);
        let accessible = if self.label.is_empty() {
            format!("Gauge {pct}%")
        } else {
            format!("{}: {pct}%", self.label)
        };

        canvas::<M>()
            .key(format!("{id_prefix}_gauge"))
            .size(size, size)
            .accessible_label(accessible)
            .paint(move |painter, rect, theme| {
                let w = rect.size.width.get();
                let h = rect.size.height.get();
                let radius = (w.min(h) * 0.42).max(6.0);
                let thickness = (radius * 0.22).clamp(3.0, 10.0);
                let center = Point::new(
                    rect.origin.x.get() + w * 0.5,
                    rect.origin.y.get() + h * 0.65,
                );

                // Background semi-circular track (left PI -> top 1.5*PI -> right 2*PI)
                painter.arc_segments(
                    center,
                    radius,
                    std::f32::consts::PI,
                    std::f32::consts::PI,
                    thickness,
                    theme_color(theme.colors.muted),
                    24,
                );

                // Active value arc
                if ratio > 0.0 {
                    painter.arc_segments(
                        center,
                        radius,
                        std::f32::consts::PI,
                        std::f32::consts::PI * ratio,
                        thickness,
                        theme_color(theme.colors.primary),
                        24,
                    );
                }

                // Needle pointing toward current angle
                let angle = std::f32::consts::PI + std::f32::consts::PI * ratio;
                let needle_len = (radius - thickness * 0.6).max(3.0);
                let tip = Point::new(
                    center.x.get() + needle_len * angle.cos(),
                    center.y.get() + needle_len * angle.sin(),
                );
                painter.line(center, tip, 2.0, theme_color(theme.colors.foreground));
                painter.circle(
                    center,
                    (thickness * 0.55).max(3.0),
                    theme_color(theme.colors.accent),
                );
            })
            .build()
    }
}

impl<M: Clone + 'static> From<GaugeBuilder<M>> for WidgetNode<M> {
    fn from(b: GaugeBuilder<M>) -> Self {
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
    fn gauge_emits_canvas_and_draws_arc_and_needle() {
        let node: WidgetNode<Msg> = gauge("load")
            .size(120.0)
            .max(100.0)
            .value(72.0)
            .label("CPU")
            .into();

        let WidgetNode::Canvas(c) = &node else {
            panic!("expected Canvas");
        };
        assert_eq!(c.id.as_ref().map(WidgetKey::as_str), Some("load_gauge"));
        assert_eq!(c.accessible_label.as_deref(), Some("CPU: 72%"));

        let theme = Theme::dark();
        let mut painter = Painter::new();
        c.paint_into(&mut painter, Rect::new(0.0, 0.0, 120.0, 120.0), &theme);
        let scene = painter.finish();
        let quads = scene
            .commands()
            .iter()
            .filter(|cmd| matches!(cmd, DrawCommand::Quad(_)))
            .count();
        assert!(quads >= 20, "expected arc + needle quads, got {quads}");
    }
}
