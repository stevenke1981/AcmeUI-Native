//! Common widget types brought into scope for convenience.
pub use crate::canvas::{Canvas, CanvasBuilder, CanvasPaintFn, canvas};
pub use crate::data::{
    CellMerge, DataGrid, DataGridColumn, DataGridRow, SortDirection, Table, TableColumn, TableRow,
    TableSelectionMode, TableState, Tree, TreeNode, VariableHeightCache, VirtualList, VisibleNode,
};
pub use crate::dirty_tracker::{DirtyState, FrameDirtyTracker};
pub use crate::foundations::{
    Card, CardVariant, Container, Label, LivePoliteness, ScrollView, Separator,
};
pub use crate::hit_bridge::{
    DispatchedPointerEvents, HitRegistry, build_hit_layer, build_hit_layer_from_root_id,
    build_hit_layer_with_binding, build_hit_layer_with_scroll, dispatch_pointer_button,
    dispatch_pointer_move, dispatch_pointer_outcomes,
};
pub use crate::inputs::{
    Button, ButtonSize, ButtonState, ButtonVariant, ResolvedButtonStyle, TextInput,
};
pub use crate::navigation::{
    Breadcrumb, BreadcrumbSegment, NavItem, NavRail, Sidebar, TabBar, TabItem,
};
pub use crate::overlay::{Dialog, Menu, MenuItem, Popover, PopoverPlacement, Tooltip};
pub use crate::overlay_manager::{OverlayLayer, OverlayManager};
pub use crate::painter_bridge::{
    layout_rect_to_logical, node_logical_rect, paint_canvas_with_theme, paint_widget_tree_canvases,
    paint_widget_tree_from_root_id, paint_widget_tree_overlays_or_primitives,
    paint_widget_tree_with_scroll, theme_color,
};
pub use crate::visual_state::VisualState;
pub use acme_core::WidgetKey;
