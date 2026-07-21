//! Mobile segmented control — N-segment row with selected highlight.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a mobile segmented control.
pub struct MobileSegmentedBuilder<M> {
    pub id: WidgetKey,
    pub segments: Vec<String>,
    pub selected_index: usize,
    pub on_select: Option<fn(usize) -> M>,
}

/// Create a mobile segmented control builder.
pub fn mobile_segmented<M: Clone + 'static>(
    segments: Vec<impl Into<String>>,
) -> MobileSegmentedBuilder<M> {
    MobileSegmentedBuilder {
        id: WidgetKey::from("mobile_segmented"),
        segments: segments.into_iter().map(Into::into).collect(),
        selected_index: 0,
        on_select: None,
    }
}

impl<M: Clone + 'static> MobileSegmentedBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn selected_index(mut self, index: usize) -> Self {
        self.selected_index = index;
        self
    }

    pub fn on_select(mut self, f: fn(usize) -> M) -> Self {
        self.on_select = Some(f);
        self
    }
}

impl<M: Clone + 'static> From<MobileSegmentedBuilder<M>> for WidgetNode<M> {
    fn from(b: MobileSegmentedBuilder<M>) -> Self {
        let mut row = crate::row::<M>().key(b.id).gap(0.0).padding(4.0);
        for (i, seg) in b.segments.iter().enumerate() {
            let prefix = if i == b.selected_index { "▣ " } else { "" };
            let text = format!("{}{}", prefix, seg);
            row = row.child(crate::label(text));
        }
        row.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Selected(usize),
    }

    #[test]
    fn mobile_segmented_produces_row() {
        let node: WidgetNode<Msg> = mobile_segmented(vec!["A", "B", "C"]).into();
        assert!(matches!(node, WidgetNode::Row(_)));
    }

    #[test]
    fn mobile_segmented_child_count_matches_segments() {
        let node: WidgetNode<Msg> = mobile_segmented(vec!["X", "Y"]).into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        assert_eq!(r.children.len(), 2);
    }

    #[test]
    fn mobile_segmented_selected_index() {
        let b = mobile_segmented::<Msg>(vec!["A", "B"]).selected_index(1);
        assert_eq!(b.selected_index, 1);
    }
}
