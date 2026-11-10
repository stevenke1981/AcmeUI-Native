use super::*;

#[test]
fn toast_message_wraps_instead_of_overlapping_its_actions() {
    let node: WidgetNode<()> = toast("A long message · 這是一則需要換行的通知訊息").into();
    let WidgetNode::Label(message) = &node.children()[0].children()[1] else {
        panic!("message Label")
    };
    assert_eq!(message.wrap, crate::LabelWrap::Character);
}

#[test]
fn toast_defaults_tones_and_independent_actions_are_preserved() {
    let defaults = toast::<()>("Saved");
    assert_eq!(defaults.id.as_str(), "toast");
    assert_eq!(defaults.duration_ms, 3000);
    assert!(defaults.dismissible && defaults.action.is_none() && defaults.on_dismiss.is_none());
    for (tone, icon) in [
        (Tone::Neutral, "ℹ"),
        (Tone::Primary, "✓"),
        (Tone::Success, "✔"),
        (Tone::Warning, "⚠"),
        (Tone::Danger, "✕"),
        (Tone::Info, "ℹ"),
    ] {
        for action in [false, true] {
            for dismissible in [false, true] {
                let mut b = toast("Saved")
                    .key("one")
                    .tone(tone)
                    .dismissible(dismissible)
                    .on_dismiss(2);
                if action {
                    b = b.action("Undo", 1);
                }
                let node: WidgetNode<i32> = b.into();
                let WidgetNode::Card(card) = &node else {
                    panic!("Card")
                };
                assert_eq!(card.key.as_ref().unwrap().as_str(), "one");
                assert!(card.background_color.is_none());
                let WidgetNode::Label(label) = &card.children[0].children()[0] else {
                    panic!("icon")
                };
                assert_eq!(label.text, icon);
                assert_eq!(
                    card.children.len(),
                    if action || dismissible { 2 } else { 1 }
                );
                if action || dismissible {
                    let buttons = card.children[1].children();
                    let expected: Vec<_> =
                        [(action, 1, "one__action"), (dismissible, 2, "one__dismiss")]
                            .into_iter()
                            .filter(|x| x.0)
                            .collect();
                    assert_eq!(buttons.len(), expected.len());
                    for (node, (_, msg, key)) in buttons.iter().zip(expected) {
                        let WidgetNode::Button(button) = node else {
                            panic!("Button")
                        };
                        assert_eq!(button.activate(), Some(&msg));
                        assert_eq!(button.key.as_str(), key);
                    }
                }
            }
        }
    }
    for duration in [0, 5000, u64::MAX] {
        assert_eq!(toast::<()>("").duration_ms(duration).duration_ms, duration);
    }
}

#[test]
fn toast_unicode_message_and_long_action_fit_real_narrow_layouts() {
    use acme_core::NodeId;
    use acme_layout::{LayoutEngine, WidgetLayoutContext};
    let context = WidgetLayoutContext {
        body_font_size: 14.0,
        body_line_height: 20.0,
        label_font_size: 13.0,
        control_height: 36.0,
        scale_factor: 1.0,
    };
    let mut fonts = acme_text::FontSystem::new();
    for width in [120.0, 240.0, 600.0] {
        let node = crate::column()
            .width(width)
            .child(
                toast("A long unbroken message 這是一則通知訊息é👨‍👩‍👧‍👦".repeat(3))
                    .action("An unusually long undo action · 復原", 1)
                    .on_dismiss(2),
            )
            .build();
        let layout = node.to_layout_with_context(NodeId::new(1), &context);
        let snapshot = LayoutEngine::new()
            .compute_with_text(&layout, (width, 2000.0), &mut fonts, 1.0)
            .unwrap();
        let card = &layout.children[0];
        let surface = snapshot.get(card.id).unwrap();
        assert_eq!(surface.width, width);
        let body = snapshot.get(card.children[0].id).unwrap();
        let message = snapshot.get(card.children[0].children[1].id).unwrap();
        assert!(message.height >= 20.0);
        assert!(message.x + message.width <= surface.x + surface.width);
        let footer = snapshot.get(card.children[1].id).unwrap();
        assert!(footer.y >= body.y + body.height);
        for child in &card.children[1].children {
            let action = snapshot.get(child.id).unwrap();
            assert_eq!(action.height, 44.0);
            assert!(action.x >= surface.x && action.x + action.width <= surface.x + surface.width);
        }
    }
}

#[test]
fn toast_content_and_dismiss_ids_survive_optional_action_and_reorder() {
    use acme_core::RetainedTree;
    use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};
    let mut tree = RetainedTree::new();
    let mut saved = None;
    for changed in [false, true, false] {
        let mut b = toast(if changed { "Changed" } else { "Saved" })
            .key("a")
            .on_dismiss(2);
        if changed {
            b = b.action("Undo", 1);
        }
        let a: WidgetNode<i32> = b.into();
        let b: WidgetNode<i32> = toast("Other").key("b").dismissible(false).into();
        let roots = if changed { vec![b, a] } else { vec![a, b] };
        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0);
        let a = roots
            .iter()
            .find(|n| n.key().unwrap().as_str() == "a")
            .unwrap();
        let dismiss = a.children()[1].children().last().unwrap();
        let ids = [
            binding.id_for(a).unwrap(),
            binding.id_for(&a.children()[0]).unwrap(),
            binding.id_for(&a.children()[0].children()[1]).unwrap(),
            binding.id_for(dismiss).unwrap(),
        ];
        if let Some(previous) = saved {
            assert_eq!(ids, previous);
        }
        saved = Some(ids);
    }
}
