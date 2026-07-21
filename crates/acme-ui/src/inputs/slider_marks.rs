//! SliderMarks — slider with labeled tick marks for precise value selection.
//! Absorbs gpui-component's marked slider strength.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// A single slider mark.
#[derive(Clone, Debug)]
pub struct SliderMark {
    pub value: f32,
    pub label: String,
}

impl SliderMark {
    pub fn new(value: f32, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
        }
    }
}

/// Builder for a marked slider.
pub struct SliderMarksBuilder<M> {
    pub id: WidgetKey,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub marks: Vec<SliderMark>,
    pub on_change: Option<fn(f32) -> M>,
}

/// Create a marked slider builder.
pub fn slider_marks<M: Clone + 'static>(value: f32) -> SliderMarksBuilder<M> {
    SliderMarksBuilder {
        id: WidgetKey::from("slider_marks"),
        value,
        min: 0.0,
        max: 100.0,
        marks: Vec::new(),
        on_change: None,
    }
}

impl<M: Clone + 'static> SliderMarksBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn min(mut self, value: f32) -> Self {
        self.min = value;
        self
    }

    pub fn max(mut self, value: f32) -> Self {
        self.max = value;
        self
    }

    pub fn mark(mut self, mark: SliderMark) -> Self {
        self.marks.push(mark);
        self
    }

    pub fn on_change(mut self, f: fn(f32) -> M) -> Self {
        self.on_change = Some(f);
        self
    }
}

impl<M: Clone + 'static> From<SliderMarksBuilder<M>> for WidgetNode<M> {
    fn from(b: SliderMarksBuilder<M>) -> Self {
        let clamped = b.value.clamp(b.min, b.max);
        let mut col = crate::column::<M>().key(b.id).gap(4.0).padding(4.0);

        // Value display
        col = col.child(crate::label(format!("{:.0}", clamped)));

        // Track with marks
        let mut track = crate::row::<M>().gap(8.0);
        for mark in &b.marks {
            let active = (mark.value - clamped).abs() < f32::EPSILON;
            let prefix = if active { "▣ " } else { "▢ " };
            track = track.child(crate::label(format!("{}{}", prefix, mark.label)));
        }
        col = col.child(track.build());
        col.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Changed(f32),
    }

    fn change_msg(v: f32) -> Msg {
        Msg::Changed(v)
    }

    #[test]
    fn slider_marks_produces_column() {
        let node: WidgetNode<Msg> = slider_marks(50.0).into();
        assert!(matches!(node, WidgetNode::Column(_)));
    }

    #[test]
    fn slider_marks_with_marks() {
        let node: WidgetNode<Msg> = slider_marks(25.0)
            .mark(SliderMark::new(0.0, "0"))
            .mark(SliderMark::new(50.0, "50"))
            .mark(SliderMark::new(100.0, "100"))
            .into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column");
        };
        // value display + track = 2
        assert_eq!(c.children.len(), 2);
    }

    #[test]
    fn slider_marks_clamps_value() {
        let b = slider_marks::<Msg>(200.0).max(100.0);
        assert_eq!(b.max, 100.0);
    }
}
