//! Immediate-mode hit-testing bridge connecting [`WidgetNode`], [`LayoutSnapshot`],
//! [`acme_core::hit_layer::HitTestLayer`], and [`acme_core::hit_layer::ClickDetector`].

use crate::WidgetNode;
use crate::painter_bridge::{
    OVERLAY_SYNTHETIC_BASE, default_layout_context, layout_detached_subtree, node_logical_rect,
    resolve_dialog_geometry, resolve_popover_overlay, resolved_cell_snapshot,
};
use crate::retained_binding::RetainedBinding;
use acme_core::{
    ClickDetector, CursorHint, HitBehavior, HitRegionSpec, HitResult, HitTestLayer, HitZLayer,
    Logical, NodeId, Point, PointerOutcome, Rect,
};
use acme_layout::{LayoutNode, LayoutSnapshot, WidgetLayoutContext};
use std::collections::HashMap;

/// Outcome of routing pointer events against a [`HitTestLayer`] and message table.
#[derive(Clone, Debug, PartialEq)]
pub struct DispatchedPointerEvents<M> {
    /// Raw outcomes returned by [`ClickDetector`].
    pub outcomes: Vec<PointerOutcome>,
    /// Application messages emitted by primary clicks in `outcomes`.
    pub messages: Vec<M>,
    /// Currently hovered node after handling the event.
    pub hovered: Option<NodeId>,
    /// Currently pressed node after handling the event.
    pub pressed: Option<NodeId>,
    /// Effective pointer cursor hint after handling the event.
    pub cursor: CursorHint,
}

/// Built hit-test layer paired with clickable `NodeId -> M` message resolution.
#[derive(Clone, Debug)]
pub struct HitRegistry<M> {
    pub layer: HitTestLayer,
    pub messages: HashMap<NodeId, M>,
}

impl<M> Default for HitRegistry<M> {
    fn default() -> Self {
        Self {
            layer: HitTestLayer::new(),
            messages: HashMap::new(),
        }
    }
}

impl<M: Clone> HitRegistry<M> {
    /// Perform a single hit test at `point`.
    pub fn hit_test(&self, point: Point<Logical>) -> Option<HitResult> {
        self.layer.hit_test(point)
    }

    /// Resolve the message bound to `id`, if any.
    pub fn message_for(&self, id: NodeId) -> Option<&M> {
        self.messages.get(&id)
    }

    /// Forward a pointer move through `detector` and collect hover/drag state.
    pub fn handle_pointer_move(
        &self,
        detector: &mut ClickDetector,
        point: Point<Logical>,
    ) -> DispatchedPointerEvents<M> {
        dispatch_pointer_move(detector, &self.layer, &self.messages, point)
    }

    /// Forward a pointer button transition through `detector` and resolve emitted messages.
    pub fn handle_pointer_button(
        &self,
        detector: &mut ClickDetector,
        button: u16,
        pressed: bool,
        point: Point<Logical>,
        time_secs: f64,
    ) -> DispatchedPointerEvents<M> {
        dispatch_pointer_button(
            detector,
            &self.layer,
            &self.messages,
            button,
            pressed,
            point,
            time_secs,
        )
    }
}

/// Build a [`HitRegistry`] from a widget tree, its matching [`LayoutNode`], [`LayoutSnapshot`],
/// and viewport bounds (used for full-screen modal scrims).
pub fn build_hit_layer<M: Clone>(
    widget: &WidgetNode<M>,
    layout: &LayoutNode,
    snapshot: &LayoutSnapshot,
    viewport: Rect<Logical>,
) -> HitRegistry<M> {
    let empty_scroll = HashMap::new();
    build_hit_layer_with_scroll(widget, layout, snapshot, viewport, &empty_scroll)
}

/// Build a [`HitRegistry`] with explicit per-`ScrollView` scroll offsets `(scroll_x, scroll_y)`.
pub fn build_hit_layer_with_scroll<M: Clone>(
    widget: &WidgetNode<M>,
    layout: &LayoutNode,
    snapshot: &LayoutSnapshot,
    viewport: Rect<Logical>,
    scroll_offsets: &HashMap<NodeId, (f32, f32)>,
) -> HitRegistry<M> {
    let mut layer = HitTestLayer::new();
    let mut messages = HashMap::new();
    let mut next_synthetic = OVERLAY_SYNTHETIC_BASE;
    collect_hit_regions(
        widget,
        layout,
        snapshot,
        viewport,
        scroll_offsets,
        &mut layer,
        &mut messages,
        &mut next_synthetic,
    );
    HitRegistry { layer, messages }
}

/// Convenience builder deriving the [`LayoutNode`] sequentially from `root_id`.
pub fn build_hit_layer_from_root_id<M: Clone>(
    widget: &WidgetNode<M>,
    root_id: NodeId,
    snapshot: &LayoutSnapshot,
    viewport: Rect<Logical>,
) -> HitRegistry<M> {
    let layout = widget.to_layout(root_id);
    build_hit_layer(widget, &layout, snapshot, viewport)
}

/// Convenience builder deriving the [`LayoutNode`] from a [`RetainedBinding`].
pub fn build_hit_layer_with_binding<M: Clone>(
    widget: &WidgetNode<M>,
    binding: &RetainedBinding<'_>,
    context: &WidgetLayoutContext,
    snapshot: &LayoutSnapshot,
    viewport: Rect<Logical>,
) -> HitRegistry<M> {
    let layout = widget.to_layout_with_binding(binding, context);
    build_hit_layer(widget, &layout, snapshot, viewport)
}

/// Extract application messages for primary-button [`PointerOutcome::Clicked`] events.
pub fn dispatch_pointer_outcomes<M: Clone>(
    outcomes: &[PointerOutcome],
    messages: &HashMap<NodeId, M>,
) -> Vec<M> {
    let mut emitted = Vec::new();
    for outcome in outcomes {
        if let PointerOutcome::Clicked { id, button, .. } = outcome
            && *button == 0
            && let Some(msg) = messages.get(id)
        {
            emitted.push(msg.clone());
        }
    }
    emitted
}

/// Process a pointer move with `detector`, `layer`, and `messages`.
pub fn dispatch_pointer_move<M: Clone>(
    detector: &mut ClickDetector,
    layer: &HitTestLayer,
    messages: &HashMap<NodeId, M>,
    point: Point<Logical>,
) -> DispatchedPointerEvents<M> {
    let outcomes = detector.on_pointer_move(layer, point);
    let emitted = dispatch_pointer_outcomes(&outcomes, messages);
    DispatchedPointerEvents {
        outcomes,
        messages: emitted,
        hovered: detector.hovered(),
        pressed: detector.pressed(),
        cursor: detector.cursor(),
    }
}

/// Process a pointer button press/release with `detector`, `layer`, and `messages`.
pub fn dispatch_pointer_button<M: Clone>(
    detector: &mut ClickDetector,
    layer: &HitTestLayer,
    messages: &HashMap<NodeId, M>,
    button: u16,
    pressed: bool,
    point: Point<Logical>,
    time_secs: f64,
) -> DispatchedPointerEvents<M> {
    let outcomes = detector.on_pointer_button(layer, button, pressed, point, time_secs);
    let emitted = dispatch_pointer_outcomes(&outcomes, messages);
    DispatchedPointerEvents {
        outcomes,
        messages: emitted,
        hovered: detector.hovered(),
        pressed: detector.pressed(),
        cursor: detector.cursor(),
    }
}

fn has_area(rect: Rect<Logical>) -> bool {
    rect.size.width.get() > 0.0 && rect.size.height.get() > 0.0
}

#[allow(clippy::too_many_arguments)]
fn collect_hit_regions<M: Clone>(
    widget: &WidgetNode<M>,
    layout: &LayoutNode,
    snapshot: &LayoutSnapshot,
    viewport: Rect<Logical>,
    scroll_offsets: &HashMap<NodeId, (f32, f32)>,
    layer: &mut HitTestLayer,
    messages: &mut HashMap<NodeId, M>,
    next_synthetic: &mut u64,
) {
    match widget {
        WidgetNode::Tooltip(t) => {
            collect_hit_regions(
                &t.child,
                layout,
                snapshot,
                viewport,
                scroll_offsets,
                layer,
                messages,
                next_synthetic,
            );
        }
        WidgetNode::Popover(p) => {
            if let Some(anchor) = p.children.first() {
                collect_hit_regions(
                    anchor,
                    layout,
                    snapshot,
                    viewport,
                    scroll_offsets,
                    layer,
                    messages,
                    next_synthetic,
                );
            }
            if p.open
                && let Some(content) = p.children.get(1)
            {
                let ctx = default_layout_context();
                if let Some((overlay_rect, content_layout, overlay_snapshot)) =
                    resolve_popover_overlay(p, layout, snapshot, &ctx, next_synthetic)
                {
                    let overlay_box_id = NodeId::new(*next_synthetic);
                    *next_synthetic = (*next_synthetic).saturating_add(1);
                    layer.with_z_layer(HitZLayer::Overlay, |layer| {
                        if has_area(overlay_rect) {
                            layer.push_region(
                                HitRegionSpec::rect(overlay_box_id, overlay_rect)
                                    .enabled(false)
                                    .behavior(HitBehavior::Opaque),
                            );
                        }
                        collect_hit_regions(
                            content,
                            &content_layout,
                            &overlay_snapshot,
                            viewport,
                            scroll_offsets,
                            layer,
                            messages,
                            next_synthetic,
                        );
                    });
                }
            }
        }
        WidgetNode::Button(b) => {
            if let Some(rect) = node_logical_rect(snapshot, layout.id)
                && has_area(rect)
            {
                let cursor = if b.disabled {
                    CursorHint::NotAllowed
                } else {
                    CursorHint::Pointer
                };
                layer.push_region(
                    HitRegionSpec::rect(layout.id, rect)
                        .enabled(!b.disabled)
                        .behavior(HitBehavior::Opaque)
                        .cursor(cursor),
                );
                if let Some(msg) = b.activate() {
                    messages.insert(layout.id, msg.clone());
                }
            }
        }
        WidgetNode::TextInput(ti) => {
            if let Some(rect) = node_logical_rect(snapshot, layout.id)
                && has_area(rect)
            {
                let cursor = if ti.disabled {
                    CursorHint::NotAllowed
                } else {
                    CursorHint::Text
                };
                layer.push_region(
                    HitRegionSpec::rect(layout.id, rect)
                        .enabled(!ti.disabled)
                        .behavior(HitBehavior::Opaque)
                        .cursor(cursor),
                );
                if !ti.disabled
                    && let Some(msg) = &ti.message
                {
                    messages.insert(layout.id, msg.clone());
                }
            }
        }
        WidgetNode::Canvas(c) => {
            if let Some(rect) = node_logical_rect(snapshot, layout.id)
                && has_area(rect)
                && (c.on_click.is_some() || c.cursor.is_some())
            {
                let default_cursor = if c.disabled {
                    CursorHint::NotAllowed
                } else if c.on_click.is_some() {
                    CursorHint::Pointer
                } else {
                    CursorHint::Default
                };
                let behavior = if c.on_click.is_some() {
                    HitBehavior::Opaque
                } else {
                    HitBehavior::Translucent
                };
                layer.push_region(
                    HitRegionSpec::rect(layout.id, rect)
                        .enabled(!c.disabled)
                        .behavior(behavior)
                        .cursor(c.cursor.unwrap_or(default_cursor)),
                );
                if let Some(msg) = c.activate() {
                    messages.insert(layout.id, msg.clone());
                }
            }
        }
        WidgetNode::Row(c) | WidgetNode::Column(c) | WidgetNode::Stack(c) => {
            layer.with_node(layout.id, |layer| {
                if let Some(msg) = &c.message
                    && let Some(rect) = node_logical_rect(snapshot, layout.id)
                    && has_area(rect)
                {
                    layer.push_region(
                        HitRegionSpec::rect(layout.id, rect)
                            .enabled(true)
                            .behavior(HitBehavior::Opaque)
                            .cursor(CursorHint::Pointer),
                    );
                    messages.insert(layout.id, msg.clone());
                }
                for (child, child_layout) in c.children.iter().zip(&layout.children) {
                    collect_hit_regions(
                        child,
                        child_layout,
                        snapshot,
                        viewport,
                        scroll_offsets,
                        layer,
                        messages,
                        next_synthetic,
                    );
                }
            });
        }
        WidgetNode::ScrollView(sv) => {
            layer.with_node(layout.id, |layer| {
                if let Some(rect) = node_logical_rect(snapshot, layout.id) {
                    let (sx, sy) = scroll_offsets
                        .get(&layout.id)
                        .copied()
                        .unwrap_or((0.0, 0.0));
                    layer.with_scroll_viewport(rect, sx, sy, |layer| {
                        for (child, child_layout) in sv.children.iter().zip(&layout.children) {
                            collect_hit_regions(
                                child,
                                child_layout,
                                snapshot,
                                viewport,
                                scroll_offsets,
                                layer,
                                messages,
                                next_synthetic,
                            );
                        }
                    });
                } else {
                    for (child, child_layout) in sv.children.iter().zip(&layout.children) {
                        collect_hit_regions(
                            child,
                            child_layout,
                            snapshot,
                            viewport,
                            scroll_offsets,
                            layer,
                            messages,
                            next_synthetic,
                        );
                    }
                }
            });
        }
        WidgetNode::Dialog(d) => {
            if d.open {
                if d.modal && has_area(viewport) {
                    layer.push_scrim(layout.id, viewport, HitZLayer::Modal, false);
                }
                let ctx = default_layout_context();
                let (dialog_rect, detached) =
                    resolve_dialog_geometry(d, layout, snapshot, &ctx, next_synthetic);
                let modal_content_z = HitZLayer::Modal.z_value() + 1;
                layer.with_z_index(modal_content_z, |layer| {
                    if has_area(dialog_rect) {
                        layer.push_region(
                            HitRegionSpec::rect(layout.id, dialog_rect)
                                .enabled(false)
                                .behavior(HitBehavior::Opaque),
                        );
                    }
                    if let Some((content_layout, content_snapshot)) = detached {
                        collect_hit_regions(
                            &d.content,
                            &content_layout,
                            &content_snapshot,
                            viewport,
                            scroll_offsets,
                            layer,
                            messages,
                            next_synthetic,
                        );
                    } else if let Some(content_layout) = layout.children.first() {
                        collect_hit_regions(
                            &d.content,
                            content_layout,
                            snapshot,
                            viewport,
                            scroll_offsets,
                            layer,
                            messages,
                            next_synthetic,
                        );
                    }
                });
            }
        }
        WidgetNode::Menu(m) => {
            if m.open
                && let Some(rect) = node_logical_rect(snapshot, layout.id)
                && has_area(rect)
            {
                layer.with_z_layer(HitZLayer::Overlay, |layer| {
                    layer.push_region(
                        HitRegionSpec::rect(layout.id, rect)
                            .enabled(false)
                            .behavior(HitBehavior::Opaque),
                    );
                });
            }
        }
        WidgetNode::Card(_)
        | WidgetNode::VirtualList(_)
        | WidgetNode::NavRail(_)
        | WidgetNode::Sidebar(_)
        | WidgetNode::TabBar(_)
        | WidgetNode::Breadcrumb(_) => {
            layer.with_node(layout.id, |layer| {
                for (child, child_layout) in widget.children().iter().zip(&layout.children) {
                    collect_hit_regions(
                        child,
                        child_layout,
                        snapshot,
                        viewport,
                        scroll_offsets,
                        layer,
                        messages,
                        next_synthetic,
                    );
                }
            });
        }
        WidgetNode::Table(t) => {
            layer.with_node(layout.id, |layer| {
                let table_rect = node_logical_rect(snapshot, layout.id);
                let col_widths: Vec<f32> = t.columns.iter().map(|c| c.width).collect();
                let ctx = default_layout_context();
                let skip_headers = layout.children.len().saturating_sub(t.rows.len());
                for (row, row_layout) in
                    t.rows.iter().zip(layout.children.iter().skip(skip_headers))
                {
                    let row_rect = node_logical_rect(snapshot, row_layout.id);
                    let col_count = row.cells.len();
                    for (col_idx, (cell, cell_layout)) in
                        row.cells.iter().zip(&row_layout.children).enumerate()
                    {
                        collect_table_grid_cell(
                            cell,
                            cell_layout,
                            table_rect,
                            row_rect,
                            &col_widths,
                            col_idx,
                            col_count,
                            snapshot,
                            viewport,
                            scroll_offsets,
                            &ctx,
                            layer,
                            messages,
                            next_synthetic,
                        );
                    }
                }
            });
        }
        WidgetNode::DataGrid(g) => {
            layer.with_node(layout.id, |layer| {
                let table_rect = node_logical_rect(snapshot, layout.id);
                let col_widths: Vec<f32> = g.columns.iter().map(|c| c.width).collect();
                let ctx = default_layout_context();
                let skip_headers = layout.children.len().saturating_sub(g.rows.len());
                for (row, row_layout) in
                    g.rows.iter().zip(layout.children.iter().skip(skip_headers))
                {
                    let row_rect = node_logical_rect(snapshot, row_layout.id);
                    let col_count = row.cells.len();
                    for (col_idx, (cell, cell_layout)) in
                        row.cells.iter().zip(&row_layout.children).enumerate()
                    {
                        collect_table_grid_cell(
                            cell,
                            cell_layout,
                            table_rect,
                            row_rect,
                            &col_widths,
                            col_idx,
                            col_count,
                            snapshot,
                            viewport,
                            scroll_offsets,
                            &ctx,
                            layer,
                            messages,
                            next_synthetic,
                        );
                    }
                }
            });
        }
        WidgetNode::Label(_) | WidgetNode::Separator(_) | WidgetNode::Tree(_) => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_table_grid_cell<M: Clone>(
    cell: &WidgetNode<M>,
    cell_layout: &LayoutNode,
    table_rect: Option<Rect<Logical>>,
    row_rect: Option<Rect<Logical>>,
    col_widths: &[f32],
    col_idx: usize,
    col_count: usize,
    snapshot: &LayoutSnapshot,
    viewport: Rect<Logical>,
    scroll_offsets: &HashMap<NodeId, (f32, f32)>,
    ctx: &WidgetLayoutContext,
    layer: &mut HitTestLayer,
    messages: &mut HashMap<NodeId, M>,
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
            collect_hit_regions(
                cell,
                &sub_layout,
                &sub_snapshot,
                viewport,
                scroll_offsets,
                layer,
                messages,
                next_synthetic,
            );
        }
    } else {
        let snap = override_snap.as_ref().unwrap_or(snapshot);
        collect_hit_regions(
            cell,
            cell_layout,
            snap,
            viewport,
            scroll_offsets,
            layer,
            messages,
            next_synthetic,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        button, canvas, column, dialog, label, nav_item, nav_rail, row, scroll_view, sidebar,
        stack, tab_bar, text_input,
    };
    use acme_layout::{LayoutEngine, WidgetLayoutContext};

    fn test_ctx() -> WidgetLayoutContext {
        WidgetLayoutContext {
            body_font_size: 14.0,
            body_line_height: 20.0,
            label_font_size: 13.0,
            control_height: 36.0,
            scale_factor: 1.0,
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    enum Msg {
        Back,
        Front,
        CanvasTap,
        RowTap,
        Tab(usize),
        Nav(usize),
        SidebarBtn,
        InputFocus,
        First,
        Second,
    }

    fn viewport_rect(w: f32, h: f32) -> Rect<Logical> {
        Rect::new(0.0, 0.0, w, h)
    }

    #[test]
    fn stack_z_order_occlusion_picks_topmost_child() {
        let root = stack::<Msg>()
            .size(200.0, 100.0)
            .child(button::<Msg>("back", "Back").on_click(Msg::Back))
            .child(
                canvas::<Msg>()
                    .key("front")
                    .size(200.0, 100.0)
                    .on_click(Msg::Front),
            )
            .build();

        let layout = root.to_layout(NodeId::new(1));
        let snapshot = LayoutEngine::new()
            .compute(&layout, (200.0, 100.0))
            .unwrap();
        let registry = build_hit_layer(&root, &layout, &snapshot, viewport_rect(200.0, 100.0));

        let mut detector = ClickDetector::new();
        let center = Point::new(50.0, 20.0);
        let mv = registry.handle_pointer_move(&mut detector, center);
        assert_eq!(mv.cursor, CursorHint::Pointer);

        let _ = registry.handle_pointer_button(&mut detector, 0, true, center, 0.1);
        let up = registry.handle_pointer_button(&mut detector, 0, false, center, 0.2);
        assert_eq!(up.messages, vec![Msg::Front]);
    }

    #[test]
    fn scroll_view_clips_outside_items_and_shifts_scrolled_items() {
        let root = scroll_view::<Msg>("sv")
            .viewport_height(50.0)
            .child(button::<Msg>("b1", "One button").on_click(Msg::First))
            .child(button::<Msg>("b2", "Two button").on_click(Msg::Second))
            .build();

        let layout = root.to_layout_with_context(NodeId::new(1), &test_ctx());
        let snapshot = LayoutEngine::new()
            .compute(&layout, (200.0, 200.0))
            .unwrap();
        let sv_rect = node_logical_rect(&snapshot, layout.id).unwrap();
        let b2_rect = node_logical_rect(&snapshot, layout.children[1].id).unwrap();
        assert!(
            b2_rect.origin.y.get() >= 30.0,
            "b2 should overflow below initial y=0: {b2_rect:?}, sv={sv_rect:?}"
        );

        let scroll_y = b2_rect.origin.y.get();
        let mut offsets = HashMap::new();
        offsets.insert(layout.id, (0.0, scroll_y));
        let registry = build_hit_layer_with_scroll(
            &root,
            &layout,
            &snapshot,
            viewport_rect(200.0, 200.0),
            &offsets,
        );

        let mut detector = ClickDetector::new();
        let in_viewport = Point::new(b2_rect.origin.x.get() + 4.0, 10.0);
        let _ = registry.handle_pointer_button(&mut detector, 0, true, in_viewport, 0.1);
        let up = registry.handle_pointer_button(&mut detector, 0, false, in_viewport, 0.2);
        assert_eq!(up.messages, vec![Msg::Second]);

        // Below scroll viewport (y=60 > viewport_height 50): clipped!
        assert!(
            registry
                .hit_test(Point::new(b2_rect.origin.x.get() + 4.0, 60.0))
                .is_none()
        );
    }

    #[test]
    fn disabled_button_and_disabled_canvas_block_activation_and_show_not_allowed() {
        let root = stack::<Msg>()
            .size(200.0, 100.0)
            .child(button::<Msg>("underlying", "Under").on_click(Msg::Back))
            .child(
                canvas::<Msg>()
                    .key("disabled_canvas")
                    .size(200.0, 100.0)
                    .on_click(Msg::CanvasTap)
                    .disabled(true),
            )
            .build();

        let layout = root.to_layout(NodeId::new(1));
        let snapshot = LayoutEngine::new()
            .compute(&layout, (200.0, 100.0))
            .unwrap();
        let registry = build_hit_layer(&root, &layout, &snapshot, viewport_rect(200.0, 100.0));

        // Disabled opaque canvas occludes underlying button and yields no hit!
        assert!(registry.hit_test(Point::new(20.0, 10.0)).is_none());

        let btn_only = button::<Msg>("disabled_btn", "No")
            .disabled(true)
            .on_click(Msg::First);
        let btn_layout = btn_only.to_layout(NodeId::new(1));
        let btn_snap = LayoutEngine::new()
            .compute(&btn_layout, (200.0, 100.0))
            .unwrap();
        let btn_reg = build_hit_layer(
            &btn_only,
            &btn_layout,
            &btn_snap,
            viewport_rect(200.0, 100.0),
        );
        assert!(btn_reg.hit_test(Point::new(10.0, 10.0)).is_none());
        assert!(btn_reg.messages.is_empty());
    }

    #[test]
    fn open_modal_dialog_scrim_blocks_underlying_clicks() {
        let open_root = stack::<Msg>()
            .size(300.0, 200.0)
            .child(button::<Msg>("under", "Under").on_click(Msg::Back))
            .child(
                dialog::<Msg>("dlg", label("Confirm"))
                    .open(true)
                    .modal(true)
                    .build(),
            )
            .build();

        let layout = open_root.to_layout(NodeId::new(1));
        let snapshot = LayoutEngine::new()
            .compute(&layout, (300.0, 200.0))
            .unwrap();
        let registry = build_hit_layer(&open_root, &layout, &snapshot, viewport_rect(300.0, 200.0));

        // Scrim blocks hits across the whole viewport, even outside dialog box or on underlying button.
        assert!(registry.hit_test(Point::new(10.0, 10.0)).is_none());
        assert!(registry.hit_test(Point::new(290.0, 190.0)).is_none());

        // When closed, the underlying button is clickable.
        let closed_root = stack::<Msg>()
            .size(300.0, 200.0)
            .child(button::<Msg>("under", "Under").on_click(Msg::Back))
            .child(
                dialog::<Msg>("dlg", label("Confirm"))
                    .open(false)
                    .modal(true)
                    .build(),
            )
            .build();
        let layout2 = closed_root.to_layout_with_context(NodeId::new(1), &test_ctx());
        let snapshot2 = LayoutEngine::new()
            .compute(&layout2, (300.0, 200.0))
            .unwrap();
        let registry2 = build_hit_layer(
            &closed_root,
            &layout2,
            &snapshot2,
            viewport_rect(300.0, 200.0),
        );
        let hit = registry2
            .hit_test(Point::new(10.0, 10.0))
            .expect("underlying button clickable when modal closed");
        assert_eq!(registry2.message_for(hit.id), Some(&Msg::Back));
    }

    #[test]
    fn clickable_containers_text_input_and_navigation_widgets_dispatch() {
        let root = column::<Msg>()
            .gap(4.0)
            .child(
                row::<Msg>()
                    .size(160.0, 32.0)
                    .on_click(Msg::RowTap)
                    .child(label("Checkbox-like row")),
            )
            .child(
                text_input::<Msg>("search")
                    .placeholder("Search")
                    .on_change(Msg::InputFocus),
            )
            .child(
                tab_bar::<Msg>("tabs")
                    .selected(0)
                    .item(crate::navigation::TabItem::new("One").on_click(Msg::Tab(0)))
                    .item(crate::navigation::TabItem::new("Two").on_click(Msg::Tab(1)))
                    .build(),
            )
            .child(
                nav_rail::<Msg>("rail")
                    .item(nav_item("Home").on_click(Msg::Nav(0)))
                    .build(),
            )
            .child(
                sidebar::<Msg>("sb")
                    .child(button::<Msg>("sb_btn", "Nav").on_click(Msg::SidebarBtn))
                    .build(),
            )
            .build();

        let layout = root.to_layout_with_context(NodeId::new(1), &test_ctx());
        let snapshot = LayoutEngine::new()
            .compute(&layout, (400.0, 400.0))
            .unwrap();
        let registry = build_hit_layer(&root, &layout, &snapshot, viewport_rect(400.0, 400.0));

        // Verify TextInput cursor is Text
        let input_id = layout.children[1].id;
        let input_rect = node_logical_rect(&snapshot, input_id).unwrap();
        let input_hit = registry
            .hit_test(Point::new(
                input_rect.origin.x.get() + 5.0,
                input_rect.origin.y.get() + 5.0,
            ))
            .expect("text input hit");
        assert_eq!(input_hit.cursor, CursorHint::Text);
        assert_eq!(registry.message_for(input_hit.id), Some(&Msg::InputFocus));

        // Clickable Row fires RowTap
        let row_rect = node_logical_rect(&snapshot, layout.children[0].id).unwrap();
        let row_pt = Point::new(
            row_rect.origin.x.get() + 10.0,
            row_rect.origin.y.get() + 10.0,
        );
        let mut detector = ClickDetector::new();
        let _ = registry.handle_pointer_button(&mut detector, 0, true, row_pt, 0.1);
        let res = registry.handle_pointer_button(&mut detector, 0, false, row_pt, 0.2);
        assert_eq!(res.messages, vec![Msg::RowTap]);

        // Unselected Tab(1), Nav(0), SidebarBtn all registered in message map
        let values: Vec<&Msg> = registry.messages.values().collect();
        assert!(values.contains(&&Msg::Tab(1)));
        assert!(values.contains(&&Msg::Nav(0)));
        assert!(values.contains(&&Msg::SidebarBtn));
    }

    #[test]
    fn open_modal_dialog_allows_clicking_confirm_button_and_blocks_underlay() {
        let root = stack::<Msg>()
            .size(400.0, 300.0)
            .child(button::<Msg>("under", "Under").on_click(Msg::Back))
            .child(
                dialog::<Msg>(
                    "confirm_dlg",
                    column::<Msg>()
                        .gap(8.0)
                        .child(label("Save changes?"))
                        .child(
                            row::<Msg>()
                                .gap(8.0)
                                .child(button::<Msg>("cancel", "Cancel").on_click(Msg::First))
                                .child(button::<Msg>("ok", "Confirm").on_click(Msg::Second))
                                .build(),
                        )
                        .build(),
                )
                .open(true)
                .modal(true)
                .width(300.0)
                .build(),
            )
            .build();

        let layout = root.to_layout_with_context(NodeId::new(1), &test_ctx());
        let snapshot = LayoutEngine::new()
            .compute(&layout, (400.0, 300.0))
            .unwrap();
        let registry = build_hit_layer(&root, &layout, &snapshot, viewport_rect(400.0, 300.0));

        // Both dialog buttons are registered with non-zero hit regions above the modal scrim.
        let mut confirm_id = None;
        for (id, msg) in &registry.messages {
            if *msg == Msg::Second {
                confirm_id = Some(*id);
            }
        }
        let confirm_id = confirm_id.expect("dialog Confirm button must register a hit region");
        let entry = registry
            .layer
            .to_flat_entries()
            .into_iter()
            .find(|e| e.id == confirm_id)
            .expect("Confirm region present in HitTestLayer");
        let r = entry.bounds;
        let center = Point::new(
            r.origin.x.get() + r.size.width.get() * 0.5,
            r.origin.y.get() + r.size.height.get() * 0.5,
        );

        let mut detector = ClickDetector::new();
        let _ = registry.handle_pointer_button(&mut detector, 0, true, center, 0.1);
        let up = registry.handle_pointer_button(&mut detector, 0, false, center, 0.2);
        assert_eq!(up.messages, vec![Msg::Second]);

        // Scrim still blocks clicks outside the dialog.
        assert!(registry.hit_test(Point::new(390.0, 290.0)).is_none());
    }

    #[test]
    fn open_popover_places_overlay_above_anchor_without_stealing_anchor_children() {
        use crate::{card, popover};

        let anchor = row::<Msg>()
            .gap(8.0)
            .child(button::<Msg>("a1", "Open").on_click(Msg::First))
            .child(button::<Msg>("a2", "Extra").on_click(Msg::Back))
            .build();
        let root = column::<Msg>()
            .child(
                popover("pop", anchor)
                    .open(true)
                    .content(
                        card::<Msg>()
                            .child(button::<Msg>("menu_item", "Pick Item").on_click(Msg::Front)),
                    )
                    .build(),
            )
            .build();

        let layout = root.to_layout_with_context(NodeId::new(1), &test_ctx());
        let snapshot = LayoutEngine::new()
            .compute(&layout, (400.0, 300.0))
            .unwrap();
        let registry = build_hit_layer(&root, &layout, &snapshot, viewport_rect(400.0, 300.0));

        // Anchor's second child stays at Base z-layer, not promoted to Overlay.
        let a2_id = layout.children[0].children[1].id;
        let a2_rect = node_logical_rect(&snapshot, a2_id).unwrap();
        let a2_center = Point::new(
            a2_rect.origin.x.get() + a2_rect.size.width.get() * 0.5,
            a2_rect.origin.y.get() + a2_rect.size.height.get() * 0.5,
        );
        assert_eq!(
            registry.hit_test(a2_center).map(|h| (h.id, h.z_layer)),
            Some((a2_id, HitZLayer::Base.z_value()))
        );

        // Popover overlay button is registered on HitZLayer::Overlay and clickable.
        let menu_id = registry
            .messages
            .iter()
            .find_map(|(id, m)| (*m == Msg::Front).then_some(*id))
            .expect("popover item registered");
        let entry = registry
            .layer
            .to_flat_entries()
            .into_iter()
            .find(|e| e.id == menu_id)
            .expect("popover item region");
        let r = entry.bounds;
        let center = Point::new(
            r.origin.x.get() + r.size.width.get() * 0.5,
            r.origin.y.get() + r.size.height.get() * 0.5,
        );
        assert_eq!(
            registry.hit_test(center).map(|h| (h.id, h.z_layer)),
            Some((menu_id, HitZLayer::Overlay.z_value()))
        );
        let mut detector = ClickDetector::new();
        let _ = registry.handle_pointer_button(&mut detector, 0, true, center, 0.1);
        let up = registry.handle_pointer_button(&mut detector, 0, false, center, 0.2);
        assert_eq!(up.messages, vec![Msg::Front]);
    }

    #[test]
    fn headerless_table_and_compound_cells_register_clickable_regions() {
        use crate::{DataGridColumn, DataGridRow, TableRow, datagrid, table};

        let root = column::<Msg>()
            .gap(8.0)
            .child(
                table::<Msg>("headless")
                    .add_row(TableRow::new(vec![
                        button::<Msg>("row0", "Row0").on_click(Msg::First),
                        row::<Msg>()
                            .child(button::<Msg>("compound_btn", "Edit").on_click(Msg::Second))
                            .build(),
                    ]))
                    .build(),
            )
            .child(
                datagrid::<Msg>("grid")
                    .column(DataGridColumn::new("c1", label("C1"), 150.0))
                    .add_row(DataGridRow::new(vec![
                        row::<Msg>()
                            .child(button::<Msg>("grid_btn", "Inspect").on_click(Msg::Front))
                            .build(),
                    ]))
                    .build(),
            )
            .build();

        let layout = root.to_layout_with_context(NodeId::new(1), &test_ctx());
        let snapshot = LayoutEngine::new()
            .compute(&layout, (400.0, 300.0))
            .unwrap();
        let registry = build_hit_layer(&root, &layout, &snapshot, viewport_rect(400.0, 300.0));

        let flat = registry.layer.to_flat_entries();
        for expected in [Msg::First, Msg::Second, Msg::Front] {
            let id = registry
                .messages
                .iter()
                .find_map(|(id, m)| (*m == expected).then_some(*id))
                .unwrap_or_else(|| panic!("missing clickable region for {expected:?}"));
            let entry = flat.iter().find(|e| e.id == id).unwrap();
            let r = entry.bounds;
            let center = Point::new(
                r.origin.x.get() + r.size.width.get() * 0.5,
                r.origin.y.get() + r.size.height.get() * 0.5,
            );
            let mut detector = ClickDetector::new();
            let _ = registry.handle_pointer_button(&mut detector, 0, true, center, 0.1);
            let up = registry.handle_pointer_button(&mut detector, 0, false, center, 0.2);
            assert_eq!(up.messages, vec![expected]);
        }
    }
}
