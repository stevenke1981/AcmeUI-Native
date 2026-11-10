use super::*;
use acme_core::{NodeId, RetainedTree};
use acme_layout::{LayoutEngine, WidgetLayoutContext};
use acme_widgets::retained_binding::bind_retained;
use acme_widgets::view_bridge::widget_to_views;

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
fn circular_sizes_defaults_placeholder_and_public_edits_preserve_noninteractive_semantics() {
    assert_eq!(mobile_avatar::<()>("A").size, MobileAvatarSize::Md);
    for size in [
        MobileAvatarSize::Sm,
        MobileAvatarSize::Md,
        MobileAvatarSize::Lg,
    ] {
        for initials in ["AB", "陳", "👨‍👩‍👧‍👦", "", "   "] {
            let mut builder = mobile_avatar::<()>("unused").key("person");
            builder.size = size;
            builder.initials = initials.into();
            let node: WidgetNode<()> = builder.into();
            let WidgetNode::Stack(stack) = &node else {
                panic!("Stack")
            };
            let d = size.diameter();
            assert_eq!(stack.width, Some(d));
            assert_eq!(stack.height, Some(d));
            assert_eq!(stack.style.border_radius, Some(d / 2.0));
            assert!(stack.style.background.is_some());
            let WidgetNode::Label(label) = &stack.children[0] else {
                panic!("initials")
            };
            assert_eq!(
                label.text,
                if initials.trim().is_empty() {
                    "?"
                } else {
                    initials
                }
            );
            assert!(label.color.is_none());
            assert_eq!(
                label.style.text_align,
                Some(acme_widgets::style::TextAlign::Center)
            );
            let views = widget_to_views(&[node]);
            assert!(!views[0].focusable && !views[0].children[0].focusable);
        }
    }
}

#[test]
fn actual_layout_keeps_exact_diameters_and_text_square_inside_the_circle() {
    let mut fonts = acme_text::FontSystem::new();
    for size in [
        MobileAvatarSize::Sm,
        MobileAvatarSize::Md,
        MobileAvatarSize::Lg,
    ] {
        let node: WidgetNode<()> = mobile_avatar("AB").size(size).into();
        let layout = node.to_layout_with_context(NodeId::new(1), &context());
        let snapshot = LayoutEngine::new()
            .compute_with_text(&layout, (240.0, 100.0), &mut fonts, 1.0)
            .unwrap();
        let root = snapshot.get(layout.id).unwrap();
        let text = snapshot.get(layout.children[0].id).unwrap();
        let d = size.diameter();
        assert_eq!((root.width, root.height), (d, d));
        assert!(text.x > root.x && text.y > root.y);
        assert!(text.width < d && text.height < d);
        assert!((text.x + text.width / 2.0 - root.x - d / 2.0).abs() <= 1.0);
        assert!((text.y + text.height / 2.0 - root.y - d / 2.0).abs() <= 1.0);
    }
}

#[test]
fn avatar_and_initials_ids_survive_size_text_changes_and_keyed_reordering() {
    let mut tree = RetainedTree::new();
    let mut saved = None;
    for changed in [false, true, false] {
        let a: WidgetNode<()> = mobile_avatar(if changed { "陳" } else { "AB" })
            .key("a")
            .size(if changed {
                MobileAvatarSize::Lg
            } else {
                MobileAvatarSize::Sm
            })
            .into();
        let b: WidgetNode<()> = mobile_avatar("").key("b").into();
        let roots = if changed { vec![b, a] } else { vec![a, b] };
        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0);
        let ids = ["a", "b"].map(|key| {
            let node = roots
                .iter()
                .find(|n| n.key().unwrap().as_str() == key)
                .unwrap();
            let l = node.to_layout_with_binding(&binding, &context());
            (l.id, l.children[0].id)
        });
        if let Some(old) = &saved {
            assert_eq!(&ids, old);
        } else {
            saved = Some(ids);
        }
    }
}
