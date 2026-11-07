//! Virtual accessibility nodes for structure that has no widget identity.
//!
//! Tree rows, table and grid rows, and column headers are not widgets: the
//! layout lowers them to synthetic leaves, and [`WidgetNode::children`] does
//! not return them. The accessibility walk creates their AccessKit nodes from
//! the parent widget's data instead:
//!
//! - a `Tree` gets one `TreeItem` child per visible row, in visible order;
//! - a `Table` or `DataGrid` whose cells fill whole rows gets a header `Row`
//!   of `ColumnHeader` nodes, then one `Row` per data row. Cells keep the
//!   role their own widget gives them (see the walk in `lib.rs`).
//!
//! Virtual nodes take IDs from `IdSource::allocate_virtual`: a counter in a
//! sequential walk, and [`VirtualIds`] in a retained walk. Both are
//! deterministic for an unchanged description. The retained-walk range sits
//! above layout's synthetic IDs, so a virtual node never reads the layout
//! rectangle of another node.
//!
//! Rectangles of virtual nodes are computed from their parent's layout
//! rectangle and the same constants layout lowering uses, not looked up by ID.

use std::collections::HashMap;

use accesskit::{Node as AccessNode, NodeId as AccessNodeId, Rect, Role};
use acme_core::{NodeId, WidgetKey};
use acme_layout::LayoutRect;
use acme_widgets::{TableSelectionMode, Tree, TreeNode, VisibleNode, WidgetNode};

use super::{IdSource, non_empty};

/// Height of one tree row, as the layout lowers each visible row.
const TREE_ROW_HEIGHT: f32 = 24.0;

/// Height of a table or grid header row and of each data row. Layout lowers
/// every header and cell leaf with `min_height` 24 and no other height, so
/// rows are 24 high. `Table::row_height` and `DataGridRow::height` do not
/// reach the layout, so they are not used here either.
const GRID_ROW_HEIGHT: f32 = 24.0;

/// First ID of the range that virtual nodes use in a retained walk.
///
/// Layout gives its own synthetic leaves (tree rows, table rows and cells) IDs
/// counting up from `SYNTHETIC_ID_BASE`, and the walk's snapshot is keyed by
/// `NodeId`. A virtual node that reused one of those IDs would read the
/// rectangle of a layout node it does not correspond to, and a click on it
/// would land there. This base is 2^56 above `SYNTHETIC_ID_BASE`, far beyond
/// any number of layout allocations, so the two ranges never meet.
pub(crate) const VIRTUAL_ID_BASE: u64 = 1 << 56;

/// Allocator for virtual nodes in a retained walk. See [`VIRTUAL_ID_BASE`].
#[derive(Debug)]
pub(crate) struct VirtualIds {
    next: u64,
}

impl VirtualIds {
    /// A fresh allocator starting at [`VIRTUAL_ID_BASE`].
    pub(crate) fn new() -> Self {
        Self {
            next: VIRTUAL_ID_BASE,
        }
    }

    /// Allocate the next virtual ID.
    pub(crate) fn allocate(&mut self) -> NodeId {
        let id = NodeId::new(self.next);
        self.next += 1;
        id
    }
}

impl Default for VirtualIds {
    fn default() -> Self {
        Self::new()
    }
}

/// Where a table cell sits in its grid, in data coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CellPlace {
    /// Zero-based data row. The header row is not counted.
    pub(crate) data_row: usize,
    pub(crate) column: usize,
}

impl CellPlace {
    /// The AccessKit row index: the header row is `0`, data rows follow it.
    pub(crate) fn row_index(self) -> usize {
        self.data_row + 1
    }
}

/// The place of cell `index` in a grid of `columns` columns and `cells` cells,
/// when the cells fill whole rows. `None` means the flat layout applies.
pub(crate) fn cell_place(columns: usize, cells: usize, index: usize) -> Option<CellPlace> {
    data_rows(columns, cells)?;
    Some(CellPlace {
        data_row: index / columns,
        column: index % columns,
    })
}

/// Number of data rows when `cells` cells fill whole rows of `columns`
/// columns. Zero columns never tile, so they always take the flat layout.
///
/// Limitation: a table or grid's `children()` is only its flat cell list, so
/// a ragged table whose cell count divides evenly by its column count (two
/// columns, one row of one cell and one row of three cells) is grouped as if
/// it were rectangular. The stored rows are not consulted, and `children()`
/// cannot show that the table is ragged.
fn data_rows(columns: usize, cells: usize) -> Option<usize> {
    if columns == 0 || cells % columns != 0 {
        return None;
    }
    Some(cells / columns)
}

/// Clips the rectangle from (`left`, `top`) to (`right`, `bottom`) to `frame`.
///
/// A corner outside the frame sits on the frame's edge, so a rectangle wholly
/// outside it keeps zero size on that edge.
fn clipped(frame: LayoutRect, left: f32, top: f32, right: f32, bottom: f32) -> Rect {
    let frame_right = frame.x + frame.width.max(0.0);
    let frame_bottom = frame.y + frame.height.max(0.0);
    let clip = |value: f32, low: f32, high: f32| value.max(low).min(high);
    Rect::new(
        f64::from(clip(left, frame.x, frame_right)),
        f64::from(clip(top, frame.y, frame_bottom)),
        f64::from(clip(right, frame.x, frame_right)),
        f64::from(clip(bottom, frame.y, frame_bottom)),
    )
}

// ============================================================================
// Trees
// ============================================================================

/// Creates one `TreeItem` node per visible row of `tree`, in visible order,
/// and returns their IDs for the `Tree` node's children.
pub(super) fn tree_item_ids<M: Clone>(
    tree: &Tree<M>,
    tree_rect: Option<LayoutRect>,
    label_override: Option<&str>,
    ids: &mut IdSource<'_, '_>,
    nodes: &mut Vec<(AccessNodeId, AccessNode)>,
) -> Vec<AccessNodeId> {
    let visible = tree.visible_nodes();
    let positions = sibling_positions(tree, &visible);
    visible
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let node = tree_item_node(
                tree,
                item,
                index,
                tree_rect,
                label_override,
                positions[index],
            );
            let id = ids.allocate_virtual();
            nodes.push((id, node));
            id
        })
        .collect()
}

/// The zero-based position and the size of each visible item among the
/// visible items that share its parent, in the order of `visible`.
fn sibling_positions<M>(tree: &Tree<M>, visible: &[VisibleNode<M>]) -> Vec<(usize, usize)> {
    let mut parents: HashMap<WidgetKey, Option<WidgetKey>> = HashMap::new();
    record_parents(&tree.children, None, &mut parents);
    let parent_of: Vec<Option<WidgetKey>> = visible
        .iter()
        .map(|item| parents.get(&item.key).cloned().flatten())
        .collect();

    let mut sizes: HashMap<Option<WidgetKey>, usize> = HashMap::new();
    for parent in &parent_of {
        *sizes.entry(parent.clone()).or_default() += 1;
    }
    let mut seen: HashMap<Option<WidgetKey>, usize> = HashMap::new();
    parent_of
        .iter()
        .map(|parent| {
            let count = seen.entry(parent.clone()).or_default();
            let position = *count;
            *count += 1;
            (position, sizes[parent])
        })
        .collect()
}

fn record_parents<M>(
    nodes: &[TreeNode<M>],
    parent: Option<&WidgetKey>,
    out: &mut HashMap<WidgetKey, Option<WidgetKey>>,
) {
    for node in nodes {
        out.insert(node.key.clone(), parent.cloned());
        record_parents(&node.children, Some(&node.key), out);
    }
}

fn tree_item_node<M>(
    tree: &Tree<M>,
    item: &VisibleNode<M>,
    index: usize,
    tree_rect: Option<LayoutRect>,
    label_override: Option<&str>,
    (position, size): (usize, usize),
) -> AccessNode {
    let mut node = AccessNode::new(Role::TreeItem);
    node.set_level(item.depth + 1);
    node.set_position_in_set(position);
    node.set_size_of_set(size);
    // Only an item with children can expand or collapse.
    if item.has_children {
        node.set_expanded(item.expanded);
    }
    // Items are selectable, so an unselected item states `false` explicitly.
    node.set_selected(tree.selected.as_ref() == Some(&item.key));
    if let Some(name) = non_empty(name_of(&item.content, label_override)) {
        node.set_label(name);
    }
    if let Some(rect) = tree_rect {
        node.set_bounds(item_bounds(rect, item.depth, index, tree.indent));
    }
    node
}

/// Bounds of the row at `index` and `depth` inside a tree at `tree`, clipped
/// to the tree's rectangle.
fn item_bounds(tree: LayoutRect, depth: usize, index: usize, indent: f32) -> Rect {
    let inset = depth as f32 * indent;
    let left = tree.x + inset;
    let top = tree.y + index as f32 * TREE_ROW_HEIGHT;
    let right = left + (tree.width - inset).max(0.0);
    clipped(tree, left, top, right, top + TREE_ROW_HEIGHT)
}

// ============================================================================
// Tables and grids
// ============================================================================

/// The columns, selection and selectable kinds of a table or grid.
struct GridParts<'a, M> {
    /// Each column's header and width, in column order.
    columns: Vec<(&'a WidgetNode<M>, f32)>,
    selected_row: Option<usize>,
    rows_selectable: bool,
}

fn grid_parts<M>(node: &WidgetNode<M>) -> Option<GridParts<'_, M>> {
    match node {
        WidgetNode::Table(table) => Some(GridParts {
            columns: table
                .columns
                .iter()
                .map(|column| (&column.header, column.width))
                .collect(),
            selected_row: table.selected_row,
            rows_selectable: matches!(
                table.selection_mode,
                TableSelectionMode::Row | TableSelectionMode::MultiRow
            ),
        }),
        WidgetNode::DataGrid(grid) => Some(GridParts {
            columns: grid
                .columns
                .iter()
                .map(|column| (&column.header, column.width))
                .collect(),
            selected_row: None,
            rows_selectable: false,
        }),
        _ => None,
    }
}

/// Whether the cells of a table or grid take a selection state, so that the
/// unselected ones state `false`. A table selects cells only in `Cell` mode;
/// a grid selects cells in any mode but `None`.
pub(crate) fn cells_selectable<M>(node: &WidgetNode<M>) -> bool {
    match node {
        WidgetNode::Table(table) => matches!(table.selection_mode, TableSelectionMode::Cell),
        WidgetNode::DataGrid(grid) => !matches!(grid.selection_mode, TableSelectionMode::None),
        _ => false,
    }
}

/// Groups the cells of a table or grid into rows.
///
/// The result is a header `Row` of `ColumnHeader` nodes followed by one `Row`
/// per data row, and the IDs of those rows are returned for the grid's
/// children. `table_rect` is the grid's layout rectangle; without it the rows
/// and headers have no bounds. Returns `None`, and allocates nothing, when
/// `node` is not a grid or its cells do not fill whole rows; the caller then
/// keeps the flat cells.
pub(super) fn grid_row_ids<M: Clone>(
    node: &WidgetNode<M>,
    cells: &[(AccessNodeId, NodeId)],
    table_rect: Option<LayoutRect>,
    label_override: Option<&str>,
    ids: &mut IdSource<'_, '_>,
    nodes: &mut Vec<(AccessNodeId, AccessNode)>,
) -> Option<Vec<AccessNodeId>> {
    let parts = grid_parts(node)?;
    let columns = parts.columns.len();
    data_rows(columns, cells.len())?;

    let spans = column_spans(&parts.columns);
    let header_cells: Vec<AccessNodeId> = parts
        .columns
        .iter()
        .zip(spans)
        .enumerate()
        .map(|(column, ((header, _), (start, end)))| {
            let mut access = AccessNode::new(Role::ColumnHeader);
            if let Some(name) = non_empty(name_of(header, label_override)) {
                access.set_label(name);
            }
            access.set_column_index(column);
            access.set_row_index(0);
            if let Some(frame) = table_rect {
                let (left, right) = (frame.x + start, frame.x + end);
                access.set_bounds(clipped(
                    frame,
                    left,
                    frame.y,
                    right,
                    frame.y + GRID_ROW_HEIGHT,
                ));
            }
            let id = ids.allocate_virtual();
            nodes.push((id, access));
            id
        })
        .collect();

    let mut row_ids = vec![push_row(
        ids,
        nodes,
        0,
        header_cells,
        row_bounds(table_rect, 0),
        None,
    )];
    for (data_row, row_cells) in cells.chunks(columns).enumerate() {
        let children = row_cells.iter().map(|&(id, _)| id).collect();
        let selected = if parts.selected_row == Some(data_row) {
            Some(true)
        } else if parts.rows_selectable {
            Some(false)
        } else {
            None
        };
        row_ids.push(push_row(
            ids,
            nodes,
            data_row + 1,
            children,
            row_bounds(table_rect, data_row + 1),
            selected,
        ));
    }
    Some(row_ids)
}

/// The horizontal extent of each column, as offsets from the table's left
/// edge. Columns sit side by side in order.
fn column_spans<M>(columns: &[(&WidgetNode<M>, f32)]) -> Vec<(f32, f32)> {
    columns
        .iter()
        .scan(0.0_f32, |start, &(_, width)| {
            let from = *start;
            *start += width;
            Some((from, *start))
        })
        .collect()
}

/// The rectangle of grid row `index` (`0` is the header row): the full table
/// width, one row high, clipped to the table.
fn row_bounds(table_rect: Option<LayoutRect>, index: usize) -> Option<Rect> {
    let top = index as f32 * GRID_ROW_HEIGHT;
    table_rect.map(|frame| {
        clipped(
            frame,
            frame.x,
            frame.y + top,
            frame.x + frame.width,
            frame.y + top + GRID_ROW_HEIGHT,
        )
    })
}

fn push_row(
    ids: &mut IdSource<'_, '_>,
    nodes: &mut Vec<(AccessNodeId, AccessNode)>,
    row_index: usize,
    children: Vec<AccessNodeId>,
    bounds: Option<Rect>,
    selected: Option<bool>,
) -> AccessNodeId {
    let mut access = AccessNode::new(Role::Row);
    access.set_row_index(row_index);
    access.set_children(children);
    if let Some(bounds) = bounds {
        access.set_bounds(bounds);
    }
    if let Some(selected) = selected {
        access.set_selected(selected);
    }
    let id = ids.allocate_virtual();
    nodes.push((id, access));
    id
}

/// Sets the row and column counts of a table or grid that has a row layout.
/// The row count includes the header row. A flat grid states no counts.
pub(super) fn set_grid_counts<M>(node: &WidgetNode<M>, access: &mut AccessNode) {
    let columns = match node {
        WidgetNode::Table(table) => table.columns.len(),
        WidgetNode::DataGrid(grid) => grid.columns.len(),
        _ => return,
    };
    if let Some(rows) = data_rows(columns, node.children().len()) {
        access.set_row_count(rows + 1);
        access.set_column_count(columns);
    }
}

// ============================================================================
// Names
// ============================================================================

/// The name of `widget`: the text of its first `Label` in depth-first order,
/// or `None`. An explicit `label_override` replaces that text, as it does for
/// every `Label` node.
fn name_of<'a, M>(widget: &'a WidgetNode<M>, label_override: Option<&'a str>) -> Option<&'a str> {
    first_label_text(widget).map(|text| label_override.unwrap_or(text))
}

fn first_label_text<M>(widget: &WidgetNode<M>) -> Option<&str> {
    if let WidgetNode::Label(label) = widget {
        return Some(label.text.as_str());
    }
    widget.children().iter().find_map(first_label_text)
}

#[cfg(test)]
mod tests;
