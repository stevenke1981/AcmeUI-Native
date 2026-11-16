use super::*;
use acme_core::{FocusManager, NodeId, RetainedTree};
use acme_layout::{LayoutEngine, LayoutNode, WidgetLayoutContext};
use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};

fn preview(value: u8, active: bool) -> WidgetNode<u8> {
    let mut builder = confirm_dialog("confirm_preview")
        .open(true)
        .message(format!("Save revision {value}?"))
        .confirm_label(format!("Save {value}"))
        .cancel_label(format!("Cancel {value}"));
    if active {
        builder = builder.on_confirm(value).on_cancel(value + 10);
    }
    builder.build()
}

fn controls(node: &WidgetNode<u8>) -> &[WidgetNode<u8>] {
    node.children()[0].children()[1].children()
}

#[test]
fn confirm_dialog_current_messages_passive_controls_and_retained_focus() {
    let mut tree = RetainedTree::new();
    let mut focus = FocusManager::new();
    let mut saved = None;
    for value in [1, 2] {
        let roots = [preview(value, true)];
        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0);
        let buttons = controls(&roots[0]);
        let ids: Vec<_> = buttons.iter().map(|b| binding.id_for(b).unwrap()).collect();
        assert_ne!(ids[0], ids[1]);
        focus.set_order(ids.iter().map(|id| (*id, tree.get(*id).unwrap().focusable)));
        if let Some(saved) = saved {
            assert_eq!(ids, saved);
            assert_eq!(focus.focused(), Some(ids[1]));
        } else {
            assert!(focus.request_focus(ids[1]));
        }
        for (button, message) in buttons.iter().zip([value + 10, value]) {
            let WidgetNode::Button(button) = button else {
                panic!("Button")
            };
            assert_eq!(button.activate(), Some(&message));
        }
        saved = Some(ids);
    }
    let roots = [preview(3, false)];
    tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
    let binding = bind_retained(&roots, &tree);
    assert_eq!(binding.mismatches(), 0);
    for button in controls(&roots[0]) {
        let WidgetNode::Button(b) = button else {
            panic!("passive Button")
        };
        assert_eq!(b.activate(), None);
        assert!(binding.id_for(button).is_some());
    }
}

fn assert_alignment(widget: &WidgetNode<u8>, layout: &LayoutNode) {
    assert_eq!(
        widget.children().len(),
        layout.children.len(),
        "Dialog content must participate in layout"
    );
    for (widget, layout) in widget.children().iter().zip(&layout.children) {
        assert_alignment(widget, layout);
    }
}

#[test]
fn confirm_dialog_plain_layout_includes_content() {
    let node = preview(1, true);
    assert_alignment(&node, &node.to_layout(NodeId::new(1)));
}

#[test]
fn confirm_dialog_gallery_layout_controls_are_reachable() {
    let context = WidgetLayoutContext {
        body_font_size: 14.0,
        body_line_height: 21.0,
        label_font_size: 13.0,
        control_height: 36.0,
        scale_factor: 1.0,
    };
    let roots = [preview(1, true)];
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
        let dialog = snapshot.get(layout.id).unwrap();
        let buttons = &layout.children[0].children[1].children;
        let mut previous_right = dialog.x;
        for (widget, button) in controls(&roots[0]).iter().zip(buttons) {
            if bound {
                assert_eq!(Some(button.id), binding.id_for(widget));
            }
            let rect = snapshot.get(button.id).expect("control layout rect");
            assert!(
                [rect.x, rect.y, rect.width, rect.height]
                    .iter()
                    .all(|v| v.is_finite())
            );
            assert!(rect.width > 0.0 && rect.height > 0.0);
            assert!(rect.x >= previous_right && rect.x + rect.width <= dialog.x + dialog.width);
            assert!(rect.y >= dialog.y && rect.y + rect.height <= dialog.y + dialog.height);
            previous_right = rect.x + rect.width;
        }
    }
}
