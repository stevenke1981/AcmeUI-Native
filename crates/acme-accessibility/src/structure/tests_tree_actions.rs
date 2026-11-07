//! ROADMAP 013: tree item actions, and the widgets inside a tree item.

use accesskit::{Action, Role, TreeUpdate};
use acme_core::{NodeId, RetainedTree, WidgetKey};
use acme_platform::{PlatformEvent, PlatformKey};
use acme_widgets::retained_binding::bind_retained;
use acme_widgets::view_bridge::widget_to_views;
use acme_widgets::{TreeNode, WidgetNode, button, column, label, row, tree};

use super::tests_support::*;
use crate::{AccessibilityAction, AccessibilityAdapter};

/// A tree whose rows hold widgets: `inbox` (expanded) holds a `row` of a
/// label and a button, with a child `draft` that holds a lone button; `trash`
/// holds a plain label. A button follows the tree in a column.
fn widget_rows() -> WidgetNode<Msg> {
    let inbox = TreeNode::new(
        "inbox",
        row::<Msg>()
            .child(label::<Msg>("Inbox"))
            .child(button::<Msg>("archive", "Archive").on_click(Msg)),
    )
    .child(TreeNode::new(
        "draft",
        button::<Msg>("send", "Send").on_click(Msg),
    ))
    .expanded(true);
    let mut mail = tree::<Msg>("mail")
        .child(inbox)
        .child(TreeNode::new("trash", label::<Msg>("Trash")));
    mail.selected = Some(WidgetKey::new("trash"));
    column::<Msg>()
        .child(mail.build())
        .child(button::<Msg>("tail", "Tail").on_click(Msg))
        .build()
}

fn items_of<'u>(update: &'u TreeUpdate, tree_node: &accesskit::Node) -> Vec<&'u accesskit::Node> {
    children_of(update, tree_node)
}

#[test]
fn every_tree_item_supports_focus_and_click() {
    both_paths(&files_tree(None, true), None, |update| {
        for item in items_of(update, root_of(update)) {
            assert!(item.supports_action(Action::Focus), "{:?}", item.label());
            assert!(item.supports_action(Action::Click), "{:?}", item.label());
        }
    });
}

#[test]
fn only_items_with_children_expose_expand_or_collapse() {
    both_paths(&files_tree(None, true), None, |update| {
        let items = items_of(update, root_of(update));
        let actions: Vec<(bool, bool)> = items
            .iter()
            .map(|i| {
                (
                    i.supports_action(Action::Expand),
                    i.supports_action(Action::Collapse),
                )
            })
            .collect();
        // src is expanded: it can collapse. The others have no children.
        assert_eq!(
            actions,
            [
                (false, true),
                (false, false),
                (false, false),
                (false, false)
            ]
        );
    });
    both_paths(&files_tree(None, false), None, |update| {
        let items = items_of(update, root_of(update));
        assert!(items[0].supports_action(Action::Expand));
        assert!(!items[0].supports_action(Action::Collapse));
        assert!(!items[1].supports_action(Action::Expand));
    });
}

#[test]
fn plain_text_tree_adds_only_the_actions() {
    both_paths(&files_tree(Some("lib"), true), None, |update| {
        // The tree, and one node per visible row: nothing for the text.
        assert_eq!(update.nodes.len(), 1 + 4);
        for item in items_of(update, root_of(update)) {
            assert!(item.children().is_empty(), "{:?}", item.label());
        }
    });
}

#[test]
fn widgets_in_item_content_become_children_of_the_item() {
    both_paths(&widget_rows(), None, |update| {
        let mail = children_of(update, root_of(update))[0];
        let items = items_of(update, mail);
        assert_eq!(items.len(), 3);

        // `inbox`: a row of a label and a button. The name rule is unchanged.
        assert_eq!(items[0].label(), Some("Inbox"));
        let content = children_of(update, items[0]);
        assert_eq!(content.len(), 1);
        assert_eq!(content[0].role(), Role::Group);
        let inner = children_of(update, content[0]);
        assert_eq!(inner[0].role(), Role::Label);
        assert_eq!(inner[1].role(), Role::Button);
        assert_eq!(inner[1].label(), Some("Archive"));
        assert!(inner[1].supports_action(Action::Click));
        assert!(inner[1].supports_action(Action::Focus));

        // `draft`: no label, so the button names the item.
        assert_eq!(items[1].label(), Some("Send"));
        let draft = children_of(update, items[1]);
        assert_eq!(draft[0].role(), Role::Button);
        assert_eq!(draft[0].label(), Some("Send"));

        // `trash`: plain text, no children.
        assert!(items[2].children().is_empty());
    });
}

#[test]
fn content_root_reads_its_row_rect_and_ids_are_unique() {
    laid_out_both_paths(&widget_rows(), |update, layout, snapshot| {
        let mail = children_of(update, root_of(update))[0];
        let items = items_of(update, mail);
        let row_rect = |index: usize| layout_bounds(snapshot, layout_at(layout, &[0, index]));
        for (index, item) in items.iter().enumerate() {
            assert_eq!(bounds_of(item), row_rect(index), "item {index}");
        }
        let content = children_of(update, items[1])[0];
        assert_eq!(bounds_of(content), row_rect(1), "content root");

        let mut ids: Vec<_> = update.nodes.iter().map(|(id, _)| *id).collect();
        let before = ids.len();
        ids.sort_by_key(|id| id.0);
        ids.dedup();
        assert_eq!(ids.len(), before, "node ids must be unique");
    });
}

#[test]
fn walking_item_content_keeps_the_layout_id_order() {
    // The tail button comes after the tree in layout. Walking the tree's item
    // content must not consume layout IDs, so the tail still reads its own
    // layout rectangle, and so does every item.
    laid_out_both_paths(&widget_rows(), |update, layout, snapshot| {
        let children = children_of(update, root_of(update));
        let mail = children[0];
        for (index, item) in items_of(update, mail).iter().enumerate() {
            assert_eq!(
                bounds_of(item),
                layout_bounds(snapshot, layout_at(layout, &[0, index])),
                "item {index}"
            );
        }
        assert_eq!(
            bounds_of(children[1]),
            layout_bounds(snapshot, layout_at(layout, &[1])),
            "button after the tree"
        );
        assert_eq!(children[1].label(), Some("Tail"));
    });
}

// ------------------------------------------------------------------
// Routing
// ------------------------------------------------------------------

/// Builds the adapter on the sequential path and on the retained path, and
/// runs `check` with each adapter and the access ID of the tree item at
/// visible index `index`.
fn adapters(index: usize, check: impl Fn(&AccessibilityAdapter, NodeId, (f32, f32))) {
    let root = files_tree(None, true);
    let layout = root.to_layout(NodeId::new(1));
    let snapshot = compute_layout(&layout);
    let mut adapter = AccessibilityAdapter::new(7);
    adapter.update(&root, &snapshot);
    let center = center_of(&snapshot, &layout, index);
    check(&adapter, item_id(&adapter, index), center);

    let mut retained = RetainedTree::new();
    let roots = std::slice::from_ref(&root);
    retained
        .reconcile_roots(&widget_to_views(roots))
        .expect("reconciliation should succeed");
    let binding = bind_retained(roots, &retained);
    let layout = root.to_layout_with_binding(&binding, &layout_context());
    let snapshot = compute_layout(&layout);
    let mut adapter = AccessibilityAdapter::new(7);
    adapter.update_with_binding(&root, &snapshot, &binding);
    let center = center_of(&snapshot, &layout, index);
    check(&adapter, item_id(&adapter, index), center);
}

fn center_of(
    snapshot: &acme_layout::LayoutSnapshot,
    layout: &acme_layout::LayoutNode,
    index: usize,
) -> (f32, f32) {
    let r = snapshot.get(layout.children[index].id).copied().unwrap();
    (r.x + r.width * 0.5, r.y + r.height * 0.5)
}

fn item_id(adapter: &AccessibilityAdapter, index: usize) -> NodeId {
    let tree = adapter.tree_ref().expect("tree");
    let tree_node = &tree
        .nodes
        .iter()
        .find(|(id, _)| *id == tree.root)
        .unwrap()
        .1;
    NodeId::new(tree_node.children()[index].0)
}

#[test]
fn click_on_a_tree_item_selects_its_row_on_both_paths() {
    adapters(2, |adapter, id, (cx, cy)| {
        let events = adapter.route_action(&AccessibilityAction::Click(id));
        assert!(
            matches!(
                events[0],
                PlatformEvent::FocusChanged { node_id, gained: true, .. } if node_id == id.get()
            ),
            "{events:?}"
        );
        match &events[1] {
            PlatformEvent::PointerButton { x, y, pressed, .. } => {
                assert!(*pressed);
                assert_eq!((*x, *y), (cx, cy), "press at the row's centre");
            }
            other => panic!("expected a pointer press, got {other:?}"),
        }
    });
}

#[test]
fn expand_and_collapse_select_the_row_then_press_the_arrow_key() {
    adapters(0, |adapter, id, (cx, cy)| {
        for (action, expected) in [
            (AccessibilityAction::Expand(id), PlatformKey::ArrowRight),
            (AccessibilityAction::Collapse(id), PlatformKey::ArrowLeft),
        ] {
            let events = adapter.route_action(&action);
            assert!(matches!(events[0], PlatformEvent::FocusChanged { .. }));
            assert!(
                matches!(events[1], PlatformEvent::PointerButton { x, y, pressed: true, .. } if (x, y) == (cx, cy)),
                "{events:?}"
            );
            let keys: Vec<_> = events
                .iter()
                .filter_map(|e| match e {
                    PlatformEvent::Key { key, pressed, .. } => Some((*key, *pressed)),
                    _ => None,
                })
                .collect();
            assert_eq!(keys, [(expected, true), (expected, false)]);
        }
    });
}
