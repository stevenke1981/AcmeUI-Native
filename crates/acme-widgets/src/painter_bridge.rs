//! Immediate-mode painting bridge between [`WidgetNode`], [`LayoutSnapshot`],
//! [`Theme`], and [`acme_core::painter::Painter`].

use crate::overlay::{Dialog, Popover, PopoverPlacement};
use crate::{Canvas, CardVariant, WidgetNode};
use acme_core::{Color, Logical, NodeId, Painter, Rect, RectStyle};
use acme_layout::{
    LayoutEngine, LayoutNode, LayoutRect, LayoutSnapshot, Length, WidgetLayoutContext,
};
use acme_theme::{Theme, ThemeColor};
use std::collections::HashMap;

pub(crate) const OVERLAY_SYNTHETIC_BASE: u64 = 1 << 56;
const DIALOG_CONTENT_PADDING: f32 = 16.0;
const POPOVER_GAP: f32 = 4.0;
const POPOVER_MAX_WIDTH: f32 = 320.0;
const POPOVER_MAX_HEIGHT: f32 = 480.0;

#[inline]
fn safe_coord(v: f32) -> f32 {
    if v.is_finite() { v } else { 0.0 }
}

#[inline]
fn safe_extent(v: f32) -> f32 {
    if v.is_finite() { v.max(0.0) } else { 0.0 }
}

pub(crate) fn theme_layout_context(theme: &Theme) -> WidgetLayoutContext {
    WidgetLayoutContext {
        body_font_size: theme.typography.body,
        body_line_height: (theme.typography.body * theme.typography.line_height).max(1.0),
        label_font_size: theme.typography.label,
        control_height: theme.control_heights.md,
        scale_factor: 1.0,
    }
}

pub(crate) fn default_layout_context() -> WidgetLayoutContext {
    WidgetLayoutContext {
        body_font_size: 14.0,
        body_line_height: 21.0,
        label_font_size: 13.0,
        control_height: 34.0,
        scale_factor: 1.0,
    }
}

fn max_layout_node_id(node: &LayoutNode) -> u64 {
    node.children
        .iter()
        .map(max_layout_node_id)
        .fold(node.id.get(), u64::max)
}

fn shift_snapshot(raw: &LayoutSnapshot, dx: f32, dy: f32) -> LayoutSnapshot {
    let mut shifted = LayoutSnapshot::default();
    let ox = safe_coord(dx);
    let oy = safe_coord(dy);
    for (id, rect) in raw.iter() {
        shifted.insert(
            id,
            LayoutRect {
                x: rect.x + ox,
                y: rect.y + oy,
                width: rect.width,
                height: rect.height,
            },
        );
    }
    shifted
}

pub(crate) fn layout_detached_subtree<M>(
    widget: &WidgetNode<M>,
    context: &WidgetLayoutContext,
    origin_x: f32,
    origin_y: f32,
    available_w: f32,
    available_h: f32,
    next_synthetic: &mut u64,
) -> Option<(LayoutNode, LayoutSnapshot)> {
    let start = (*next_synthetic).max(OVERLAY_SYNTHETIC_BASE);
    let layout = widget.to_layout_subtree_with_context(NodeId::new(start), context);
    *next_synthetic = max_layout_node_id(&layout)
        .saturating_add(1)
        .max(start.saturating_add(1));
    let w = safe_extent(available_w).max(1.0);
    let h = safe_extent(available_h).max(1.0);
    let raw = LayoutEngine::new().compute(&layout, (w, h)).ok()?;
    let shifted = shift_snapshot(&raw, origin_x, origin_y);
    Some((layout, shifted))
}

pub(crate) fn resolve_dialog_geometry<M>(
    d: &Dialog<M>,
    layout: &LayoutNode,
    snapshot: &LayoutSnapshot,
    context: &WidgetLayoutContext,
    next_synthetic: &mut u64,
) -> (Rect<Logical>, Option<(LayoutNode, LayoutSnapshot)>) {
    let raw_rect =
        node_logical_rect(snapshot, layout.id).unwrap_or_else(|| Rect::new(0.0, 0.0, 0.0, 0.0));
    let fallback_w = match d.width {
        Length::Px(w) if w > 0.0 => w,
        Length::Percent(p) if p > 0.0 => (480.0 * p).max(120.0),
        _ => 480.0,
    };
    let box_w = if raw_rect.size.width.get() > 0.0 {
        raw_rect.size.width.get()
    } else {
        fallback_w
    };
    let max_content_h = match d.height {
        Length::Px(h) if h > 0.0 => (h - DIALOG_CONTENT_PADDING * 2.0).max(1.0),
        _ => 600.0,
    };
    let content_avail_w = (box_w - DIALOG_CONTENT_PADDING * 2.0).max(1.0);

    if layout.children.is_empty() {
        let content_x = raw_rect.origin.x.get() + DIALOG_CONTENT_PADDING;
        let content_y = raw_rect.origin.y.get() + DIALOG_CONTENT_PADDING;
        let detached = layout_detached_subtree(
            &d.content,
            context,
            content_x,
            content_y,
            content_avail_w,
            max_content_h,
            next_synthetic,
        );
        let measured_h = detached
            .as_ref()
            .and_then(|(l, s)| s.get(l.id).map(|r| r.height))
            .unwrap_or(0.0);
        let box_h = if raw_rect.size.height.get() > 0.0 {
            raw_rect.size.height.get()
        } else {
            (measured_h + DIALOG_CONTENT_PADDING * 2.0).max(48.0)
        };
        (
            Rect::new(
                raw_rect.origin.x.get(),
                raw_rect.origin.y.get(),
                box_w,
                box_h,
            ),
            detached,
        )
    } else {
        let box_h = if raw_rect.size.height.get() > 0.0 {
            raw_rect.size.height.get()
        } else {
            match d.height {
                Length::Px(h) if h > 0.0 => h,
                _ => 200.0,
            }
        };
        (
            Rect::new(
                raw_rect.origin.x.get(),
                raw_rect.origin.y.get(),
                box_w,
                box_h,
            ),
            None,
        )
    }
}

pub(crate) fn resolve_popover_overlay<M>(
    p: &Popover<M>,
    anchor_layout: &LayoutNode,
    snapshot: &LayoutSnapshot,
    context: &WidgetLayoutContext,
    next_synthetic: &mut u64,
) -> Option<(Rect<Logical>, LayoutNode, LayoutSnapshot)> {
    if !p.open {
        return None;
    }
    let content = p.children.get(1)?;
    let anchor = node_logical_rect(snapshot, anchor_layout.id)?;
    let (content_layout, local_snap) = layout_detached_subtree(
        content,
        context,
        0.0,
        0.0,
        POPOVER_MAX_WIDTH,
        POPOVER_MAX_HEIGHT,
        next_synthetic,
    )?;
    let root_rect = local_snap.get(content_layout.id).copied()?;
    let w = safe_extent(root_rect.width).max(1.0);
    let h = safe_extent(root_rect.height).max(1.0);
    let ax = anchor.origin.x.get();
    let ay = anchor.origin.y.get();
    let aw = anchor.size.width.get();
    let ah = anchor.size.height.get();
    let (ox, oy) = match p.placement {
        PopoverPlacement::Bottom => (ax, ay + ah + POPOVER_GAP),
        PopoverPlacement::Top => (ax, (ay - h - POPOVER_GAP).max(0.0)),
        PopoverPlacement::Right => (ax + aw + POPOVER_GAP, ay),
        PopoverPlacement::Left => ((ax - w - POPOVER_GAP).max(0.0), ay),
    };
    let shifted = shift_snapshot(&local_snap, ox, oy);
    Some((Rect::new(ox, oy, w, h), content_layout, shifted))
}

pub(crate) fn resolved_cell_snapshot(
    table_rect: Option<Rect<Logical>>,
    row_rect: Option<Rect<Logical>>,
    col_widths: &[f32],
    col_idx: usize,
    col_count: usize,
    cell_layout: &LayoutNode,
    snapshot: &LayoutSnapshot,
) -> Option<(Rect<Logical>, Option<LayoutSnapshot>)> {
    let raw = node_logical_rect(snapshot, cell_layout.id);
    if let Some(r) = raw
        && has_area(r)
    {
        return Some((r, None));
    }
    let count = col_count.max(1);
    let col_width_at = |idx: usize| -> f32 {
        col_widths
            .get(idx)
            .copied()
            .filter(|w| w.is_finite() && *w > 0.0)
            .unwrap_or_else(|| {
                let total_w = row_rect
                    .map(|r| r.size.width.get())
                    .filter(|w| *w > 0.0)
                    .or_else(|| table_rect.map(|r| r.size.width.get()).filter(|w| *w > 0.0))
                    .unwrap_or(120.0 * count as f32);
                total_w / count as f32
            })
    };
    let origin = row_rect
        .map(|r| r.origin)
        .or_else(|| raw.map(|r| r.origin))
        .or_else(|| table_rect.map(|r| r.origin))?;
    let x = origin.x.get() + (0..col_idx).map(col_width_at).sum::<f32>();
    let y = raw.map(|r| r.origin.y.get()).unwrap_or(origin.y.get());
    let w = col_width_at(col_idx);
    let h = raw
        .map(|r| r.size.height.get())
        .filter(|h| *h > 0.0)
        .or_else(|| row_rect.map(|r| r.size.height.get()).filter(|h| *h > 0.0))
        .unwrap_or(24.0);
    let resolved = Rect::new(x, y, w, h);
    let mut cell_snap = LayoutSnapshot::default();
    cell_snap.insert(
        cell_layout.id,
        LayoutRect {
            x,
            y,
            width: w,
            height: h,
        },
    );
    Some((resolved, Some(cell_snap)))
}

/// Convert a semantic [`ThemeColor`] into an [`acme_core::geometry::Color`].
#[inline]
pub fn theme_color(color: ThemeColor) -> Color {
    Color::rgba(color.red, color.green, color.blue, color.alpha)
}

/// Convert a laid-out [`LayoutRect`] into a logical [`Rect<Logical>`], sanitizing
/// any non-finite or negative extents.
#[inline]
pub fn layout_rect_to_logical(rect: LayoutRect) -> Rect<Logical> {
    Rect::new(
        safe_coord(rect.x),
        safe_coord(rect.y),
        safe_extent(rect.width),
        safe_extent(rect.height),
    )
}

/// Resolve the logical bounding rectangle of `node_id` from `snapshot`.
#[inline]
pub fn node_logical_rect(snapshot: &LayoutSnapshot, node_id: NodeId) -> Option<Rect<Logical>> {
    snapshot.get(node_id).copied().map(layout_rect_to_logical)
}

/// Paint a single [`Canvas`] widget into `painter` using `rect` and semantic `theme` tokens.
pub fn paint_canvas_with_theme<M>(
    canvas: &Canvas<M>,
    rect: Rect<Logical>,
    theme: &Theme,
    painter: &mut Painter<'_>,
) {
    canvas.paint_into(painter, rect, theme);
}

/// Paint only [`Canvas`] nodes in a widget tree (plus `ScrollView` clipping/scrolling),
/// leaving standard widgets to an external primitive renderer.
pub fn paint_widget_tree_canvases<M>(
    widget: &WidgetNode<M>,
    layout: &LayoutNode,
    snapshot: &LayoutSnapshot,
    theme: &Theme,
    painter: &mut Painter<'_>,
) {
    let empty_scroll = HashMap::new();
    let mut next_synthetic = OVERLAY_SYNTHETIC_BASE;
    paint_tree_inner(
        widget,
        layout,
        snapshot,
        theme,
        painter,
        &empty_scroll,
        false,
        &mut next_synthetic,
    );
}

/// Paint a widget tree into `painter` using `layout`, `snapshot`, and semantic [`Theme`] tokens,
/// emitting primitives for [`Canvas`], [`Card`], [`Button`], [`TextInput`], [`Separator`],
/// and open [`Dialog`] panels while clipping [`ScrollView`] viewports.
pub fn paint_widget_tree_overlays_or_primitives<M>(
    widget: &WidgetNode<M>,
    layout: &LayoutNode,
    snapshot: &LayoutSnapshot,
    theme: &Theme,
    painter: &mut Painter<'_>,
) {
    let empty_scroll = HashMap::new();
    let mut next_synthetic = OVERLAY_SYNTHETIC_BASE;
    paint_tree_inner(
        widget,
        layout,
        snapshot,
        theme,
        painter,
        &empty_scroll,
        true,
        &mut next_synthetic,
    );
}

/// Variant of [`paint_widget_tree_overlays_or_primitives`] that applies per-`ScrollView`
/// scroll offsets `(scroll_x, scroll_y)` alongside viewport clipping.
pub fn paint_widget_tree_with_scroll<M>(
    widget: &WidgetNode<M>,
    layout: &LayoutNode,
    snapshot: &LayoutSnapshot,
    theme: &Theme,
    scroll_offsets: &HashMap<NodeId, (f32, f32)>,
    painter: &mut Painter<'_>,
) {
    let mut next_synthetic = OVERLAY_SYNTHETIC_BASE;
    paint_tree_inner(
        widget,
        layout,
        snapshot,
        theme,
        painter,
        scroll_offsets,
        true,
        &mut next_synthetic,
    );
}

/// Convenience helper that compiles sequential layout IDs starting at `root_id`
/// and invokes [`paint_widget_tree_overlays_or_primitives`].
pub fn paint_widget_tree_from_root_id<M: Clone>(
    widget: &WidgetNode<M>,
    root_id: NodeId,
    snapshot: &LayoutSnapshot,
    theme: &Theme,
    painter: &mut Painter<'_>,
) {
    let layout = widget.to_layout(root_id);
    paint_widget_tree_overlays_or_primitives(widget, &layout, snapshot, theme, painter);
}

fn has_area(rect: Rect<Logical>) -> bool {
    rect.size.width.get() > 0.0 && rect.size.height.get() > 0.0
}

#[allow(clippy::too_many_arguments)]
fn paint_tree_inner<M>(
    widget: &WidgetNode<M>,
    layout: &LayoutNode,
    snapshot: &LayoutSnapshot,
    theme: &Theme,
    painter: &mut Painter<'_>,
    scroll_offsets: &HashMap<NodeId, (f32, f32)>,
    draw_primitives: bool,
    next_synthetic: &mut u64,
) {
    match widget {
        WidgetNode::Tooltip(t) => {
            paint_tree_inner(
                &t.child,
                layout,
                snapshot,
                theme,
                painter,
                scroll_offsets,
                draw_primitives,
                next_synthetic,
            );
        }
        WidgetNode::Popover(p) => {
            if let Some(anchor) = p.children.first() {
                paint_tree_inner(
                    anchor,
                    layout,
                    snapshot,
                    theme,
                    painter,
                    scroll_offsets,
                    draw_primitives,
                    next_synthetic,
                );
            }
            if p.open
                && let Some(content) = p.children.get(1)
            {
                let ctx = theme_layout_context(theme);
                if let Some((_overlay_rect, content_layout, overlay_snapshot)) =
                    resolve_popover_overlay(p, layout, snapshot, &ctx, next_synthetic)
                {
                    paint_tree_inner(
                        content,
                        &content_layout,
                        &overlay_snapshot,
                        theme,
                        painter,
                        scroll_offsets,
                        draw_primitives,
                        next_synthetic,
                    );
                }
            }
        }
        WidgetNode::Canvas(c) => {
            if let Some(rect) = node_logical_rect(snapshot, layout.id)
                && has_area(rect)
            {
                paint_canvas_with_theme(c, rect, theme, painter);
            }
        }
        WidgetNode::ScrollView(sv) => {
            if let Some(rect) = node_logical_rect(snapshot, layout.id) {
                let (sx, sy) = scroll_offsets
                    .get(&layout.id)
                    .copied()
                    .unwrap_or((0.0, 0.0));
                painter.with_clip(rect, |p| {
                    p.with_offset(-safe_coord(sx), -safe_coord(sy), |p| {
                        for (child, child_layout) in sv.children.iter().zip(&layout.children) {
                            paint_tree_inner(
                                child,
                                child_layout,
                                snapshot,
                                theme,
                                p,
                                scroll_offsets,
                                draw_primitives,
                                next_synthetic,
                            );
                        }
                    });
                });
            } else {
                for (child, child_layout) in sv.children.iter().zip(&layout.children) {
                    paint_tree_inner(
                        child,
                        child_layout,
                        snapshot,
                        theme,
                        painter,
                        scroll_offsets,
                        draw_primitives,
                        next_synthetic,
                    );
                }
            }
        }
        WidgetNode::Card(card) => {
            if draw_primitives
                && let Some(rect) = node_logical_rect(snapshot, layout.id)
                && has_area(rect)
            {
                let bg = card.background_color.unwrap_or(match card.variant {
                    CardVariant::Muted => theme.colors.muted,
                    _ => theme.colors.surface,
                });
                let (border_width, border_color) = match card.variant {
                    // Interactive marks the keyboard-owner / focused card: draw the
                    // focus ring token, 2px unless the card styles its own width.
                    CardVariant::Interactive => (
                        card.style.border_width.unwrap_or(2.0),
                        theme_color(theme.colors.ring),
                    ),
                    CardVariant::Outlined | CardVariant::Elevated => {
                        (1.0, theme_color(theme.colors.border))
                    }
                    CardVariant::Plain | CardVariant::Muted => (0.0, Color::TRANSPARENT),
                };
                painter.rect(
                    rect,
                    RectStyle::solid(theme_color(bg))
                        .radius(card.border_radius.unwrap_or(theme.radii.md))
                        .border(border_width, border_color),
                );
            }
            for (child, child_layout) in card.children.iter().zip(&layout.children) {
                paint_tree_inner(
                    child,
                    child_layout,
                    snapshot,
                    theme,
                    painter,
                    scroll_offsets,
                    draw_primitives,
                    next_synthetic,
                );
            }
        }
        WidgetNode::Button(b) => {
            if draw_primitives
                && let Some(rect) = node_logical_rect(snapshot, layout.id)
                && has_area(rect)
            {
                let resolved = b.resolve_style(theme, crate::inputs::ButtonState::default());
                painter.rect(
                    rect,
                    RectStyle::solid(theme_color(resolved.background))
                        .radius(theme.radii.md)
                        .border(1.0, theme_color(resolved.border)),
                );
            }
        }
        WidgetNode::TextInput(ti) => {
            if draw_primitives
                && let Some(rect) = node_logical_rect(snapshot, layout.id)
                && has_area(rect)
            {
                let fill = if ti.disabled {
                    theme.colors.disabled_bg
                } else {
                    theme.colors.surface
                };
                let border = if ti.invalid {
                    theme.colors.danger
                } else {
                    theme.colors.input
                };
                painter.rect(
                    rect,
                    RectStyle::solid(theme_color(fill))
                        .radius(theme.radii.sm)
                        .border(1.0, theme_color(border)),
                );
            }
        }
        WidgetNode::Separator(_) => {
            if draw_primitives
                && let Some(rect) = node_logical_rect(snapshot, layout.id)
                && has_area(rect)
            {
                painter.fill_rect(rect, theme_color(theme.colors.border));
            }
        }
        WidgetNode::Dialog(d) => {
            if d.open {
                let ctx = theme_layout_context(theme);
                let (panel_rect, detached) =
                    resolve_dialog_geometry(d, layout, snapshot, &ctx, next_synthetic);
                if draw_primitives && has_area(panel_rect) {
                    painter.rect(
                        panel_rect,
                        RectStyle::solid(theme_color(theme.colors.surface_overlay))
                            .radius(theme.radii.lg)
                            .border(1.0, theme_color(theme.colors.border)),
                    );
                }
                if let Some((content_layout, content_snapshot)) = detached {
                    paint_tree_inner(
                        &d.content,
                        &content_layout,
                        &content_snapshot,
                        theme,
                        painter,
                        scroll_offsets,
                        draw_primitives,
                        next_synthetic,
                    );
                } else if let Some(content_layout) = layout.children.first() {
                    paint_tree_inner(
                        &d.content,
                        content_layout,
                        snapshot,
                        theme,
                        painter,
                        scroll_offsets,
                        draw_primitives,
                        next_synthetic,
                    );
                }
            }
        }
        WidgetNode::Table(t) => {
            let table_rect = node_logical_rect(snapshot, layout.id);
            let col_widths: Vec<f32> = t.columns.iter().map(|c| c.width).collect();
            let ctx = theme_layout_context(theme);
            let skip_headers = layout.children.len().saturating_sub(t.rows.len());
            for (row, row_layout) in t.rows.iter().zip(layout.children.iter().skip(skip_headers)) {
                let row_rect = node_logical_rect(snapshot, row_layout.id);
                let col_count = row.cells.len();
                for (col_idx, (cell, cell_layout)) in
                    row.cells.iter().zip(&row_layout.children).enumerate()
                {
                    paint_table_grid_cell(
                        cell,
                        cell_layout,
                        table_rect,
                        row_rect,
                        &col_widths,
                        col_idx,
                        col_count,
                        snapshot,
                        theme,
                        &ctx,
                        painter,
                        scroll_offsets,
                        draw_primitives,
                        next_synthetic,
                    );
                }
            }
        }
        WidgetNode::DataGrid(g) => {
            let table_rect = node_logical_rect(snapshot, layout.id);
            let col_widths: Vec<f32> = g.columns.iter().map(|c| c.width).collect();
            let ctx = theme_layout_context(theme);
            let skip_headers = layout.children.len().saturating_sub(g.rows.len());
            for (row, row_layout) in g.rows.iter().zip(layout.children.iter().skip(skip_headers)) {
                let row_rect = node_logical_rect(snapshot, row_layout.id);
                let col_count = row.cells.len();
                for (col_idx, (cell, cell_layout)) in
                    row.cells.iter().zip(&row_layout.children).enumerate()
                {
                    paint_table_grid_cell(
                        cell,
                        cell_layout,
                        table_rect,
                        row_rect,
                        &col_widths,
                        col_idx,
                        col_count,
                        snapshot,
                        theme,
                        &ctx,
                        painter,
                        scroll_offsets,
                        draw_primitives,
                        next_synthetic,
                    );
                }
            }
        }
        WidgetNode::Row(_)
        | WidgetNode::Column(_)
        | WidgetNode::Stack(_)
        | WidgetNode::VirtualList(_)
        | WidgetNode::NavRail(_)
        | WidgetNode::Sidebar(_)
        | WidgetNode::TabBar(_)
        | WidgetNode::Breadcrumb(_) => {
            for (child, child_layout) in widget.children().iter().zip(&layout.children) {
                paint_tree_inner(
                    child,
                    child_layout,
                    snapshot,
                    theme,
                    painter,
                    scroll_offsets,
                    draw_primitives,
                    next_synthetic,
                );
            }
        }
        WidgetNode::Label(_) | WidgetNode::Menu(_) | WidgetNode::Tree(_) => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_table_grid_cell<M>(
    cell: &WidgetNode<M>,
    cell_layout: &LayoutNode,
    table_rect: Option<Rect<Logical>>,
    row_rect: Option<Rect<Logical>>,
    col_widths: &[f32],
    col_idx: usize,
    col_count: usize,
    snapshot: &LayoutSnapshot,
    theme: &Theme,
    ctx: &WidgetLayoutContext,
    painter: &mut Painter<'_>,
    scroll_offsets: &HashMap<NodeId, (f32, f32)>,
    draw_primitives: bool,
    next_synthetic: &mut u64,
) {
    let Some((cell_rect, override_snap)) = resolved_cell_snapshot(
        table_rect,
        row_rect,
        col_widths,
        col_idx,
        col_count,
        cell_layout,
        snapshot,
    ) else {
        return;
    };
    if !cell.children().is_empty() && cell_layout.children.is_empty() {
        if let Some((sub_layout, sub_snapshot)) = layout_detached_subtree(
            cell,
            ctx,
            cell_rect.origin.x.get(),
            cell_rect.origin.y.get(),
            cell_rect.size.width.get(),
            cell_rect.size.height.get(),
            next_synthetic,
        ) {
            paint_tree_inner(
                cell,
                &sub_layout,
                &sub_snapshot,
                theme,
                painter,
                scroll_offsets,
                draw_primitives,
                next_synthetic,
            );
        }
    } else {
        let snap = override_snap.as_ref().unwrap_or(snapshot);
        paint_tree_inner(
            cell,
            cell_layout,
            snap,
            theme,
            painter,
            scroll_offsets,
            draw_primitives,
            next_synthetic,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{button, canvas, card, column, scroll_view};
    use acme_core::DrawCommand;
    use acme_layout::LayoutEngine;

    #[test]
    fn theme_color_and_rect_helpers_convert_cleanly() {
        let tc = Theme::dark().colors.primary;
        let c = theme_color(tc);
        assert_eq!(c, Color::rgba(tc.red, tc.green, tc.blue, tc.alpha));

        let lr = LayoutRect {
            x: f32::NAN,
            y: 12.0,
            width: -5.0,
            height: 24.0,
        };
        let logical = layout_rect_to_logical(lr);
        assert_eq!(logical, Rect::new(0.0, 12.0, 0.0, 24.0));
    }

    #[test]
    fn paint_widget_tree_invokes_canvas_with_theme_tokens() {
        let theme = Theme::light();
        let expected_color = theme_color(theme.colors.accent);
        let root = column::<()>()
            .child(
                canvas::<()>()
                    .key("bar")
                    .size(120.0, 40.0)
                    .paint(|painter, rect, th| {
                        painter.fill_rect(rect, theme_color(th.colors.accent));
                    }),
            )
            .build();

        let layout = root.to_layout(NodeId::new(1));
        let snapshot = LayoutEngine::new()
            .compute(&layout, (400.0, 200.0))
            .unwrap();
        let mut painter = Painter::new();
        paint_widget_tree_overlays_or_primitives(&root, &layout, &snapshot, &theme, &mut painter);
        let scene = painter.finish();

        let quads: Vec<_> = scene
            .commands()
            .iter()
            .filter_map(|cmd| match cmd {
                DrawCommand::Quad(q) => Some(q),
                _ => None,
            })
            .collect();
        assert_eq!(quads.len(), 1);
        assert_eq!(quads[0].color, expected_color);
        assert_eq!(quads[0].rect.size.width.get(), 120.0);
        assert_eq!(quads[0].rect.size.height.get(), 40.0);
    }

    #[test]
    fn paint_widget_tree_clips_scroll_view_and_skips_offscreen_canvas() {
        let theme = Theme::dark();
        let root = scroll_view::<()>("sv")
            .viewport_height(50.0)
            .child(
                column::<()>()
                    .child(
                        canvas::<()>()
                            .key("top")
                            .size(100.0, 40.0)
                            .paint(|p, r, t| p.fill_rect(r, theme_color(t.colors.primary))),
                    )
                    .child(
                        canvas::<()>()
                            .key("offscreen")
                            .size(100.0, 40.0)
                            .paint(|p, r, t| p.fill_rect(r, theme_color(t.colors.danger))),
                    ),
            )
            .build();

        let layout = root.to_layout(NodeId::new(1));
        let snapshot = LayoutEngine::new().compute(&layout, (200.0, 50.0)).unwrap();
        let mut offsets = HashMap::new();
        offsets.insert(layout.id, (0.0, 60.0));

        let mut painter = Painter::new();
        paint_widget_tree_with_scroll(&root, &layout, &snapshot, &theme, &offsets, &mut painter);
        let scene = painter.finish();

        let quads: Vec<_> = scene
            .commands()
            .iter()
            .filter_map(|cmd| match cmd {
                DrawCommand::Quad(q) => Some(q),
                _ => None,
            })
            .collect();
        // Only the second canvas (y=40..80 shifted by -60 => -20..20) intersects clip 0..50!
        // First canvas (y=0..40 shifted by -60 => -60..-20) is culled.
        assert_eq!(quads.len(), 1);
        assert_eq!(quads[0].color, theme_color(theme.colors.danger));
    }

    #[test]
    fn paint_widget_tree_canvases_skips_button_and_card_chrome() {
        let theme = Theme::light();
        let root = card::<()>()
            .child(button::<()>("ok", "OK").on_click(()))
            .child(
                canvas::<()>()
                    .size(50.0, 20.0)
                    .paint(|p, r, t| p.fill_rect(r, theme_color(t.colors.info))),
            )
            .build();

        let layout = root.to_layout(NodeId::new(1));
        let snapshot = LayoutEngine::new()
            .compute(&layout, (200.0, 200.0))
            .unwrap();
        let mut painter = Painter::new();
        paint_widget_tree_canvases(&root, &layout, &snapshot, &theme, &mut painter);
        let scene = painter.finish();

        assert_eq!(scene.commands().len(), 1);
    }

    /// Paints a single fixed-size card and returns its chrome quad's
    /// (border_width, border_color).
    fn card_chrome(builder: crate::CardBuilder<()>, theme: &Theme) -> (f32, Color) {
        use acme_layout::Length;
        use acme_style::prelude::*;
        let root = builder.w(Length::px(120.0)).h(Length::px(60.0)).build();
        let layout = root.to_layout(NodeId::new(1));
        let snapshot = LayoutEngine::new()
            .compute(&layout, (200.0, 200.0))
            .unwrap();
        let mut painter = Painter::new();
        paint_widget_tree_overlays_or_primitives(&root, &layout, &snapshot, theme, &mut painter);
        let scene = painter.finish();
        let quad = scene
            .commands()
            .iter()
            .find_map(|cmd| match cmd {
                DrawCommand::Quad(q) => Some(q),
                _ => None,
            })
            .expect("card chrome quad");
        (quad.border_width, quad.border_color)
    }

    #[test]
    fn interactive_card_draws_ring_border_and_other_variants_are_unchanged() {
        use acme_style::prelude::*;
        let theme = Theme::light();
        let ring = theme_color(theme.colors.ring);
        let border = theme_color(theme.colors.border);
        assert_ne!(ring, border, "test needs distinct ring/border tokens");

        // Interactive: focus ring colour at the default 2px.
        assert_eq!(
            card_chrome(card::<()>().variant(CardVariant::Interactive), &theme),
            (2.0, ring)
        );
        // Interactive honours an explicit style border width.
        assert_eq!(
            card_chrome(
                card::<()>()
                    .variant(CardVariant::Interactive)
                    .border_width(3.0),
                &theme
            ),
            (3.0, ring)
        );
        // Outlined / Elevated keep the 1px theme border.
        for variant in [CardVariant::Outlined, CardVariant::Elevated] {
            assert_eq!(
                card_chrome(card::<()>().variant(variant), &theme),
                (1.0, border),
                "{variant:?}"
            );
        }
        // Plain / Muted stay borderless.
        for variant in [CardVariant::Plain, CardVariant::Muted] {
            assert_eq!(
                card_chrome(card::<()>().variant(variant), &theme).0,
                0.0,
                "{variant:?}"
            );
        }
    }

    #[test]
    fn paint_widget_tree_renders_open_dialog_content_and_popover_overlay() {
        use crate::{dialog, label, popover, row};

        let theme = Theme::light();
        let root = column::<()>()
            .child(
                popover(
                    "pop",
                    row::<()>()
                        .child(button("anchor_a", "A"))
                        .child(button("anchor_b", "B")),
                )
                .open(true)
                .content(
                    card::<()>().child(
                        canvas::<()>()
                            .key("pop_canvas")
                            .size(48.0, 24.0)
                            .paint(|p, r, t| p.fill_rect(r, theme_color(t.colors.success))),
                    ),
                )
                .build(),
            )
            .child(
                dialog(
                    "dlg",
                    column::<()>()
                        .child(label("Proceed?"))
                        .child(button("confirm", "Confirm"))
                        .child(
                            canvas::<()>()
                                .key("dlg_canvas")
                                .size(32.0, 16.0)
                                .paint(|p, r, t| p.fill_rect(r, theme_color(t.colors.accent))),
                        )
                        .build(),
                )
                .open(true)
                .width(280.0)
                .build(),
            )
            .build();

        let layout = root.to_layout(NodeId::new(1));
        let snapshot = LayoutEngine::new()
            .compute(&layout, (400.0, 400.0))
            .unwrap();
        let mut painter = Painter::new();
        paint_widget_tree_overlays_or_primitives(&root, &layout, &snapshot, &theme, &mut painter);
        let scene = painter.finish_checked().expect("valid scene");

        let quads: Vec<_> = scene
            .commands()
            .iter()
            .filter_map(|cmd| match cmd {
                DrawCommand::Quad(q) => Some(q),
                _ => None,
            })
            .collect();
        // Should include anchor buttons, popover canvas/card, dialog panel, confirm button, and dialog canvas.
        assert!(
            quads
                .iter()
                .any(|q| q.color == theme_color(theme.colors.success)),
            "popover overlay canvas must paint"
        );
        assert!(
            quads
                .iter()
                .any(|q| q.color == theme_color(theme.colors.accent)),
            "dialog content canvas must paint even with Length::Auto height"
        );
    }

    #[test]
    fn paint_widget_tree_renders_headerless_table_and_compound_cells() {
        use crate::{DataGridColumn, DataGridRow, TableRow, datagrid, row, table};

        let theme = Theme::light();
        let root = column::<()>()
            .child(
                table::<()>("headless")
                    .add_row(TableRow::new(vec![
                        row::<()>()
                            .child(button("row0_btn", "Row0"))
                            .child(
                                canvas::<()>()
                                    .size(20.0, 12.0)
                                    .paint(|p, r, t| p.fill_rect(r, theme_color(t.colors.warning))),
                            )
                            .build(),
                    ]))
                    .build(),
            )
            .child(
                datagrid::<()>("grid")
                    .column(DataGridColumn::new("col", crate::label("Col"), 160.0))
                    .add_row(DataGridRow::new(vec![
                        row::<()>()
                            .child(
                                canvas::<()>()
                                    .size(20.0, 12.0)
                                    .paint(|p, r, t| p.fill_rect(r, theme_color(t.colors.danger))),
                            )
                            .build(),
                    ]))
                    .build(),
            )
            .build();

        let layout = root.to_layout(NodeId::new(1));
        let snapshot = LayoutEngine::new()
            .compute(&layout, (300.0, 200.0))
            .unwrap();
        let mut painter = Painter::new();
        paint_widget_tree_overlays_or_primitives(&root, &layout, &snapshot, &theme, &mut painter);
        let scene = painter.finish_checked().expect("valid scene");

        let colors: Vec<_> = scene
            .commands()
            .iter()
            .filter_map(|cmd| match cmd {
                DrawCommand::Quad(q) => Some(q.color),
                _ => None,
            })
            .collect();
        assert!(
            colors.contains(&theme_color(theme.colors.warning)),
            "headerless table row 0 compound cell must render"
        );
        assert!(
            colors.contains(&theme_color(theme.colors.danger)),
            "datagrid compound cell must render"
        );
    }
}
