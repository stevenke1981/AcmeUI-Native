//! AccessKit integration for AcmeUI Native.
//!
//! Provides functions to build [`TreeUpdate`] values from the widget tree
//! and layout snapshot used by the AcmeUI framework.

#![forbid(unsafe_op_in_unsafe_fn)]

use accesskit::{Action, Node as AccessNode, NodeId as AccessNodeId, Rect, Role, Tree, TreeUpdate};
use acme_core::NodeId;
use acme_layout::LayoutSnapshot;
use acme_widgets::WidgetNode;

/// Converts an AcmeUI [`NodeId`] to an AccessKit [`AccessNodeId`].
#[inline]
pub fn to_access_id(id: NodeId) -> AccessNodeId {
    AccessNodeId::from(id.get())
}

/// Build a full [`TreeUpdate`] from a widget tree and its layout snapshot.
///
/// Walks the widget tree in DFS order, assigning sequential IDs that match
/// those produced by [`WidgetNode::to_layout`]. Each widget variant is mapped
/// to the AccessKit role most appropriate for assistive technology consumption:
///
/// | Widget variant          | AccessKit role    |
/// |-------------------------|-------------------|
/// | `Row` / `Column` / `Stack` / `Card` | `Group`       |
/// | `Label`                 | `Label`           |
/// | `Button`                | `Button`          |
/// | `Separator`             | `Splitter`        |
/// | `ScrollView`            | `ScrollView`      |
///
/// # Parameters
///
/// * `root` – The root of the widget tree.
/// * `snapshot` – A layout snapshot whose keys match the walk order of `root`.
/// * `focus_id` – The ID of the node that currently holds keyboard focus, if any.
/// * `label_override` – If `Some`, replaces the display text of every `Label` node
///   in the tree with this string.
pub fn build_tree_update<M>(
    root: &WidgetNode<M>,
    snapshot: &LayoutSnapshot,
    focus_id: Option<NodeId>,
    label_override: Option<&str>,
) -> TreeUpdate
where
    M: Clone + 'static,
{
    let mut next_id: u64 = 1;
    let mut nodes: Vec<(AccessNodeId, AccessNode)> = Vec::new();
    let root_id = walk_node(root, snapshot, &mut next_id, label_override, &mut nodes);

    let focus = match focus_id {
        Some(id) => AccessNodeId::from(id.get()),
        None => root_id,
    };

    TreeUpdate {
        nodes,
        tree: Some(Tree::new(root_id)),
        focus,
    }
}

/// Recursively walk the widget tree, producing AccessKit nodes.
fn walk_node<M: Clone + 'static>(
    node: &WidgetNode<M>,
    snapshot: &LayoutSnapshot,
    next_id: &mut u64,
    label_override: Option<&str>,
    nodes: &mut Vec<(AccessNodeId, AccessNode)>,
) -> AccessNodeId {
    let id = AccessNodeId::from(*next_id);
    let layout_id = *next_id;
    *next_id += 1;

    let role = match node {
        WidgetNode::Row(_) | WidgetNode::Column(_) | WidgetNode::Stack(_) | WidgetNode::Card(_) => {
            Role::Group
        }
        WidgetNode::Label(_) => Role::Label,
        WidgetNode::Button(_) => Role::Button,
        WidgetNode::Separator(_) => Role::Splitter,
        WidgetNode::ScrollView(_) => Role::ScrollView,
    };

    let mut access_node = AccessNode::new(role);

    // Map the layout bounding box into an AccessKit rectangle.
    if let Some(rect) = snapshot.get(layout_id) {
        access_node.set_bounds(Rect::new(
            rect.x as f64,
            rect.y as f64,
            (rect.x + rect.width) as f64,
            (rect.y + rect.height) as f64,
        ));
    }

    // Determine the accessible name.
    let label = match node {
        WidgetNode::Label(l) => label_override.unwrap_or(&l.text).to_string(),
        WidgetNode::Button(b) => b.label.clone(),
        _ => String::new(),
    };
    if !label.is_empty() {
        access_node.set_label(label);
    }

    // Interactive state.
    if let WidgetNode::Button(b) = node {
        if b.disabled {
            access_node.set_disabled();
        }
        access_node.add_action(Action::Focus);
        access_node.add_action(Action::Click);
    }

    // Process children.
    let child_ids: Vec<AccessNodeId> = node
        .children()
        .iter()
        .map(|child| walk_node(child, snapshot, next_id, label_override, nodes))
        .collect();
    if !child_ids.is_empty() {
        access_node.set_children(child_ids);
    }

    nodes.push((id, access_node));
    id
}

/// Create a minimal initial [`TreeUpdate`] with a single root window node.
///
/// The root node is sized to the window default (1080×720). Applications
/// should call [`build_tree_update`] with the real widget tree as soon as
/// it is available to replace this placeholder.
pub fn initial_tree(snapshot: &LayoutSnapshot) -> TreeUpdate {
    let root_id = AccessNodeId::from(1u64);

    // Use a default window size; we cannot iterate the private rect map.
    let _ = snapshot; // snapshot size is logged or used by the caller.
    let bounds = Rect::new(0.0, 0.0, 1080.0, 720.0);

    let mut node = AccessNode::new(Role::Window);
    node.set_bounds(bounds);

    TreeUpdate {
        nodes: vec![(root_id, node)],
        tree: Some(Tree::new(root_id)),
        focus: root_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::{NodeId, RetainedTree, ViewNode};
    use acme_layout::LayoutEngine;
    use acme_widgets::{WidgetNode, button, column, label, row, scroll_view, separator, stack};

    #[derive(Clone, Debug, PartialEq)]
    struct TestMsg;

    /// Create a `NodeId` with a specific `u64` value by reconciling that many roots.
    fn make_node_id(n: u64) -> NodeId {
        let mut tree = RetainedTree::new();
        let keys: Vec<&str> = (0..n)
            .map(|i| match i {
                0 => "_a",
                1 => "_b",
                2 => "_c",
                3 => "_d",
                _ => "_x",
            })
            .collect();
        let views: Vec<ViewNode> = keys.iter().map(|k| ViewNode::new(*k, "_")).collect();
        tree.reconcile_roots(&views)
            .expect("reconciliation should succeed");
        let roots = tree.roots();
        roots[roots.len() - 1]
    }

    // ------------------------------------------------------------------
    // Role mapping tests — no layout needed.
    // ------------------------------------------------------------------

    #[test]
    fn label_uses_label_role() {
        let tree = column::<TestMsg>().child(label("Hello")).build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&tree, &snapshot, None, None);

        let count = update
            .nodes
            .iter()
            .filter(|(_, n)| n.role() == Role::Label)
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn label_node_carries_text() {
        let tree = column::<TestMsg>().child(label("Hello World")).build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&tree, &snapshot, None, None);

        let label_node = update
            .nodes
            .iter()
            .find(|(_, n)| n.role() == Role::Label)
            .expect("expected a StaticText node");
        assert_eq!(label_node.1.label(), Some("Hello World"));
    }

    #[test]
    fn button_uses_button_role() {
        let tree = column::<TestMsg>()
            .child(button("ok", "OK").on_click(TestMsg {}))
            .build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&tree, &snapshot, None, None);

        let count = update
            .nodes
            .iter()
            .filter(|(_, n)| n.role() == Role::Button)
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn button_node_carries_label() {
        let tree = column::<TestMsg>()
            .child(button("cancel", "Cancel").on_click(TestMsg {}))
            .build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&tree, &snapshot, None, None);

        let btn_node = update
            .nodes
            .iter()
            .find(|(_, n)| n.role() == Role::Button)
            .expect("expected a Button node");
        assert_eq!(btn_node.1.label(), Some("Cancel"));
    }

    #[test]
    fn button_supports_focus_and_click_actions() {
        let tree = column::<TestMsg>()
            .child(button("go", "Go").on_click(TestMsg {}))
            .build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&tree, &snapshot, None, None);

        let btn_node = update
            .nodes
            .iter()
            .find(|(_, n)| n.role() == Role::Button)
            .expect("expected a Button node");
        assert!(btn_node.1.supports_action(Action::Focus));
        assert!(btn_node.1.supports_action(Action::Click));
    }

    #[test]
    fn disabled_button_has_disabled_flag() {
        let tree = column::<TestMsg>()
            .child(button("save", "Save").disabled(true).on_click(TestMsg {}))
            .build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&tree, &snapshot, None, None);

        let btn_node = update
            .nodes
            .iter()
            .find(|(_, n)| n.role() == Role::Button)
            .expect("expected a Button node");
        assert!(btn_node.1.is_disabled());
    }

    #[test]
    fn separator_uses_splitter_role() {
        let tree = column::<TestMsg>().child(separator::<TestMsg>()).build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&tree, &snapshot, None, None);

        let count = update
            .nodes
            .iter()
            .filter(|(_, n)| n.role() == Role::Splitter)
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn scroll_view_uses_scroll_view_role() {
        let tree = column::<TestMsg>()
            .child(scroll_view::<TestMsg>("sv").child(label("inside")).build())
            .build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&tree, &snapshot, None, None);

        let count = update
            .nodes
            .iter()
            .filter(|(_, n)| n.role() == Role::ScrollView)
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn containers_use_group_role() {
        let tree = column::<TestMsg>()
            .child(row::<TestMsg>().child(label("A")).build())
            .child(stack::<TestMsg>().child(label("B")).build())
            .build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&tree, &snapshot, None, None);

        // Column + Row + Stack = 3 Group nodes
        let count = update
            .nodes
            .iter()
            .filter(|(_, n)| n.role() == Role::Group)
            .count();
        assert_eq!(count, 3);
    }

    // ------------------------------------------------------------------
    // label_override
    // ------------------------------------------------------------------

    #[test]
    fn label_override_replaces_label_text() {
        let tree = column::<TestMsg>().child(label("Original")).build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&tree, &snapshot, None, Some("Overridden"));

        let label_node = update
            .nodes
            .iter()
            .find(|(_, n)| n.role() == Role::Label)
            .expect("expected a StaticText node");
        assert_eq!(label_node.1.label(), Some("Overridden"));
    }

    // ------------------------------------------------------------------
    // Focus
    // ------------------------------------------------------------------

    #[test]
    fn focus_is_root_when_no_focus_id_provided() {
        let tree = column::<TestMsg>().child(label("Hi")).build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&tree, &snapshot, None, None);

        let root_id = AccessNodeId::from(1u64);
        assert_eq!(update.focus, root_id);
    }

    #[test]
    fn focus_is_mapped_from_node_id() {
        let tree = column::<TestMsg>().child(label("Hi")).build();
        let snapshot = LayoutSnapshot::default();
        // Node ID 2 corresponds to the label (second DFS walk).
        let focus_id = make_node_id(2);
        let update = build_tree_update(&tree, &snapshot, Some(focus_id), None);

        assert_eq!(update.focus, AccessNodeId::from(2u64));
    }

    // ------------------------------------------------------------------
    // Snapshot bounds mapping
    // ------------------------------------------------------------------

    fn compute_snapshot(
        tree: &WidgetNode<TestMsg>,
        start_id: u64,
        viewport: (f32, f32),
    ) -> LayoutSnapshot {
        let layout = tree.to_layout(&mut start_id.clone());
        LayoutEngine::new()
            .compute(&layout, viewport)
            .expect("layout computation should succeed")
    }

    #[test]
    fn bounds_are_mapped_from_snapshot() {
        let tree = column::<TestMsg>()
            .child(label("Hello"))
            .child(separator::<TestMsg>())
            .child(button("btn", "Click").on_click(TestMsg {}))
            .build();
        let snapshot = compute_snapshot(&tree, 1, (200.0, 300.0));
        let update = build_tree_update(&tree, &snapshot, None, None);

        assert!(
            !update.nodes.is_empty(),
            "there should be at least one node"
        );

        // Every node should have a valid (non-negative width/height) bounding box.
        for (id, node) in &update.nodes {
            let bounds = node
                .bounds()
                .unwrap_or_else(|| panic!("node {id:?} is missing bounds"));
            assert!(bounds.width() >= 0.0, "node {id:?} has negative width");
            assert!(bounds.height() >= 0.0, "node {id:?} has negative height");
        }
    }

    #[test]
    fn all_bounds_are_present_after_layout() {
        // Verifies that every widget node gets a `Some(…)` bounds from the
        // layout snapshot.  Width/height may be zero for auto-sized nodes
        // because Taffy has no intrinsic content measurement.
        let tree = column::<TestMsg>()
            .child(label("Hello"))
            .child(button("a", "A").on_click(TestMsg {}))
            .build();
        let snapshot = compute_snapshot(&tree, 1, (400.0, 100.0));
        let update = build_tree_update(&tree, &snapshot, None, None);

        for (id, node) in &update.nodes {
            let bounds = node
                .bounds()
                .unwrap_or_else(|| panic!("node {id:?} is missing bounds after layout"));
            // Non-negative guarantee (NaN would also fail here).
            assert!(bounds.width() >= 0.0, "node {id:?} has negative width");
            assert!(bounds.height() >= 0.0, "node {id:?} has negative height");
        }
    }

    // ------------------------------------------------------------------
    // Tree structure: parent-child relationships
    // ------------------------------------------------------------------

    #[test]
    fn container_has_correct_child_ids() {
        let tree = column::<TestMsg>()
            .child(label("A"))
            .child(label("B"))
            .child(label("C"))
            .build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&tree, &snapshot, None, None);

        // Root (column, ID 1) should have children [2, 3, 4].
        let root_node = update
            .nodes
            .iter()
            .find(|(id, _)| *id == AccessNodeId::from(1u64))
            .expect("expected root node");

        let children = root_node.1.children();
        assert_eq!(
            children,
            &[
                AccessNodeId::from(2u64),
                AccessNodeId::from(3u64),
                AccessNodeId::from(4u64),
            ]
        );
    }

    // ------------------------------------------------------------------
    // Initial tree
    // ------------------------------------------------------------------

    #[test]
    fn initial_tree_has_window_role() {
        let snapshot = LayoutSnapshot::default();
        let update = initial_tree(&snapshot);

        assert_eq!(update.nodes.len(), 1);
        assert_eq!(update.nodes[0].1.role(), Role::Window);
        assert_eq!(update.focus, AccessNodeId::from(1u64));
        assert!(update.tree.is_some());
    }

    #[test]
    fn initial_tree_uses_default_bounds() {
        let snapshot = LayoutSnapshot::default();
        let update = initial_tree(&snapshot);

        let bounds = update.nodes[0]
            .1
            .bounds()
            .expect("window should have bounds");
        assert_eq!(bounds, Rect::new(0.0, 0.0, 1080.0, 720.0));
    }
}
