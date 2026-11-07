//! Interaction tests for `mobile_segmented`.

use super::*;
use crate::mobile::mobile_toggle::test_support::{Laid, lay_out};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Selected(usize),
}

fn laid(builder: MobileSegmentedBuilder<Msg>) -> Laid<Msg> {
    lay_out(builder.key("seg").into())
}

fn variants(node: &WidgetNode<Msg>) -> Vec<crate::ButtonVariant> {
    node.children()
        .iter()
        .filter_map(|c| match c {
            WidgetNode::Button(b) => Some(b.variant),
            _ => None,
        })
        .collect()
}

#[test]
fn pressing_each_segment_emits_its_index() {
    let l = laid(mobile_segmented(vec!["A", "B", "C"]).on_select(Msg::Selected));
    for index in 0..3 {
        assert_eq!(l.press_button(index), Some(Msg::Selected(index)));
    }
}

#[test]
fn the_selected_segment_uses_the_theme_primary_variant() {
    let node: WidgetNode<Msg> = mobile_segmented(vec!["A", "B", "C"])
        .selected_index(1)
        .on_select(Msg::Selected)
        .into();
    use crate::ButtonVariant::{Primary, Secondary};
    assert_eq!(variants(&node), [Secondary, Primary, Secondary]);
}

#[test]
fn an_out_of_range_selected_index_highlights_nothing_and_does_not_panic() {
    let node: WidgetNode<Msg> = mobile_segmented(vec!["A", "B"])
        .selected_index(9)
        .on_select(Msg::Selected)
        .into();
    assert!(
        variants(&node)
            .iter()
            .all(|v| *v != crate::ButtonVariant::Primary)
    );
    let l = laid(
        mobile_segmented(vec!["A", "B"])
            .selected_index(usize::MAX)
            .on_select(Msg::Selected),
    );
    assert_eq!(l.press_button(1), Some(Msg::Selected(1)));
}

#[test]
fn zero_segments_is_an_empty_row() {
    let l = laid(mobile_segmented::<Msg>(Vec::<String>::new()).on_select(Msg::Selected));
    assert!(l.hits().is_empty());
    assert_eq!(l.press_at(10.0, 10.0), None);
}

#[test]
fn one_segment_is_pressable() {
    let l = laid(mobile_segmented(vec!["Only"]).on_select(Msg::Selected));
    assert_eq!(l.press_button(0), Some(Msg::Selected(0)));
}

#[test]
fn without_on_select_nothing_is_delivered() {
    let l = laid(mobile_segmented(vec!["A", "B"]));
    assert_eq!(l.press_button(0), None);
}

#[test]
fn disabled_segments_emit_nothing() {
    let l = laid(
        mobile_segmented(vec!["A", "B"])
            .disabled(true)
            .on_select(Msg::Selected),
    );
    assert_eq!(l.press_button(0), None);
    assert_eq!(l.press_button(1), None);
}

#[test]
fn segments_sit_side_by_side_and_are_44_px_touch_targets() {
    let l = laid(mobile_segmented(vec!["A", "B", "C"]).on_select(Msg::Selected));
    let hits = l.hits();
    assert_eq!(hits.len(), 3);
    for hit in &hits {
        assert!(hit.rect[3] >= 44.0, "height {}", hit.rect[3]);
    }
    for pair in hits.windows(2) {
        assert!(pair[0].rect[0] + pair[0].rect[2] <= pair[1].rect[0] + 0.01);
    }
}

#[test]
fn the_selected_segment_keeps_a_non_colour_cue() {
    let node: WidgetNode<Msg> = mobile_segmented(vec!["A", "B"]).selected_index(0).into();
    let labels: Vec<String> = node
        .children()
        .iter()
        .filter_map(|c| match c {
            WidgetNode::Button(b) => Some(b.label.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(labels, ["▣ A (selected)", "B"]);
}
