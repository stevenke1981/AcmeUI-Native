//! ROADMAP 010: the accessibility walk mirrors layout IDs.
//!
//! Every test lays out a real tree with the crate's own layout path and checks
//! the walk against it, on the sequential and the retained ID paths.

use accesskit::{Role, TreeUpdate};
use acme_core::RetainedTree;
use acme_layout::{LayoutNode, LayoutSnapshot};
use acme_widgets::retained_binding::bind_retained;
use acme_widgets::view_bridge::widget_to_views;
use acme_widgets::{
    TableColumn, TableRow, WidgetNode, button, column, dialog, label, popover, table, tooltip,
    virtual_list,
};

use super::tests_support::*;
use crate::build_tree_update_with_binding;

#[test]
fn every_tree_item_row_header_cell_and_widget_reads_its_layout_rect_on_both_paths() {
    laid_out_both_paths(&tree_table_button(), |update, layout, snapshot| {
        let rect = |path: &[usize]| layout_bounds(snapshot, layout_at(layout, path));
        let children = children_of(update, root_of(update));
        let [tree_node, table_node, button_node] = children.as_slice() else {
            panic!("expected tree, table and button");
        };
        for (index, item) in children_of(update, tree_node).iter().enumerate() {
            assert_eq!(bounds_of(item), rect(&[0, index]), "tree item {index}");
        }
        let table_rows = children_of(update, table_node);
        for (r, row_node) in table_rows.iter().enumerate() {
            assert_eq!(bounds_of(row_node), rect(&[1, r]), "row {r}");
            for (c, cell) in children_of(update, row_node).iter().enumerate() {
                assert_eq!(bounds_of(cell), rect(&[1, r, c]), "cell {r},{c}");
            }
        }
        assert_eq!(
            bounds_of(button_node),
            rect(&[2]),
            "button after tree and table"
        );
    });
}

#[test]
fn unbound_widget_after_a_tree_and_a_table_reads_its_own_layout_rect() {
    // Bind one description, then lay out and walk a copy whose widgets the
    // binding does not know. Every widget is then unbound.
    let original = vec![tree_table_button()];
    let mut retained = RetainedTree::new();
    retained
        .reconcile_roots(&widget_to_views(&original))
        .expect("reconciliation should succeed");
    let binding = bind_retained(&original, &retained);

    let copy = tree_table_button();
    let layout = copy.to_layout_with_binding(&binding, &layout_context());
    let snapshot = compute_layout(&layout);
    let update = build_tree_update_with_binding(&copy, &snapshot, &binding, None, None);

    let children = children_of(&update, root_of(&update));
    assert_eq!(
        bounds_of(children[2]),
        layout_bounds(&snapshot, &layout.children[2]),
        "unbound button after a tree and a table"
    );
    let table_rows = children_of(&update, children[1]);
    assert_eq!(
        bounds_of(children_of(&update, table_rows[1])[0]),
        layout_bounds(&snapshot, &layout.children[1].children[1].children[0]),
        "unbound cell after a tree and a table"
    );
}

#[test]
fn retained_cell_widget_keeps_its_retained_id_and_reads_its_cell_leaf_rect() {
    let roots = vec![
        table::<Msg>("actions")
            .column(TableColumn::new("act", label::<Msg>("Act"), 80.0))
            .add_row(TableRow::new(vec![
                button::<Msg>("delete", "Delete").on_click(Msg),
            ]))
            .build(),
    ];
    let mut retained = RetainedTree::new();
    retained
        .reconcile_roots(&widget_to_views(&roots))
        .expect("reconciliation should succeed");
    let binding = bind_retained(&roots, &retained);
    let layout = roots[0].to_layout_with_binding(&binding, &layout_context());
    let snapshot = compute_layout(&layout);
    let update = build_tree_update_with_binding(&roots[0], &snapshot, &binding, None, None);

    let button_id = binding
        .id_for(&roots[0].children()[0])
        .expect("the cell button should be bound");
    let delete = node_by_id(&update, crate::to_access_id(button_id));
    assert_eq!(delete.role(), Role::Button);
    // Header is child 0, the data row is child 1, the cell leaf is its child 0.
    assert_eq!(
        bounds_of(delete),
        layout_bounds(&snapshot, &layout.children[1].children[0])
    );
}

#[test]
fn ragged_rows_group_by_their_actual_cells_on_both_paths() {
    let ragged = table::<Msg>("ragged")
        .column(TableColumn::new("a", label::<Msg>("A"), 80.0))
        .column(TableColumn::new("b", label::<Msg>("B"), 80.0))
        .add_row(TableRow::new(vec![label::<Msg>("a1")]))
        .add_row(TableRow::new(vec![
            label::<Msg>("b1"),
            label::<Msg>("b2"),
            label::<Msg>("b3"),
        ]))
        .build();
    laid_out_both_paths(&ragged, |update, layout, snapshot| {
        let table_node = root_of(update);
        assert_eq!(table_node.row_count(), Some(3));
        assert_eq!(
            table_node.column_count(),
            Some(3),
            "the widest row has three cells"
        );
        let column_count = table_node
            .column_count()
            .expect("a grouped table states its columns");
        for row_node in children_of(update, table_node) {
            for cell in children_of(update, row_node) {
                assert!(
                    cell.column_index()
                        .is_some_and(|column| column < column_count)
                );
            }
        }
        let rows = children_of(update, table_node);
        let sizes: Vec<usize> = rows.iter().map(|row| row.children().len()).collect();
        assert_eq!(sizes, [2, 1, 3], "header, then the rows as they are");
        for (r, row_node) in rows.iter().enumerate() {
            assert_eq!(
                bounds_of(row_node),
                layout_bounds(snapshot, &layout.children[r])
            );
            for (c, cell) in children_of(update, row_node).iter().enumerate() {
                assert_eq!(
                    bounds_of(cell),
                    layout_bounds(snapshot, &layout.children[r].children[c])
                );
            }
        }
        assert_eq!(
            named(&children_of(update, rows[2])),
            [
                Some("b1".to_string()),
                Some("b2".to_string()),
                Some("b3".to_string())
            ]
        );
    });
}

// ------------------------------------------------------------------
// Tail widgets after containers that layout does not fully number.
// ------------------------------------------------------------------

/// Lays out what `make` builds on three ID paths and runs `check` on each: the
/// sequential path, the bound path (the binding of the same instance), and the
/// unbound path (the binding of another instance, so that no widget of the
/// laid-out tree has a retained ID).
fn laid_out_all_paths(
    make: impl Fn() -> WidgetNode<Msg>,
    check: impl Fn(&TreeUpdate, &LayoutNode, &LayoutSnapshot),
) {
    laid_out_both_paths(&make(), &check);

    let original = vec![make()];
    let mut retained = RetainedTree::new();
    retained
        .reconcile_roots(&widget_to_views(&original))
        .expect("reconciliation should succeed");
    let binding = bind_retained(&original, &retained);
    let copy = make();
    let layout = copy.to_layout_with_binding(&binding, &layout_context());
    let snapshot = compute_layout(&layout);
    let update = build_tree_update_with_binding(&copy, &snapshot, &binding, None, None);
    check(&update, &layout, &snapshot);
}

/// The layout node of the last child of the root.
fn last_child(layout: &LayoutNode) -> &LayoutNode {
    layout.children.last().expect("the root has a tail child")
}

fn tail_button() -> WidgetNode<Msg> {
    button::<Msg>("tail", "Tail").on_click(Msg)
}

fn tail_after_dialog() -> WidgetNode<Msg> {
    column::<Msg>()
        .child(dialog::<Msg>("dialog", people_table()).build())
        .child(tail_button())
        .build()
}

fn tail_after_virtual_list() -> WidgetNode<Msg> {
    column::<Msg>()
        .child(
            virtual_list::<Msg>("list")
                .child(button::<Msg>("inside", "Inside").on_click(Msg))
                .child(files_tree(None, true))
                .build(),
        )
        .child(tail_button())
        .build()
}

fn tail_after_popover_with_extra_content() -> WidgetNode<Msg> {
    column::<Msg>()
        .child(
            popover::<Msg>("popover", button::<Msg>("anchor", "Anchor").on_click(Msg))
                .content(files_tree(None, true))
                .build(),
        )
        .child(tail_button())
        .build()
}

fn table_with_tooltip_cell_then_tail() -> WidgetNode<Msg> {
    column::<Msg>()
        .child(
            table::<Msg>("tips")
                .column(TableColumn::new("a", label::<Msg>("A"), 80.0))
                .add_row(TableRow::new(vec![
                    tooltip::<Msg>("tip", label::<Msg>("x"), "help").into(),
                ]))
                .build(),
        )
        .child(tail_button())
        .build()
}

fn headless_table_then_tail() -> WidgetNode<Msg> {
    column::<Msg>()
        .child(
            table::<Msg>("headless")
                .add_row(TableRow::new(vec![label::<Msg>("x"), label::<Msg>("y")]))
                .build(),
        )
        .child(tail_button())
        .build()
}

#[test]
fn tail_after_a_dialog_reads_its_own_layout_rect_on_all_paths() {
    laid_out_all_paths(tail_after_dialog, |update, layout, snapshot| {
        let children = children_of(update, root_of(update));
        assert_eq!(
            bounds_of(children[1]),
            layout_bounds(snapshot, last_child(layout)),
            "tail button after a dialog holding a table"
        );
        assert!(bounds_of(children[1]).is_some());
    });
}

#[test]
fn tail_after_a_virtual_list_reads_its_own_layout_rect_on_all_paths() {
    laid_out_all_paths(tail_after_virtual_list, |update, layout, snapshot| {
        let children = children_of(update, root_of(update));
        assert_eq!(
            bounds_of(children[1]),
            layout_bounds(snapshot, last_child(layout)),
            "tail button after a virtual list of a button and a tree"
        );
    });
}

#[test]
fn tail_after_a_popover_reads_its_own_layout_rect_on_all_paths() {
    laid_out_all_paths(
        tail_after_popover_with_extra_content,
        |update, layout, snapshot| {
            let children = children_of(update, root_of(update));
            assert_eq!(
                bounds_of(children[1]),
                layout_bounds(snapshot, last_child(layout)),
                "tail button after a popover with extra content"
            );
        },
    );
}

#[test]
fn tooltip_in_a_table_cell_reads_its_cell_leaf_on_all_paths() {
    laid_out_all_paths(
        table_with_tooltip_cell_then_tail,
        |update, layout, snapshot| {
            let children = children_of(update, root_of(update));
            assert_eq!(
                bounds_of(children[1]),
                layout_bounds(snapshot, last_child(layout)),
                "tail button after a table with a tooltip cell"
            );
            let data_row = children_of(update, children[0])[1];
            let cell = children_of(update, data_row)[0];
            assert_eq!(cell.role(), Role::Tooltip);
            assert_eq!(
                bounds_of(cell),
                layout_bounds(snapshot, &layout.children[0].children[1].children[0]),
                "the tooltip reads its cell leaf"
            );
        },
    );
}

#[test]
fn table_without_columns_reads_its_cell_leaves_and_tail_on_all_paths() {
    laid_out_all_paths(headless_table_then_tail, |update, layout, snapshot| {
        let children = children_of(update, root_of(update));
        assert_eq!(
            bounds_of(children[1]),
            layout_bounds(snapshot, last_child(layout)),
            "tail button after a table with no columns"
        );
        let cells = children_of(update, children[0]);
        assert_eq!(cells.len(), 2, "a headless table stays flat");
        for (c, cell) in cells.iter().enumerate() {
            assert_eq!(
                bounds_of(cell),
                layout_bounds(snapshot, &layout.children[0].children[0].children[c]),
                "cell {c} reads its leaf"
            );
        }
    });
}
