use super::*;
use acme_core::{NodeId, RetainedTree};
use acme_layout::{LayoutEngine, WidgetLayoutContext};
use acme_widgets::retained_binding::bind_retained;
use acme_widgets::view_bridge::widget_to_views;

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Tap,
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
fn chip_actions_are_separate_and_surface_uses_semantic_tokens() {
    let defaults = mobile_chip::<Msg>("Tag");
    assert!(!defaults.selected && defaults.on_tap.is_none() && defaults.on_dismiss.is_none());
    for selected in [false, true] {
        for tap in [false, true] {
            for dismiss in [false, true] {
                let mut b = mobile_chip("標籤").key("tag").selected(selected);
                if tap {
                    b = b.on_tap(Msg::Tap);
                }
                if dismiss {
                    b = b.on_dismiss(Msg::Dismiss);
                }
                let node: WidgetNode<Msg> = b.into();
                let WidgetNode::Row(row) = &node else {
                    panic!("Row")
                };
                assert!(
                    row.message.is_none(),
                    "dismiss must not trigger an ancestor tap action"
                );
                assert!(row.style.background.is_some());
                assert!(row.style.border_radius.is_some_and(|r| r > 0.0));
                assert_eq!(row.children.len(), 1 + usize::from(dismiss));
                let text = match &row.children[0] {
                    WidgetNode::Button(button) => {
                        assert!(tap);
                        assert_eq!(button.key.as_str(), "tag__tap");
                        assert_eq!(button.activate(), Some(&Msg::Tap));
                        &button.label
                    }
                    WidgetNode::Label(label) => {
                        assert!(!tap);
                        &label.text
                    }
                    _ => panic!("tag text"),
                };
                assert_eq!(text, if selected { "✓ 標籤" } else { "標籤" });
                if dismiss {
                    let WidgetNode::Button(button) = &row.children[1] else {
                        panic!("dismiss")
                    };
                    assert_eq!(button.key.as_str(), "tag__dismiss");
                    assert!(button.label.contains('✕'));
                    assert_eq!(button.activate(), Some(&Msg::Dismiss));
                }
            }
        }
    }
}

#[test]
fn chip_layout_has_separate_nonzero_hit_targets_and_compact_height() {
    let mut fonts = acme_text::FontSystem::new();
    for width in [320.0, 640.0] {
        let node: WidgetNode<Msg> = mobile_chip("Rust · 原生")
            .on_tap(Msg::Tap)
            .on_dismiss(Msg::Dismiss)
            .into();
        let layout = node.to_layout_with_context(NodeId::new(1), &context());
        let snapshot = LayoutEngine::new()
            .compute_with_text(&layout, (width, 100.0), &mut fonts, 1.0)
            .unwrap();
        let root = snapshot.get(layout.id).unwrap();
        assert!(root.width <= width && root.height <= 48.0);
        let a = snapshot.get(layout.children[0].id).unwrap();
        let b = snapshot.get(layout.children[1].id).unwrap();
        assert!(a.width > 0.0 && b.width > 0.0);
        assert!(a.height >= 30.0 && b.height >= 30.0);
        assert!(a.x + a.width <= b.x);
        assert!(b.x + b.width <= root.x + root.width);
    }
}

#[test]
fn retained_chip_and_tap_ids_survive_selection_dismiss_changes_and_reorder() {
    let mut tree = RetainedTree::new();
    let mut saved = None;
    for selected in [false, true, false] {
        let mut a = mobile_chip("A")
            .key("a")
            .selected(selected)
            .on_tap(Msg::Tap);
        if selected {
            a = a.on_dismiss(Msg::Dismiss);
        }
        let a: WidgetNode<Msg> = a.into();
        let b: WidgetNode<Msg> = mobile_chip("B")
            .key("b")
            .on_tap(Msg::Tap)
            .on_dismiss(Msg::Dismiss)
            .into();
        let roots = if selected { vec![b, a] } else { vec![a, b] };
        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0);
        let ids = ["a", "b"].map(|key| {
            let n = roots
                .iter()
                .find(|n| n.key().unwrap().as_str() == key)
                .unwrap();
            let l = n.to_layout_with_binding(&binding, &context());
            (l.id, l.children[0].id)
        });
        if let Some(old) = &saved {
            assert_eq!(&ids, old);
        } else {
            saved = Some(ids);
        }
    }
}

#[test]
fn empty_and_unicode_static_tags_are_noninteractive_and_preserve_public_field_edits() {
    for text in ["", "標籤👨‍👩‍👧‍👦é"] {
        let mut builder = mobile_chip::<Msg>("unused");
        builder.text = text.into();
        builder.selected = true;
        let node: WidgetNode<Msg> = builder.into();
        let WidgetNode::Label(label) = &node.children()[0] else {
            panic!("static label")
        };
        assert_eq!(label.text, format!("✓ {text}"));
        assert!(!widget_to_views(&[node])[0].children[0].focusable);
    }
}
