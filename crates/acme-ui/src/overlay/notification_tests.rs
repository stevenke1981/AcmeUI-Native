use super::*;
use acme_core::{FocusManager, RetainedTree};
use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};

fn preview(key: &str, description: bool, action: bool, close: bool, message: u8) -> WidgetNode<u8> {
    let mut b = notification("Title").key(key);
    if description {
        b = b.description("Details");
    }
    if action {
        b = b.action("Undo", message);
    }
    if close {
        b = b.on_close(message + 10);
    }
    b.into()
}

#[test]
fn notification_current_typed_messages() {
    for message in [1, 2] {
        for action in [false, true] {
            for close in [false, true] {
                let node = preview("a", true, action, close, message);
                let messages: Vec<_> = node
                    .children()
                    .iter()
                    .filter_map(|n| {
                        if let WidgetNode::Button(b) = n {
                            b.activate().copied()
                        } else {
                            None
                        }
                    })
                    .collect();
                let expected: Vec<_> = [action.then_some(message), close.then_some(message + 10)]
                    .into_iter()
                    .flatten()
                    .collect();
                assert_eq!(messages, expected);
            }
        }
    }
}

#[test]
fn notification_instances_reorder_with_distinct_control_identity_and_focus() {
    for (owner, control) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
        let mut tree = RetainedTree::new();
        let mut focus = FocusManager::new();
        let mut saved = None;
        for (message, keys) in [(1, ["a", "b"]), (2, ["b", "a"])] {
            let roots = [crate::column()
                .child(preview(keys[0], true, true, true, message))
                .child(preview(keys[1], true, true, true, message))
                .build()];
            tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
            let binding = bind_retained(&roots, &tree);
            assert_eq!(binding.mismatches(), 0);
            let instances = roots[0].children();
            let a = &instances[keys.iter().position(|k| *k == "a").unwrap()];
            let b = &instances[keys.iter().position(|k| *k == "b").unwrap()];
            for node in [a, b] {
                for (index, expected) in [(1, message), (2, message + 10)] {
                    let WidgetNode::Button(button) = &node.children()[index] else {
                        panic!("Button")
                    };
                    assert_eq!(button.activate(), Some(&expected));
                }
            }
            let ids = [a, b].map(|n| {
                [
                    binding.id_for(&n.children()[1]).unwrap(),
                    binding.id_for(&n.children()[2]).unwrap(),
                ]
            });
            assert_ne!(
                ids[0][0], ids[1][0],
                "different notifications need distinct action identities"
            );
            assert_ne!(
                ids[0][1], ids[1][1],
                "different notifications need distinct close identities"
            );
            focus.set_order(
                ids.into_iter()
                    .flatten()
                    .map(|id| (id, tree.get(id).unwrap().focusable)),
            );
            if let Some(old) = saved {
                assert_eq!(ids, old);
                assert_eq!(focus.focused(), Some(ids[owner][control]));
            } else {
                assert!(focus.request_focus(ids[owner][control]));
            }
            saved = Some(ids);
        }
    }
}

#[test]
fn notification_optional_siblings_preserve_body_and_surviving_control_focus() {
    for keep_action in [false, true] {
        let mut tree = RetainedTree::new();
        let mut focus = FocusManager::new();
        let mut saved = None;
        for (description, other) in [(false, true), (true, false), (false, true), (true, true)] {
            let roots = [preview(
                "a",
                description,
                keep_action || other,
                !keep_action || other,
                1,
            )];
            tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
            let binding = bind_retained(&roots, &tree);
            assert_eq!(binding.mismatches(), 0);
            let children = roots[0].children();
            let control = children.iter().find(|n| matches!(n, WidgetNode::Button(b) if b.activate() == Some(&if keep_action { 1 } else { 11 }))).unwrap();
            let ids = [
                binding.id_for(&children[0]).unwrap(),
                binding.id_for(control).unwrap(),
                binding.id_for(&children[0].children()[0]).unwrap(),
                binding.id_for(&children[0].children()[1]).unwrap(),
                binding
                    .id_for(&children[0].children()[1].children()[0])
                    .unwrap(),
            ];
            // The body is passive; assert identity for Row/icon/content/title and
            // focus continuity only for the surviving interactive control.
            assert!(!tree.get(ids[0]).unwrap().focusable);
            focus.set_order([(ids[1], tree.get(ids[1]).unwrap().focusable)]);
            if let Some(old) = saved {
                assert_eq!(
                    ids, old,
                    "optional siblings must not remount body or surviving control"
                );
                assert_eq!(focus.focused(), Some(ids[1]));
            } else {
                assert!(focus.request_focus(ids[1]));
            }
            saved = Some(ids);
        }
    }
}

#[test]
fn notification_passive_body_survives_all_optional_combinations() {
    let mut tree = RetainedTree::new();
    let mut saved = None;
    for description in [false, true, false] {
        for action in [false, true, false] {
            for close in [false, true, false] {
                let roots = [preview("body", description, action, close, 1)];
                tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
                let binding = bind_retained(&roots, &tree);
                assert_eq!(binding.mismatches(), 0);
                let body = &roots[0].children()[0];
                let content = &body.children()[1];
                let ids = [body, &body.children()[0], content, &content.children()[0]]
                    .map(|node| binding.id_for(node).unwrap());
                assert!(ids.iter().all(|id| !tree.get(*id).unwrap().focusable));
                if let Some(old) = saved {
                    assert_eq!(ids, old, "all optional combinations preserve passive body");
                }
                saved = Some(ids);
            }
        }
    }
}
