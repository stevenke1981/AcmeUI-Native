//! Hit region collection and `HitTestLayer` construction — DFS walks for button, canvas, tree, table, and virtual-list hits.

use std::hash::{Hash, Hasher};

use acme_core::{CursorHint, HitRegionSpec, HitTestLayer, Rect};
use acme_layout::{LayoutNode, LayoutSnapshot};
use acme_widgets::WidgetNode;

use crate::helpers::table_display_order;
use crate::render::tree_key_static;
use crate::types::{GalleryMessage, HitRegion};

/// Walk the entire widget+layout tree and collect interactive hit regions (DFS order).
pub fn collect_hit_regions(
    widget: &WidgetNode<GalleryMessage>,
    layout: &LayoutNode,
    snapshot: &LayoutSnapshot,
    scrolled: bool,
    result: &mut Vec<HitRegion>,
) {
    match widget {
        WidgetNode::Button(btn) => {
            if let Some(msg) = btn.activate()
                && let Some(rect) = snapshot.get(layout.id)
            {
                result.push(HitRegion {
                    rect: [rect.x, rect.y, rect.width, rect.height],
                    message: *msg,
                    scrolled,
                    node: layout.id,
                });
            }
        }
        WidgetNode::Canvas(c) => {
            if let Some(msg) = c.activate()
                && let Some(rect) = snapshot.get(layout.id)
                && rect.width > 0.0
                && rect.height > 0.0
            {
                result.push(HitRegion {
                    rect: [rect.x, rect.y, rect.width, rect.height],
                    message: *msg,
                    scrolled,
                    node: layout.id,
                });
            }
        }
        WidgetNode::ScrollView(_) => {
            for (w, l) in widget.children().iter().zip(layout.children.iter()) {
                collect_hit_regions(w, l, snapshot, true, result);
            }
        }
        WidgetNode::Tooltip(t) => collect_hit_regions(&t.child, layout, snapshot, scrolled, result),
        WidgetNode::Popover(p) => {
            collect_hit_regions(&p.children[0], layout, snapshot, scrolled, result)
        }
        WidgetNode::Tree(_) | WidgetNode::Table(_) | WidgetNode::VirtualList(_) => {}
        WidgetNode::Row(v) | WidgetNode::Column(v) | WidgetNode::Stack(v) => {
            if let Some(msg) = &v.message
                && let Some(rect) = snapshot.get(layout.id)
            {
                result.push(HitRegion {
                    rect: [rect.x, rect.y, rect.width, rect.height],
                    message: *msg,
                    scrolled,
                    node: layout.id,
                });
            }
            for (w, l) in widget.children().iter().zip(layout.children.iter()) {
                collect_hit_regions(w, l, snapshot, scrolled, result);
            }
        }
        _ => {
            for (w, l) in widget.children().iter().zip(layout.children.iter()) {
                collect_hit_regions(w, l, snapshot, scrolled, result);
            }
        }
    }
}

/// Append Tree / Table hit regions and cache viewport rects for scroll routing.
#[allow(clippy::too_many_arguments)]
pub fn collect_data_widget_hits(
    widget: &WidgetNode<GalleryMessage>,
    layout: &LayoutNode,
    snapshot: &LayoutSnapshot,
    table_sort_col: Option<usize>,
    table_sort_asc: bool,
    result: &mut Vec<HitRegion>,
    tree_viewport: &mut [f32; 4],
    table_viewport: &mut [f32; 4],
    vlist_viewport: &mut [f32; 4],
) {
    match widget {
        WidgetNode::Tree(t) => {
            if let Some(rect) = snapshot.get(layout.id) {
                *tree_viewport = [rect.x, rect.y, rect.width, rect.height];
            }
            let visible = t.visible_nodes();
            for (node, child_layout) in visible.iter().zip(layout.children.iter()) {
                let Some(rect) = snapshot.get(child_layout.id) else {
                    continue;
                };
                let full = [rect.x, rect.y, rect.width, rect.height];
                let Some(key) = tree_key_static(node.key.as_str()) else {
                    continue;
                };
                if node.has_children {
                    let chevron_w = 20.0_f32.min(rect.width);
                    result.push(HitRegion {
                        rect: [rect.x, rect.y, chevron_w, rect.height],
                        message: GalleryMessage::TreeToggleKey(key),
                        scrolled: true,
                        node: child_layout.id,
                    });
                    result.push(HitRegion {
                        rect: [
                            rect.x + chevron_w,
                            rect.y,
                            (rect.width - chevron_w).max(0.0),
                            rect.height,
                        ],
                        message: GalleryMessage::TreeSelectKey(key),
                        scrolled: true,
                        node: child_layout.id,
                    });
                } else {
                    result.push(HitRegion {
                        rect: full,
                        message: GalleryMessage::TreeSelectKey(key),
                        scrolled: true,
                        node: child_layout.id,
                    });
                }
            }
        }
        WidgetNode::Table(t) => {
            if let Some(rect) = snapshot.get(layout.id) {
                *table_viewport = [rect.x, rect.y, rect.width, rect.height];
            }
            let mut row_i = 0usize;
            if !t.columns.is_empty() {
                if let Some(header_row) = layout.children.get(row_i) {
                    for (ci, cell_layout) in header_row.children.iter().enumerate() {
                        if !t.columns.get(ci).is_some_and(|c| c.sortable) {
                            continue;
                        }
                        if let Some(rect) = snapshot.get(cell_layout.id) {
                            result.push(HitRegion {
                                rect: [rect.x, rect.y, rect.width, rect.height],
                                message: GalleryMessage::TableSort(ci),
                                scrolled: true,
                                node: cell_layout.id,
                            });
                        }
                    }
                }
                row_i += 1;
            }
            let order = table_display_order(table_sort_col, table_sort_asc);
            for (display_i, row_layout) in layout.children.iter().skip(row_i).enumerate() {
                let Some(&orig) = order.get(display_i) else {
                    break;
                };
                if let Some(rect) = snapshot.get(row_layout.id) {
                    result.push(HitRegion {
                        rect: [rect.x, rect.y, rect.width, rect.height],
                        message: GalleryMessage::TableSelectRow(orig),
                        scrolled: true,
                        node: row_layout.id,
                    });
                }
            }
        }
        WidgetNode::VirtualList(_) => {
            if let Some(rect) = snapshot.get(layout.id) {
                *vlist_viewport = [rect.x, rect.y, rect.width, rect.height];
            }
        }
        WidgetNode::Tooltip(t) => {
            collect_data_widget_hits(
                &t.child,
                layout,
                snapshot,
                table_sort_col,
                table_sort_asc,
                result,
                tree_viewport,
                table_viewport,
                vlist_viewport,
            );
        }
        WidgetNode::Popover(p) => {
            collect_data_widget_hits(
                &p.children[0],
                layout,
                snapshot,
                table_sort_col,
                table_sort_asc,
                result,
                tree_viewport,
                table_viewport,
                vlist_viewport,
            );
        }
        _ => {
            let wc = widget.children();
            for (w, l) in wc.iter().zip(layout.children.iter()) {
                collect_data_widget_hits(
                    w,
                    l,
                    snapshot,
                    table_sort_col,
                    table_sort_asc,
                    result,
                    tree_viewport,
                    table_viewport,
                    vlist_viewport,
                );
            }
        }
    }
}

/// Hash the geometry inputs used to build [`HitTestLayer`] so out-of-band test
/// mutations trigger an on-demand rebuild before pointer queries.
pub fn hit_layer_fingerprint(
    button_info: &[HitRegion],
    scroll: f32,
    scroll_clip_rect: [f32; 4],
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    button_info.len().hash(&mut hasher);
    scroll.to_bits().hash(&mut hasher);
    for v in scroll_clip_rect {
        v.to_bits().hash(&mut hasher);
    }
    for hr in button_info {
        hr.node.hash(&mut hasher);
        for v in hr.rect {
            v.to_bits().hash(&mut hasher);
        }
        hr.scrolled.hash(&mut hasher);
    }
    hasher.finish()
}

/// Construct an [`acme_core::HitTestLayer`] from collected [`HitRegion`]s,
/// wrapping `scrolled` regions in `with_scroll_viewport` when a viewport clip
/// rect is present.
pub fn build_hit_layer(
    button_info: &[HitRegion],
    scroll: f32,
    scroll_clip_rect: [f32; 4],
) -> HitTestLayer {
    let mut layer = HitTestLayer::new();
    let has_clip = scroll_clip_rect[2] > 0.0 && scroll_clip_rect[3] > 0.0;
    let clip_rect = Rect::new(
        scroll_clip_rect[0],
        scroll_clip_rect[1],
        scroll_clip_rect[2],
        scroll_clip_rect[3],
    );
    for (idx, hr) in button_info.iter().enumerate() {
        if hr.rect[2] <= 0.0 || hr.rect[3] <= 0.0 {
            continue;
        }
        let spec = HitRegionSpec::rect(
            hr.node,
            Rect::new(hr.rect[0], hr.rect[1], hr.rect[2], hr.rect[3]),
        )
        .cursor(CursorHint::Pointer)
        .tag(idx as u64);
        if hr.scrolled {
            if has_clip {
                layer.with_scroll_viewport(clip_rect, 0.0, scroll, |layer| {
                    layer.push_region(spec);
                });
            } else {
                layer.push_region(
                    HitRegionSpec::rect(
                        hr.node,
                        Rect::new(hr.rect[0], hr.rect[1] - scroll, hr.rect[2], hr.rect[3]),
                    )
                    .cursor(CursorHint::Pointer)
                    .tag(idx as u64),
                );
            }
        } else {
            layer.push_region(spec);
        }
    }
    layer
}
