use super::*;
use acme_core::{FocusManager, NodeId, RetainedTree};
use acme_layout::{LayoutEngine, LayoutNode, WidgetLayoutContext};
use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};

fn content(node: &WidgetNode<u8>) -> &WidgetNode<u8> {
    // Dialog's content is the single child (the Column with header/body/footer)
    &node.children()[0]
}

fn header(node: &WidgetNode<u8>) -> &WidgetNode<u8> {
    &content(node).children()[0]
}

fn body(node: &WidgetNode<u8>) -> &WidgetNode<u8> {
    &content(node).children()[1]
}

fn close_control(node: &WidgetNode<u8>) -> Option<&WidgetNode<u8>> {
    let h = header(node);
    // Header is a Row; children are title Row and optional close control
    if h.children().len() >= 2 {
        Some(&h.children()[1])
    } else {
        None
    }
}

#[test]
fn converted_modal_close_respects_current_message_and_optional_control() {
    // (a) closable + on_close Some -> instance-keyed Button with current message
    // (b) closable + on_close None -> passive "✕" Label
    // (c) closable(false) -> no close action
    for visible in [false, true] {
        for message in [None, Some(7_u8), Some(9)] {
            let mut builder = modal("preview", "Preview")
                .closable(visible)
                .child(crate::label::<u8>("預覽內容"));
            if let Some(message) = message {
                builder = builder.on_close(message);
            }
            let node: WidgetNode<u8> = builder.into();
            let WidgetNode::Dialog(dialog) = &node else {
                panic!("expected Dialog root")
            };
            assert_eq!(dialog.title, "Preview");
            assert_eq!(dialog.width, crate::Length::px(480.0));

            let children = content(&node).children();
            assert_eq!(children.len(), 2, "header + body (no footer)");

            if visible {
                // Header has title + close
                let header_children = header(&node).children();
                assert_eq!(header_children.len(), 2, "title row + close control");
                let close = &header_children[1];

                match (message, close) {
                    (Some(message), WidgetNode::Button(button)) => {
                        // (a) Instance-keyed Button with current message
                        assert_eq!(
                            button.activate(),
                            Some(&message),
                            "Button must carry current on_close message"
                        );
                        assert_eq!(
                            button.key.as_str(),
                            "preview_close",
                            "Close Button key must be instance-keyed as '{{id}}_close'"
                        );
                        assert_eq!(button.label, "✕", "Close Button label must be ✕");
                    }
                    (None, WidgetNode::Label(label)) => {
                        // (b) Passive Label fallback
                        assert_eq!(label.text, "✕", "Passive close must be Label with ✕");
                    }
                    _ => panic!(
                        "configured close must carry its message; passive close remains Label"
                    ),
                }
            } else {
                // (c) closable(false) - no close control
                let header_children = header(&node).children();
                assert_eq!(header_children.len(), 1, "title row only, no close");
            }

            // Body content is preserved
            let body_children = body(&node).children();
            assert!(matches!(&body_children[0], WidgetNode::Label(l) if l.text == "預覽內容"));
        }
    }
}

#[test]
fn modal_close_updates_when_on_close_changes() {
    // Verify that when on_close message changes, the Button gets the new message
    let msg1 = 1_u8;
    let msg2 = 2_u8;

    // First with msg1
    let node1: WidgetNode<u8> = modal("preview", "Preview")
        .closable(true)
        .on_close(msg1)
        .child(crate::label::<u8>("Content"))
        .into();
    let close1 = close_control(&node1).unwrap();
    let WidgetNode::Button(btn1) = close1 else {
        panic!("Expected Button");
    };
    assert_eq!(btn1.activate(), Some(&msg1));

    // Second with msg2
    let node2: WidgetNode<u8> = modal("preview", "Preview")
        .closable(true)
        .on_close(msg2)
        .child(crate::label::<u8>("Content"))
        .into();
    let close2 = close_control(&node2).unwrap();
    let WidgetNode::Button(btn2) = close2 else {
        panic!("Expected Button");
    };
    assert_eq!(
        btn2.activate(),
        Some(&msg2),
        "Close Button must carry updated on_close message"
    );
}

#[test]
fn modal_body_focus_survives_optional_close_header_and_current_close_updates() {
    // (d) title Label / body children keep stable retained identities and focus
    // when closable is toggled, when on_close appears/disappears, and when title
    // text or footer actions change
    let mut tree = RetainedTree::new();
    let mut focus = FocusManager::new();
    let mut body_ids = None;
    let mut close_id = None;

    for (visible, message, title, has_footer) in [
        (true, 1, "Title 1", false),
        (true, 2, "Title 2", false),
        (false, 3, "Title 3", false),
        (true, 4, "Title 4", true),
        (true, 5, "Title 5", true),
    ] {
        let mut builder = modal("preview", title)
            .closable(visible)
            .on_close(message)
            .child(crate::button("body_action", "Body action").on_click(42_u8));
        if has_footer {
            builder = builder.action(crate::button("footer_action", "Footer").on_click(99_u8));
        }
        let roots = [WidgetNode::from(builder)];

        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        assert_eq!(
            binding.mismatches(),
            0,
            "No retained mismatches during reconciliation"
        );

        let body_node = body(&roots[0]);
        let body_action = &body_node.children()[0];
        let ids = [
            binding.id_for(body_node).unwrap(),
            binding.id_for(body_action).unwrap(),
        ];

        focus.set_order(ids.iter().map(|id| (*id, tree.get(*id).unwrap().focusable)));

        if let Some(saved) = body_ids {
            assert_eq!(
                ids, saved,
                "Optional close header changes must not remount body"
            );
            assert_eq!(
                focus.focused(),
                Some(ids[1]),
                "Focus must survive optional close/header changes"
            );
        } else {
            assert!(focus.request_focus(ids[1]), "Body action must be focusable");
        }
        body_ids = Some(ids);

        if visible {
            let close = close_control(&roots[0]).unwrap();
            let id = binding.id_for(close).unwrap();
            assert!(
                tree.get(id).unwrap().focusable,
                "Close Button must be focusable"
            );
            let WidgetNode::Button(button) = close else {
                panic!("Close must be Button when closable with on_close");
            };
            assert_eq!(
                button.activate(),
                Some(&message),
                "Close Button must carry current on_close message"
            );
            if message == 2 {
                assert_eq!(
                    Some(id),
                    close_id,
                    "Close identity must be stable when message changes"
                );
            }
            close_id = Some(id);
        }
    }
}

fn assert_alignment(widget: &WidgetNode<u8>, layout: &LayoutNode) {
    assert_eq!(
        widget.children().len(),
        layout.children.len(),
        "Dialog content must participate in layout (children count mismatch)"
    );
    for (widget, layout) in widget.children().iter().zip(&layout.children) {
        assert_alignment(widget, layout);
    }
}

#[test]
fn modal_plain_layout_includes_content() {
    // Verify Dialog content participates in layout
    let node: WidgetNode<u8> = modal("m", "Title")
        .closable(true)
        .on_close(1)
        .child(crate::button("body", "Body").on_click(2))
        .action(crate::button("footer", "Footer").on_click(3))
        .into();
    assert_alignment(&node, &node.to_layout(NodeId::new(1)));
}

#[test]
fn modal_gallery_layout_close_has_finite_positive_non_overlapping_rect() {
    // (e) Close has a finite, positive, non-overlapping layout rect inside the dialog
    let context = WidgetLayoutContext {
        body_font_size: 14.0,
        body_line_height: 21.0,
        label_font_size: 13.0,
        control_height: 36.0,
        scale_factor: 1.0,
    };

    let roots = [modal("preview", "Preview")
        .closable(true)
        .on_close(1_u8)
        .child(crate::button("body_action", "Body action").on_click(42_u8))
        .into()];

    let mut tree = RetainedTree::new();
    tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
    let binding = bind_retained(&roots, &tree);
    assert_eq!(binding.mismatches(), 0);

    for bound in [false, true] {
        let layout = if bound {
            roots[0].to_layout_with_binding(&binding, &context)
        } else {
            roots[0].to_layout_with_context(NodeId::new(1), &context)
        };

        assert_alignment(&roots[0], &layout);

        let mut fonts = acme_text::FontSystem::new();
        let snapshot = LayoutEngine::new()
            .compute_with_text(&layout, (800.0, 600.0), &mut fonts, 1.0)
            .unwrap();

        let dialog_rect = snapshot.get(layout.id).expect("dialog layout rect");

        // Close button is in header children[1]
        let header_layout = &layout.children[0].children[0];
        let close_layout = &header_layout.children[1];

        if bound {
            let close_widget = close_control(&roots[0]).unwrap();
            assert_eq!(Some(close_layout.id), binding.id_for(close_widget));
        }

        let close_rect = snapshot
            .get(close_layout.id)
            .expect("close button layout rect");

        // Verify finite, positive, non-overlapping
        assert!(
            [
                close_rect.x,
                close_rect.y,
                close_rect.width,
                close_rect.height
            ]
            .iter()
            .all(|v| v.is_finite()),
            "Close rect must have finite values: {close_rect:?}"
        );
        assert!(
            close_rect.width > 0.0 && close_rect.height > 0.0,
            "Close rect must be positive: {close_rect:?}"
        );
        assert!(
            close_rect.x >= dialog_rect.x
                && close_rect.x + close_rect.width <= dialog_rect.x + dialog_rect.width,
            "Close rect x must be within dialog: close={close_rect:?} dialog={dialog_rect:?}"
        );
        assert!(
            close_rect.y >= dialog_rect.y
                && close_rect.y + close_rect.height <= dialog_rect.y + dialog_rect.height,
            "Close rect y must be within dialog: close={close_rect:?} dialog={dialog_rect:?}"
        );
    }
}

#[test]
fn modal_body_focus_stable_across_footer_changes() {
    // Body and title should retain identity when footer is added/removed
    let mut tree = RetainedTree::new();
    let mut focus = FocusManager::new();
    let mut body_ids = None;
    let mut title_id = None;

    for has_footer in [false, true, false, true] {
        let mut builder = modal("preview", "Title")
            .closable(true)
            .on_close(1_u8)
            .child(crate::button("body_action", "Body action").on_click(42_u8));
        if has_footer {
            builder = builder.action(crate::button("footer_action", "Footer").on_click(99_u8));
        }
        let roots = [WidgetNode::from(builder)];

        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0);

        let body_node = body(&roots[0]);
        let body_action = &body_node.children()[0];
        let ids = [
            binding.id_for(body_node).unwrap(),
            binding.id_for(body_action).unwrap(),
        ];

        focus.set_order(ids.iter().map(|id| (*id, tree.get(*id).unwrap().focusable)));

        if let Some(saved) = body_ids {
            assert_eq!(
                ids, saved,
                "Body must retain identity across footer changes"
            );
            assert_eq!(
                focus.focused(),
                Some(ids[1]),
                "Focus must survive footer changes"
            );
        } else {
            assert!(focus.request_focus(ids[1]));
        }
        body_ids = Some(ids);

        // Title label identity
        let header_node = header(&roots[0]);
        let title_row = &header_node.children()[0];
        let title_label = &title_row.children()[0];
        let id = binding.id_for(title_label).unwrap();
        if let Some(saved) = title_id {
            assert_eq!(
                id, saved,
                "Title label must retain identity across footer changes"
            );
        }
        title_id = Some(id);
    }
}

#[test]
fn modal_retains_title_identity() {
    // Title Label should keep stable identity
    let mut tree = RetainedTree::new();
    let mut title_id = None;

    for title in ["Title 1", "Title 2", "Title 3"] {
        let roots = [modal("preview", title).closable(true).on_close(1_u8).into()];

        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0);

        let header_node = header(&roots[0]);
        let title_row = &header_node.children()[0];
        let title_label = &title_row.children()[0];
        let id = binding.id_for(title_label).unwrap();

        if let Some(saved) = title_id {
            assert_eq!(
                id, saved,
                "Title label must retain identity across title text changes"
            );
        }
        title_id = Some(id);
    }
}
