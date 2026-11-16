use super::*;
use acme_core::{FocusManager, NodeId, RetainedTree};
use acme_layout::{LayoutEngine, LayoutNode, LayoutRect, WidgetLayoutContext};
use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};

fn dropdown_card(node: &WidgetNode<u8>) -> &WidgetNode<u8> {
    &node.children()[1]
}

fn menu_items(node: &WidgetNode<u8>) -> &[WidgetNode<u8>] {
    dropdown_card(node).children()
}

fn item_key(menu_id: &str, item_key: &str, index: usize) -> String {
    if item_key.is_empty() {
        format!("{menu_id}_item_{index}")
    } else {
        format!("{menu_id}_{item_key}")
    }
}

fn layout_context() -> WidgetLayoutContext {
    WidgetLayoutContext {
        body_font_size: 14.0,
        body_line_height: 21.0,
        label_font_size: 13.0,
        control_height: 36.0,
        scale_factor: 1.0,
    }
}

fn assert_alignment(widget: &WidgetNode<u8>, layout: &LayoutNode) {
    assert_eq!(
        widget.children().len(),
        layout.children.len(),
        "widget and layout children must stay aligned"
    );
    for (widget, layout) in widget.children().iter().zip(&layout.children) {
        assert_alignment(widget, layout);
    }
}

fn sample_open_menu(cut: u8, copy: Option<u8>) -> WidgetNode<u8> {
    let mut copy_item = menu_item("copy", "Copy");
    if let Some(message) = copy {
        copy_item = copy_item.on_click(message);
    }
    context_menu("preview")
        .anchor(crate::button("anchor_action", "Anchor").on_click(99_u8))
        .item(menu_item("cut", "Cut").on_click(cut))
        .item(copy_item)
        .item(menu_item("lock", "Lock").disabled(true).on_click(7_u8))
        .separator()
        .item(menu_item("", "Empty key").on_click(8_u8))
        .open(true)
        .into()
}

#[test]
fn converted_context_menu_items_respect_current_message_and_optional_control() {
    let node = sample_open_menu(1, None);
    let WidgetNode::Stack(_) = &node else {
        panic!("open context menu must stay a Stack");
    };
    let items = menu_items(&node);
    assert_eq!(items.len(), 5, "cut, copy, disabled, separator, empty-key");

    let WidgetNode::Button(cut) = &items[0] else {
        panic!(
            "enabled item with a message must be a Button, got {:?}",
            items[0]
        );
    };
    assert_eq!(
        cut.activate(),
        Some(&1_u8),
        "Cut must carry the current message"
    );
    assert_eq!(cut.key.as_str(), "preview_cut");
    assert_eq!(cut.label, "Cut");

    let WidgetNode::Label(copy) = &items[1] else {
        panic!(
            "item without a message must stay a passive Label, got {:?}",
            items[1]
        );
    };
    assert_eq!(copy.text, "Copy");

    match &items[2] {
        WidgetNode::Label(label) => assert_eq!(label.text, "Lock"),
        WidgetNode::Button(button) => {
            assert!(button.disabled || button.activate().is_none());
            assert_eq!(button.label, "Lock");
        }
        other => panic!("disabled item must not activate, got {other:?}"),
    }

    assert!(
        matches!(&items[3], WidgetNode::Separator(_)),
        "separators must stay Separator nodes"
    );

    let WidgetNode::Button(empty) = &items[4] else {
        panic!("empty-key item with a message must be a Button");
    };
    assert_eq!(empty.activate(), Some(&8_u8));
    assert_eq!(empty.key.as_str(), "preview_item_4");
}

#[test]
fn context_menu_item_message_updates_when_changed() {
    for message in [1_u8, 2, 3] {
        let node = sample_open_menu(message, Some(4));
        let items = menu_items(&node);
        let WidgetNode::Button(cut) = &items[0] else {
            panic!("Cut must be a Button");
        };
        assert_eq!(
            cut.activate(),
            Some(&message),
            "Cut Button must carry the updated message"
        );
        assert_eq!(cut.key.as_str(), item_key("preview", "cut", 0));
        let WidgetNode::Button(copy) = &items[1] else {
            panic!("Copy with a message must be a Button");
        };
        assert_eq!(copy.activate(), Some(&4_u8));
        assert_eq!(copy.key.as_str(), "preview_copy");
    }
}

#[test]
fn context_menu_closed_still_renders_only_the_anchor() {
    let node: WidgetNode<u8> = context_menu("preview")
        .anchor(label("Right-click me"))
        .item(menu_item("cut", "Cut").on_click(1_u8))
        .open(false)
        .into();
    assert!(
        matches!(node, WidgetNode::Label(ref label) if label.text == "Right-click me"),
        "closed menus must still render just the anchor"
    );

    let empty: WidgetNode<u8> = context_menu("preview").open(false).into();
    assert!(matches!(empty, WidgetNode::Label(_)));
}

#[test]
fn context_menu_anchor_identity_and_focus_survive_open_item_changes() {
    let mut tree = RetainedTree::new();
    let mut focus = FocusManager::new();
    let mut anchor_id = None;

    let variants: [(_, _, bool, bool); 5] = [
        (1, None, false, true),
        (2, Some(4), false, true),
        (3, None, true, true),
        (4, Some(5), false, false),
        (5, None, false, true),
    ];

    for (cut, copy, lock_disabled, with_separator) in variants {
        let mut builder = context_menu("preview")
            .anchor(crate::button("anchor_action", "Anchor").on_click(99_u8))
            .item(menu_item("cut", "Cut").on_click(cut))
            .item({
                let mut item = menu_item("copy", "Copy");
                if let Some(message) = copy {
                    item = item.on_click(message);
                }
                item
            })
            .item(
                menu_item("lock", "Lock")
                    .disabled(lock_disabled)
                    .on_click(7_u8),
            );
        if with_separator {
            builder = builder.separator();
        }
        let roots = [WidgetNode::from(builder.open(true))];

        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        assert_eq!(
            binding.mismatches(),
            0,
            "retained binding must stay aligned"
        );

        let anchor = &roots[0].children()[0];
        let id = binding
            .id_for(anchor)
            .expect("anchor must have a retained id");
        assert!(
            tree.get(id).unwrap().focusable,
            "anchor Button must stay focusable"
        );

        focus.set_order(std::iter::once((id, true)));
        if let Some(saved) = anchor_id {
            assert_eq!(
                id, saved,
                "item changes must not remount the open-menu anchor"
            );
            assert_eq!(
                focus.focused(),
                Some(id),
                "anchor focus must survive item message/disabled/separator changes"
            );
        } else {
            assert!(focus.request_focus(id), "anchor Button must accept focus");
        }
        anchor_id = Some(id);

        let cut_button = &menu_items(&roots[0])[0];
        let WidgetNode::Button(button) = cut_button else {
            panic!("Cut must stay a Button");
        };
        assert_eq!(button.activate(), Some(&cut));
    }
}

#[test]
fn context_menu_actionable_items_have_finite_positive_non_overlapping_rects() {
    let roots = [sample_open_menu(1, Some(4))];
    let mut tree = RetainedTree::new();
    tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
    let binding = bind_retained(&roots, &tree);
    assert_eq!(binding.mismatches(), 0);
    let context = layout_context();

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

        let stack_rect = snapshot.get(layout.id).expect("stack layout rect");
        let card_layout = &layout.children[1];
        let mut seen: Vec<LayoutRect> = Vec::new();

        for (index, item) in menu_items(&roots[0]).iter().enumerate() {
            if !matches!(item, WidgetNode::Button(_)) {
                continue;
            }
            let item_layout = &card_layout.children[index];
            if bound {
                assert_eq!(Some(item_layout.id), binding.id_for(item));
            }
            let rect = snapshot
                .get(item_layout.id)
                .expect("actionable item must have a layout rect");
            assert!(
                [rect.x, rect.y, rect.width, rect.height]
                    .iter()
                    .all(|value| value.is_finite()),
                "item rect must be finite: {rect:?}"
            );
            assert!(
                rect.width > 0.0 && rect.height > 0.0,
                "item rect must be positive: {rect:?}"
            );
            // Stack may report a zero height in this layout path; only require
            // containment when the parent actually has a positive box.
            if stack_rect.width > 0.0 && stack_rect.height > 0.0 {
                assert!(
                    rect.x >= stack_rect.x
                        && rect.x + rect.width <= stack_rect.x + stack_rect.width
                        && rect.y >= stack_rect.y
                        && rect.y + rect.height <= stack_rect.y + stack_rect.height,
                    "item rect must stay inside the menu: item={rect:?} stack={stack_rect:?}"
                );
            }
            for previous in &seen {
                let overlap_x =
                    rect.x < previous.x + previous.width && previous.x < rect.x + rect.width;
                let overlap_y =
                    rect.y < previous.y + previous.height && previous.y < rect.y + rect.height;
                assert!(
                    !(overlap_x && overlap_y),
                    "actionable item rects must not overlap: {rect:?} vs {previous:?}"
                );
            }
            seen.push(*rect);
        }
        assert_eq!(
            seen.len(),
            3,
            "Cut, Copy and empty-key items must be laid out"
        );
    }
}
