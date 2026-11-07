//! Retained identity evidence for the actual Playground builder and frame path.
//! The compatibility adapter deliberately preserves legacy `to_layout` styles
//! and measure specs, rather than switching to intrinsic text measurement.

use std::collections::HashSet;

use acme_core::ViewNode;
use acme_layout::{LayoutRect, LayoutSnapshot};

use super::*;

const VIEWPORTS: [(f32, f32); 3] = [(1080.0, 720.0), (640.0, 480.0), (1920.0, 1080.0)];
const STATES: [(bool, u32, bool); 5] = [
    (false, 0, false),
    (true, 1, false),
    (true, 42, true),
    (false, 999, true),
    (false, u32::MAX, false),
];

/// Allocate an unrelated identity first so retained and sequential IDs differ.
fn playground() -> Playground {
    let mut playground = Playground::new();
    playground
        .retained
        .reconcile_roots(&[ViewNode::new("previous-root", "column")])
        .expect("seed retained IDs");
    playground
}

fn select_state(playground: &mut Playground, (dark, count, danger): (bool, u32, bool)) {
    playground.dark = dark;
    playground.click_count = count;
    playground.danger_mode = danger;
}

fn ids(root: &LayoutNode) -> Vec<NodeId> {
    let mut all = vec![root.id];
    for child in &root.children {
        all.extend(ids(child));
    }
    all
}

fn rects(root: &LayoutNode, snapshot: &LayoutSnapshot) -> Vec<LayoutRect> {
    let mut all = vec![*snapshot.get(root.id).expect("node has geometry")];
    for child in &root.children {
        all.extend(rects(child, snapshot));
    }
    all
}

fn assert_legacy_shape(retained: &LayoutNode, sequential: &LayoutNode) {
    assert_eq!(retained.style, sequential.style);
    assert_eq!(retained.measure, sequential.measure);
    assert_eq!(retained.children.len(), sequential.children.len());
    for (retained, sequential) in retained.children.iter().zip(&sequential.children) {
        assert_legacy_shape(retained, sequential);
    }
}

fn variant_buttons(
    description: &mut WidgetNode<PlaygroundMessage>,
) -> &mut Vec<WidgetNode<PlaygroundMessage>> {
    let WidgetNode::Column(root) = description else {
        panic!("Playground root is a column");
    };
    let WidgetNode::Column(content) = &mut root.children[4] else {
        panic!("Playground content is a column");
    };
    let WidgetNode::Row(buttons) = &mut content.children[1] else {
        panic!("Playground variants use a row");
    };
    &mut buttons.children
}

fn variant_ids(root: &LayoutNode) -> Vec<NodeId> {
    root.children[4].children[1]
        .children
        .iter()
        .map(|button| button.id)
        .collect()
}

#[test]
fn actual_builder_binds_completely_and_keeps_ids_across_state_changes() {
    let mut playground = playground();
    let mut first = None;
    for state in STATES {
        select_state(&mut playground, state);
        let description = playground.build_tree();
        let root = playground.layout_root(&description);
        assert!(!playground.retained_error_logged);
        let binding = bind_retained(std::slice::from_ref(&description), &playground.retained);
        assert_eq!(binding.mismatches(), 0);
        assert_eq!(binding.len(), 32);
        let current = ids(&root);
        assert_eq!(current.len(), 32);
        assert_eq!(current.iter().copied().collect::<HashSet<_>>().len(), 32);
        assert_eq!(root.id, playground.retained.roots()[0]);
        assert_ne!(root.id, NodeId::new(1));
        if let Some(first) = &first {
            assert_eq!(&current, first, "IDs changed for state {state:?}");
        } else {
            first = Some(current);
        }
    }
}

#[test]
fn retained_layout_preserves_legacy_styles_measurements_and_computed_geometry() {
    let mut playground = playground();
    for state in STATES {
        select_state(&mut playground, state);
        for viewport in VIEWPORTS {
            let description = playground.build_tree();
            let mut retained = playground.layout_root(&description);
            let mut sequential = description.to_layout(NodeId::new(1));
            assert_legacy_shape(&retained, &sequential);
            apply_playground_styles(&mut retained, viewport.0, viewport.1);
            apply_playground_styles(&mut sequential, viewport.0, viewport.1);
            assert_legacy_shape(&retained, &sequential);
            let retained_snapshot = playground
                .layout
                .compute(&retained, viewport)
                .expect("layout");
            let sequential_snapshot = playground
                .layout
                .compute(&sequential, viewport)
                .expect("layout");
            assert_eq!(
                rects(&retained, &retained_snapshot),
                rects(&sequential, &sequential_snapshot),
                "geometry changed for state {state:?}, viewport {viewport:?}"
            );
            let retained_ids = extract_playground_ids(&retained);
            let sequential_ids = extract_playground_ids(&sequential);
            assert_eq!(
                retained_snapshot.scroll_metrics(retained_ids.content),
                sequential_snapshot.scroll_metrics(sequential_ids.content)
            );
        }
    }
}

#[test]
fn keyed_buttons_keep_ids_through_reorder_and_insertion_in_actual_builder() {
    let mut playground = playground();
    let first = playground.build_tree();
    let before = variant_ids(&playground.layout_root(&first));
    let mut reordered = playground.build_tree();
    variant_buttons(&mut reordered).rotate_right(1);
    let after = variant_ids(&playground.layout_root(&reordered));
    assert_eq!(after, vec![before[3], before[0], before[1], before[2]]);

    variant_buttons(&mut reordered).insert(0, button("inserted", "Inserted").into());
    let inserted = variant_ids(&playground.layout_root(&reordered));
    assert_eq!(&inserted[1..], &after);
    assert!(!before.contains(&inserted[0]));
    assert!(!playground.retained_error_logged);
}

#[test]
fn duplicate_keys_use_wholly_sequential_ids_then_recover_previous_identities() {
    let mut playground = playground();
    let description = playground.build_tree();
    let before = ids(&playground.layout_root(&description));
    let mut invalid = playground.build_tree();
    let buttons = variant_buttons(&mut invalid);
    let WidgetNode::Button(second) = &mut buttons[1] else {
        panic!("variant is a button");
    };
    second.key = "primary".into();
    for _ in 0..2 {
        let fallback = playground.layout_root(&invalid);
        assert_eq!(fallback, invalid.to_layout(NodeId::new(1)));
        assert!(playground.retained_error_logged);
    }
    let recovered = playground.layout_root(&description);
    assert_eq!(ids(&recovered), before);
    assert!(!playground.retained_error_logged);
}

#[test]
fn unsupported_widget_falls_back_without_mixing_ids_and_recovers() {
    let mut playground = playground();
    let description = playground.build_tree();
    let before = ids(&playground.layout_root(&description));
    let mut unsupported = playground.build_tree();
    variant_buttons(&mut unsupported).push(acme_widgets::text_input("input").build());
    let fallback = playground.layout_root(&unsupported);
    assert_eq!(fallback, unsupported.to_layout(NodeId::new(1)));
    assert!(playground.retained_error_logged);
    assert_eq!(ids(&playground.layout_root(&description)), before);
    assert!(!playground.retained_error_logged);
}

#[test]
fn incomplete_binding_or_layout_shape_is_rejected() {
    let mut playground = playground();
    let description = playground.build_tree();
    playground.layout_root(&description);
    let mut stale_description = playground.build_tree();
    variant_buttons(&mut stale_description).push(button("new", "New").into());
    let binding = bind_retained(
        std::slice::from_ref(&stale_description),
        &playground.retained,
    );
    assert!(binding.mismatches() > 0);
    let mut root = stale_description.to_layout(NodeId::new(1));
    assert!(!assign_retained_ids(
        &stale_description,
        &binding,
        &mut root
    ));

    let binding = bind_retained(std::slice::from_ref(&description), &playground.retained);
    let mut incomplete_layout = description.to_layout(NodeId::new(1));
    incomplete_layout.children.pop();
    assert!(!assign_retained_ids(
        &description,
        &binding,
        &mut incomplete_layout
    ));
}

#[test]
fn real_frame_uses_retained_ids_for_button_geometry_and_preserves_hit_rects() {
    let mut playground = playground();
    let mut first_ids = None;
    for state in STATES {
        select_state(&mut playground, state);
        for viewport in VIEWPORTS {
            playground.scroll = 100.0;
            let scene = playground.frame(FrameContext {
                window: WindowId(0),
                logical_width: viewport.0,
                logical_height: viewport.1,
                scale_factor: 1.0,
            });
            assert!(!scene.commands().is_empty());
            let description = playground.build_tree();
            let mut bound = playground.layout_root(&description);
            apply_playground_styles(&mut bound, viewport.0, viewport.1);
            let bound_snapshot = playground
                .layout
                .compute(&bound, viewport)
                .expect("bound layout");
            let expected_ids = extract_playground_ids(&bound).button_ids;
            let (actual_ids, actual_snapshot) =
                playground.last_frame_layout.as_ref().expect("frame layout");
            assert_eq!(*actual_ids, expected_ids);
            assert_eq!(actual_snapshot, &bound_snapshot);
            assert_ne!(
                *actual_ids,
                extract_playground_ids(&description.to_layout(NodeId::new(1))).button_ids
            );
            if let Some(first) = first_ids {
                assert_eq!(*actual_ids, first);
            } else {
                first_ids = Some(*actual_ids);
            }

            let mut sequential = description.to_layout(NodeId::new(1));
            apply_playground_styles(&mut sequential, viewport.0, viewport.1);
            let snapshot = playground
                .layout
                .compute(&sequential, viewport)
                .expect("sequential layout");
            let sequential_ids = extract_playground_ids(&sequential);
            for (index, id) in sequential_ids.button_ids.into_iter().enumerate() {
                let rect = snapshot.get(id).expect("button geometry");
                let retained_rect = actual_snapshot
                    .get(actual_ids[index])
                    .expect("retained button geometry");
                assert_eq!(retained_rect, rect);
                assert_eq!(
                    playground.buttons[index],
                    [rect.x, rect.y, rect.width, rect.height]
                );
                assert!(rect.width > 0.0 && rect.height > 0.0);
                playground.cursor = (rect.x + rect.width * 0.5, rect.y + rect.height * 0.5);
                assert_eq!(playground.hit(), Some(index));
            }
            assert!(playground.scroll <= playground.max_scroll);
        }
    }
}
