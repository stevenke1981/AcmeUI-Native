//! Mobile segmented control — N-segment row with selected highlight.

use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_layout::Length;
use acme_widgets::style::prelude::*;

/// Builder for a mobile segmented control.
pub struct MobileSegmentedBuilder<M> {
    pub id: WidgetKey,
    pub segments: Vec<String>,
    pub selected_index: usize,
    pub disabled: bool,
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
        disabled: false,
        on_select: None,
    }
}

impl<M: Clone + 'static> MobileSegmentedBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    /// Select an index. Out-of-range indices leave all segments unselected.
    pub fn selected_index(mut self, index: usize) -> Self {
        self.selected_index = index;
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    pub fn on_select(mut self, f: fn(usize) -> M) -> Self {
        self.on_select = Some(f);
        self
    }
}

impl<M: Clone + 'static> From<MobileSegmentedBuilder<M>> for WidgetNode<M> {
    fn from(b: MobileSegmentedBuilder<M>) -> Self {
        let id = b.id.as_str();
        let mut row = crate::row::<M>().key(b.id.clone()).gap(0.0).padding(4.0);
        for (i, seg) in b.segments.iter().enumerate() {
            let selected = i == b.selected_index;
            let text = if selected {
                format!("▣ {seg} (selected)")
            } else {
                seg.clone()
            };
            let button = crate::button(format!("{id}__segment_{i}"), text)
                .variant(if selected {
                    crate::ButtonVariant::Primary
                } else {
                    crate::ButtonVariant::Secondary
                })
                .h(Length::px(44.0))
                .min_h(Length::px(44.0))
                .disabled(b.disabled);
            let node: WidgetNode<M> = if let Some(f) = b.on_select {
                button.on_click(f(i))
            } else {
                button.into()
            };
            row = row.child(node);
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

    #[test]
    fn mobile_segmented_on_select_maps_index_to_message() {
        let b = mobile_segmented::<Msg>(vec!["A", "B"]).on_select(Msg::Selected);
        let f = b.on_select.expect("on_select stored");
        assert_eq!(f(1), Msg::Selected(1));
    }
}

#[cfg(test)]
#[path = "mobile_segmented_tests.rs"]
mod interaction_tests;
