use super::*;
use acme_core::RetainedTree;
use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};

#[derive(Clone, Debug, PartialEq)]
enum Message {
    Toggle,
    Action(usize),
}

fn dial(key: &str, open: bool) -> WidgetNode<Message> {
    speed_dial()
        .key(key)
        .open(open)
        .action(SpeedDialAction::new("Edit · 編輯").icon(crate::IconName::Plus))
        .action(SpeedDialAction::new("Share · 分享"))
        .on_toggle(Message::Toggle)
        .on_action(Message::Action)
        .into()
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
        let layout = node.to_layout(acme_core::NodeId::new(1));
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
