use super::*;
use acme_core::{NodeId, RetainedTree};
use acme_layout::{LayoutEngine, WidgetLayoutContext};
use acme_widgets::retained_binding::bind_retained;
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
fn buttons(node: &WidgetNode<Msg>) -> Vec<(&str, &Msg)> {
    let mut out = Vec::new();
    fn collect<'a>(node: &'a WidgetNode<Msg>, out: &mut Vec<(&'a str, &'a Msg)>) {
        if let WidgetNode::Button(b) = node {
            out.push((b.key.as_str(), b.activate().unwrap()));
        }
        for c in node.children() {
            collect(c, out);
        }
    }
    collect(node, &mut out);
    out
}
#[test]
fn defaults_public_optional_fields_and_unicode_keep_distinct_actions() {
    let defaults = mobile_notification::<Msg>("Notice");
    assert_eq!(defaults.id.as_str(), "mobile_notification");
    assert!(
        defaults.body.is_none()
            && defaults.action_label.is_none()
            && defaults.on_action.is_none()
            && defaults.on_dismiss.is_none()
    );
    for body in [None, Some(""), Some("訊息👨‍👩‍👧‍👦é")] {
        for label in [None, Some(""), Some("Open")] {
            for action in [None, Some(Msg::Action)] {
                for dismiss in [None, Some(Msg::Dismiss)] {
                    let mut b = mobile_notification("unused").key("notice");
                    b.title = "通知".into();
                    b.body = body.map(String::from);
                    b.action_label = label.map(String::from);
                    b.on_action = action.clone();
                    b.on_dismiss = dismiss.clone();
                    let node: WidgetNode<Msg> = b.into();
                    let WidgetNode::Card(card) = &node else {
                        panic!("card")
                    };
                    assert_eq!(card.variant, crate::CardVariant::Elevated);
                    assert!(card.background_color.is_none());
                    let WidgetNode::Column(content) = &card.children[0] else {
                        panic!("content")
                    };
                    assert_eq!(content.key.as_ref().unwrap().as_str(), "notice__content");
                    assert_eq!(content.children.len(), 1 + usize::from(body.is_some()));
                    for (child, expected) in content
                        .children
                        .iter()
                        .zip(std::iter::once("通知").chain(body))
                    {
                        let WidgetNode::Label(l) = child else {
                            panic!("label")
                        };
                        assert_eq!(l.text, expected);
                        assert_eq!(l.wrap, crate::LabelWrap::Character);
                        assert!(l.color.is_none());
                    }
                    let mut expected = Vec::new();
                    if label.is_some() && action.is_some() {
                        expected.push(("notice__action", &Msg::Action));
                    }
                    if dismiss.is_some() {
                        expected.push(("notice__dismiss", &Msg::Dismiss));
                    }
                    assert_eq!(buttons(&node), expected);
                    assert!(!widget_to_views(&[node])[0].focusable);
                }
            }
        }
    }
}
#[test]
fn long_title_and_body_fit_narrow_cards_and_actions_stay_below_text() {
    let mut fonts = acme_text::FontSystem::new();
    for width in [180.0, 320.0, 600.0] {
        for body in [
            None,
            Some(""),
            Some("LongUnbrokenMessageLongUnbrokenMessageLongUnbrokenMessage"),
            Some("這是一則較長的通知內容，請先儲存工作，再繼續下一步。這是一則較長的通知內容。"),
        ] {
            let mut b = mobile_notification("Long title · 較長的通知標題")
                .action("Open", Msg::Action)
                .on_dismiss(Msg::Dismiss);
            b.body = body.map(String::from);
            let node: WidgetNode<Msg> = b.into();
            let layout = node.to_layout_with_context(NodeId::new(1), &context());
            let snapshot = LayoutEngine::new()
                .compute_with_text(&layout, (width, 2000.0), &mut fonts, 1.0)
                .unwrap();
            let root = snapshot.get(layout.id).unwrap();
            assert_eq!(root.width, width);
            let content = snapshot.get(layout.children[0].id).unwrap();
            for text in &layout.children[0].children {
                let r = snapshot.get(text.id).unwrap();
                assert_eq!(r.width, width - 24.0);
                assert!(r.x >= root.x + 12.0 && r.x + r.width <= root.x + root.width - 11.0);
                assert!(r.y + r.height <= content.y + content.height + 1.0);
            }
            let footer = &layout.children[1];
            let f = snapshot.get(footer.id).unwrap();
            assert!(f.y >= content.y + content.height + 7.0);
            for c in &footer.children {
                let r = snapshot.get(c.id).unwrap();
                assert!(r.height >= 44.0);
                assert!(r.x + r.width <= root.x + root.width - 11.0);
            }
            if width == 180.0 && body.is_some_and(|s| !s.is_empty()) {
                assert!(
                    snapshot
                        .get(layout.children[0].children[1].id)
                        .unwrap()
                        .height
                        > 20.0
                );
            }
        }
    }
}
#[test]
fn retained_title_content_and_dismiss_survive_optional_changes_and_keyed_reorder() {
    let mut tree = RetainedTree::new();
    let mut saved = None;
    for changed in [false, true, false] {
        let mut a = mobile_notification(if changed { "更新" } else { "Notice" })
            .key("a")
            .on_dismiss(Msg::Dismiss);
        if changed {
            a = a.body("新內容").action("Open", Msg::Action);
        }
        let a: WidgetNode<Msg> = a.into();
        let b: WidgetNode<Msg> = mobile_notification("B")
            .key("b")
            .body("Body")
            .action("Go", Msg::Action)
            .on_dismiss(Msg::Dismiss)
            .into();
        let roots = if changed { vec![b, a] } else { vec![a, b] };
        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0);
        let ids = ["a", "b"].map(|key| {
            let n = roots
                .iter()
                .find(|n| n.key().unwrap().as_str() == key)
                .unwrap();
            let content = &n.children()[0];
            let footer = n.children().last().unwrap();
            [
                binding.id_for(n).unwrap(),
                binding.id_for(content).unwrap(),
                binding.id_for(&content.children()[0]).unwrap(),
                binding.id_for(footer.children().last().unwrap()).unwrap(),
            ]
        });
        assert_ne!(ids[0][3], ids[1][3]);
        if let Some(saved) = saved {
            assert_eq!(ids, saved);
        }
        saved = Some(ids);
    }
}
