//! ProgressRing — circular/radial progress indicator.
//! Absorbs gpui-component's circular progress strength (complements linear progress).

use crate::WidgetNode;
use acme_core::{Point, WidgetKey};
use acme_widgets::painter_bridge::theme_color;
use acme_widgets::{canvas, label, stack};

/// Builder for a circular progress ring.
pub struct ProgressRingBuilder<M> {
    pub id: WidgetKey,
    pub value: f32,
    pub size: f32,
    pub show_label: bool,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a progress ring builder.
pub fn progress_ring<M: Clone + 'static>(value: f32) -> ProgressRingBuilder<M> {
    ProgressRingBuilder {
        id: WidgetKey::from("progress_ring"),
        value: value.clamp(0.0, 100.0),
        size: 48.0,
        show_label: true,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> ProgressRingBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn size(mut self, value: f32) -> Self {
        self.size = value;
        self
    }

    pub fn show_label(mut self, value: bool) -> Self {
        self.show_label = value;
        self
    }
}

impl<M: Clone + 'static> From<ProgressRingBuilder<M>> for WidgetNode<M> {
    fn from(b: ProgressRingBuilder<M>) -> Self {
        let ratio = (b.value / 100.0).clamp(0.0, 1.0);
        let pct = (ratio * 100.0).round() as u32;
        let ring_size = b.size.max(8.0);
        let id_str = b.id.as_str().to_string();

        let ring_canvas = canvas::<M>()
            .key(format!("{id_str}_ring"))
            .size(ring_size, ring_size)
            .accessible_label(format!("Progress {pct}%"))
            .paint(move |painter, rect, theme| {
                let w = rect.size.width.get();
                let h = rect.size.height.get();
                let center =
                    Point::new(rect.origin.x.get() + w * 0.5, rect.origin.y.get() + h * 0.5);
                let outer_r = (w.min(h) * 0.5 - 2.0).max(2.0);
                let thickness = (outer_r * 0.22).clamp(2.0, 8.0);
                let arc_r = (outer_r - thickness * 0.5).max(1.0);

                painter.ring(center, outer_r, thickness, theme_color(theme.colors.muted));
                if ratio > 0.0 {
                    painter.arc_segments(
                        center,
                        arc_r,
                        -std::f32::consts::FRAC_PI_2,
                        std::f32::consts::TAU * ratio,
                        thickness,
                        theme_color(theme.colors.primary),
                        32,
                    );
                }
            });

        let mut root = stack::<M>().key(b.id).size(ring_size, ring_size);
        root = root.child(ring_canvas);
        if b.show_label {
            root = root.child(label(format!("{pct}%")));
        }
        root.build()
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
    fn progress_ring_produces_stack() {
        let node: WidgetNode<Msg> = progress_ring(50.0).into();
        assert!(matches!(node, WidgetNode::Stack(_)));
    }

    #[test]
    fn progress_ring_with_label() {
        let node: WidgetNode<Msg> = progress_ring(75.0).into();
        let WidgetNode::Stack(s) = &node else {
            panic!("expected Stack");
        };
        assert_eq!(s.children.len(), 2);
        assert!(matches!(s.children[0], WidgetNode::Canvas(_)));
        assert!(matches!(s.children[1], WidgetNode::Label(_)));
    }

    #[test]
    fn progress_ring_value_clamped() {
        let b = progress_ring::<Msg>(150.0);
        assert_eq!(b.value, 100.0);
    }

    #[test]
    fn progress_ring_no_label() {
        let node: WidgetNode<Msg> = progress_ring(50.0).show_label(false).into();
        let WidgetNode::Stack(s) = &node else {
            panic!("expected Stack");
        };
        assert_eq!(s.children.len(), 1);
        assert!(matches!(s.children[0], WidgetNode::Canvas(_)));
    }

    #[test]
    fn progress_ring_canvas_paints_track_and_arc_commands() {
        let node: WidgetNode<Msg> = progress_ring(60.0).into();
        let WidgetNode::Stack(s) = &node else {
            panic!("expected Stack");
        };
        let WidgetNode::Canvas(c) = &s.children[0] else {
            panic!("expected Canvas child");
        };
        let theme = Theme::dark();
        let mut painter = Painter::new();
        c.paint_into(&mut painter, Rect::new(0.0, 0.0, 48.0, 48.0), &theme);
        let scene = painter.finish();
        assert!(
            scene
                .commands()
                .iter()
                .any(|cmd| matches!(cmd, DrawCommand::Quad(_))),
            "expected ring and arc quads in scene"
        );
    }
}
