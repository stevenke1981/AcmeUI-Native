use super::*;
use acme_core::{FocusManager, RetainedTree};
use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};

fn section(key: &str, open: bool, disabled: bool, message: u8) -> WidgetNode<u8> {
    collapsible(key)
        .title(format!("詳細資料 {message}"))
        .open(open)
        .disabled(disabled)
        .on_toggle(message)
        .child(label("Body · 內容"))
        .into()
}

#[test]
fn converted_collapsible_header_binds_current_message_only_when_enabled() {
    for open in [false, true] {
        for disabled in [false, true] {
            for message in [Some(7_u8), Some(9), None] {
                let mut builder = collapsible("section")
                    .title("Details")
                    .open(open)
                    .disabled(disabled)
                    .child(label("Body"));
                if let Some(message) = message {
                    builder = builder.on_toggle(message);
                }
                let node: WidgetNode<u8> = builder.into();
                assert!(matches!(node, WidgetNode::Column(_)));
                assert_eq!(node.children().len(), if open { 2 } else { 1 });
                let WidgetNode::Row(header) = &node.children()[0] else {
                    panic!("preserve header Row")
                };
                assert_eq!(header.message, if disabled { None } else { message });
                assert_eq!(header.key.as_ref().unwrap().as_str(), "section_header");
                assert!(
                    matches!(&header.children[0], WidgetNode::Label(l) if l.text == if open { "▾" } else { "▸" })
                );
                assert!(matches!(&header.children[1], WidgetNode::Label(l) if l.text == "Details"));
                assert_eq!(
                    widget_to_views(&[node])[0].children[0].focusable,
                    !disabled && message.is_some()
                );
            }
        }
    }
}

#[test]
fn collapsible_header_retains_focus_through_updates_and_loses_eligibility_when_disabled() {
    let mut tree = RetainedTree::new();
    let mut focus = FocusManager::new();
    let mut saved = None;
    for (open, disabled, message, reverse) in [
        (false, false, 1, false),
        (true, false, 2, true),
        (false, false, 3, false),
        (true, true, 4, true),
        (true, false, 5, false),
    ] {
        let mut roots = vec![
            section("a", open, disabled, message),
            section("b", open, disabled, message),
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
            assert!(!tree.get(binding.id_for(root).unwrap()).unwrap().focusable);
            binding.id_for(&root.children()[0]).unwrap()
        });
        assert_ne!(ids[0], ids[1]);
        focus.set_order(ids.map(|id| (id, tree.get(id).unwrap().focusable)));
        if let Some(saved) = saved {
            assert_eq!(ids, saved);
        }
        if disabled {
            assert_eq!(focus.focused(), None);
            assert!(!focus.request_focus(ids[0]));
        } else {
            if message == 2 || message == 3 {
                assert_eq!(focus.focused(), Some(ids[0]));
            }
            if message == 5 {
                assert_eq!(focus.focused(), None);
            }
            assert!(focus.request_focus(ids[0]));
        }
        saved = Some(ids);
    }
}
