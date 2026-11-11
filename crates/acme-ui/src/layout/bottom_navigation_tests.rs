use super::*;
use acme_core::RetainedTree;
use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};

#[derive(Clone, Debug, PartialEq)]
enum Message {
    Select(usize),
    Updated(usize),
}

fn navigation(key: &str, selected: usize, labels: bool, updated: bool) -> WidgetNode<Message> {
    bottom_navigation()
        .key(key)
        .action(BottomNavigationAction::new("首頁").icon(crate::IconName::User))
        .action(BottomNavigationAction::new("搜尋").icon(crate::IconName::Search))
        .selected_index(selected)
        .show_labels(labels)
        .on_select(if updated {
            Message::Updated
        } else {
            Message::Select
        })
        .into()
}

#[test]
fn converted_actions_carry_current_indexed_messages_and_preserve_display_text() {
    for selected in [0, 1, usize::MAX] {
        for labels in [true, false] {
            for updated in [false, true] {
                let node = navigation("nav", selected, labels, updated);
                for (index, child) in node.children().iter().enumerate() {
                    let WidgetNode::Button(button) = child else {
                        panic!("configured navigation action must be a Button");
                    };
                    let expected = if updated {
                        Message::Updated(index)
                    } else {
                        Message::Select(index)
                    };
                    assert_eq!(button.activate(), Some(&expected));
                    let icon = if index == 0 {
                        crate::IconName::User
                    } else {
                        crate::IconName::Search
                    };
                    let text = if labels {
                        format!(
                            "{} {} {}",
                            icon.char(),
                            if selected == index { "▣" } else { "▢" },
                            if index == 0 { "首頁" } else { "搜尋" }
                        )
                    } else {
                        icon.char().to_string()
                    };
                    assert_eq!(button.label, text);
                    assert_eq!(button.key.as_str(), format!("nav:action:{index}"));
                }
            }
        }
    }
}

#[test]
fn missing_callback_preserves_passive_labels_and_empty_actions_stay_empty() {
    for labels in [false, true] {
        let node: WidgetNode<Message> = bottom_navigation()
            .action(BottomNavigationAction::new("Read only"))
            .show_labels(labels)
            .into();
        assert!(
            matches!(&node.children()[0], WidgetNode::Label(l) if l.text == if labels { " ▣ Read only" } else { "" })
        );
        assert!(!widget_to_views(&[node])[0].children[0].focusable);
    }
    let empty: WidgetNode<Message> = bottom_navigation().on_select(Message::Select).into();
    assert!(empty.children().is_empty());
}

#[test]
fn action_identity_and_focus_survive_selection_label_changes_and_instance_reordering() {
    let mut tree = RetainedTree::new();
    let mut saved = None;
    for (selected, labels, reverse) in [
        (0, true, false),
        (1, false, true),
        (usize::MAX, true, true),
        (0, true, false),
    ] {
        let mut roots = vec![
            navigation("a", selected, labels, reverse),
            navigation("b", selected, labels, reverse),
        ];
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
            let root_id = binding.id_for(root).unwrap();
            assert!(!tree.get(root_id).unwrap().focusable);
            let children = root
                .children()
                .iter()
                .map(|n| binding.id_for(n).unwrap())
                .collect::<Vec<_>>();
            for &id in &children {
                assert!(tree.get(id).unwrap().focusable);
            }
            (root_id, children)
        });
        if let Some(saved) = &saved {
            assert_eq!(&ids, saved);
        }
        assert_ne!(ids[0].1[0], ids[0].1[1]);
        assert_ne!(ids[0].1[0], ids[1].1[0]);
        saved = Some(ids);
    }
}

#[test]
fn configured_navigation_has_finite_nonoverlapping_real_layout() {
    for width in [240.0, 600.0] {
        for labels in [false, true] {
            let node = navigation("layout", 1, labels, false);
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
                .compute_with_text(&layout, (width, 120.0), &mut fonts, 1.0)
                .unwrap();
            let parent = snapshot.get(layout.id).unwrap();
            assert_eq!(parent.height, 56.0);
            let mut right = parent.x;
            for child in &layout.children {
                let r = snapshot.get(child.id).unwrap();
                assert!(r.width.is_finite() && r.width > 0.0 && r.height > 0.0);
                assert!(r.x >= right && r.x + r.width <= parent.x + parent.width + 0.01);
                assert!(r.y >= parent.y && r.y + r.height <= parent.y + parent.height + 0.01);
                right = r.x + r.width;
            }
        }
    }
}
