use super::*;
use acme_core::{NodeId, RetainedTree};
use acme_layout::{LayoutEngine, WidgetLayoutContext};
use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};
#[test]
fn gestures_require_begin_dead_zone_and_directional_release_threshold() {
    let mut state = SwipeActionsState::default();
    assert!(!state.end(-100.0, 0.0));
    state.begin(0.0, 0.0);
    assert!(!state.update(-11.0, 0.0));
    assert!(!state.end(-11.0, 0.0));
    state.begin(0.0, 0.0);
    assert!(state.update(-12.0, 0.0));
    assert!(!state.end(-47.0, 0.0));
    assert!(!state.revealed());
    state.begin(0.0, 0.0);
    assert!(state.end(-48.0, 0.0));
    assert!(state.revealed());
    assert!(!state.end(200.0, 0.0));
    assert!(state.revealed());
    state.begin(0.0, 0.0);
    assert!(state.end(48.0, 0.0));
    assert!(!state.revealed());
}
#[test]
fn vertical_cancel_and_nonfinite_sequences_never_change_visibility() {
    for initial in [false, true] {
        let mut state = SwipeActionsState::default();
        state.set_revealed(initial);
        state.begin(0.0, 0.0);
        assert!(!state.update(1.0, 20.0));
        assert!(!state.end(-100.0, 21.0));
        assert_eq!(state.revealed(), initial);
        state.begin(0.0, 0.0);
        state.update(-80.0, 0.0);
        state.cancel();
        assert!(!state.end(-80.0, 0.0));
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            state.begin(invalid, 0.0);
            assert!(!state.end(-100.0, 0.0));
            state.begin(0.0, 0.0);
            assert!(!state.update(invalid, 0.0));
            assert!(!state.end(-100.0, 0.0));
            state.begin(0.0, 0.0);
            assert!(!state.end(-100.0, invalid));
        }
        state.begin(f32::MAX, 0.0);
        assert!(!state.end(-f32::MAX, 0.0));
        assert_eq!(state.revealed(), initial);
    }
}
#[test]
fn controlled_actions_are_optional_keyed_and_never_fire_on_reveal() {
    for revealed in [false, true] {
        for archive in [false, true] {
            for delete in [false, true] {
                let mut b = swipe_actions("row", crate::label("Item")).revealed(revealed);
                if archive {
                    b = b.on_archive(1);
                }
                if delete {
                    b = b.on_delete(2);
                }
                let node: WidgetNode<i32> = b.into();
                let WidgetNode::Row(root) = &node else {
                    panic!("Row")
                };
                assert!(root.message.is_none());
                assert_eq!(
                    root.children.len(),
                    if revealed && (archive || delete) {
                        2
                    } else {
                        1
                    }
                );
                if root.children.len() == 2 {
                    let actions = root.children[1].children();
                    let expected: Vec<_> = [
                        (archive, 1, "row__archive", crate::ButtonVariant::Secondary),
                        (delete, 2, "row__delete", crate::ButtonVariant::Danger),
                    ]
                    .into_iter()
                    .filter(|x| x.0)
                    .collect();
                    assert_eq!(actions.len(), expected.len());
                    for (node, (_, msg, key, variant)) in actions.iter().zip(expected) {
                        let WidgetNode::Button(button) = node else {
                            panic!("Button")
                        };
                        assert_eq!(button.key.as_str(), key);
                        assert_eq!(button.activate(), Some(&msg));
                        assert_eq!(button.variant, variant);
                    }
                }
            }
        }
    }
}
#[test]
fn wrapping_list_and_trailing_action_targets_fit_real_layouts() {
    let context = WidgetLayoutContext {
        body_font_size: 14.0,
        body_line_height: 20.0,
        label_font_size: 13.0,
        control_height: 36.0,
        scale_factor: 1.0,
    };
    let mut fonts = acme_text::FontSystem::new();
    for width in [240.0, 360.0, 600.0] {
        for revealed in [false, true] {
            let node = crate::column()
                .width(width)
                .child(
                    swipe_actions(
                        "row",
                        crate::mobile::mobile_list_item(
                            "A longer title · 請閱讀這則訊息",
                            Some("Details wrap without covering the actions. 內容會換行。"),
                        ),
                    )
                    .revealed(revealed)
                    .on_archive(1)
                    .on_delete(2),
                )
                .build();
            let layout = node.to_layout_with_context(NodeId::new(1), &context);
            let snapshot = LayoutEngine::new()
                .compute_with_text(&layout, (width, 2000.0), &mut fonts, 1.0)
                .unwrap();
            let root = &layout.children[0];
            let rect = snapshot.get(root.id).unwrap();
            assert_eq!(rect.width, width);
            for (_, r) in snapshot.iter() {
                assert!(r.x >= 0.0 && r.x + r.width <= width && r.height.is_finite());
            }
            if revealed {
                let body = snapshot.get(root.children[0].id).unwrap();
                let actions = &root.children[1];
                let panel = snapshot.get(actions.id).unwrap();
                assert!(body.x + body.width <= panel.x);
                for action in &actions.children {
                    let r = snapshot.get(action.id).unwrap();
                    assert_eq!((r.width, r.height), (72.0, 44.0));
                }
            }
        }
    }
}
#[test]
fn content_ids_survive_reveal_optional_actions_and_instance_reorder() {
    let mut tree = RetainedTree::new();
    let mut saved = None;
    for changed in [false, true, false] {
        let a: WidgetNode<i32> =
            swipe_actions("a", crate::label(if changed { "Changed" } else { "Item" }))
                .revealed(changed)
                .on_delete(2)
                .into();
        let b: WidgetNode<i32> = swipe_actions("b", crate::label("Other")).into();
        let roots = if changed { vec![b, a] } else { vec![a, b] };
        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0);
        let a = roots
            .iter()
            .find(|n| n.key().unwrap().as_str() == "a")
            .unwrap();
        let ids = [
            binding.id_for(a).unwrap(),
            binding.id_for(&a.children()[0]).unwrap(),
            binding.id_for(&a.children()[0].children()[0]).unwrap(),
        ];
        if let Some(previous) = saved {
            assert_eq!(ids, previous);
        }
        saved = Some(ids);
    }
}
