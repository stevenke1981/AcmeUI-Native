use crate::retained_binding::bind_retained;
use crate::view_bridge::widget_to_views;
use crate::{WidgetNode, column, label, row, stack};
use acme_core::{FocusManager, RetainedTree};

#[test]
fn container_handlers_join_retained_focus_order_and_survive_reorders() {
    let mut tree = RetainedTree::new();
    let mut focus = FocusManager::new();
    let mut first_ids = None;
    for (reverse, active) in [(false, true), (true, true), (false, false)] {
        let mut a = row().key("a").child(label("Open"));
        if active {
            a = a.on_click(1);
        }
        let b = column().key("b").on_click(2).build();
        let roots = if reverse {
            vec![b, a.build()]
        } else {
            vec![a.build(), b]
        };
        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        let ids = ["a", "b"].map(|key| {
            binding
                .id_for(
                    roots
                        .iter()
                        .find(|n| n.key().unwrap().as_str() == key)
                        .unwrap(),
                )
                .unwrap()
        });
        if let Some(first) = first_ids {
            assert_eq!(ids, first);
        }
        first_ids = Some(ids);
        focus.set_order(
            tree.roots()
                .iter()
                .map(|&id| (id, tree.get(id).unwrap().focusable)),
        );
        assert!(tree.get(ids[1]).unwrap().focusable);
        if active {
            assert!(tree.get(ids[0]).unwrap().focusable);
            if reverse {
                assert_eq!(focus.focused(), Some(ids[0]));
            }
            assert!(focus.request_focus(ids[0]));
            assert_eq!(focus.focus_next(reverse), Some(ids[1]));
            assert!(focus.request_focus(ids[0]));
        } else {
            assert!(!tree.get(ids[0]).unwrap().focusable);
            assert_eq!(focus.focused(), None);
            assert_eq!(focus.focus_next(false), Some(ids[1]));
        }
    }
    for node in [row::<()>().build(), column().build(), stack().build()] {
        assert!(!widget_to_views(&[node])[0].focusable);
    }
    let node: WidgetNode<i32> = stack().on_click(3).build();
    assert!(widget_to_views(&[node])[0].focusable);
}
