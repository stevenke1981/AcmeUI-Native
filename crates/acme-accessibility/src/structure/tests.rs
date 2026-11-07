use std::collections::HashSet;

use accesskit::{Action, Node as AccessNode, NodeId as AccessNodeId, Role, TreeUpdate};
use acme_core::{NodeId, RetainedTree};
use acme_layout::LayoutSnapshot;
use acme_widgets::retained_binding::bind_retained;
use acme_widgets::view_bridge::widget_to_views;
use acme_widgets::{
    DataGridColumn, DataGridRow, TableColumn, TableRow, TableSelectionMode, TreeNode, WidgetNode,
    button, column, datagrid, label, row, table, tree,
};

use super::VIRTUAL_ID_BASE;
use super::tests_support::*;
use crate::{build_tree_update, build_tree_update_with_binding};

// ------------------------------------------------------------------
// Tree items
// ------------------------------------------------------------------

#[test]
fn tree_has_one_tree_item_per_visible_row_in_visible_order() {
    both_paths(&files_tree(None, true), None, |update| {
        let tree_node = root_of(update);
        assert_eq!(tree_node.role(), Role::Tree);

        let items = children_of(update, tree_node);
        let roles: Vec<Role> = items.iter().map(|item| item.role()).collect();
        assert_eq!(roles, [Role::TreeItem; 4]);
        assert_eq!(
            named(&items),
            [
                Some("src".to_string()),
                Some("lib.rs".to_string()),
                Some("main.rs".to_string()),
                Some("docs".to_string()),
            ]
        );
        let levels: Vec<Option<usize>> = items.iter().map(|item| item.level()).collect();
        assert_eq!(levels, [Some(1), Some(2), Some(2), Some(1)]);
    });
}

#[test]
fn tree_items_report_expansion_only_when_they_have_children() {
    both_paths(&files_tree(None, true), None, |update| {
        let items = children_of(update, root_of(update));
        let expanded: Vec<Option<bool>> = items.iter().map(|item| item.is_expanded()).collect();
        assert_eq!(expanded, [Some(true), None, None, None]);
    });
    both_paths(&files_tree(None, false), None, |update| {
        let items = children_of(update, root_of(update));
        let expanded: Vec<Option<bool>> = items.iter().map(|item| item.is_expanded()).collect();
        assert_eq!(expanded, [Some(false), None]);
    });
}

#[test]
fn collapsed_branch_children_are_absent() {
    both_paths(&files_tree(None, false), None, |update| {
        assert_eq!(nodes_with_role(update, Role::TreeItem).len(), 2);
        assert!(
            update
                .nodes
                .iter()
                .all(|(_, node)| node.label() != Some("lib.rs")),
            "children of a collapsed item must not be in the tree"
        );
    });
}

#[test]
fn tree_items_mark_only_the_selected_key_as_selected() {
    both_paths(&files_tree(Some("lib"), true), None, |update| {
        let items = children_of(update, root_of(update));
        let selected: Vec<Option<bool>> = items.iter().map(|item| item.is_selected()).collect();
        // Items are selectable, so the unselected ones state `false` explicitly.
        assert_eq!(
            selected,
            [Some(false), Some(true), Some(false), Some(false)]
        );
    });
}

#[test]
fn tree_item_bounds_follow_depth_indent_and_row_height() {
    let root_rect = rect(10.0, 20.0, 200.0, 120.0);
    both_paths(&files_tree(None, true), Some(root_rect), |update| {
        let items = children_of(update, root_of(update));
        let bounds: Vec<_> = items.iter().map(|item| bounds_of(item)).collect();
        assert_eq!(
            bounds,
            [
                Some((10.0, 20.0, 210.0, 44.0)),
                Some((30.0, 44.0, 210.0, 68.0)),
                Some((30.0, 68.0, 210.0, 92.0)),
                Some((10.0, 92.0, 210.0, 116.0)),
            ]
        );
    });
}

#[test]
fn tree_item_bounds_are_clipped_to_the_tree_rect() {
    // The tree ends at y = 60, so `lib.rs` is cut short and `main.rs`
    // and `docs` fall outside it with no height left.
    let root_rect = rect(10.0, 20.0, 200.0, 40.0);
    both_paths(&files_tree(None, true), Some(root_rect), |update| {
        let items = children_of(update, root_of(update));
        assert_eq!(bounds_of(items[0]), Some((10.0, 20.0, 210.0, 44.0)));
        assert_eq!(bounds_of(items[1]), Some((30.0, 44.0, 210.0, 60.0)));
        let main = bounds_of(items[2]).expect("clipped item keeps a rect");
        assert_eq!(main.1, main.3, "item outside the tree has no height");
        assert!(main.3 <= 60.0);
    });
}

#[test]
fn tree_items_have_no_bounds_without_a_tree_rect() {
    both_paths(&files_tree(None, true), None, |update| {
        for item in nodes_with_role(update, Role::TreeItem) {
            assert_eq!(bounds_of(item), None);
        }
    });
}

#[test]
fn tree_item_name_falls_back_to_first_label_inside_its_content() {
    let folder = row::<Msg>().child(label::<Msg>("Folder")).build();
    let blank = row::<Msg>().child(button::<Msg>("noop", "")).build();
    let root = tree::<Msg>("t")
        .child(TreeNode::new("f", folder))
        .child(TreeNode::new("b", blank))
        .build();
    both_paths(&root, None, |update| {
        let items = children_of(update, root_of(update));
        assert_eq!(named(&items), [Some("Folder".to_string()), None]);
    });
}

// ------------------------------------------------------------------
// Table and DataGrid rows
// ------------------------------------------------------------------

#[test]
fn table_groups_cells_into_rows_under_a_column_header_row() {
    both_paths(&people_table(), None, |update| {
        let table_node = root_of(update);
        assert_eq!(table_node.role(), Role::Table);
        assert_eq!(table_node.row_count(), Some(3), "header row counts");
        assert_eq!(table_node.column_count(), Some(2));

        let rows = children_of(update, table_node);
        assert_eq!(rows.len(), 3);
        assert!(rows.iter().all(|row| row.role() == Role::Row));
        let row_indices: Vec<Option<usize>> = rows.iter().map(|r| r.row_index()).collect();
        assert_eq!(row_indices, [Some(0), Some(1), Some(2)]);

        let header = children_of(update, rows[0]);
        assert_eq!(header.len(), 2);
        assert!(header.iter().all(|h| h.role() == Role::ColumnHeader));
        assert_eq!(
            named(&header),
            [Some("Name".to_string()), Some("Age".to_string())]
        );
        let columns: Vec<Option<usize>> = header.iter().map(|h| h.column_index()).collect();
        assert_eq!(columns, [Some(0), Some(1)]);

        for (row_node, expected) in rows[1..].iter().zip([["Ada", "36"], ["Bob", "25"]]) {
            let cells = children_of(update, row_node);
            assert!(cells.iter().all(|c| c.role() == Role::Cell));
            assert_eq!(named(&cells), expected.map(|s| Some(s.to_string())));
            let columns: Vec<Option<usize>> = cells.iter().map(|c| c.column_index()).collect();
            assert_eq!(columns, [Some(0), Some(1)]);
        }
        let data_rows: Vec<Option<usize>> = rows[1..].iter().map(|r| r.row_index()).collect();
        assert_eq!(data_rows, [Some(1), Some(2)]);
    });
}

#[test]
fn data_grid_groups_cells_into_rows_with_the_same_index_convention() {
    both_paths(&one_by_one_grid(), None, |update| {
        let grid = root_of(update);
        assert_eq!(grid.role(), Role::Grid);
        assert_eq!(grid.row_count(), Some(2));
        assert_eq!(grid.column_count(), Some(1));
        let rows = children_of(update, grid);
        assert_eq!(rows.len(), 2);
        let cells = children_of(update, rows[1]);
        assert_eq!(named(&cells), [Some("Ada".to_string())]);
        assert_eq!(cells[0].row_index(), Some(1));
        assert_eq!(cells[0].column_index(), Some(0));
    });
}

#[test]
fn table_selected_row_and_selected_cell_are_marked() {
    let mut people = match people_table() {
        WidgetNode::Table(table) => table,
        _ => panic!("expected a Table"),
    };
    people.selected_row = Some(1);
    people.selected_cell = Some((0, 1));
    let root = WidgetNode::Table(people);

    both_paths(&root, None, |update| {
        let rows = children_of(update, root_of(update));
        assert_eq!(rows[2].is_selected(), Some(true), "Bob's row is selected");
        assert_eq!(rows[1].is_selected(), Some(false), "Ada's row is not");
        let ada_cells = children_of(update, rows[1]);
        assert_eq!(ada_cells[1].label(), Some("36"));
        assert_eq!(ada_cells[1].is_selected(), Some(true));
        assert_ne!(ada_cells[0].is_selected(), Some(true));
    });
}

#[test]
fn data_grid_selected_cell_is_marked() {
    let mut grid = match one_by_one_grid() {
        WidgetNode::DataGrid(grid) => grid,
        _ => panic!("expected a DataGrid"),
    };
    grid.selected_cell = Some((0, 0));
    let root = WidgetNode::DataGrid(grid);

    both_paths(&root, None, |update| {
        let rows = children_of(update, root_of(update));
        let cells = children_of(update, rows[1]);
        assert_eq!(cells[0].is_selected(), Some(true));
    });
}

#[test]
fn control_in_a_cell_keeps_its_role_inside_its_row() {
    let actions = table::<Msg>("actions")
        .column(TableColumn::new("act", label::<Msg>("Act"), 80.0))
        .add_row(TableRow::new(vec![
            button::<Msg>("delete", "Delete").on_click(Msg),
        ]))
        .build();

    both_paths(&actions, None, |update| {
        let rows = children_of(update, root_of(update));
        let data_row = rows[1];
        let cells = children_of(update, data_row);
        assert_eq!(cells.len(), 1);
        assert_eq!(cells[0].role(), Role::Button);
        assert_eq!(cells[0].label(), Some("Delete"));
        assert!(cells[0].supports_action(Action::Click));
        assert!(
            update
                .nodes
                .iter()
                .all(|(_, node)| node.role() != Role::Cell),
            "a control is not wrapped in a Cell"
        );
    });
}

#[test]
fn rows_changed_after_build_fall_back_to_flat_cells() {
    // `children()` is fixed at build time. A row added afterwards is not in it,
    // so the flattened cells no longer match the rows and the grid stays flat.
    let mut people = match people_table() {
        WidgetNode::Table(table) => table,
        _ => panic!("expected a Table"),
    };
    people
        .rows
        .push(TableRow::new(vec![label::<Msg>("Cy"), label::<Msg>("9")]));
    let root = WidgetNode::Table(people);

    both_paths(&root, None, |update| {
        let table_node = root_of(update);
        assert_eq!(table_node.row_count(), None);
        assert_eq!(table_node.column_count(), None);
        assert!(nodes_with_role(update, Role::Row).is_empty());
        assert!(nodes_with_role(update, Role::ColumnHeader).is_empty());
        let cells = children_of(update, table_node);
        assert_eq!(cells.len(), 4);
        assert!(cells.iter().all(|c| c.role() == Role::Cell));
    });
}

#[test]
fn table_without_columns_falls_back_to_flat_cells() {
    let headless = table::<Msg>("headless")
        .add_row(TableRow::new(vec![label::<Msg>("x"), label::<Msg>("y")]))
        .build();

    both_paths(&headless, None, |update| {
        let table_node = root_of(update);
        assert_eq!(table_node.row_count(), None);
        assert!(nodes_with_role(update, Role::Row).is_empty());
        let cells = children_of(update, table_node);
        assert_eq!(cells.len(), 2);
        assert!(cells.iter().all(|c| c.role() == Role::Cell));
    });
}

#[test]
fn label_override_renames_tree_items_and_column_headers() {
    let snapshot = LayoutSnapshot::default();
    let update = build_tree_update(&files_tree(None, true), &snapshot, None, Some("X"));
    let items = children_of(&update, root_of(&update));
    assert!(items.iter().all(|item| item.label() == Some("X")));

    let update = build_tree_update(&people_table(), &snapshot, None, Some("X"));
    let headers = nodes_with_role(&update, Role::ColumnHeader);
    assert_eq!(headers.len(), 2);
    assert!(headers.iter().all(|header| header.label() == Some("X")));
}

#[test]
fn table_row_bounds_are_clipped_to_the_table_rect() {
    // The table is 24 high, so only the header row fits. The data rows collapse
    // onto its bottom edge, and the column headers keep their column widths.
    let mut snapshot = LayoutSnapshot::default();
    snapshot.insert(NodeId::new(1), rect(0.0, 0.0, 200.0, 24.0));

    let update = build_tree_update(&people_table(), &snapshot, None, None);
    let rows = children_of(&update, root_of(&update));
    assert_eq!(bounds_of(rows[0]), Some((0.0, 0.0, 200.0, 24.0)));
    assert_eq!(bounds_of(rows[1]), Some((0.0, 24.0, 200.0, 24.0)));
    assert_eq!(bounds_of(rows[2]), Some((0.0, 24.0, 200.0, 24.0)));
    let headers = children_of(&update, rows[0]);
    assert_eq!(bounds_of(headers[0]), Some((0.0, 0.0, 120.0, 24.0)));
    assert_eq!(bounds_of(headers[1]), Some((120.0, 0.0, 200.0, 24.0)));
}

#[test]
fn table_rows_and_headers_have_no_bounds_without_a_table_rect() {
    let update = build_tree_update(&people_table(), &LayoutSnapshot::default(), None, None);
    let rows_and_headers = nodes_with_role(&update, Role::Row)
        .into_iter()
        .chain(nodes_with_role(&update, Role::ColumnHeader));
    for node in rows_and_headers {
        assert_eq!(bounds_of(node), None);
    }
}

#[test]
fn table_rows_and_headers_have_the_bounds_of_the_layout_rows() {
    laid_out_both_paths(&people_table(), |update, layout, snapshot| {
        let rows = children_of(update, root_of(update));
        assert_eq!(
            rows.len(),
            layout.children.len(),
            "header row plus data rows"
        );
        for (row_node, layout_row) in rows.iter().zip(&layout.children) {
            assert!(layout_bounds(snapshot, layout_row).is_some());
            assert_eq!(bounds_of(row_node), layout_bounds(snapshot, layout_row));
        }
        let headers = children_of(update, rows[0]);
        for (header, layout_leaf) in headers.iter().zip(&layout.children[0].children) {
            assert_eq!(bounds_of(header), layout_bounds(snapshot, layout_leaf));
        }
    });
}

#[test]
fn retained_path_virtual_ids_are_not_layout_keys_and_tree_items_read_their_layout_rows() {
    let roots = vec![mixed_frame()];
    let mut retained = RetainedTree::new();
    retained
        .reconcile_roots(&widget_to_views(&roots))
        .expect("reconciliation should succeed");
    let binding = bind_retained(&roots, &retained);
    let layout = roots[0].to_layout_with_binding(&binding, &layout_context());
    let snapshot = compute_layout(&layout);
    let update = build_tree_update_with_binding(&roots[0], &snapshot, &binding, None, None);

    // The tree is the column's first child, and its layout node holds one row
    // per visible item: src, lib, main and docs. Each item reads its row.
    let tree_node = only_with_role(&update, Role::Tree);
    let items = children_of(&update, tree_node);
    assert_eq!(items.len(), layout.children[0].children.len());
    for (index, item) in items.iter().enumerate() {
        assert_eq!(
            bounds_of(item),
            layout_bounds(&snapshot, &layout.children[0].children[index]),
            "tree item {index} reads its layout row"
        );
    }

    for (id, node) in &update.nodes {
        if matches!(node.role(), Role::TreeItem | Role::Row | Role::ColumnHeader) {
            assert!(
                snapshot.get(NodeId::new(id.0)).is_none(),
                "virtual {:?} ID {} must not be a layout key",
                node.role(),
                id.0
            );
        }
    }
}

fn only_with_role(update: &TreeUpdate, role: Role) -> &AccessNode {
    let found = nodes_with_role(update, role);
    assert_eq!(found.len(), 1, "expected exactly one {role:?} node");
    found[0]
}

#[test]
fn tree_items_report_position_and_size_among_visible_siblings() {
    both_paths(&files_tree(None, true), None, |update| {
        let items = children_of(update, root_of(update));
        // Roots src and docs; children of src are lib.rs and main.rs.
        let positions: Vec<Option<usize>> =
            items.iter().map(|item| item.position_in_set()).collect();
        let sizes: Vec<Option<usize>> = items.iter().map(|item| item.size_of_set()).collect();
        assert_eq!(positions, [Some(0), Some(0), Some(1), Some(1)]);
        assert_eq!(sizes, [Some(2), Some(2), Some(2), Some(2)]);
    });
    both_paths(&files_tree(None, false), None, |update| {
        let items = children_of(update, root_of(update));
        let positions: Vec<Option<usize>> =
            items.iter().map(|item| item.position_in_set()).collect();
        assert_eq!(
            positions,
            [Some(0), Some(1)],
            "collapsed children are not siblings"
        );
    });
}

#[test]
fn unselected_cells_state_false_when_the_grid_selects_cells() {
    let grid = datagrid::<Msg>("grid")
        .column(DataGridColumn::new("a", label::<Msg>("A"), 80.0))
        .column(DataGridColumn::new("b", label::<Msg>("B"), 80.0))
        .add_row(DataGridRow::new(vec![label::<Msg>("x"), label::<Msg>("y")]))
        .build();
    let mut grid = match grid {
        WidgetNode::DataGrid(grid) => grid,
        _ => panic!("expected a DataGrid"),
    };
    grid.selected_cell = Some((0, 1));
    let root = WidgetNode::DataGrid(grid);

    both_paths(&root, None, |update| {
        let rows = children_of(update, root_of(update));
        let cells = children_of(update, rows[1]);
        assert_eq!(cells[0].is_selected(), Some(false));
        assert_eq!(cells[1].is_selected(), Some(true));
    });
}

#[test]
fn table_in_none_mode_states_no_selection_on_rows_or_cells() {
    let mut people = match people_table() {
        WidgetNode::Table(table) => table,
        _ => panic!("expected a Table"),
    };
    people.selection_mode = TableSelectionMode::None;
    let root = WidgetNode::Table(people);

    both_paths(&root, None, |update| {
        for row_node in children_of(update, root_of(update)) {
            assert_eq!(row_node.is_selected(), None);
            for cell in children_of(update, row_node) {
                assert_eq!(cell.is_selected(), None);
            }
        }
    });
}

// ------------------------------------------------------------------
// IDs
// ------------------------------------------------------------------

/// A column holding a tree and a table, so every kind of virtual node
/// appears in one update.
fn mixed_frame() -> WidgetNode<Msg> {
    column::<Msg>()
        .child(files_tree(Some("docs"), true))
        .child(people_table())
        .build()
}

#[test]
fn retained_path_ids_are_stable_across_two_frames() {
    let roots = vec![mixed_frame()];
    let mut retained = RetainedTree::new();
    let snapshot = LayoutSnapshot::default();

    let mut ids_per_frame = Vec::new();
    for _ in 0..2 {
        retained
            .reconcile_roots(&widget_to_views(&roots))
            .expect("reconciliation should succeed");
        let binding = bind_retained(&roots, &retained);
        let update = build_tree_update_with_binding(&roots[0], &snapshot, &binding, None, None);
        ids_per_frame.push(update.nodes.iter().map(|(id, _)| *id).collect::<Vec<_>>());
    }
    assert_eq!(ids_per_frame[0], ids_per_frame[1]);
}

#[test]
fn sequential_path_ids_are_deterministic_and_unique() {
    let root = mixed_frame();
    let snapshot = LayoutSnapshot::default();
    let first = build_tree_update(&root, &snapshot, None, None);
    let second = build_tree_update(&root, &snapshot, None, None);

    let first_ids: Vec<AccessNodeId> = first.nodes.iter().map(|(id, _)| *id).collect();
    let second_ids: Vec<AccessNodeId> = second.nodes.iter().map(|(id, _)| *id).collect();
    assert_eq!(first_ids, second_ids);
    let unique: HashSet<AccessNodeId> = first_ids.iter().copied().collect();
    assert_eq!(unique.len(), first_ids.len(), "every node has its own ID");
}

#[test]
fn virtual_nodes_use_synthetic_ids_and_widgets_keep_retained_ids() {
    let roots = vec![mixed_frame()];
    let mut retained = RetainedTree::new();
    retained
        .reconcile_roots(&widget_to_views(&roots))
        .expect("reconciliation should succeed");
    let binding = bind_retained(&roots, &retained);
    let update =
        build_tree_update_with_binding(&roots[0], &LayoutSnapshot::default(), &binding, None, None);

    // Virtual nodes sit at or above VIRTUAL_ID_BASE, widgets below it.
    // Retained IDs are below SYNTHETIC_ID_BASE, and widgets without a
    // retained ID take layout's synthetic range, which is below
    // VIRTUAL_ID_BASE too.
    let virtual_roles = [Role::TreeItem, Role::Row, Role::ColumnHeader];
    for (id, node) in &update.nodes {
        let is_virtual = virtual_roles.contains(&node.role());
        assert_eq!(
            id.0 >= VIRTUAL_ID_BASE,
            is_virtual,
            "{:?} node {} has ID {}",
            node.role(),
            node.label().unwrap_or(""),
            id.0
        );
    }
    let unique: HashSet<AccessNodeId> = update.nodes.iter().map(|(id, _)| *id).collect();
    assert_eq!(unique.len(), update.nodes.len());
}

#[test]
fn table_in_cell_mode_states_false_on_unselected_cells() {
    let mut people = match people_table() {
        WidgetNode::Table(table) => table,
        _ => panic!("expected a Table"),
    };
    people.selection_mode = TableSelectionMode::Cell;
    people.selected_cell = Some((1, 0));
    let root = WidgetNode::Table(people);

    both_paths(&root, None, |update| {
        let rows = children_of(update, root_of(update));
        assert_eq!(
            rows[1].is_selected(),
            None,
            "rows are not selectable in cell mode"
        );
        let ada = children_of(update, rows[1]);
        assert_eq!(
            ada.iter().map(|c| c.is_selected()).collect::<Vec<_>>(),
            [Some(false), Some(false)]
        );
        let bob = children_of(update, rows[2]);
        assert_eq!(bob[0].is_selected(), Some(true));
        assert_eq!(bob[1].is_selected(), Some(false));
    });
}
