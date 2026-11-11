use super::*;
use acme_core::{FocusManager, RetainedTree};
use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};

fn content(node: &WidgetNode<u8>) -> &WidgetNode<u8> {
    &node.children()[0]
}

#[test]
fn converted_fullscreen_close_respects_current_message_and_optional_control() {
    for visible in [false, true] {
        for message in [None, Some(7_u8), Some(9)] {
            let mut builder = fullscreen("preview")
                .close_button(visible)
                .child(crate::label("預覽內容"));
            if let Some(message) = message {
                builder = builder.on_close(message);
            }
            let node: WidgetNode<u8> = builder.into();
            let WidgetNode::Card(card) = &node else {
                panic!("existing Card root")
            };
            assert_eq!(card.variant, crate::CardVariant::Plain);
            assert_eq!(card.border_radius, Some(0.0));
            let children = content(&node).children();
            assert_eq!(children.len(), if visible { 2 } else { 1 });
            if visible {
                let header = children[0].children();
                match (message, &header[1]) {
                    (Some(message), WidgetNode::Button(button)) => {
                        assert_eq!(button.activate(), Some(&message));
                        assert_eq!(button.key.as_str(), "preview_close");
                        assert_eq!(button.label, "✕");
                    }
                    (None, WidgetNode::Label(label)) => assert_eq!(label.text, "✕"),
                    _ => panic!(
                        "configured close must carry its message; passive close remains Label"
                    ),
                }
            }
            assert!(
                matches!(&children.last().unwrap().children()[0], WidgetNode::Label(l) if l.text == "預覽內容")
            );
        }
    }
}

#[test]
fn fullscreen_body_focus_survives_optional_close_header_and_current_close_updates() {
    let mut tree = RetainedTree::new();
    let mut focus = FocusManager::new();
    let mut body_ids = None;
    let mut close_id = None;
    for (visible, message) in [(true, 1), (true, 2), (false, 3), (true, 4)] {
        let roots = [WidgetNode::from(
            fullscreen("preview")
                .close_button(visible)
                .on_close(message)
                .child(crate::button("body_action", "Body action").on_click(42_u8)),
        )];
        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0);
        let children = content(&roots[0]).children();
        let body = children.last().unwrap();
        let body_action = &body.children()[0];
        let ids = [
            binding.id_for(body).unwrap(),
            binding.id_for(body_action).unwrap(),
        ];
        focus.set_order([(ids[1], tree.get(ids[1]).unwrap().focusable)]);
        if let Some(saved) = body_ids {
            assert_eq!(ids, saved, "optional header must not remount body");
            assert_eq!(focus.focused(), Some(ids[1]));
        } else {
            assert!(focus.request_focus(ids[1]));
        }
        body_ids = Some(ids);
        if visible {
            let close = &children[0].children()[1];
            let id = binding.id_for(close).unwrap();
            assert!(tree.get(id).unwrap().focusable);
            let WidgetNode::Button(button) = close else {
                panic!("Close Button")
            };
            assert_eq!(button.activate(), Some(&message));
            if message == 2 {
                assert_eq!(Some(id), close_id);
            }
            close_id = Some(id);
        }
    }
}
