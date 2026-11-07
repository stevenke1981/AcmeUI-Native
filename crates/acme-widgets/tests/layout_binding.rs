//! Retained-binding layout tests (ROADMAP 003, item A).
//!
//! Public API only: `WidgetNode::to_layout_with_binding` must produce the same
//! layout as `to_layout_with_context` except for node IDs, keep keyed widgets on
//! their retained IDs across reconciles, and never hand out colliding IDs.

use std::collections::HashSet;

use acme_core::{NodeId, RetainedTree};
use acme_layout::{LayoutNode, WidgetLayoutContext};
use acme_widgets::retained_binding::{SYNTHETIC_ID_BASE, bind_retained, is_synthetic};
use acme_widgets::view_bridge::widget_to_views;
use acme_widgets::{
    DataGridColumn, DataGridRow, TableColumn, TableRow, TreeNode, WidgetNode, breadcrumb, button,
    card, column, dialog, label, menu, menu_item, nav_item, nav_rail, popover, row, scroll_view,
    separator, sidebar, stack, tab_bar, table, text_input, tooltip, tree, virtual_list,
};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Save,
    Pick(usize),
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

/// Reconcile `roots` into a fresh retained tree, as the Gallery frame does.
fn reconciled(roots: &[WidgetNode<Msg>]) -> RetainedTree {
    let mut tree = RetainedTree::new();
    tree.reconcile_roots(&widget_to_views(roots))
        .expect("reconcile must succeed for a valid description");
    tree
}

/// Every ID the retained tree holds, reachable from its roots.
fn retained_ids(tree: &RetainedTree) -> HashSet<NodeId> {
    let mut out = HashSet::new();
    let mut stack: Vec<NodeId> = tree.roots().to_vec();
    while let Some(id) = stack.pop() {
        if !out.insert(id) {
            continue;
        }
        if let Some(node) = tree.get(id) {
            stack.extend(node.children.iter().copied());
        }
    }
    out
}

fn collect_ids(node: &LayoutNode, out: &mut Vec<NodeId>) {
    out.push(node.id);
    for child in &node.children {
        collect_ids(child, out);
    }
}

fn all_layout_ids(node: &LayoutNode) -> Vec<NodeId> {
    let mut out = Vec::new();
    collect_ids(node, &mut out);
    out
}

/// Structural equality of two layouts with node IDs ignored.
fn assert_same_shape_ignoring_ids(bound: &LayoutNode, sequential: &LayoutNode, path: &str) {
    assert_eq!(bound.style, sequential.style, "style differs at {path}");
    assert_eq!(
        bound.measure, sequential.measure,
        "measure differs at {path}"
    );
    assert_eq!(
        bound.children.len(),
        sequential.children.len(),
        "child count differs at {path}"
    );
    for (i, (b, s)) in bound.children.iter().zip(&sequential.children).enumerate() {
        assert_same_shape_ignoring_ids(b, s, &format!("{path}/{i}"));
    }
}

/// Every widget kind the layout pipeline knows, grouped by case name.
fn parity_cases() -> Vec<(&'static str, Vec<WidgetNode<Msg>>)> {
    let containers = column::<Msg>()
        .key("root")
        .gap(8.0)
        .padding(12.0)
        .child(label("Title"))
        .child(
            row::<Msg>()
                .key("actions")
                .gap(4.0)
                .child(button::<Msg>("save", "Save").primary().on_click(Msg::Save))
                .child(button::<Msg>("cancel", "Cancel").disabled(true))
                .build(),
        )
        .child(stack::<Msg>().child(label("layered")).build())
        .build();

    let inputs = column::<Msg>()
        .child(text_input::<Msg>("name").label("Name").value("Ada").build())
        .child(separator::<Msg>())
        .child(button::<Msg>("submit", "Submit").full_width(true))
        .build();

    let scroll_and_card = column::<Msg>()
        .child(
            scroll_view::<Msg>("scroll")
                .viewport_height(120.0)
                .child(label("inner"))
                .build(),
        )
        .child(card::<Msg>().key("card").child(label("card body")).build())
        .build();

    let overlays = column::<Msg>()
        .child(tooltip::<Msg>(
            "tt",
            button::<Msg>("tt_btn", "Hover"),
            "help text",
        ))
        .child(
            popover::<Msg>("pop", button::<Msg>("pop_anchor", "Anchor"))
                .content(label("Popover body"))
                .open(true)
                .build(),
        )
        .child(
            dialog::<Msg>("dlg", label("Dialog body"))
                .title("Confirm")
                .width(320.0)
                .height(180.0)
                .build(),
        )
        .child(
            menu::<Msg>("file_menu")
                .item(menu_item::<Msg>("m_open", "Open").on_click(Msg::Save))
                .separator()
                .item(menu_item::<Msg>("m_quit", "Quit").disabled(true))
                .open(true)
                .build(),
        )
        .build();

    let overlay_roots = vec![
        tooltip::<Msg>("root_tt", label("root tooltip child"), "help").into(),
        popover::<Msg>("root_pop", button::<Msg>("root_anchor", "Anchor")).into(),
    ];

    let tree_widget = tree::<Msg>("tree")
        .indent(16.0)
        .viewport_height(200.0)
        .child(
            TreeNode::new("n_root", label::<Msg>("Root"))
                .expanded(true)
                .child(TreeNode::new("n_a", label::<Msg>("Alpha")))
                .child(
                    TreeNode::new("n_b", button::<Msg>("n_b_btn", "Beta"))
                        .expanded(true)
                        .child(TreeNode::new("n_b1", label::<Msg>("Beta child"))),
                ),
        )
        .build();

    let table_widget = table::<Msg>("table")
        .column(TableColumn::new("c_name", label::<Msg>("Name"), 120.0))
        .column(TableColumn::new("c_qty", label::<Msg>("Qty"), 60.0))
        .add_row(TableRow::new(vec![
            label::<Msg>("Apple"),
            label::<Msg>("3"),
        ]))
        .add_row(TableRow::new(vec![
            button::<Msg>("row_open", "Open").into(),
            label::<Msg>("7"),
        ]))
        .build();

    let grid_widget = acme_widgets::datagrid::<Msg>("grid")
        .column(DataGridColumn::new("g_name", label::<Msg>("Name"), 140.0))
        .column(DataGridColumn::new("g_qty", label::<Msg>("Qty"), 80.0))
        .add_row(DataGridRow::new(vec![
            label::<Msg>("Apple"),
            label::<Msg>("3"),
        ]))
        .frozen_rows(1)
        .frozen_cols(1)
        .build();

    let mut list = virtual_list::<Msg>("list")
        .viewport_height(120.0)
        .item_height(Some(24.0));
    for i in 0..20 {
        list = list.child(label::<Msg>(format!("row {i}")));
    }
    let list_widget = list.build();

    let navigation = vec![
        nav_rail::<Msg>("rail")
            .item(nav_item::<Msg>("Home"))
            .item(nav_item::<Msg>("Settings"))
            .selected(0)
            .build(),
        sidebar::<Msg>("side")
            .width(220.0)
            .header("Menu")
            .child(label::<Msg>("sidebar body"))
            .build(),
        tab_bar::<Msg>("tabs")
            .tab("Overview")
            .tab("Logs")
            .selected(1)
            .build(),
        breadcrumb::<Msg>("crumbs")
            .segment("Home")
            .segment("Docs")
            .separator(">")
            .build(),
    ];

    let mut cases = vec![
        ("containers, labels and buttons", vec![containers]),
        ("inputs and separator", vec![inputs]),
        ("scroll view and card", vec![scroll_and_card]),
        ("tooltip, popover, dialog and menu", vec![overlays]),
        ("overlay roots", overlay_roots),
        ("tree", vec![tree_widget]),
        ("table", vec![table_widget]),
        ("data grid", vec![grid_widget]),
        ("virtual list", vec![list_widget]),
    ];
    for (i, widget) in navigation.into_iter().enumerate() {
        let name = ["nav rail", "sidebar", "tab bar", "breadcrumb"][i];
        cases.push((name, vec![widget]));
    }
    cases
}

// ---------------------------------------------------------------------------
// 1. Parity: the bound layout equals the sequential layout ignoring IDs.
// ---------------------------------------------------------------------------

#[test]
fn bound_layout_matches_sequential_layout_ignoring_ids() {
    let context = layout_context();
    for (name, roots) in parity_cases() {
        let tree = reconciled(&roots);
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0, "{name}: binding mismatch");
        assert!(!binding.is_empty(), "{name}: nothing was bound");

        for (i, root) in roots.iter().enumerate() {
            let bound = root.to_layout_with_binding(&binding, &context);
            let sequential = root.to_layout_with_context(NodeId::new(1), &context);
            assert_same_shape_ignoring_ids(&bound, &sequential, &format!("{name}[{i}]"));
        }
    }
}

#[test]
fn bound_layout_matches_sequential_layout_when_nothing_is_bound() {
    // Without a reconcile, every widget is unbound and takes a synthetic ID;
    // the shape must still match the sequential layout exactly.
    let context = layout_context();
    for (name, roots) in parity_cases() {
        let empty = RetainedTree::new();
        let binding = bind_retained(&roots, &empty);
        assert!(binding.is_empty(), "{name}: unexpected binding");
        for (i, root) in roots.iter().enumerate() {
            let bound = root.to_layout_with_binding(&binding, &context);
            let sequential = root.to_layout_with_context(NodeId::new(1), &context);
            assert_same_shape_ignoring_ids(&bound, &sequential, &format!("{name}[{i}]"));
        }
    }
}

// ---------------------------------------------------------------------------
// 2. Stable IDs across keyed reorder / insert / remove.
// ---------------------------------------------------------------------------

fn keyed_toolbar(keys: &[&'static str]) -> Vec<WidgetNode<Msg>> {
    let mut toolbar = row::<Msg>().key("toolbar").gap(4.0);
    for (i, key) in keys.iter().enumerate() {
        toolbar = toolbar.child(button::<Msg>(*key, *key).on_click(Msg::Pick(i)));
    }
    vec![toolbar.build()]
}

/// Reconcile one frame and return `(button key, layout node id)` in frame order.
fn frame_layout_ids(tree: &mut RetainedTree, roots: &[WidgetNode<Msg>]) -> Vec<(String, NodeId)> {
    tree.reconcile_roots(&widget_to_views(roots))
        .expect("reconcile");
    let binding = bind_retained(roots, tree);
    assert_eq!(binding.mismatches(), 0);
    let layout = roots[0].to_layout_with_binding(&binding, &layout_context());
    roots[0]
        .children()
        .iter()
        .zip(&layout.children)
        .map(|(widget, node)| {
            let key = widget.key().expect("keyed button").as_str().to_owned();
            (key, node.id)
        })
        .collect()
}

fn id_of(frame: &[(String, NodeId)], key: &str) -> NodeId {
    frame
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, id)| *id)
        .unwrap_or_else(|| panic!("key {key} missing from frame"))
}

fn keys_of(frame: &[(String, NodeId)]) -> Vec<&str> {
    frame.iter().map(|(k, _)| k.as_str()).collect()
}

#[test]
fn keyed_reorder_keeps_each_layout_id() {
    let mut tree = RetainedTree::new();
    let before = frame_layout_ids(&mut tree, &keyed_toolbar(&["a", "b", "c"]));
    let after = frame_layout_ids(&mut tree, &keyed_toolbar(&["c", "a", "b"]));

    assert_eq!(keys_of(&after), ["c", "a", "b"]);
    for key in ["a", "b", "c"] {
        assert_eq!(
            id_of(&before, key),
            id_of(&after, key),
            "id of {key} changed"
        );
    }
}

#[test]
fn keyed_removal_keeps_surviving_layout_ids() {
    let mut tree = RetainedTree::new();
    let before = frame_layout_ids(&mut tree, &keyed_toolbar(&["a", "b", "c"]));
    let after = frame_layout_ids(&mut tree, &keyed_toolbar(&["a", "c"]));

    assert_eq!(keys_of(&after), ["a", "c"]);
    assert_eq!(id_of(&before, "a"), id_of(&after, "a"));
    assert_eq!(id_of(&before, "c"), id_of(&after, "c"));
}

#[test]
fn keyed_insertion_before_existing_keys_keeps_their_layout_ids() {
    let mut tree = RetainedTree::new();
    let before = frame_layout_ids(&mut tree, &keyed_toolbar(&["a", "b"]));
    let after = frame_layout_ids(&mut tree, &keyed_toolbar(&["new", "a", "b"]));

    assert_eq!(keys_of(&after), ["new", "a", "b"]);
    assert_eq!(id_of(&before, "a"), id_of(&after, "a"));
    assert_eq!(id_of(&before, "b"), id_of(&after, "b"));
    assert!(!before.iter().any(|(_, id)| *id == id_of(&after, "new")));
}

#[test]
fn sequential_ids_shift_on_insertion_where_bound_ids_do_not() {
    // Motivation check: the sequential numbering moves a surviving widget's ID
    // when a sibling is inserted before it, which is exactly what the binding
    // is meant to prevent.
    let context = layout_context();
    let sequential_a = |keys: &[&'static str]| {
        let roots = keyed_toolbar(keys);
        let layout = roots[0].to_layout_with_context(NodeId::new(1), &context);
        let index = keys.iter().position(|k| *k == "a").expect("a present");
        layout.children[index].id
    };
    assert_ne!(
        sequential_a(&["a", "b"]),
        sequential_a(&["new", "a", "b"]),
        "sequential IDs are expected to shift on insertion"
    );
}

// ---------------------------------------------------------------------------
// 3. Uniqueness and provenance of IDs.
// ---------------------------------------------------------------------------

#[test]
fn bound_layout_ids_are_unique_and_non_synthetic_ids_are_retained_ids() {
    let context = layout_context();
    for (name, roots) in parity_cases() {
        let tree = reconciled(&roots);
        let binding = bind_retained(&roots, &tree);
        let retained = retained_ids(&tree);
        assert_eq!(
            binding.len(),
            retained.len(),
            "{name}: every retained node should be bound exactly once"
        );

        for (i, root) in roots.iter().enumerate() {
            let layout = root.to_layout_with_binding(&binding, &context);
            assert!(
                retained.contains(&layout.id),
                "{name}[{i}]: root layout ID is not a retained ID"
            );
            let ids = all_layout_ids(&layout);
            let unique: HashSet<NodeId> = ids.iter().copied().collect();
            assert_eq!(unique.len(), ids.len(), "{name}[{i}]: duplicate layout IDs");

            for id in &ids {
                assert_eq!(
                    is_synthetic(*id),
                    !retained.contains(id),
                    "{name}[{i}]: ID {} is neither a retained ID nor synthetic",
                    id.get()
                );
                if is_synthetic(*id) {
                    assert!(id.get() >= SYNTHETIC_ID_BASE);
                }
            }
        }
    }
}

#[test]
fn unbound_layout_uses_only_synthetic_unique_ids() {
    let context = layout_context();
    for (name, roots) in parity_cases() {
        let binding = bind_retained(&roots, &RetainedTree::new());
        for (i, root) in roots.iter().enumerate() {
            let layout = root.to_layout_with_binding(&binding, &context);
            let ids = all_layout_ids(&layout);
            assert!(
                ids.iter().all(|id| is_synthetic(*id)),
                "{name}[{i}]: unbound layout has a non-synthetic ID"
            );
            let unique: HashSet<NodeId> = ids.iter().copied().collect();
            assert_eq!(
                unique.len(),
                ids.len(),
                "{name}[{i}]: duplicate synthetic IDs"
            );
        }
    }
}

#[test]
fn layout_only_widgets_take_their_retained_ids() {
    // Every widget here yields exactly one layout node, so the non-synthetic
    // layout IDs are exactly the retained IDs.
    let roots = vec![
        column::<Msg>()
            .child(label("a"))
            .child(button::<Msg>("b", "B"))
            .child(
                row::<Msg>()
                    .key("r")
                    .child(label("c"))
                    .child(button::<Msg>("d", "D"))
                    .build(),
            )
            .child(
                scroll_view::<Msg>("s")
                    .viewport_height(40.0)
                    .child(label("e"))
                    .build(),
            )
            .build(),
    ];
    let tree = reconciled(&roots);
    let binding = bind_retained(&roots, &tree);
    let layout = roots[0].to_layout_with_binding(&binding, &layout_context());

    let non_synthetic: HashSet<NodeId> = all_layout_ids(&layout)
        .into_iter()
        .filter(|id| !is_synthetic(*id))
        .collect();
    assert_eq!(non_synthetic, retained_ids(&tree));
}

// ---------------------------------------------------------------------------
// 4. Tooltip and Popover are layout-transparent and carry the child's ID.
// ---------------------------------------------------------------------------

#[test]
fn tooltip_layout_node_carries_child_retained_id() {
    let roots = vec![
        column::<Msg>()
            .child(tooltip::<Msg>(
                "tt",
                button::<Msg>("ok", "OK").primary().on_click(Msg::Save),
                "help",
            ))
            .build(),
    ];
    let tree = reconciled(&roots);
    let binding = bind_retained(&roots, &tree);

    let WidgetNode::Column(column_node) = &roots[0] else {
        panic!("expected column root");
    };
    let tooltip_widget = &column_node.children[0];
    let WidgetNode::Tooltip(tooltip_node) = tooltip_widget else {
        panic!("expected tooltip child");
    };
    let child_id = binding
        .id_for(tooltip_node.child.as_ref())
        .expect("tooltip child is bound");
    let tooltip_id = binding.id_for(tooltip_widget).expect("tooltip is bound");
    assert_ne!(child_id, tooltip_id);

    let layout = roots[0].to_layout_with_binding(&binding, &layout_context());
    assert_eq!(layout.children.len(), 1);
    assert_eq!(layout.children[0].id, child_id);
    assert!(
        layout.children.iter().all(|node| node.id != tooltip_id),
        "the wrapper's own ID must not appear in the layout"
    );
}

#[test]
fn tooltip_at_root_layout_is_the_child_node() {
    let roots: Vec<WidgetNode<Msg>> = vec![tooltip::<Msg>("tt", label("hi"), "help").into()];
    let tree = reconciled(&roots);
    let binding = bind_retained(&roots, &tree);

    let WidgetNode::Tooltip(tooltip_node) = &roots[0] else {
        panic!("expected tooltip root");
    };
    let child_id = binding
        .id_for(tooltip_node.child.as_ref())
        .expect("child bound");
    let layout = roots[0].to_layout_with_binding(&binding, &layout_context());
    assert_eq!(layout.id, child_id);
    assert!(layout.children.is_empty());
}

#[test]
fn popover_layout_node_carries_anchor_retained_id() {
    let roots = vec![
        popover::<Msg>("pop", button::<Msg>("anchor", "Anchor"))
            .content(label("Body"))
            .open(true)
            .build(),
    ];
    let tree = reconciled(&roots);
    let binding = bind_retained(&roots, &tree);

    let WidgetNode::Popover(popover_node) = &roots[0] else {
        panic!("expected popover root");
    };
    let anchor_id = binding
        .id_for(&popover_node.children[0])
        .expect("anchor bound");
    let layout = roots[0].to_layout_with_binding(&binding, &layout_context());
    assert_eq!(layout.id, anchor_id);
}
