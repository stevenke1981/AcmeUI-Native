use super::*;
use acme_core::RetainedTree;
use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};

#[derive(Clone, Debug, PartialEq)]
enum Message {
    Toggle,
    Action(usize),
}

fn dial(key: &str, open: bool) -> WidgetNode<Message> {
    directional_dial(key, open, SpeedDialDirection::Up)
}

const DIRECTIONS: [SpeedDialDirection; 4] = [
    SpeedDialDirection::Up,
    SpeedDialDirection::Down,
    SpeedDialDirection::Left,
    SpeedDialDirection::Right,
];

fn directional_dial(key: &str, open: bool, direction: SpeedDialDirection) -> WidgetNode<Message> {
    speed_dial()
        .key(key)
        .open(open)
        .direction(direction)
        .action(SpeedDialAction::new("Edit · 編輯").icon(crate::IconName::Plus))
        .action(SpeedDialAction::new("Share · 分享"))
        .on_toggle(Message::Toggle)
        .on_action(Message::Action)
        .into()
}

#[test]
fn directions_preserve_messages_passive_fallback_and_closed_toggle() {
    assert_eq!(speed_dial::<Message>().direction, SpeedDialDirection::Up);
    for direction in DIRECTIONS {
        let toggle_first = matches!(
            direction,
            SpeedDialDirection::Down | SpeedDialDirection::Right
        );
        for configured in [false, true] {
            let mut builder = speed_dial()
                .open(true)
                .direction(direction)
                .action(SpeedDialAction::new("Edit"))
                .action(SpeedDialAction::new("Share"))
                .on_toggle(Message::Toggle);
            if configured {
                builder = builder.on_action(Message::Action);
            }
            let node: WidgetNode<Message> = builder.into();
            let children = node.children();
            assert_eq!(children.len(), 3);
            let toggle_index = if toggle_first { 0 } else { 2 };
            assert!(
                matches!(&children[toggle_index], WidgetNode::Button(b) if b.activate() == Some(&Message::Toggle))
            );
            for (index, child) in children
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != toggle_index)
                .map(|(_, child)| child)
                .enumerate()
            {
                if configured {
                    assert!(
                        matches!(child, WidgetNode::Button(b) if b.activate() == Some(&Message::Action(index)))
                    );
                } else {
                    assert!(
                        matches!(child, WidgetNode::Label(l) if l.text == ["Edit", "Share"][index])
                    );
                }
            }
        }
        let closed = directional_dial("closed", false, direction);
        assert_eq!(closed.children().len(), 1);
        assert!(
            matches!(&closed.children()[0], WidgetNode::Button(b) if b.activate() == Some(&Message::Toggle))
        );
    }
}

#[test]
fn direction_changes_and_keyed_reorders_preserve_current_controls_and_focus() {
    let mut tree = RetainedTree::new();
    let mut focus = acme_core::FocusManager::new();
    let mut saved = None;
    for (step, direction) in DIRECTIONS.into_iter().enumerate() {
        let mut roots = vec![
            directional_dial("a", true, direction),
            directional_dial("b", true, direction),
        ];
        if step % 2 == 1 {
            roots.reverse();
        }
        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0);
        let ids = ["a", "b"].map(|key| {
            let root = roots.iter().find(|n| n.key().unwrap().as_str() == key).unwrap();
            let control_ids = [Message::Toggle, Message::Action(0), Message::Action(1)].map(|message| {
                let child = root.children().iter().find(|n| matches!(n, WidgetNode::Button(b) if b.activate() == Some(&message))).unwrap();
                binding.id_for(child).unwrap()
            });
            (binding.id_for(root).unwrap(), control_ids)
        });
        focus.set_order(
            ids.iter()
                .flat_map(|(_, controls)| controls.iter().copied())
                .map(|id| (id, tree.get(id).unwrap().focusable)),
        );
        if let Some(saved) = saved {
            assert_eq!(ids, saved);
            assert_eq!(focus.focused(), Some(ids[0].1[1]));
        }
        assert_ne!(ids[0].1, ids[1].1);
        assert!(focus.request_focus(ids[0].1[1]));
        saved = Some(ids);
    }
}

#[test]
fn directional_real_layout_places_nonoverlapping_actions_on_requested_side() {
    let mut fonts = acme_text::FontSystem::new();
    for direction in DIRECTIONS {
        for width in [400.0, 600.0] {
            for scale in [1.0, 1.25, 2.0] {
                let node = directional_dial("layout", true, direction);
                let context = acme_layout::WidgetLayoutContext {
                    body_font_size: 14.0,
                    body_line_height: 20.0,
                    label_font_size: 13.0,
                    control_height: 36.0,
                    scale_factor: scale,
                };
                let layout = node.to_layout_with_context(acme_core::NodeId::new(1), &context);
                let snapshot = acme_layout::LayoutEngine::new()
                    .compute_with_text(&layout, (width, 400.0), &mut fonts, scale)
                    .unwrap();
                let parent = snapshot.get(layout.id).unwrap();
                let rects: Vec<_> = layout
                    .children
                    .iter()
                    .map(|child| snapshot.get(child.id).unwrap())
                    .collect();
                let horizontal = matches!(
                    direction,
                    SpeedDialDirection::Left | SpeedDialDirection::Right
                );
                for r in &rects {
                    assert!([r.x, r.y, r.width, r.height].iter().all(|n| n.is_finite()));
                    assert!(r.width > 0.0 && r.height > 0.0);
                    assert!(r.x >= parent.x && r.x + r.width <= parent.x + parent.width + 0.01);
                    assert!(r.y >= parent.y && r.y + r.height <= parent.y + parent.height + 0.01);
                }
                for pair in rects.windows(2) {
                    if horizontal {
                        assert!(pair[0].x + pair[0].width < pair[1].x);
                    } else {
                        assert!(pair[0].y + pair[0].height < pair[1].y);
                    }
                }
                let toggle_index = if matches!(
                    direction,
                    SpeedDialDirection::Down | SpeedDialDirection::Right
                ) {
                    0
                } else {
                    2
                };
                let toggle = rects[toggle_index];
                for (_, r) in rects.iter().enumerate().filter(|(i, _)| *i != toggle_index) {
                    match direction {
                        SpeedDialDirection::Up => assert!(r.y + r.height < toggle.y),
                        SpeedDialDirection::Down => assert!(r.y > toggle.y + toggle.height),
                        SpeedDialDirection::Left => assert!(r.x + r.width < toggle.x),
                        SpeedDialDirection::Right => assert!(r.x > toggle.x + toggle.width),
                    }
                }
            }
        }
    }
}

#[test]
fn converted_actions_carry_indexed_messages_icons_and_toggle_independently() {
    let node = dial("dial", true);
    for (index, child) in node.children()[..2].iter().enumerate() {
        let WidgetNode::Button(button) = child else {
            panic!("configured SpeedDial action must be a Button");
        };
        assert_eq!(button.activate(), Some(&Message::Action(index)));
        assert!(button.label.contains(if index == 0 {
            "Edit · 編輯"
        } else {
            "Share · 分享"
        }));
        if index == 0 {
            assert!(button.label.starts_with(crate::IconName::Plus.char()));
        }
    }
    let WidgetNode::Button(toggle) = &node.children()[2] else {
        panic!("toggle");
    };
    assert_eq!(toggle.activate(), Some(&Message::Toggle));
    assert_eq!(dial("dial", false).children().len(), 1);
}

#[test]
fn missing_callback_preserves_passive_action_labels() {
    let node: WidgetNode<Message> = speed_dial()
        .open(true)
        .action(SpeedDialAction::new("Read only"))
        .into();
    assert!(matches!(&node.children()[0],WidgetNode::Label(l) if l.text == "Read only"));
    let WidgetNode::Button(toggle) = &node.children()[1] else {
        panic!("toggle");
    };
    assert!(toggle.activate().is_none());
}

#[test]
fn toggle_identity_survives_open_close_and_keyed_instance_reordering() {
    let mut tree = RetainedTree::new();
    let mut saved = None;
    for (open, reverse) in [(false, false), (true, true), (false, true), (true, false)] {
        let mut roots = vec![dial("a", open), dial("b", open)];
        if reverse {
            roots.reverse();
        }
        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0);
        let ids = ["a", "b"].map(|key| {
            let root = roots
                .iter()
                .find(|n| n.key().unwrap().as_str() == key)
                .unwrap();
            let toggle = root.children().last().unwrap();
            (
                binding.id_for(root).unwrap(),
                binding.id_for(toggle).unwrap(),
            )
        });
        if let Some(saved) = saved {
            assert_eq!(ids, saved);
        }
        saved = Some(ids);
        for root in &roots {
            if open {
                assert_ne!(
                    binding.id_for(&root.children()[0]),
                    binding.id_for(&root.children()[1])
                );
            }
        }
    }
}

#[test]
fn configured_actions_have_bounded_real_layout_at_narrow_and_wide_widths() {
    for width in [240.0, 600.0] {
        let node = dial("layout", true);
        let context = acme_layout::WidgetLayoutContext {
            body_font_size: 14.0,
            body_line_height: 20.0,
            label_font_size: 13.0,
            control_height: 36.0,
            scale_factor: 1.0,
        };
        let layout = node.to_layout_with_context(acme_core::NodeId::new(1), &context);
        let mut fonts = acme_text::FontSystem::new();
        let snapshot = acme_layout::LayoutEngine::new()
            .compute_with_text(&layout, (width, 400.0), &mut fonts, 1.0)
            .unwrap();
        let parent = snapshot.get(layout.id).unwrap();
        for child in &layout.children {
            let r = snapshot.get(child.id).unwrap();
            assert!(r.width > 0.0 && r.height > 0.0);
            assert!(r.x >= parent.x && r.x + r.width <= parent.x + parent.width + 0.01);
        }
    }
}
