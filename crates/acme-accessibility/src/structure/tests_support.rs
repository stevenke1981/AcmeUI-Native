//! Shared fixtures and helpers for the structure tests.
//!
//! Split out of `tests.rs` so that each test file stays under 800 lines.

use accesskit::{Node as AccessNode, NodeId as AccessNodeId, Rect, Role, TreeUpdate};
use acme_core::{NodeId, RetainedTree, WidgetKey};
use acme_layout::{LayoutEngine, LayoutNode, LayoutRect, LayoutSnapshot, WidgetLayoutContext};
use acme_widgets::retained_binding::bind_retained;
use acme_widgets::view_bridge::widget_to_views;
use acme_widgets::{
    DataGridColumn, DataGridRow, TableColumn, TableRow, TreeNode, WidgetNode, button, column,
    datagrid, label, table, tree,
};

use crate::{build_tree_update, build_tree_update_with_binding};

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Msg;

// ------------------------------------------------------------------
// Fixtures and helpers
// ------------------------------------------------------------------

/// A tree rooted at `files`: `src` (with children `lib.rs` and `main.rs`)
/// and a leaf `docs`. `src` starts expanded or collapsed, and `selected`
/// names the key of the selected node.
pub(super) fn files_tree(selected: Option<&str>, src_expanded: bool) -> WidgetNode<Msg> {
    let src = TreeNode::new("src", label::<Msg>("src"))
        .child(TreeNode::new("lib", label::<Msg>("lib.rs")))
        .child(TreeNode::new("main", label::<Msg>("main.rs")))
        .expanded(src_expanded);
    let mut files = tree::<Msg>("files")
        .child(src)
        .child(TreeNode::new("docs", label::<Msg>("docs")));
    files.selected = selected.map(WidgetKey::new);
    files.build()
}

/// A table with two columns (`Name`, `Age`) and two rows.
pub(super) fn people_table() -> WidgetNode<Msg> {
    table::<Msg>("people")
        .column(TableColumn::new("name", label::<Msg>("Name"), 120.0))
        .column(TableColumn::new("age", label::<Msg>("Age"), 80.0))
        .add_row(TableRow::new(vec![label::<Msg>("Ada"), label::<Msg>("36")]))
        .add_row(TableRow::new(vec![label::<Msg>("Bob"), label::<Msg>("25")]))
        .build()
}

/// A grid with one column (`Name`) and one row.
pub(super) fn one_by_one_grid() -> WidgetNode<Msg> {
    datagrid::<Msg>("grid")
        .column(DataGridColumn::new("name", label::<Msg>("Name"), 120.0))
        .add_row(DataGridRow::new(vec![label::<Msg>("Ada")]))
        .build()
}

/// Builds `root` on both paths and runs `check` on each result.
///
/// The sequential path numbers the root `1`. The retained path binds a
/// fresh frame and places `root_rect` under the root's retained ID.
pub(super) fn both_paths(
    root: &WidgetNode<Msg>,
    root_rect: Option<LayoutRect>,
    check: impl Fn(&TreeUpdate),
) {
    let mut sequential_snapshot = LayoutSnapshot::default();
    if let Some(rect) = root_rect {
        sequential_snapshot.insert(NodeId::new(1), rect);
    }
    check(&build_tree_update(root, &sequential_snapshot, None, None));

    let mut retained = RetainedTree::new();
    let roots = std::slice::from_ref(root);
    retained
        .reconcile_roots(&widget_to_views(roots))
        .expect("reconciliation should succeed");
    let binding = bind_retained(roots, &retained);
    let mut bound_snapshot = LayoutSnapshot::default();
    if let (Some(rect), Some(id)) = (root_rect, binding.id_for(root)) {
        bound_snapshot.insert(id, rect);
    }
    check(&build_tree_update_with_binding(
        root,
        &bound_snapshot,
        &binding,
        None,
        None,
    ));
}

pub(super) fn rect(x: f32, y: f32, width: f32, height: f32) -> LayoutRect {
    LayoutRect {
        x,
        y,
        width,
        height,
    }
}

pub(super) fn node_by_id(update: &TreeUpdate, id: AccessNodeId) -> &AccessNode {
    update
        .nodes
        .iter()
        .find(|(node_id, _)| *node_id == id)
        .map(|(_, node)| node)
        .expect("child id should be in the update")
}

pub(super) fn root_of(update: &TreeUpdate) -> &AccessNode {
    let root = update
        .tree
        .as_ref()
        .expect("update should name a tree")
        .root;
    node_by_id(update, root)
}

pub(super) fn children_of<'u>(update: &'u TreeUpdate, node: &AccessNode) -> Vec<&'u AccessNode> {
    node.children()
        .iter()
        .map(|id| node_by_id(update, *id))
        .collect()
}

pub(super) fn named(nodes: &[&AccessNode]) -> Vec<Option<String>> {
    nodes
        .iter()
        .map(|node| node.label().map(str::to_string))
        .collect()
}

pub(super) fn bounds_of(node: &AccessNode) -> Option<(f64, f64, f64, f64)> {
    node.bounds()
        .map(|Rect { x0, y0, x1, y1 }| (x0, y0, x1, y1))
}

pub(super) fn nodes_with_role(update: &TreeUpdate, role: Role) -> Vec<&AccessNode> {
    update
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == role)
        .map(|(_, node)| node)
        .collect()
}

// ------------------------------------------------------------------
// Real layout: the crate's own layout path, on both ID paths.
// ------------------------------------------------------------------

pub(super) fn layout_context() -> WidgetLayoutContext {
    WidgetLayoutContext {
        body_font_size: 14.0,
        body_line_height: 21.0,
        label_font_size: 13.0,
        control_height: 36.0,
        scale_factor: 1.0,
    }
}

pub(super) fn compute_layout(layout: &LayoutNode) -> LayoutSnapshot {
    LayoutEngine::new()
        .compute(layout, (400.0, 300.0))
        .expect("layout should succeed")
}

/// Lays `root` out with the crate's layout path on both ID paths and runs
/// `check` with the update, the layout tree and its snapshot.
pub(super) fn laid_out_both_paths(
    root: &WidgetNode<Msg>,
    check: impl Fn(&TreeUpdate, &LayoutNode, &LayoutSnapshot),
) {
    let layout = root.to_layout(NodeId::new(1));
    let snapshot = compute_layout(&layout);
    check(
        &build_tree_update(root, &snapshot, None, None),
        &layout,
        &snapshot,
    );

    let mut retained = RetainedTree::new();
    let roots = std::slice::from_ref(root);
    retained
        .reconcile_roots(&widget_to_views(roots))
        .expect("reconciliation should succeed");
    let binding = bind_retained(roots, &retained);
    let layout = root.to_layout_with_binding(&binding, &layout_context());
    let snapshot = compute_layout(&layout);
    check(
        &build_tree_update_with_binding(root, &snapshot, &binding, None, None),
        &layout,
        &snapshot,
    );
}

/// The accessibility bounds that a layout node's rectangle maps to.
pub(super) fn layout_bounds(
    snapshot: &LayoutSnapshot,
    layout: &LayoutNode,
) -> Option<(f64, f64, f64, f64)> {
    snapshot.get(layout.id).map(|r| {
        (
            f64::from(r.x),
            f64::from(r.y),
            f64::from(r.x + r.width),
            f64::from(r.y + r.height),
        )
    })
}

/// A column holding a tree, a table and a button, in that order.
pub(super) fn tree_table_button() -> WidgetNode<Msg> {
    column::<Msg>()
        .child(files_tree(Some("lib"), true))
        .child(people_table())
        .child(button::<Msg>("save", "Save").on_click(Msg))
        .build()
}

/// The layout node at `path`, where each index selects a child.
pub(super) fn layout_at<'l>(layout: &'l LayoutNode, path: &[usize]) -> &'l LayoutNode {
    path.iter()
        .fold(layout, |node, &index| &node.children[index])
}
