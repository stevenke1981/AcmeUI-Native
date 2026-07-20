use crate::WidgetNode;
use acme_core::WidgetKey;

// ============================================================================
// Re-use types from table module
// ============================================================================

pub use crate::data::table::{SortDirection, TableSelectionMode, TableState};

// ============================================================================
// DataGridColumn
// ============================================================================

/// A column definition for a data grid.
#[derive(Clone, Debug, PartialEq)]
pub struct DataGridColumn<M> {
    pub key: WidgetKey,
    /// Header widget.
    pub header: WidgetNode<M>,
    /// Column width in logical pixels.
    pub width: f32,
    /// Minimum width.
    pub min_width: f32,
    /// Maximum width.
    pub max_width: f32,
    /// Whether the column is sortable.
    pub sortable: bool,
    /// Whether the column is frozen (stays visible during horizontal scroll).
    pub frozen: bool,
}

impl<M> DataGridColumn<M> {
    pub fn new(key: impl Into<WidgetKey>, header: impl Into<WidgetNode<M>>, width: f32) -> Self {
        Self {
            key: key.into(),
            header: header.into(),
            width,
            min_width: 20.0,
            max_width: f32::MAX,
            sortable: false,
            frozen: false,
        }
    }

    /// Builder: set min width.
    pub fn min_width(mut self, value: f32) -> Self {
        self.min_width = crate::finite(value).max(0.0);
        self
    }

    /// Builder: set max width.
    pub fn max_width(mut self, value: f32) -> Self {
        self.max_width = crate::finite(value).max(self.min_width);
        self
    }

    /// Builder: set sortable.
    pub fn sortable(mut self, value: bool) -> Self {
        self.sortable = value;
        self
    }

    /// Builder: set frozen (sticky column).
    pub fn frozen(mut self, value: bool) -> Self {
        self.frozen = value;
        self
    }
}

// ============================================================================
// DataGridRow
// ============================================================================

/// A single row in a data grid.
#[derive(Clone, Debug, PartialEq)]
pub struct DataGridRow<M> {
    pub cells: Vec<WidgetNode<M>>,
    pub height: Option<f32>,
    /// Optional row number / line number prefix.
    pub row_number: Option<String>,
}

impl<M> DataGridRow<M> {
    pub fn new(cells: Vec<WidgetNode<M>>) -> Self {
        Self {
            cells,
            height: None,
            row_number: None,
        }
    }

    /// Builder: set row height.
    pub fn height(mut self, value: f32) -> Self {
        self.height = Some(crate::finite(value).max(0.0));
        self
    }

    /// Builder: set row number.
    pub fn row_number(mut self, value: impl Into<String>) -> Self {
        self.row_number = Some(value.into());
        self
    }
}

// ============================================================================
// CellMerge — colspan/rowspan descriptor
// ============================================================================

/// Describes a merged cell span in the data grid.
#[derive(Clone, Debug, PartialEq)]
pub struct CellMerge {
    /// Number of columns this cell spans (default 1).
    pub colspan: usize,
    /// Number of rows this cell spans (default 1).
    pub rowspan: usize,
}

impl CellMerge {
    pub fn new(colspan: usize, rowspan: usize) -> Self {
        Self {
            colspan: colspan.max(1),
            rowspan: rowspan.max(1),
        }
    }
}

// ============================================================================
// DataGrid
// ============================================================================

/// A data grid with 2D virtualization, frozen rows/columns, cell merging,
/// and row numbers.
///
/// Both rows and columns are virtualized — only cells within the visible
/// viewport (+ overscan) enter the layout tree.
pub struct DataGrid<M> {
    pub key: WidgetKey,
    /// Column definitions.
    pub columns: Vec<DataGridColumn<M>>,
    /// Data rows.
    pub rows: Vec<DataGridRow<M>>,
    /// Number of frozen rows at the top (always visible).
    pub frozen_rows: usize,
    /// Number of frozen columns at the left (always visible).
    pub frozen_cols: usize,
    /// Horizontal scroll offset.
    pub scroll_x: f32,
    /// Vertical scroll offset.
    pub scroll_y: f32,
    /// Currently selected cell `(row, col)`.
    pub selected_cell: Option<(usize, usize)>,
    /// Selection mode.
    pub selection_mode: TableSelectionMode,
    /// Default column width.
    pub default_col_width: f32,
    /// Default row height.
    pub default_row_height: f32,
    /// Overscan for both axes.
    pub overscan: usize,
    /// High-level state.
    pub state: TableState<M>,
    /// Cell merge descriptors. Keyed by `(row, col)`.
    pub merged_cells: std::collections::HashMap<(usize, usize), CellMerge>,
    /// Total viewport width (set by layout).
    pub viewport_width: f32,
    /// Total viewport height (set by layout).
    pub viewport_height: f32,
    /// Flat list of all cell widgets (for reconciliation / children()).
    pub(crate) all_cells: Vec<WidgetNode<M>>,
}

// Manual Clone / Debug / PartialEq

impl<M: Clone> Clone for DataGrid<M> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            columns: self.columns.clone(),
            rows: self.rows.clone(),
            frozen_rows: self.frozen_rows,
            frozen_cols: self.frozen_cols,
            scroll_x: self.scroll_x,
            scroll_y: self.scroll_y,
            selected_cell: self.selected_cell,
            selection_mode: self.selection_mode,
            default_col_width: self.default_col_width,
            default_row_height: self.default_row_height,
            overscan: self.overscan,
            state: self.state.clone(),
            merged_cells: self.merged_cells.clone(),
            viewport_width: self.viewport_width,
            viewport_height: self.viewport_height,
            all_cells: self.all_cells.clone(),
        }
    }
}

impl<M: std::fmt::Debug> std::fmt::Debug for DataGrid<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataGrid")
            .field("key", &self.key)
            .field("frozen_rows", &self.frozen_rows)
            .field("frozen_cols", &self.frozen_cols)
            .field("scroll_x", &self.scroll_x)
            .field("scroll_y", &self.scroll_y)
            .field("selected_cell", &self.selected_cell)
            .field("selection_mode", &self.selection_mode)
            .field("state", &self.state)
            .field("overscan", &self.overscan)
            .finish()
    }
}

impl<M: PartialEq> PartialEq for DataGrid<M> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.columns == other.columns
            && self.rows == other.rows
            && self.frozen_rows == other.frozen_rows
            && self.frozen_cols == other.frozen_cols
            && self.scroll_x == other.scroll_x
            && self.scroll_y == other.scroll_y
            && self.selected_cell == other.selected_cell
            && self.selection_mode == other.selection_mode
            && self.default_col_width == other.default_col_width
            && self.default_row_height == other.default_row_height
            && self.overscan == other.overscan
            && self.state == other.state
            && self.viewport_width == other.viewport_width
            && self.viewport_height == other.viewport_height
    }
}

// ── Builder / factory ───────────────────────────────────────────────────────

/// Create a new `DataGrid` builder.
pub fn datagrid<M>(key: impl Into<WidgetKey>) -> DataGrid<M> {
    DataGrid {
        key: key.into(),
        columns: Vec::new(),
        rows: Vec::new(),
        frozen_rows: 0,
        frozen_cols: 0,
        scroll_x: 0.0,
        scroll_y: 0.0,
        selected_cell: None,
        selection_mode: TableSelectionMode::Cell,
        default_col_width: 120.0,
        default_row_height: 28.0,
        overscan: 3,
        state: TableState::Normal,
        merged_cells: std::collections::HashMap::new(),
        viewport_width: 0.0,
        viewport_height: 0.0,
        all_cells: Vec::new(),
    }
}

impl<M> DataGrid<M> {
    /// Add a column.
    pub fn column(mut self, col: DataGridColumn<M>) -> Self {
        self.columns.push(col);
        self
    }

    /// Add a row.
    pub fn add_row(mut self, row: DataGridRow<M>) -> Self {
        self.rows.push(row);
        self
    }

    /// Set frozen row count.
    pub fn frozen_rows(mut self, value: usize) -> Self {
        self.frozen_rows = value;
        self
    }

    /// Set frozen column count.
    pub fn frozen_cols(mut self, value: usize) -> Self {
        self.frozen_cols = value;
        self
    }

    /// Set default column width.
    pub fn default_col_width(mut self, value: f32) -> Self {
        self.default_col_width = crate::finite(value).max(1.0);
        self
    }

    /// Set default row height.
    pub fn default_row_height(mut self, value: f32) -> Self {
        self.default_row_height = crate::finite(value).max(1.0);
        self
    }

    /// Add a merged cell descriptor.
    pub fn merge_cells(mut self, row: usize, col: usize, colspan: usize, rowspan: usize) -> Self {
        self.merged_cells
            .insert((row, col), CellMerge::new(colspan, rowspan));
        self
    }

    /// Set viewport width.
    pub fn viewport_width(mut self, value: f32) -> Self {
        self.viewport_width = crate::finite(value).max(1.0);
        self
    }

    /// Set viewport height.
    pub fn viewport_height(mut self, value: f32) -> Self {
        self.viewport_height = crate::finite(value).max(1.0);
        self
    }

    /// Set table state.
    pub fn state(mut self, value: TableState<M>) -> Self {
        self.state = value;
        self
    }

    /// Build into a `WidgetNode`.
    pub fn build(mut self) -> WidgetNode<M>
    where
        M: Clone,
    {
        self.all_cells = self
            .rows
            .iter()
            .flat_map(|r| r.cells.iter().cloned())
            .collect();
        WidgetNode::DataGrid(self)
    }
}

// ── Core methods ─────────────────────────────────────────────────────────────

impl<M> DataGrid<M> {
    // ── Column helpers ────────────────────────────────────────────────────

    /// Get the width of a column (falls back to default).
    pub fn col_width(&self, index: usize) -> f32 {
        self.columns
            .get(index)
            .map(|c| c.width)
            .unwrap_or(self.default_col_width)
    }

    /// Get the height of a row (falls back to default).
    pub fn row_height_at(&self, index: usize) -> f32 {
        self.rows
            .get(index)
            .and_then(|r| r.height)
            .unwrap_or(self.default_row_height)
    }

    /// Total content width.
    pub fn total_width(&self) -> f32 {
        let col_count = self.columns.len().max(self.max_col_count());
        col_count as f32 * self.default_col_width
    }

    /// Total content height.
    pub fn total_height(&self) -> f32 {
        self.rows.len() as f32 * self.default_row_height
    }

    /// Maximum number of columns across all rows.
    fn max_col_count(&self) -> usize {
        self.rows
            .iter()
            .map(|r| r.cells.len())
            .max()
            .unwrap_or(0)
            .max(self.columns.len())
    }

    // ── 2D Viewport / virtualization ──────────────────────────────────────

    /// Calculate visible column range `(first, one_past_last)`.
    pub fn visible_columns(&self) -> (usize, usize) {
        if self.columns.is_empty() || self.viewport_width <= 0.0 {
            return (0, 0);
        }
        let col_count = self.columns.len();
        let mut x = 0.0_f32;
        let mut first = 0;
        while first < col_count {
            let w = self.col_width(first);
            if x + w > self.scroll_x {
                break;
            }
            x += w;
            first += 1;
        }
        let overscan_first = first.saturating_sub(self.overscan);

        let mut last = first;
        let mut vx = x;
        while last < col_count && vx < self.scroll_x + self.viewport_width {
            vx += self.col_width(last);
            last += 1;
        }
        let overscan_last = (last + self.overscan).min(col_count);

        (overscan_first, overscan_last)
    }

    /// Calculate visible row range `(first, one_past_last)`.
    pub fn visible_rows(&self) -> (usize, usize) {
        if self.rows.is_empty() || self.viewport_height <= 0.0 {
            return (0, 0);
        }
        let row_count = self.rows.len();
        let mut y = 0.0_f32;
        let mut first = 0;
        while first < row_count {
            let h = self.row_height_at(first);
            if y + h > self.scroll_y {
                break;
            }
            y += h;
            first += 1;
        }
        let overscan_first = first.saturating_sub(self.overscan);

        let mut last = first;
        let mut vy = y;
        while last < row_count && vy < self.scroll_y + self.viewport_height {
            vy += self.row_height_at(last);
            last += 1;
        }
        let overscan_last = (last + self.overscan).min(row_count);

        (overscan_first, overscan_last)
    }

    /// Check if a cell is covered by a merge (i.e., is not the origin cell).
    pub fn is_merged_cell(&self, row: usize, col: usize) -> bool {
        // A cell is "merged" if it's within the span of another cell's merge
        for (&(r, c), merge) in &self.merged_cells {
            if r == row && c == col {
                return false; // origin cell is not "covered"
            }
            if row >= r && row < r + merge.rowspan && col >= c && col < c + merge.colspan {
                return true;
            }
        }
        false
    }

    /// Get the origin of a merge that covers `(row, col)`, if any.
    pub fn merge_origin(&self, row: usize, col: usize) -> Option<(usize, usize)> {
        for (&(r, c), merge) in &self.merged_cells {
            if row >= r && row < r + merge.rowspan && col >= c && col < c + merge.colspan {
                return Some((r, c));
            }
        }
        None
    }

    /// Get the merge descriptor for a cell origin.
    pub fn get_merge(&self, row: usize, col: usize) -> Option<&CellMerge> {
        self.merged_cells.get(&(row, col))
    }

    // ── Selection ─────────────────────────────────────────────────────────

    pub fn select_cell(&mut self, row: usize, col: usize) {
        if row < self.rows.len() && col < self.columns.len() {
            self.selected_cell = Some((row, col));
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected_cell = None;
    }

    // ── Keyboard navigation ──────────────────────────────────────────────

    pub fn select_down(&mut self) {
        if let Some((row, col)) = self.selected_cell {
            let next = (row + 1).min(self.rows.len().saturating_sub(1));
            self.selected_cell = Some((next, col));
        } else if !self.rows.is_empty() && !self.columns.is_empty() {
            self.selected_cell = Some((0, 0));
        }
    }

    pub fn select_up(&mut self) {
        if let Some((row, col)) = self.selected_cell {
            let prev = row.saturating_sub(1);
            self.selected_cell = Some((prev, col));
        } else if !self.rows.is_empty() && !self.columns.is_empty() {
            self.selected_cell = Some((self.rows.len() - 1, 0));
        }
    }

    pub fn select_right(&mut self) {
        if let Some((row, col)) = self.selected_cell {
            let next = (col + 1).min(self.columns.len().saturating_sub(1));
            self.selected_cell = Some((row, next));
        }
    }

    pub fn select_left(&mut self) {
        if let Some((row, col)) = self.selected_cell {
            let prev = col.saturating_sub(1);
            self.selected_cell = Some((row, prev));
        }
    }

    /// Scroll to ensure a cell is visible.
    pub fn scroll_to_cell(&mut self, row: usize, col: usize) {
        // Vertical
        let cell_top = row as f32 * self.default_row_height;
        let cell_bottom = cell_top + self.default_row_height;
        if cell_top < self.scroll_y {
            self.scroll_y = cell_top;
        } else if cell_bottom > self.scroll_y + self.viewport_height {
            self.scroll_y = cell_bottom - self.viewport_height;
        }
        // Horizontal
        let cell_left = col as f32 * self.default_col_width;
        let cell_right = cell_left + self.default_col_width;
        if cell_left < self.scroll_x {
            self.scroll_x = cell_left;
        } else if cell_right > self.scroll_x + self.viewport_width {
            self.scroll_x = cell_right - self.viewport_width;
        }
    }
}

// ── From conversion ──────────────────────────────────────────────────────────

impl<M> From<DataGrid<M>> for WidgetNode<M> {
    fn from(value: DataGrid<M>) -> Self {
        WidgetNode::DataGrid(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::label;

    fn sample_grid() -> DataGrid<()> {
        datagrid::<()>("g")
            .column(DataGridColumn::new("name", label("Name"), 150.0))
            .column(DataGridColumn::new("score", label("Score"), 100.0))
            .add_row(DataGridRow::new(vec![label("Alice"), label("95")]))
            .add_row(DataGridRow::new(vec![label("Bob"), label("87")]))
            .add_row(DataGridRow::new(vec![label("Charlie"), label("92")]))
    }

    #[test]
    fn datagrid_creates_columns_and_rows() {
        let g = sample_grid();
        assert_eq!(g.columns.len(), 2);
        assert_eq!(g.rows.len(), 3);
        assert_eq!(g.columns[0].key.as_str(), "name");
    }

    #[test]
    fn visible_columns_at_start() {
        let g = sample_grid().viewport_width(300.0);
        let (first, last) = g.visible_columns();
        assert_eq!(first, 0);
        assert!(last > 0);
    }

    #[test]
    fn visible_rows_at_start() {
        let g = sample_grid().viewport_height(200.0);
        let (first, last) = g.visible_rows();
        assert_eq!(first, 0);
        assert!(last > 0);
    }

    #[test]
    fn frozen_rows_and_cols() {
        let g = sample_grid().frozen_rows(1).frozen_cols(1);
        assert_eq!(g.frozen_rows, 1);
        assert_eq!(g.frozen_cols, 1);
    }

    #[test]
    fn cell_merge_origin() {
        let g = datagrid::<()>("g")
            .column(DataGridColumn::new("a", label("A"), 100.0))
            .column(DataGridColumn::new("b", label("B"), 100.0))
            .add_row(DataGridRow::new(vec![label("A1"), label("B1")]))
            .add_row(DataGridRow::new(vec![label("A2"), label("B2")]))
            .merge_cells(0, 0, 2, 1); // A1 spans 2 columns, 1 row

        assert_eq!(g.merged_cells.len(), 1);
        assert!(!g.is_merged_cell(0, 0)); // origin
        assert!(g.is_merged_cell(0, 1)); // covered by merge
        assert!(!g.is_merged_cell(1, 0)); // outside
        assert_eq!(g.merge_origin(0, 1), Some((0, 0)));
    }

    #[test]
    fn select_cell_and_navigate() {
        let mut g = sample_grid();
        g.select_cell(1, 0);
        assert_eq!(g.selected_cell, Some((1, 0)));

        g.select_down();
        assert_eq!(g.selected_cell, Some((2, 0)));

        g.select_right();
        assert_eq!(g.selected_cell, Some((2, 1)));

        g.select_up();
        assert_eq!(g.selected_cell, Some((1, 1)));

        g.select_left();
        assert_eq!(g.selected_cell, Some((1, 0)));
    }

    #[test]
    fn clear_selection_resets() {
        let mut g = sample_grid();
        g.select_cell(0, 0);
        g.clear_selection();
        assert!(g.selected_cell.is_none());
    }

    #[test]
    fn scroll_to_cell_adjusts_scroll() {
        let mut g = sample_grid()
            .default_row_height(28.0)
            .viewport_width(300.0)
            .viewport_height(200.0);
        g.scroll_to_cell(10, 5);
        // Should set scroll_y/scroll_x since the cell is out of view
        assert!(g.scroll_y > 0.0 || g.scroll_x > 0.0);
    }

    #[test]
    fn row_number_is_optional() {
        let row: DataGridRow<()> = DataGridRow::new(vec![label("test")]);
        assert!(row.row_number.is_none());

        let row: DataGridRow<()> = DataGridRow::new(vec![label("test")]).row_number("1");
        assert_eq!(row.row_number.as_deref(), Some("1"));
    }

    #[test]
    fn datagrid_state_default_is_normal() {
        let g = sample_grid();
        assert_eq!(g.state, TableState::Normal);
    }

    #[test]
    fn datagrid_total_width_and_height() {
        let g = sample_grid()
            .default_col_width(100.0)
            .default_row_height(30.0);
        assert!(g.total_width() > 0.0);
        assert!((g.total_height() - 90.0).abs() < f32::EPSILON);
    }
}
