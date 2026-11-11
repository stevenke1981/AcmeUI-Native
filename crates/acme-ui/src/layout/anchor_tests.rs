use super::*;
use acme_core::{FocusManager, RetainedTree};
use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};

#[derive(Clone, Debug, PartialEq)]
enum Message {
    Navigate(String),
    Updated(String),
}

fn navigation(key: &str, active: &str, updated: bool) -> WidgetNode<Message> {
    anchor()
        .key(key)
        .item(AnchorItem::new("#intro", "簡介"))
        .item(AnchorItem::new("#api", "API").child(AnchorItem::new("#get", "GET")))
        .active_href(active)
        .on_navigate(if updated {
            |href| Message::Updated(href.into())
        } else {
            |href| Message::Navigate(href.into())
        })
        .into()
}

#[test]
fn converted_anchor_links_emit_current_hrefs_and_preserve_passive_text() {
    for updated in [false, true] {
        let node = navigation("links", "#api", updated);
        assert!(matches!(node, WidgetNode::Column(_)));
        for (index, (href, text, path)) in [
            ("#intro", "  簡介", "0"),
            ("#api", "▸ API", "1"),
            ("#get", "    · GET", "1:0"),
        ]
        .into_iter()
        .enumerate()
        {
            let WidgetNode::Button(button) = &node.children()[index] else {
                panic!("configured rendered links must be Buttons")
            };
            assert_eq!(button.label, text);
            assert_eq!(button.key.as_str(), format!("links:link:{path}"));
            assert_eq!(
                button.activate(),
                Some(&if updated {
                    Message::Updated(href.into())
                } else {
                    Message::Navigate(href.into())
                })
            );
        }
    }
    let passive: WidgetNode<Message> = anchor()
        .item(AnchorItem::new("#api", "API").child(AnchorItem::new("#get", "GET")))
        .active_href("#api")
        .into();
    for (child, text) in passive.children().iter().zip(["▸ API", "    · GET"]) {
        assert!(matches!(child, WidgetNode::Label(label) if label.text == text));
    }
    assert!(
        widget_to_views(&[passive])[0]
            .children
            .iter()
            .all(|v| !v.focusable)
    );
    let empty: WidgetNode<Message> = anchor().on_navigate(|s| Message::Navigate(s.into())).into();
    assert!(empty.children().is_empty());
}

#[test]
fn anchor_focus_survives_active_href_and_callback_updates_and_instance_reorder() {
    let mut tree = RetainedTree::new();
    let mut focus = FocusManager::new();
    let mut saved = None;
    for (active, updated, reverse) in [
        ("#intro", false, false),
        ("#get", true, true),
        ("#api", false, false),
    ] {
        let mut roots = vec![
            navigation("a", active, updated),
            navigation("b", active, updated),
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
            root.children()
                .iter()
                .map(|n| binding.id_for(n).unwrap())
                .collect::<Vec<_>>()
        });
        assert_ne!(ids[0][0], ids[1][0]);
        focus.set_order(
            ids.iter()
                .flatten()
                .map(|&id| (id, tree.get(id).unwrap().focusable)),
        );
        if let Some(saved) = &saved {
            assert_eq!(&ids, saved);
            assert_eq!(focus.focused(), Some(ids[0][2]));
        } else {
            assert!(focus.request_focus(ids[0][2]));
        }
        saved = Some(ids);
    }
}
