use super::*;
use acme_core::{NodeId, RetainedTree};
use acme_layout::{LayoutEngine, WidgetLayoutContext};
use acme_widgets::style::ColorToken;
use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};
#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Press,
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
fn defaults_and_public_states_control_activation_and_semantic_colours() {
    let b = mobile_action::<Msg>("Delete");
    assert_eq!(b.id.as_str(), "mobile_action");
    assert_eq!(b.label, "Delete");
    assert!(!b.disabled && !b.destructive && b.icon.is_none() && b.on_press.is_none());
    for disabled in [false, true] {
        for destructive in [false, true] {
            for message in [None, Some(Msg::Press)] {
                for icon in [None, Some(crate::IconName::Star)] {
                    let mut b = mobile_action("unused").key("action");
                    b.label = "刪除👨‍👩‍👧‍👦é".into();
                    b.disabled = disabled;
                    b.destructive = destructive;
                    b.on_press = message.clone();
                    b.icon = icon;
                    let node: WidgetNode<Msg> = b.into();
                    let WidgetNode::Row(row) = &node else {
                        panic!("Row")
                    };
                    assert_eq!(row.message, if disabled { None } else { message.clone() });
                    assert_eq!(
                        row.style.background,
                        Some(if disabled {
                            ColorToken::DisabledBg
                        } else if destructive {
                            ColorToken::DangerSoft
                        } else {
                            ColorToken::Surface
                        })
                    );
                    assert_eq!(row.children.len(), 1 + usize::from(icon.is_some()));
                    let label = row
                        .children
                        .iter()
                        .find(|n| n.key().is_some_and(|k| k.as_str() == "action__label"))
                        .expect("stable label key");
                    let WidgetNode::Label(l) = &label.children()[0] else {
                        panic!("Label")
                    };
                    assert_eq!(l.text, "刪除👨‍👩‍👧‍👦é");
                    assert_eq!(l.wrap, crate::LabelWrap::Character);
                    assert!(l.color.is_none());
                    assert_eq!(
                        l.style.text_color,
                        Some(if disabled {
                            ColorToken::DisabledText
                        } else if destructive {
                            ColorToken::Danger
                        } else {
                            ColorToken::Foreground
                        })
                    );
                }
            }
        }
    }
}
#[test]
fn narrow_and_wide_real_layouts_wrap_without_icon_overlap() {
    let mut fonts = acme_text::FontSystem::new();
    for width in [180.0, 320.0, 640.0] {
        for icon in [false, true] {
            let mut b = mobile_action::<Msg>("LongUnbrokenActionLabel · 請閱讀後再執行這個操作");
            if icon {
                b = b.icon(crate::IconName::Star);
            }
            let node: WidgetNode<Msg> = b.into();
            let layout = node.to_layout_with_context(NodeId::new(1), &context());
            let snapshot = LayoutEngine::new()
                .compute_with_text(&layout, (width, 2000.0), &mut fonts, 1.0)
                .unwrap();
            let root = snapshot.get(layout.id).unwrap();
            assert_eq!(root.width, width);
            assert!(root.height >= 44.0);
            let text = snapshot.get(layout.children[usize::from(icon)].id).unwrap();
            assert!(text.x >= root.x + 14.0);
            assert!(text.x + text.width <= root.x + root.width - 13.0);
            assert!(text.y + text.height <= root.y + root.height - 13.0);
            if width == 180.0 {
                assert!(text.height > 20.0);
            }
            if icon {
                let r = snapshot.get(layout.children[0].id).unwrap();
                assert_eq!(r.width, 20.0);
                assert!(r.x + r.width + 11.0 <= text.x);
            }
        }
    }
}
#[test]
fn keyed_label_identity_survives_icon_state_and_instance_reorder() {
    let mut tree = RetainedTree::new();
    let mut saved = None;
    for changed in [false, true, false] {
        let mut a = mobile_action("Share").key("a");
        if changed {
            a = a
                .icon(crate::IconName::Star)
                .destructive(true)
                .disabled(true)
                .on_press(Msg::Press);
        }
        let a: WidgetNode<Msg> = a.into();
        let b: WidgetNode<Msg> = mobile_action("B").key("b").on_press(Msg::Press).into();
        let roots = if changed { vec![b, a] } else { vec![a, b] };
        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0);
        let ids = ["a", "b"].map(|key| {
            let root = roots
                .iter()
                .find(|n| n.key().unwrap().as_str() == key)
                .unwrap();
            let label = root
                .children()
                .iter()
                .find(|n| {
                    n.key()
                        .is_some_and(|k| k.as_str() == format!("{key}__label"))
                })
                .unwrap();
            [
                binding.id_for(root).unwrap(),
                binding.id_for(label).unwrap(),
            ]
        });
        if let Some(previous) = saved {
            assert_eq!(ids, previous);
        }
        saved = Some(ids);
    }
}
