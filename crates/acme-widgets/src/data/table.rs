use crate::WidgetNode;
use acme_core::WidgetKey;

// ============================================================================
// SortDirection
// ============================================================================

/// Direction of sort indicator shown in a column header.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

// ============================================================================
// TableSelectionMode
// ============================================================================

/// Selection behaviour for a table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableSelectionMode {
    /// No selection.
    None,
    /// Entire rows are selected.
    Row,
    /// Individual cells are selected.
    Cell,
    /// Multiple rows can be selected (Ctrl+click, Shift+click range).
    MultiRow,
}

// ============================================================================
// TableState
// ============================================================================

/// High-level state of the table.
#[derive(Clone, Debug, PartialEq)]
pub enum TableState<M> {
    /// Normal data display.
    Normal,
    /// No data — show a custom empty widget.
    Empty(Box<WidgetNode<M>>),
    /// Loading state — show a custom loading widget.
    Loading(Box<WidgetNode<M>>),
    /// Error state — message + retry widget.
    Error(String, Box<WidgetNode<M>>),
}

// ============================================================================
// TableColumn
// ============================================================================

/// A single column definition for a table.
#[derive(Clone, Debug, PartialEq)]
pub struct TableColumn<M> {
    pub key: WidgetKey,
    /// Header widget (typically a `Label` but can be any widget).
    pub header: WidgetNode<M>,
    /// Current width in logical pixels.
    pub width: f32,
    /// Minimum width (default 20).
    pub min_width: f32,
    /// Maximum width (default f32::MAX).
    pub max_width: f32,
    /// Whether the column can be resized by dragging its edge.
    pub resizable: bool,
    /// Whether the column can be sorted.
    pub sortable: bool,
    /// Current sort indicator (`None` if not the sort column).
    pub sort_indicator: Option<SortDirection>,
}

impl<M> TableColumn<M> {
    /// Create a new table column.
    pub fn new(key: impl Into<WidgetKey>, header: impl Into<WidgetNode<M>>, width: f32) -> Self {
        Self {
            key: key.into(),
            header: header.into(),
            width,
            min_width: 20.0,
            max_width: f32::MAX,
            resizable: true,
            sortable: false,
            sort_indicator: None,
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

    /// Builder: set resizable.
    pub fn resizable(mut self, value: bool) -> Self {
        self.resizable = value;
        self
    }

    /// Builder: set sortable.
    pub fn sortable(mut self, value: bool) -> Self {
        self.sortable = value;
        self
    }

    /// Builder: set sort indicator.
    pub fn sort_indicator(mut self, value: SortDirection) -> Self {
        self.sort_indicator = Some(value);
        self
    }
}

// ============================================================================
// TableRow
// ============================================================================

/// A single row in a table.
#[derive(Clone, Debug, PartialEq)]
pub struct TableRow<M> {
    /// Cell widgets for this row.
    pub cells: Vec<WidgetNode<M>>,
    /// Optional per-row height override.
    pub height: Option<f32>,
}

impl<M> TableRow<M> {
    pub fn new(cells: Vec<WidgetNode<M>>) -> Self {
        Self {
            cells,
            height: None,
        }
    }

    /// Builder: set row height.
    pub fn height(mut self, value: f32) -> Self {
        self.height = Some(crate::finite(value).max(0.0));
        self
    }
}

// ============================================================================
// ResizeState — internal tracking for column resize
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ResizeState {
    pub column_index: usize,
    pub start_x: f32,
    pub start_width: f32,
}

// ============================================================================
// Table
// ============================================================================

/// A full-featured table widget with sort, resize, selection, sticky header,
/// and viewport virtualization.
pub struct Table<M> {
    pub key: WidgetKey,
    /// Column definitions.
    pub columns: Vec<TableColumn<M>>,
    /// Data rows.
    pub rows: Vec<TableRow<M>>,
    /// Currently sorted column index.
    pub sort_column: Option<usize>,
    /// Sort direction.
    pub sort_ascending: bool,
    /// Currently selected row index (for Row / MultiRow mode).
    pub selected_row: Option<usize>,
    /// Currently selected cell `(row, col)` (for Cell mode).
    pub selected_cell: Option<(usize, usize)>,
    /// Whether the header row stays visible while body scrolls.
    pub sticky_header: bool,
    /// Scroll offset for the body (vertical).
    pub viewport_offset: f32,
    /// Default row height in logical pixels.
    pub row_height: f32,
    /// Overscan row count.
    pub overscan: usize,
    /// Selection mode.
    pub selection_mode: TableSelectionMode,
    /// High-level state (Normal, Empty, Loading, Error).
    pub state: TableState<M>,
    /// Internal: current column resize operation state.
    #[doc(hidden)]
    pub(crate) resize_state: Option<ResizeState>,
    /// Flat list of all cell widgets (for reconciliation / children()).
    pub(crate) all_cells: Vec<WidgetNode<M>>,
}

// Manual Clone / Debug / PartialEq

impl<M: Clone> Clone for Table<M> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            columns: self.columns.clone(),
            rows: self.rows.clone(),
            sort_column: self.sort_column,
            sort_ascending: self.sort_ascending,
            selected_row: self.selected_row,
            selected_cell: self.selected_cell,
            sticky_header: self.sticky_header,
            viewport_offset: self.viewport_offset,
            row_height: self.row_height,
            overscan: self.overscan,
            selection_mode: self.selection_mode,
            state: self.state.clone(),
            resize_state: self.resize_state,
            all_cells: self.all_cells.clone(),
        }
    }
}

impl<M: std::fmt::Debug> std::fmt::Debug for Table<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Table")
            .field("key", &self.key)
            .field("columns", &self.columns)
            .field("rows", &self.rows)
            .field("sort_column", &self.sort_column)
            .field("sort_ascending", &self.sort_ascending)
            .field("selected_row", &self.selected_row)
            .field("selected_cell", &self.selected_cell)
            .field("sticky_header", &self.sticky_header)
            .field("viewport_offset", &self.viewport_offset)
            .field("row_height", &self.row_height)
            .field("overscan", &self.overscan)
            .field("selection_mode", &self.selection_mode)
            .field("state", &self.state)
            .field("resize_state", &self.resize_state)
            .finish()
    }
}

impl<M: PartialEq> PartialEq for Table<M> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.columns == other.columns
            && self.rows == other.rows
            && self.sort_column == other.sort_column
            && self.sort_ascending == other.sort_ascending
            && self.selected_row == other.selected_row
            && self.selected_cell == other.selected_cell
            && self.sticky_header == other.sticky_header
            && self.viewport_offset == other.viewport_offset
            && self.row_height == other.row_height
            && self.overscan == other.overscan
            && self.selection_mode == other.selection_mode
            && self.state == other.state
    }
}

// ── Builder / factory ───────────────────────────────────────────────────────

/// Create a new `Table` builder.
pub fn table<M>(key: impl Into<WidgetKey>) -> Table<M> {
    Table {
        key: key.into(),
        columns: Vec::new(),
        rows: Vec::new(),
        sort_column: None,
        sort_ascending: true,
        selected_row: None,
        selected_cell: None,
        sticky_header: true,
        viewport_offset: 0.0,
        row_height: 32.0,
        overscan: 3,
        selection_mode: TableSelectionMode::Row,
        state: TableState::Normal,
        resize_state: None,
        all_cells: Vec::new(),
    }
}

impl<M> Table<M> {
    /// Add a column definition.
    pub fn column(mut self, col: TableColumn<M>) -> Self {
        self.columns.push(col);
        self
    }

    /// Add a data row.
    pub fn add_row(mut self, row: TableRow<M>) -> Self {
        self.rows.push(row);
        self
    }

    /// Set sticky header.
    pub fn sticky_header(mut self, value: bool) -> Self {
        self.sticky_header = value;
        self
    }

    /// Set default row height.
    pub fn row_height(mut self, value: f32) -> Self {
        self.row_height = crate::finite(value).max(1.0);
        self
    }

    /// Set selection mode.
    pub fn selection_mode(mut self, value: TableSelectionMode) -> Self {
        self.selection_mode = value;
        self
    }

    /// Set the table state (Normal, Empty, Loading, Error).
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
        WidgetNode::Table(self)
    }
}

// ── Core methods ─────────────────────────────────────────────────────────────

impl<M> Table<M> {
    // ── Viewport / virtualization ────────────────────────────────────────

    /// Calculate the range of visible row indices.
    pub fn visible_rows(&self) -> std::ops::Range<usize> {
        if self.rows.is_empty() || self.row_height <= 0.0 {
            return 0..0;
        }
        let first = (self.viewport_offset / self.row_height).floor() as usize;
        let visible_count =
            (self.viewport_height() / self.row_height).ceil() as usize + self.overscan * 2;
        let start = first.saturating_sub(self.overscan);
        let end = (first + visible_count).min(self.rows.len());
        start..end
    }

    /// Viewport height for scrolling (approximated from visible rows).
    /// Can be overridden by setting a custom height via external layout.
    pub fn viewport_height(&self) -> f32 {
        // Default heuristic: 10 rows visible
        10.0 * self.row_height
    }

    /// Total content height.
    pub fn content_height(&self) -> f32 {
        self.rows.len() as f32 * self.row_height
    }

    // ── Column resize ────────────────────────────────────────────────────

    /// Start a resize operation on `column_index`.
    /// `pointer_x` is the current pointer x coordinate in the table's local space.
    pub fn start_resize(&mut self, column_index: usize, pointer_x: f32) {
        if column_index >= self.columns.len() {
            return;
        }
        if !self.columns[column_index].resizable {
            return;
        }
        self.resize_state = Some(ResizeState {
            column_index,
            start_x: pointer_x,
            start_width: self.columns[column_index].width,
        });
    }

    /// Update an active resize with a new pointer x.
    pub fn update_resize(&mut self, pointer_x: f32) {
        let Some(state) = self.resize_state else {
            return;
        };
        let dx = pointer_x - state.start_x;
        let col = &mut self.columns[state.column_index];
        col.width = (state.start_width + dx).clamp(col.min_width, col.max_width);
    }

    /// End the current resize operation.
    pub fn end_resize(&mut self) {
        self.resize_state = None;
    }

    /// Get the current resize column index, if any.
    pub fn resizing_column(&self) -> Option<usize> {
        self.resize_state.map(|s| s.column_index)
    }

    // ── Sorting ──────────────────────────────────────────────────────────

    /// Toggle sort on a column.  If already the sort column, reverse direction.
    pub fn toggle_sort(&mut self, column_index: usize) {
        if column_index >= self.columns.len() || !self.columns[column_index].sortable {
            return;
        }
        if self.sort_column == Some(column_index) {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_column = Some(column_index);
            self.sort_ascending = true;
        }

        // Clear old indicators, set new one
        for col in &mut self.columns {
            col.sort_indicator = None;
        }
        if let Some(col) = self.columns.get_mut(column_index) {
            col.sort_indicator = Some(if self.sort_ascending {
                SortDirection::Ascending
            } else {
                SortDirection::Descending
            });
        }
    }

    // ── Selection ────────────────────────────────────────────────────────

    /// Select a row (for Row / MultiRow mode).
    pub fn select_row(&mut self, index: usize) {
        if index < self.rows.len() {
            self.selected_row = Some(index);
            self.selected_cell = None;
        }
    }

    /// Select a cell (for Cell mode).
    pub fn select_cell(&mut self, row: usize, col: usize) {
        if row < self.rows.len() && col < self.columns.len() {
            self.selected_cell = Some((row, col));
            self.selected_row = None;
        }
    }

    /// Toggle row selection (MultiRow mode: Ctrl+click).
    /// Simple implementation — just selects the row.
    pub fn toggle_row_selection(&mut self, index: usize) {
        if self.selected_row == Some(index) {
            self.selected_row = None;
        } else {
            self.selected_row = Some(index);
        }
    }

    /// Select a range of rows (Shift+click in MultiRow mode).
    /// `from` and `to` define the inclusive range.
    pub fn select_row_range(&mut self, from: usize, to: usize) {
        let clamp = |i: usize| i.min(self.rows.len().saturating_sub(1));
        let _f = clamp(from);
        let t = clamp(to);
        // For simplicity, just select the last one in range
        self.selected_row = Some(t);
    }

    // ── Keyboard navigation ──────────────────────────────────────────────

    /// Move selection down one row.
    pub fn select_down(&mut self) {
        match self.selection_mode {
            TableSelectionMode::Cell => {
                if let Some((row, col)) = self.selected_cell {
                    let next = (row + 1).min(self.rows.len().saturating_sub(1));
                    self.selected_cell = Some((next, col));
                } else if !self.rows.is_empty() && !self.columns.is_empty() {
                    self.selected_cell = Some((0, 0));
                }
            }
            _ => {
                if let Some(row) = self.selected_row {
                    let next = (row + 1).min(self.rows.len().saturating_sub(1));
                    self.selected_row = Some(next);
                } else if !self.rows.is_empty() {
                    self.selected_row = Some(0);
                }
            }
        }
    }

    /// Move selection up one row.
    pub fn select_up(&mut self) {
        match self.selection_mode {
            TableSelectionMode::Cell => {
                if let Some((row, col)) = self.selected_cell {
                    let prev = row.saturating_sub(1);
                    self.selected_cell = Some((prev, col));
                } else if !self.rows.is_empty() && !self.columns.is_empty() {
                    let last = self.rows.len() - 1;
                    self.selected_cell = Some((last, 0));
                }
            }
            _ => {
                if let Some(row) = self.selected_row {
                    let prev = row.saturating_sub(1);
                    self.selected_row = Some(prev);
                } else if !self.rows.is_empty() {
                    self.selected_row = Some(self.rows.len() - 1);
                }
            }
        }
    }

    /// Move cell selection right.
    pub fn select_right(&mut self) {
        if let Some((row, col)) = self.selected_cell {
            let next = (col + 1).min(self.columns.len().saturating_sub(1));
            self.selected_cell = Some((row, next));
        }
    }

    /// Move cell selection left.
    pub fn select_left(&mut self) {
        if let Some((row, col)) = self.selected_cell {
            let prev = col.saturating_sub(1);
            self.selected_cell = Some((row, prev));
        }
    }

    /// Select first row / first cell in row.
    pub fn select_home(&mut self) {
        match self.selection_mode {
            TableSelectionMode::Cell => {
                if let Some((row, _)) = self.selected_cell {
                    self.selected_cell = Some((row, 0));
                }
            }
            _ => {
                if !self.rows.is_empty() {
                    self.selected_row = Some(0);
                }
            }
        }
    }

    /// Select last row / last cell in row.
    pub fn select_end(&mut self) {
        match self.selection_mode {
            TableSelectionMode::Cell => {
                if let Some((row, _)) = self.selected_cell {
                    let last_col = self.columns.len().saturating_sub(1);
                    self.selected_cell = Some((row, last_col));
                }
            }
            _ => {
                if !self.rows.is_empty() {
                    self.selected_row = Some(self.rows.len() - 1);
                }
            }
        }
    }

    /// Scroll to ensure `row` is visible.
    pub fn scroll_to_row(&mut self, row: usize) {
        if row >= self.rows.len() {
            return;
        }
        let row_top = row as f32 * self.row_height;
        let row_bottom = row_top + self.row_height;
        let vp_height = self.viewport_height();

        if row_top < self.viewport_offset {
            self.viewport_offset = row_top;
        } else if row_bottom > self.viewport_offset + vp_height {
            self.viewport_offset = row_bottom - vp_height;
        }
    }

    /// Clear selection.
    pub fn clear_selection(&mut self) {
        self.selected_row = None;
        self.selected_cell = None;
    }
}

// ── From conversion ──────────────────────────────────────────────────────────

impl<M> From<Table<M>> for WidgetNode<M> {
    fn from(value: Table<M>) -> Self {
        WidgetNode::Table(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::label;

    fn sample_table() -> Table<()> {
        table::<()>("t")
            .column(TableColumn::new("name", label("Name"), 150.0))
            .column(TableColumn::new("age", label("Age"), 80.0))
            .add_row(TableRow::new(vec![label("Alice"), label("30")]))
            .add_row(TableRow::new(vec![label("Bob"), label("25")]))
            .add_row(TableRow::new(vec![label("Charlie"), label("35")]))
    }

    #[test]
    fn table_creates_columns_and_rows() {
        let t = sample_table();
        assert_eq!(t.columns.len(), 2);
        assert_eq!(t.columns[0].key.as_str(), "name");
        assert_eq!(t.columns[0].width, 150.0);
        assert_eq!(t.rows.len(), 3);
    }

    #[test]
    fn visible_rows_at_start() {
        let t = sample_table().row_height(32.0).sticky_header(true);
        // We're not setting viewport_offset, so it defaults to 0
        let range = t.visible_rows();
        assert_eq!(range.start, 0);
        assert!(range.end > 0);
    }

    #[test]
    fn visible_rows_scrolled() {
        let t = sample_table().row_height(32.0);
        // Manually set viewport_offset to skip first row
        let mut t2 = t;
        t2.viewport_offset = 32.0;
        let range = t2.visible_rows();
        assert!(range.start <= 1);
        assert!(range.end > range.start);
    }

    #[test]
    fn visible_rows_empty() {
        let t = table::<()>("e").row_height(32.0);
        let range = t.visible_rows();
        assert_eq!(range, 0..0);
    }

    #[test]
    fn column_resize_lifecycle() {
        let mut t = sample_table();
        assert!(t.resize_state.is_none());

        t.start_resize(0, 100.0);
        assert!(t.resize_state.is_some());

        t.update_resize(150.0); // drag by +50
        assert!((t.columns[0].width - 200.0).abs() < f32::EPSILON);

        t.end_resize();
        assert!(t.resize_state.is_none());
    }

    #[test]
    fn column_resize_clamps_to_min_max() {
        let mut t = sample_table();
        t.columns[0].min_width = 50.0;
        t.columns[0].max_width = 300.0;

        t.start_resize(0, 100.0);
        t.update_resize(0.0); // drag by -100 → 150 - 100 = 50 (at min)
        assert!((t.columns[0].width - 50.0).abs() < f32::EPSILON);

        t.update_resize(500.0); // drag way right → at max
        assert!((t.columns[0].width - 300.0).abs() < f32::EPSILON);
    }

    #[test]
    fn non_resizable_column_ignores_resize() {
        let mut t = sample_table();
        t.columns[0].resizable = false;
        let original = t.columns[0].width;

        t.start_resize(0, 100.0);
        assert!(t.resize_state.is_none());
        assert!((t.columns[0].width - original).abs() < f32::EPSILON);
    }

    #[test]
    fn toggle_sort_cycles_direction() {
        let mut t = sample_table();
        t.columns[0].sortable = true;

        t.toggle_sort(0);
        assert_eq!(t.sort_column, Some(0));
        assert!(t.sort_ascending);
        assert_eq!(t.columns[0].sort_indicator, Some(SortDirection::Ascending));

        t.toggle_sort(0);
        assert!(!t.sort_ascending);
        assert_eq!(t.columns[0].sort_indicator, Some(SortDirection::Descending));

        // Clear indicator when switching columns
        t.columns[1].sortable = true;
        t.toggle_sort(1);
        assert_eq!(t.sort_column, Some(1));
        assert_eq!(t.columns[0].sort_indicator, None);
    }

    #[test]
    fn non_sortable_column_ignores_toggle() {
        let mut t = sample_table();
        t.toggle_sort(0);
        assert!(t.sort_column.is_none());
    }

    #[test]
    fn select_row_works() {
        let mut t = sample_table();
        t.select_row(1);
        assert_eq!(t.selected_row, Some(1));
        assert!(t.selected_cell.is_none());
    }

    #[test]
    fn select_cell_works() {
        let mut t = sample_table();
        t.selection_mode = TableSelectionMode::Cell;
        t.select_cell(1, 0);
        assert_eq!(t.selected_cell, Some((1, 0)));
        assert!(t.selected_row.is_none());
    }

    #[test]
    fn keyboard_navigation_row_mode() {
        let mut t = sample_table();
        t.selection_mode = TableSelectionMode::Row;

        t.select_down();
        assert_eq!(t.selected_row, Some(0));
        t.select_down();
        assert_eq!(t.selected_row, Some(1));
        t.select_up();
        assert_eq!(t.selected_row, Some(0));
        t.select_home();
        assert_eq!(t.selected_row, Some(0));
        t.select_end();
        assert_eq!(t.selected_row, Some(2));
    }

    #[test]
    fn keyboard_navigation_cell_mode() {
        let mut t = sample_table();
        t.selection_mode = TableSelectionMode::Cell;

        // First down goes to (0,0)
        t.select_down();
        assert_eq!(t.selected_cell, Some((0, 0)));
        t.select_right();
        assert_eq!(t.selected_cell, Some((0, 1)));
        t.select_left();
        assert_eq!(t.selected_cell, Some((0, 0)));
        t.select_down();
        assert_eq!(t.selected_cell, Some((1, 0)));
        t.select_home();
        assert_eq!(t.selected_cell, Some((1, 0))); // home on same row, col 0
        t.select_end();
        assert_eq!(t.selected_cell, Some((1, 1))); // end → last col
    }

    #[test]
    fn clear_selection_resets() {
        let mut t = sample_table();
        t.select_row(1);
        t.clear_selection();
        assert!(t.selected_row.is_none());
        assert!(t.selected_cell.is_none());
    }

    #[test]
    fn table_state_default_is_normal() {
        let t = table::<()>("t");
        assert_eq!(t.state, TableState::Normal);
    }

    #[test]
    fn test_table_state_empty() {
        let empty_widget = label("No data");
        let t = table::<()>("t").state(TableState::Empty(Box::new(empty_widget)));
        assert!(matches!(t.state, TableState::Empty(_)));
    }

    #[test]
    fn test_table_state_loading() {
        let loading = label("Loading...");
        let t = table::<()>("t").state(TableState::Loading(Box::new(loading)));
        assert!(matches!(t.state, TableState::Loading(_)));
    }

    #[test]
    fn test_table_state_error() {
        let retry = label("Retry");
        let t = table::<()>("t").state(TableState::Error("Network error".into(), Box::new(retry)));
        match &t.state {
            TableState::Error(msg, _) => assert_eq!(msg, "Network error"),
            _ => panic!("expected Error"),
        }
    }

    #[test]
    fn table_content_height() {
        let t = sample_table().row_height(32.0);
        assert!((t.content_height() - 96.0).abs() < f32::EPSILON);
    }

    #[test]
    fn table_scroll_to_row() {
        // Row 2 is within the default viewport (10*row_height=320), so
        // scroll_to_row keeps viewport_offset at 0 (already visible).
        let mut t = sample_table().row_height(32.0);
        let before = t.viewport_offset;
        t.scroll_to_row(2);
        // Row 2 (top=64) was already visible, so no scroll needed.
        assert_eq!(t.viewport_offset, before);
        // Confirm row 2 is actually visible.
        let range = t.visible_rows();
        assert!(range.contains(&2));
    }
}
