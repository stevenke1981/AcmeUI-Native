use super::*;
use acme_core::RetainedTree;
use acme_layout::{LayoutEngine, WidgetLayoutContext};
use acme_widgets::retained_binding::{SYNTHETIC_ID_BASE, bind_retained};
use acme_widgets::view_bridge::widget_to_views;
use acme_widgets::{button, column, label, row, stack};

fn sample() -> WidgetNode<i32> {
    column()
        .key("root")
        .child(
            row()
                .key("open")
                .on_click(1)
                .child(column().child(label("Open · 開啟")).child(label("Details")))
                .child(button("child", "Separate child").on_click(2)),
        )
        .child(column().key("column").on_click(3).child(label("Column")))
        .child(stack().key("stack").on_click(4).child(label("Stack")))
        .child(row().key("passive").child(label("Read only")))
        .child(button("disabled", "Disabled").disabled(true).on_click(5))
        .build()
}

#[test]
fn active_containers_have_named_button_semantics_and_independent_child_actions() {
    let roots = [sample()];
    let root = &roots[0];
    let mut tree = RetainedTree::new();
    tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
    let binding = bind_retained(&roots, &tree);
    let layout = root.to_layout_with_binding(
        &binding,
        &WidgetLayoutContext {
            body_font_size: 14.0,
            body_line_height: 20.0,
            label_font_size: 13.0,
            control_height: 36.0,
            scale_factor: 1.0,
        },
    );
    let snapshot = LayoutEngine::new()
        .compute(&layout, (400.0, 600.0))
        .unwrap();
    let update = build_tree_update_with_binding(root, &snapshot, &binding, None, None);
    let access = |w| {
        update
            .nodes
            .iter()
            .find(|(id, _)| *id == to_access_id(binding.id_for(w).unwrap()))
            .unwrap()
    };
    for (widget, name) in
        root.children()
            .iter()
            .zip(["Open · 開啟 Details", "Column", "Stack", "", "Disabled"])
    {
        let (_, node) = access(widget);
        if name.is_empty() {
            assert_eq!(node.role(), Role::Group);
            assert!(!node.supports_action(Action::Focus) && !node.supports_action(Action::Click));
        } else {
            assert_eq!(node.role(), Role::Button);
            assert_eq!(node.label(), Some(name));
            assert!(node.supports_action(Action::Focus));
            assert_eq!(node.supports_action(Action::Click), name != "Disabled");
        }
    }
    let parent = &root.children()[0];
    let child = &parent.children()[1];
    assert_eq!(access(child).1.label(), Some("Separate child"));
    for (widget, message) in [
        (parent, 1),
        (child, 2),
        (&root.children()[1], 3),
        (&root.children()[2], 4),
    ] {
        let id = binding.id_for(widget).unwrap();
        let request = ActionRequest {
            action: Action::Click,
            target: to_access_id(id),
            data: None,
        };
        assert_eq!(
            AccessibilityAction::from_request(&request)
                .unwrap()
                .activation_message(root, &binding),
            Some(message)
        );
        assert_eq!(
            AccessibilityAction::Activate(id).activation_message(root, &binding),
            Some(message)
        );
        let rect = snapshot.get(id).unwrap();
        let b = access(widget).1.bounds().unwrap();
        assert_eq!(
            (b.x0, b.y0, b.x1, b.y1),
            (
                rect.x as f64,
                rect.y as f64,
                (rect.x + rect.width) as f64,
                (rect.y + rect.height) as f64
            )
        );
    }
    for widget in [&root.children()[3], &root.children()[4]] {
        let id = binding.id_for(widget).unwrap();
        assert_eq!(
            AccessibilityAction::Click(id).activation_message(root, &binding),
            None
        );
    }
    // Sequential/unbound consumers receive the same semantics.
    let update = build_tree_update(root, &Default::default(), None, None);
    assert!(
        update
            .nodes
            .iter()
            .any(|(_, n)| n.role() == Role::Button && n.label() == Some("Open · 開啟 Details"))
    );
}

#[test]
fn typed_activation_uses_current_handlers_and_ids_after_reorder_removal_and_passivation() {
    let mut tree = RetainedTree::new();
    let mut saved = None;
    for (reverse, handler, present) in [
        (false, Some(1), true),
        (true, Some(9), true),
        (false, None, true),
        (true, None, false),
    ] {
        let mut row_a = row().key("a").child(label("A"));
        if let Some(message) = handler {
            row_a = row_a.on_click(message);
        }
        let mut children = vec![row().key("b").on_click(2).build()];
        if present {
            children.push(row_a.build());
        }
        if reverse {
            children.reverse();
        }
        let roots = [WidgetNode::Column(acme_widgets::Container {
            key: Some("root".into()),
            children,
            ..Default::default()
        })];
        tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
        let binding = bind_retained(&roots, &tree);
        if let Some(a) = roots[0]
            .children()
            .iter()
            .find(|w| w.key().unwrap().as_str() == "a")
        {
            let id = binding.id_for(a).unwrap();
            if let Some(saved) = saved {
                assert_eq!(id, saved);
            }
            saved = Some(id);
        }
        assert_eq!(
            AccessibilityAction::Click(saved.unwrap()).activation_message(&roots[0], &binding),
            handler
        );
    }
}

#[test]
fn typed_activation_rejects_partial_foreign_missing_synthetic_and_nonactivation_targets() {
    let roots = [sample()];
    let mut tree = RetainedTree::new();
    tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
    let binding = bind_retained(&roots, &tree);
    let id = binding.id_for(&roots[0].children()[0]).unwrap();
    for action in [
        AccessibilityAction::Focus(id),
        AccessibilityAction::SetValue(id, "x".into()),
        AccessibilityAction::Click(NodeId::new(u64::MAX)),
        AccessibilityAction::Click(NodeId::new(SYNTHETIC_ID_BASE)),
    ] {
        assert_eq!(action.activation_message(&roots[0], &binding), None);
    }
    let foreign = sample();
    assert_eq!(
        AccessibilityAction::Click(id).activation_message(&foreign, &binding),
        None
    );
    let other_roots = [row().on_click(100).build()];
    let partial = bind_retained(&other_roots, &tree);
    assert_ne!(partial.mismatches(), 0);
    assert_eq!(
        AccessibilityAction::Click(tree.roots()[0]).activation_message(&other_roots[0], &partial),
        None
    );
}
