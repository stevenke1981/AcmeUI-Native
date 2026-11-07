//! Virtual accessibility nodes for structure that has no widget identity.
//!
//! Tree rows, table and grid rows, and column headers are not widgets. Layout
//! lowers them to leaves and containers with IDs of their own, and
//! [`WidgetNode::children`] does not return them. The walk takes those IDs in
//! layout's order (see `super::IdSource`), so each virtual node reads the
//! rectangle of its layout node:
//!
//! - a `Tree` gets one `TreeItem` per visible row, in visible order;
//! - a `Table` or `DataGrid` gets a header `Row` of `ColumnHeader` nodes and
//!   one `Row` per data row, grouped by each row's own cells. A grid whose
//!   flattened cells do not match its rows, or that has no columns, keeps its
//!   cells flat. Cells keep the role their own widget gives them.
//!
//! Node IDs: in a sequential walk a virtual node's ID is its layout ID. In a
//! retained walk it comes from [`VirtualIds`], above layout's synthetic IDs,
//! so it never reads another node's rectangle.

use std::collections::HashMap;

use accesskit::{Action, Node as AccessNode, NodeId as AccessNodeId, Rect, Role};
use acme_core::{NodeId, WidgetKey};
use acme_layout::LayoutRect;
use acme_widgets::{TableSelectionMode, Tree, TreeNode, VisibleNode, WidgetNode};

use super::{Ctx, IdSource, Lowering, Slot, Walked, non_empty, walk};

/// Height of one tree row, as the layout lowers each visible row.
const TREE_ROW_HEIGHT: f32 = 24.0;

/// Height of a table or grid header row and of each data row. Layout lowers
/// every header and cell leaf with `min_height` 24 and no other height, so
/// rows are 24 high. `Table::row_height` and `DataGridRow::height` do not
/// reach the layout, so they are not used here either.
const GRID_ROW_HEIGHT: f32 = 24.0;

/// The first ID of the range that virtual nodes use: the absolute value 2^56.
///
/// The walk's snapshot is keyed by `NodeId`. Layout numbers its synthetic
/// leaves from `SYNTHETIC_ID_BASE` (2^48) upward, one per layout node, so they
/// stay far below 2^56. A virtual node in this range never reads another
/// node's rectangle.
pub(crate) const VIRTUAL_ID_BASE: u64 = 1 << 56;

/// Allocator for virtual nodes. See [`VIRTUAL_ID_BASE`].
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

/// The AccessKit rectangle of a layout rectangle.
pub(super) fn rect_of(rect: LayoutRect) -> Rect {
    Rect::new(
        f64::from(rect.x),
        f64::from(rect.y),
        f64::from(rect.x + rect.width),
        f64::from(rect.y + rect.height),
    )
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

/// The snapshot rectangle of a layout node, if it has one.
fn layout_rect(ctx: Ctx<'_>, layout: Option<NodeId>) -> Option<Rect> {
    layout
        .and_then(|id| ctx.snapshot.get(id).copied())
        .map(rect_of)
}

// ============================================================================
// Trees
// ============================================================================

/// The layout IDs of a tree's visible rows, one per row in visible order. The
/// rows are lowered, and so take IDs, only when `lowered` is set.
pub(super) fn tree_row_layouts<M: Clone>(
    tree: &Tree<M>,
    lowered: bool,
    ids: &mut IdSource<'_, '_>,
) -> Vec<Option<NodeId>> {
    tree.visible_nodes()
        .iter()
        .map(|_| lowered.then(|| ids.structural()))
        .collect()
}

/// Creates one `TreeItem` node per visible row of `tree`, in visible order,
/// and returns their IDs for the `Tree` node's children. `own` is the tree's
/// layout ID, whose rectangle is the fallback frame for the rows.
///
/// A row whose content holds more than plain text (a button, a text input, a
/// nested composite) has that content walked as the item's child. Layout
/// numbers no node for row content, so the walk takes no layout ID for it
/// ([`Lowering::Unlowered`]) and the layout-ID order is unchanged. The
/// content root reads the row's rectangle, since it has none of its own.
pub(super) fn tree_item_ids<M: Clone + 'static>(
    tree: &Tree<M>,
    own: Option<NodeId>,
    rows: &[Option<NodeId>],
    ctx: Ctx<'_>,
    ids: &mut IdSource<'_, '_>,
    nodes: &mut Vec<(AccessNodeId, AccessNode)>,
) -> Vec<AccessNodeId> {
    let frame = own.and_then(|id| ctx.snapshot.get(id).copied());
    let visible = tree.visible_nodes();
    let positions = sibling_positions(tree, &visible);
    visible
        .iter()
        .enumerate()
        .zip(positions)
        .map(|((index, item), position)| {
            let layout = rows.get(index).copied().flatten();
            let bounds = layout_rect(ctx, layout)
                .or_else(|| frame.map(|frame| item_bounds(frame, item.depth, index, tree.indent)));
            let mut node = tree_item_node(tree, item, ctx.label_override, position, bounds);
            let id = ids.structural_identity(layout);
            if !is_plain_content(&item.content) {
                let content = walk(
                    &item.content,
                    ctx,
                    ids,
                    nodes,
                    Slot::Plain,
                    Lowering::Unlowered,
                );
                if let Some(bounds) = bounds
                    && let Some((_, root)) =
                        nodes.iter_mut().rev().find(|(id, _)| *id == content.id)
                    && root.bounds().is_none()
                {
                    root.set_bounds(bounds);
                }
                node.set_children(vec![content.id]);
            }
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
    label_override: Option<&str>,
    (position, size): (usize, usize),
    bounds: Option<Rect>,
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
    // Every row can be focused and clicked (selected); a row with children can
    // also expand or collapse, whichever it is not now.
    node.add_action(Action::Focus);
    node.add_action(Action::Click);
    if item.has_children {
        node.add_action(if item.expanded {
            Action::Collapse
        } else {
            Action::Expand
        });
    }
    let name = name_of(&item.content, label_override).or_else(|| control_name(&item.content));
    if let Some(name) = non_empty(name) {
        node.set_label(name);
    }
    if let Some(bounds) = bounds {
        node.set_bounds(bounds);
    }
    node
}

/// Fallback bounds of the row at `index` and `depth` in a tree at `tree`,
/// clipped to the tree. Used only when the layout has no rectangle for the row.
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

/// The shape of a table or grid: its column count, the cell count of each
/// row, and whether layout lowers a header row for it.
struct Shape {
    columns: usize,
    rows: Vec<usize>,
    header: bool,
}

fn shape<M>(node: &WidgetNode<M>) -> Option<Shape> {
    match node {
        WidgetNode::Table(table) => Some(Shape {
            columns: table.columns.len(),
            rows: table.rows.iter().map(|row| row.cells.len()).collect(),
            // Layout lowers a Table's header only when it has columns.
            header: !table.columns.is_empty(),
        }),
        WidgetNode::DataGrid(grid) => Some(Shape {
            columns: grid.columns.len(),
            rows: grid.rows.iter().map(|row| row.cells.len()).collect(),
            header: true,
        }),
        _ => None,
    }
}

/// Whether a grid's cells group into rows: it has columns, and its flattened
/// children are exactly the cells of its rows. Otherwise the cells stay flat.
pub(super) fn grouped<M>(node: &WidgetNode<M>) -> bool {
    shape(node)
        .is_some_and(|s| s.columns > 0 && s.rows.iter().sum::<usize>() == node.children().len())
}

/// The selection of a table or grid, as its mode and selected cell allow.
struct Selection {
    row: Option<usize>,
    rows_selectable: bool,
    cell: Option<(usize, usize)>,
    cells_selectable: bool,
}

fn selection<M>(node: &WidgetNode<M>) -> Selection {
    match node {
        WidgetNode::Table(table) => Selection {
            row: table.selected_row,
            rows_selectable: matches!(
                table.selection_mode,
                TableSelectionMode::Row | TableSelectionMode::MultiRow
            ),
            cell: table.selected_cell,
            cells_selectable: matches!(table.selection_mode, TableSelectionMode::Cell),
        },
        WidgetNode::DataGrid(grid) => Selection {
            row: None,
            rows_selectable: false,
            cell: grid.selected_cell,
            cells_selectable: !matches!(grid.selection_mode, TableSelectionMode::None),
        },
        _ => Selection {
            row: None,
            rows_selectable: false,
            cell: None,
            cells_selectable: false,
        },
    }
}

/// Each column's header widget and width, in column order.
fn grid_columns<M>(node: &WidgetNode<M>) -> Vec<(&WidgetNode<M>, f32)> {
    match node {
        WidgetNode::Table(table) => table
            .columns
            .iter()
            .map(|column| (&column.header, column.width))
            .collect(),
        WidgetNode::DataGrid(grid) => grid
            .columns
            .iter()
            .map(|column| (&column.header, column.width))
            .collect(),
        _ => Vec::new(),
    }
}

/// The layout IDs of one grid line: the leaves of its cells (or of its column
/// headers, for the header line), and its row container. `None` where layout
/// does not lower the grid.
pub(super) struct Line {
    leaves: Vec<Option<NodeId>>,
    row: Option<NodeId>,
}

/// The layout IDs of a table or grid, in the order layout lowering takes them:
/// the header leaves and the header row, then each row's cell leaves and its
/// row container.
#[derive(Default)]
pub(super) struct GridLayout {
    header: Option<Line>,
    rows: Vec<Line>,
}

/// Takes the layout IDs of a table or grid, in layout's order. Nothing is
/// taken unless `lowered` is set.
pub(super) fn plan_grid<M>(
    node: &WidgetNode<M>,
    lowered: bool,
    ids: &mut IdSource<'_, '_>,
) -> GridLayout {
    let Some(shape) = shape(node) else {
        return GridLayout::default();
    };
    let mut take = || lowered.then(|| ids.structural());
    let header = shape.header.then(|| Line {
        leaves: (0..shape.columns).map(|_| take()).collect(),
        row: take(),
    });
    let rows = shape
        .rows
        .iter()
        .map(|&cells| Line {
            leaves: (0..cells).map(|_| take()).collect(),
            row: take(),
        })
        .collect();
    GridLayout { header, rows }
}

/// How layout lowers one flat child of a grid, and where that child sits.
pub(super) struct CellSlot {
    pub(super) lowering: Lowering,
    pub(super) place: Option<CellPlace>,
    pub(super) selected: Option<bool>,
}

/// How layout lowers each flat child (cell) of a grid, and where it sits. The
/// cells are matched to layout's leaves through the grid's rows, which is only
/// possible when the flat children are exactly those rows' cells; otherwise
/// every cell is unlowered and flat.
pub(super) fn cell_slots<M>(node: &WidgetNode<M>, layout: &GridLayout) -> Vec<CellSlot> {
    let cells = node.children().len();
    let positions: Vec<(usize, usize, Option<NodeId>)> = layout
        .rows
        .iter()
        .enumerate()
        .flat_map(|(data_row, line)| {
            line.leaves
                .iter()
                .enumerate()
                .map(move |(column, leaf)| (data_row, column, *leaf))
        })
        .collect();
    let aligned = positions.len() == cells;
    let is_grouped = grouped(node);
    let selection = selection(node);
    (0..cells)
        .map(|index| {
            if !aligned {
                return CellSlot {
                    lowering: Lowering::Unlowered,
                    place: None,
                    selected: None,
                };
            }
            let (data_row, column, leaf) = positions[index];
            // A flat grid has no row structure, so its cells state no place.
            let place = is_grouped.then_some(CellPlace { data_row, column });
            let selected = place.and_then(|place| {
                if selection.cell == Some((place.data_row, place.column)) {
                    Some(true)
                } else if selection.cells_selectable {
                    Some(false)
                } else {
                    None
                }
            });
            CellSlot {
                lowering: leaf.map_or(Lowering::Unlowered, Lowering::Cell),
                place,
                selected,
            }
        })
        .collect()
}

/// Creates the header and data rows of a grouped grid as nodes, and returns
/// their IDs for the grid's children. Returns `None` when the grid is flat, in
/// which case nothing is created. `own` is the grid's layout ID; its rectangle
/// is the fallback frame for the rows.
pub(super) fn grid_row_ids<M: Clone>(
    node: &WidgetNode<M>,
    layout: &GridLayout,
    own: Option<NodeId>,
    children: &[Walked],
    ctx: Ctx<'_>,
    ids: &mut IdSource<'_, '_>,
    nodes: &mut Vec<(AccessNodeId, AccessNode)>,
) -> Option<Vec<AccessNodeId>> {
    if !grouped(node) {
        return None;
    }
    let header = layout.header.as_ref()?;
    let frame = own.and_then(|id| ctx.snapshot.get(id).copied());
    let header_children = column_header_ids(node, header, frame, ctx, ids, nodes);
    let header_bounds = row_bounds(ctx, frame, header.row, 0);
    let mut row_ids = vec![push_row(
        ids,
        nodes,
        header.row,
        0,
        header_children,
        header_bounds,
        None,
    )];
    let selection = selection(node);
    row_ids.extend(data_row_ids(
        layout, &selection, frame, children, ctx, ids, nodes,
    ));
    Some(row_ids)
}

/// Creates the column header nodes of a grouped grid, one per column in order,
/// and returns their IDs.
fn column_header_ids<M: Clone>(
    node: &WidgetNode<M>,
    header: &Line,
    frame: Option<LayoutRect>,
    ctx: Ctx<'_>,
    ids: &mut IdSource<'_, '_>,
    nodes: &mut Vec<(AccessNodeId, AccessNode)>,
) -> Vec<AccessNodeId> {
    let columns = grid_columns(node);
    let spans = column_spans(columns.iter().map(|&(_, width)| width));
    let mut header_ids = Vec::with_capacity(columns.len());
    for (column, ((widget, _), (start, end))) in columns.iter().zip(spans).enumerate() {
        let leaf = header.leaves.get(column).copied().flatten();
        let mut access = AccessNode::new(Role::ColumnHeader);
        if let Some(name) = non_empty(name_of(widget, ctx.label_override)) {
            access.set_label(name);
        }
        access.set_column_index(column);
        access.set_row_index(0);
        let bounds = layout_rect(ctx, leaf).or_else(|| {
            frame.map(|frame| {
                clipped(
                    frame,
                    frame.x + start,
                    frame.y,
                    frame.x + end,
                    frame.y + GRID_ROW_HEIGHT,
                )
            })
        });
        if let Some(bounds) = bounds {
            access.set_bounds(bounds);
        }
        let id = ids.structural_identity(leaf);
        nodes.push((id, access));
        header_ids.push(id);
    }
    header_ids
}

/// Creates the data row nodes of a grouped grid, one per row of `layout`, each
/// holding the walked cells of that row, and returns their IDs.
fn data_row_ids(
    layout: &GridLayout,
    selection: &Selection,
    frame: Option<LayoutRect>,
    children: &[Walked],
    ctx: Ctx<'_>,
    ids: &mut IdSource<'_, '_>,
    nodes: &mut Vec<(AccessNodeId, AccessNode)>,
) -> Vec<AccessNodeId> {
    let mut offset = 0;
    let mut row_ids = Vec::with_capacity(layout.rows.len());
    for (data_row, line) in layout.rows.iter().enumerate() {
        let len = line.leaves.len();
        let row_children = children[offset..offset + len]
            .iter()
            .map(|child| child.id)
            .collect();
        offset += len;
        let selected = if selection.row == Some(data_row) {
            Some(true)
        } else if selection.rows_selectable {
            Some(false)
        } else {
            None
        };
        let bounds = row_bounds(ctx, frame, line.row, data_row + 1);
        row_ids.push(push_row(
            ids,
            nodes,
            line.row,
            data_row + 1,
            row_children,
            bounds,
            selected,
        ));
    }
    row_ids
}

/// The horizontal extent of each column, as offsets from the table's left
/// edge. Columns sit side by side in order.
fn column_spans(widths: impl Iterator<Item = f32>) -> Vec<(f32, f32)> {
    widths
        .scan(0.0_f32, |start, width| {
            let from = *start;
            *start += width;
            Some((from, *start))
        })
        .collect()
}

/// The rectangle of grid row `index` (`0` is the header row). The layout's
/// rectangle is used; without one, the table's width one row high at `index`,
/// clipped to the table.
fn row_bounds(
    ctx: Ctx<'_>,
    frame: Option<LayoutRect>,
    layout: Option<NodeId>,
    index: usize,
) -> Option<Rect> {
    layout_rect(ctx, layout).or_else(|| {
        frame.map(|frame| {
            let top = frame.y + index as f32 * GRID_ROW_HEIGHT;
            clipped(
                frame,
                frame.x,
                top,
                frame.x + frame.width,
                top + GRID_ROW_HEIGHT,
            )
        })
    })
}

fn push_row(
    ids: &mut IdSource<'_, '_>,
    nodes: &mut Vec<(AccessNodeId, AccessNode)>,
    layout: Option<NodeId>,
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
    let id = ids.structural_identity(layout);
    nodes.push((id, access));
    id
}

/// Sets the row and column counts of a grouped table or grid. The row count
/// includes the header row. The column count is the wider of the column list
/// and the widest row, so every cell's column index is below it. A flat grid
/// states no counts.
pub(super) fn set_grid_counts<M>(node: &WidgetNode<M>, access: &mut AccessNode) {
    if !grouped(node) {
        return;
    }
    if let Some(shape) = shape(node) {
        let widest = shape.rows.iter().copied().max().unwrap_or(0);
        access.set_row_count(shape.rows.len() + 1);
        access.set_column_count(shape.columns.max(widest));
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

/// Whether row content is only text in plain containers, which adds no node.
fn is_plain_content<M>(widget: &WidgetNode<M>) -> bool {
    match widget {
        WidgetNode::Label(_) => true,
        WidgetNode::Row(_) | WidgetNode::Column(_) | WidgetNode::Stack(_) | WidgetNode::Card(_) => {
            widget.children().iter().all(is_plain_content)
        }
        _ => false,
    }
}

/// The name of the first control in `widget`, for content without a label.
fn control_name<M>(widget: &WidgetNode<M>) -> Option<&str> {
    match widget {
        WidgetNode::Button(button) => non_empty(Some(button.label.as_str())),
        WidgetNode::TextInput(input) => non_empty(Some(super::text_input_name(input))),
        _ => None,
    }
    .or_else(|| widget.children().iter().find_map(control_name))
}

fn first_label_text<M>(widget: &WidgetNode<M>) -> Option<&str> {
    if let WidgetNode::Label(label) = widget {
        return Some(label.text.as_str());
    }
    widget.children().iter().find_map(first_label_text)
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_layout_ids;
#[cfg(test)]
mod tests_support;
#[cfg(test)]
mod tests_tree_actions;
