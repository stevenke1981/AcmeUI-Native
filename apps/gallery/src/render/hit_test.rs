//! Hit region collection — DFS walks for button, tree, table, and virtual-list hits.

use acme_layout::{LayoutNode, LayoutSnapshot};
use acme_widgets::WidgetNode;

use crate::helpers::table_display_order;
use crate::render::tree_key_static;
use crate::types::{GalleryMessage, HitRegion};

/// Walk the entire widget+layout tree and collect button hit regions (DFS order).
pub fn collect_hit_regions(
    widget: &WidgetNode<GalleryMessage>,
    layout: &LayoutNode,
    snapshot: &LayoutSnapshot,
    scrolled: bool,
    result: &mut Vec<HitRegion>,
) {
    #[allow(clippy::collapsible_if)]
    match widget {
        WidgetNode::Button(btn) => {
            if let Some(msg) = btn.activate() {
                if let Some(rect) = snapshot.get(layout.id) {
                    result.push(HitRegion {
                        rect: [rect.x, rect.y, rect.width, rect.height],
                        message: *msg,
                        scrolled,
                    });
                }
            }
        }
        WidgetNode::ScrollView(_) => {
            let wc = widget.children();
            for (w, l) in wc.iter().zip(layout.children.iter()) {
                collect_hit_regions(w, l, snapshot, true, result);
            }
        }
        WidgetNode::Tooltip(t) => {
            collect_hit_regions(&t.child, layout, snapshot, scrolled, result);
        }
        WidgetNode::Popover(p) => {
            collect_hit_regions(&p.children[0], layout, snapshot, scrolled, result);
        }
        WidgetNode::Tree(_) | WidgetNode::Table(_) | WidgetNode::VirtualList(_) => {}
        _ => {
            let wc = widget.children();
            for (w, l) in wc.iter().zip(layout.children.iter()) {
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
                    });
                } else {
                    result.push(HitRegion {
                        rect: full,
                        message: GalleryMessage::TreeSelectKey(key),
                        scrolled: true,
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
