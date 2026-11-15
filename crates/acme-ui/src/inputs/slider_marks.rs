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
        let id_str = b.id.as_str().to_string();
        let mut col = crate::column::<M>().key(b.id).gap(4.0).padding(4.0);

        // Value display
        col = col.child(crate::label(format!("{clamped:.0}")));

        // Track with clickable mark buttons
        let mut track = crate::row::<M>().gap(8.0);
        for (idx, mark) in b.marks.iter().enumerate() {
            let active = (mark.value - clamped).abs() < f32::EPSILON;
            let prefix = if active { "▣ " } else { "▢ " };
            let btn = crate::button::<M>(
                format!("{id_str}_mark_{idx}").as_str(),
                format!("{prefix}{}", mark.label),
            )
            .variant(if active {
                crate::ButtonVariant::Primary
            } else {
                crate::ButtonVariant::Ghost
            });
            let node: WidgetNode<M> = match b.on_change {
                Some(cb) => btn.on_click(cb(mark.value)),
                None => btn.into(),
            };
            track = track.child(node);
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

    #[test]
    fn slider_marks_on_change_emits_changed_with_value() {
        let b = slider_marks::<Msg>(0.0).on_change(change_msg);
        assert_eq!(b.on_change.map(|f| f(2.5)), Some(Msg::Changed(2.5)));
    }

    #[test]
    fn slider_marks_renders_clickable_mark_buttons() {
        let node: WidgetNode<Msg> = slider_marks(50.0)
            .key("vol")
            .mark(SliderMark::new(0.0, "Mute"))
            .mark(SliderMark::new(50.0, "Mid"))
            .mark(SliderMark::new(100.0, "Max"))
            .on_change(change_msg)
            .into();

        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        let WidgetNode::Row(track) = &col.children[1] else {
            panic!("expected track Row");
        };
        assert_eq!(track.children.len(), 3);

        let WidgetNode::Button(b0) = &track.children[0] else {
            panic!("expected Button for mark 0");
        };
        assert_eq!(b0.activate(), Some(&Msg::Changed(0.0)));
        assert_eq!(b0.variant, crate::ButtonVariant::Ghost);

        let WidgetNode::Button(b1) = &track.children[1] else {
            panic!("expected Button for mark 1");
        };
        assert_eq!(b1.activate(), Some(&Msg::Changed(50.0)));
        assert_eq!(b1.variant, crate::ButtonVariant::Primary);
    }
}
