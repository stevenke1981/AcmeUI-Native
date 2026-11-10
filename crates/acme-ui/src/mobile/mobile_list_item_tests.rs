use super::*;
use acme_core::{NodeId, RetainedTree};
use acme_layout::{LayoutEngine, WidgetLayoutContext};
use acme_widgets::retained_binding::bind_retained;
use acme_widgets::style::ColorToken;
use acme_widgets::view_bridge::widget_to_views;
#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Tap,
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
fn defaults_public_fields_optional_detail_icon_and_tap_are_preserved() {
    let defaults = mobile_list_item::<Msg>("Title", None::<&str>);
    assert_eq!(defaults.id.as_str(), "mobile_list_item");
    assert_eq!(defaults.title, "Title");
    assert!(
        defaults.detail.is_none() && defaults.trailing_icon.is_none() && defaults.on_tap.is_none()
    );
    for detail in [None, Some(""), Some("細節👨‍👩‍👧‍👦é")] {
        for icon in [
            None,
            Some(crate::IconName::ChevronRight),
            Some(crate::IconName::Settings),
        ] {
            for tap in [None, Some(Msg::Tap)] {
                let mut b = mobile_list_item("unused", None::<&str>).key("setting");
                b.title = "設定".into();
                b.detail = detail.map(String::from);
                b.trailing_icon = icon;
                b.on_tap = tap.clone();
                let node: WidgetNode<Msg> = b.into();
                let WidgetNode::Row(row) = &node else {
                    panic!("Row")
                };
                assert_eq!(row.message, tap);
                assert_eq!(row.style.background, Some(ColorToken::Surface));
                let WidgetNode::Column(text) = &row.children[0] else {
                    panic!("text column")
                };
                assert_eq!(text.key.as_ref().unwrap().as_str(), "setting__text");
                assert_eq!(text.children.len(), 1 + usize::from(detail.is_some()));
                for (child, expected) in text
                    .children
                    .iter()
                    .zip(std::iter::once("設定").chain(detail))
                {
                    let WidgetNode::Label(l) = child else {
                        panic!("text")
                    };
                    assert_eq!(l.text, expected);
                    assert_eq!(l.wrap, crate::LabelWrap::Character);
                    assert!(l.color.is_none());
                }
                assert_eq!(row.children.len(), 1 + usize::from(icon.is_some()));
                if let Some(icon) = icon {
                    let WidgetNode::Label(l) = &row.children[1] else {
                        panic!("icon")
                    };
                    assert_eq!(l.text, icon.char());
                    assert!(l.color.is_none());
                }
            }
        }
    }
}
#[test]
fn full_width_wrapped_text_and_fixed_trailing_icon_fit_real_narrow_layouts() {
    let mut fonts = acme_text::FontSystem::new();
    for width in [180.0, 320.0, 640.0] {
        for detail in [
            None,
            Some(""),
            Some("LongUnbrokenDetailLongUnbrokenDetailLongUnbrokenDetail"),
            Some("這是一段較長的詳細資訊，請閱讀內容後再繼續下一步。這是一段較長的詳細資訊。"),
        ] {
            for icon in [None, Some(crate::IconName::ChevronRight)] {
                let mut b = mobile_list_item::<Msg>("Long title · 較長的清單項目標題", detail);
                b.trailing_icon = icon;
                let node: WidgetNode<Msg> = b.into();
                let layout = node.to_layout_with_context(NodeId::new(1), &context());
                let snapshot = LayoutEngine::new()
                    .compute_with_text(&layout, (width, 2000.0), &mut fonts, 1.0)
                    .unwrap();
                let root = snapshot.get(layout.id).unwrap();
                assert_eq!(root.width, width);
                assert!(root.height >= 44.0);
                let text = snapshot.get(layout.children[0].id).unwrap();
                assert!(text.width >= 20.0);
                for child in &layout.children[0].children {
                    let r = snapshot.get(child.id).unwrap();
                    assert_eq!(r.width, text.width);
                    assert!(r.x >= root.x + 14.0 && r.x + r.width <= root.x + root.width - 13.0);
                    assert!(r.y + r.height <= root.y + root.height - 13.0);
                }
                if icon.is_some() {
                    let r = snapshot.get(layout.children[1].id).unwrap();
                    assert_eq!(r.width, 20.0);
                    assert!(r.x >= text.x + text.width + 11.0);
                    assert!(r.x + r.width <= root.x + root.width - 13.0);
                }
                if width == 180.0 {
                    assert!(
                        snapshot
                            .get(layout.children[0].children[0].id)
                            .unwrap()
                            .height
                            > 20.0
                    );
                }
            }
        }
    }
}
#[test]
fn retained_root_text_and_title_survive_optional_changes_tap_changes_and_keyed_reorder() {
    let mut tree = RetainedTree::new();
    let mut saved = None;
    for changed in [false, true, false] {
        let mut a = mobile_list_item("A", None::<&str>).key("a");
        if changed {
            a.title = "設定".into();
            a.detail = Some("已連線".into());
            a = a.trailing_icon(crate::IconName::Check).on_tap(Msg::Tap);
        }
        let a: WidgetNode<Msg> = a.into();
        let b: WidgetNode<Msg> = mobile_list_item("B", Some("Detail"))
            .key("b")
            .trailing_icon(crate::IconName::ChevronRight)
            .on_tap(Msg::Tap)
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
            let text = &n.children()[0];
            [
                binding.id_for(n).unwrap(),
                binding.id_for(text).unwrap(),
                binding.id_for(&text.children()[0]).unwrap(),
            ]
        });
        assert_ne!(ids[0][0], ids[1][0]);
        if let Some(saved) = saved {
            assert_eq!(ids, saved);
        }
        saved = Some(ids);
    }
}
