//! ProgressRing — circular/radial progress indicator.
//! Absorbs gpui-component's circular progress strength (complements linear progress).

use crate::WidgetNode;
use acme_core::WidgetKey;

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
        // Approximate the ring fill with a fraction-of-8 glyph.
        let eighths = ((b.value / 100.0) * 8.0).round() as usize;
        let ring_glyph = match eighths {
            0 => "○",
            1 => "◔",
            2 | 3 => "◑",
            4 | 5 | 6 | 7 => "◕",
            _ => "●",
        };

        let mut stack = crate::stack::<M>().key(b.id).size(b.size, b.size);
        stack = stack.child(crate::label(ring_glyph));
        if b.show_label {
            stack = stack.child(crate::label(format!("{:.0}%", b.value)));
        }
        stack.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }
}
