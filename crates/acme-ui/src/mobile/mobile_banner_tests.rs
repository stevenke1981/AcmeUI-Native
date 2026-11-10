use super::*;
use acme_core::{NodeId, RetainedTree};
use acme_layout::{LayoutEngine, WidgetLayoutContext};
use acme_widgets::retained_binding::bind_retained;
use acme_widgets::style::ColorToken;
use acme_widgets::view_bridge::widget_to_views;

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Action,
    Dismiss,
}

fn context() -> WidgetLayoutContext {
    WidgetLayoutContext {
        body_font_size: 14.0,
        body_line_height: 20.0,
        label_font_size: 13.0,
        control_height: 36.0,
        scale_factor: 1.0,
    }
}

#[test]
fn defaults_tones_and_public_optional_fields_preserve_separate_actions() {
    let defaults = mobile_banner::<Msg>("Hello");
    assert_eq!(defaults.id.as_str(), "mobile_banner");
    assert_eq!(defaults.tone, MobileBannerTone::Info);
    assert!(defaults.action_label.is_none() && defaults.on_action.is_none());
    assert!(defaults.on_dismiss.is_none());
    for (tone, background) in [
        (MobileBannerTone::Info, ColorToken::InfoSoft),
        (MobileBannerTone::Success, ColorToken::SuccessSoft),
        (MobileBannerTone::Warning, ColorToken::WarningSoft),
        (MobileBannerTone::Danger, ColorToken::DangerSoft),
    ] {
        for label in [None, Some(""), Some("Retry")] {
            for action in [None, Some(Msg::Action)] {
                for dismiss in [None, Some(Msg::Dismiss)] {
                    let mut b = mobile_banner("unused").key("alert").tone(tone);
                    b.text = "更新👨‍👩‍👧‍👦é".into();
                    b.action_label = label.map(String::from);
                    b.on_action = action.clone();
                    b.on_dismiss = dismiss.clone();
                    let node: WidgetNode<Msg> = b.into();
                    let WidgetNode::Row(row) = &node else {
                        panic!("Row")
                    };
                    assert_eq!(row.style.background, Some(background));
                    assert!(row.message.is_none());
                    let WidgetNode::Label(icon) = &row.children[0] else {
                        panic!("icon")
                    };
                    assert_eq!(icon.text, tone.icon_char());
                    let WidgetNode::Label(text) = &row.children[1] else {
                        panic!("text")
                    };
                    assert_eq!(text.text, "更新👨‍👩‍👧‍👦é");
                    assert_eq!(text.wrap, crate::LabelWrap::Character);
                    assert!(icon.color.is_none() && text.color.is_none());
                    let mut expected = Vec::new();
                    if label.is_some() && action.is_some() {
                        expected.push(("alert__action", Msg::Action));
                    }
                    if dismiss.is_some() {
                        expected.push(("alert__dismiss", Msg::Dismiss));
                    }
                    assert_eq!(row.children.len(), 2 + expected.len());
                    for (child, (key, msg)) in row.children[2..].iter().zip(expected) {
                        let WidgetNode::Button(button) = child else {
                            panic!("action")
                        };
                        assert_eq!(button.key.as_str(), key);
                        assert_eq!(button.activate(), Some(&msg));
                    }
                    let views = widget_to_views(&[node]);
                    assert!(!views[0].focusable);
                    assert!(views[0].children[..2].iter().all(|v| !v.focusable));
                    assert!(views[0].children[2..].iter().all(|v| v.focusable));
                }
            }
        }
    }
}

#[test]
fn full_width_long_unicode_text_wraps_without_overlapping_actions() {
    let mut fonts = acme_text::FontSystem::new();
    for width in [240.0, 400.0, 640.0] {
        for text in [
            "",
            "正在更新資料，請稍候，完成後即可繼續使用應用程式。",
            "LongUnbrokenMessageLongUnbrokenMessageLongUnbrokenMessage",
        ] {
            for actions in [false, true] {
                let mut b = mobile_banner(text);
                if actions {
                    b = b.action("Retry", Msg::Action).on_dismiss(Msg::Dismiss);
                }
                let node: WidgetNode<Msg> = b.into();
                let layout = node.to_layout_with_context(NodeId::new(1), &context());
                let snapshot = LayoutEngine::new()
                    .compute_with_text(&layout, (width, 2000.0), &mut fonts, 1.0)
                    .unwrap();
                let root = snapshot.get(layout.id).unwrap();
                assert_eq!(root.width, width);
                let text_rect = snapshot.get(layout.children[1].id).unwrap();
                assert!(text_rect.width >= 20.0, "{text_rect:?}");
                if !text.is_empty() && width == 240.0 {
                    assert!(text_rect.height > 20.0);
                }
                for pair in layout.children.windows(2) {
                    let a = snapshot.get(pair[0].id).unwrap();
                    let b = snapshot.get(pair[1].id).unwrap();
                    assert!(a.x + a.width <= b.x);
                }
                for child in &layout.children {
                    let rect = snapshot.get(child.id).unwrap();
                    assert!(
                        rect.x >= root.x + 12.0
                            && rect.x + rect.width <= root.x + root.width - 11.0,
                        "width={width}, actions={actions}, text={text:?}, root={root:?}, child={rect:?}"
                    );
                    assert!(rect.y + rect.height <= root.y + root.height - 11.0);
                }
                if actions {
                    for child in &layout.children[2..] {
                        assert!(snapshot.get(child.id).unwrap().height >= 44.0);
                    }
                }
            }
        }
    }
}

#[test]
fn retained_text_and_dismiss_survive_action_insertion_and_instance_reordering() {
    let mut tree = RetainedTree::new();
    let mut saved = None;
    for changed in [false, true, false] {
        let mut a = mobile_banner(if changed { "更新" } else { "Update" })
            .key("a")
            .on_dismiss(Msg::Dismiss);
        if changed {
            a = a
                .tone(MobileBannerTone::Danger)
                .action("Retry", Msg::Action);
        }
        let a: WidgetNode<Msg> = a.into();
        let b: WidgetNode<Msg> = mobile_banner("B")
            .key("b")
            .action("Go", Msg::Action)
            .on_dismiss(Msg::Dismiss)
            .into();
        let roots = if changed { vec![b, a] } else { vec![a, b] };
        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0);
        let ids = ["a", "b"].map(|key| {
            let node = roots
                .iter()
                .find(|n| n.key().unwrap().as_str() == key)
                .unwrap();
            [
                binding.id_for(node).unwrap(),
                binding.id_for(&node.children()[0]).unwrap(),
                binding.id_for(&node.children()[1]).unwrap(),
                binding.id_for(node.children().last().unwrap()).unwrap(),
            ]
        });
        assert_ne!(ids[0][3], ids[1][3]);
        if let Some(saved) = saved {
            assert_eq!(ids, saved);
        }
        saved = Some(ids);
    }
}
