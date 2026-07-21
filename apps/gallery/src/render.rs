//! Render helpers, layout extractors, style application, and geometry utilities.
//!
//! Functions not yet imported by main.rs are kept here for future modularization.
#![allow(unused)]

use acme_core::NodeId;
use acme_layout::{LayoutKind, LayoutNode, LayoutStyle, Length};
use acme_render_wgpu::{Frame, Quad, TextRun};
use acme_text::{FontSystem, GlyphAtlas, TextConstraints, TextStyle};
use acme_theme::{Theme, ThemeColor};
use acme_widgets::{ButtonState, WidgetNode};

use crate::types::*;

// ── Layout ID Extractors ────────────────────────────────────────────────────

pub struct GalleryNodeIds {
    #[allow(dead_code)]
    pub _root: NodeId,
    pub sidebar: NodeId,
    pub sidebar_label: NodeId,
    pub _sidebar_separator: NodeId,
    pub sidebar_buttons: [NodeId; 8],
    #[allow(dead_code)]
    pub content_area: NodeId,
    pub toolbar: NodeId,
    pub toolbar_buttons: [NodeId; 3],
    pub scroll_view: NodeId,
}

pub fn extract_gallery_ids(root: &LayoutNode) -> GalleryNodeIds {
    let sb = &root.children[0];
    let ca = &root.children[1];
    let tb = &ca.children[0];
    GalleryNodeIds {
        _root: root.id,
        sidebar: sb.id,
        sidebar_label: sb.children[0].id,
        _sidebar_separator: sb.children[1].id,
        sidebar_buttons: [
            sb.children[2].id,
            sb.children[3].id,
            sb.children[4].id,
            sb.children[5].id,
            sb.children[6].id,
            sb.children[7].id,
            sb.children[8].id,
            sb.children[9].id,
        ],
        content_area: ca.id,
        toolbar: tb.id,
        toolbar_buttons: [tb.children[0].id, tb.children[1].id, tb.children[2].id],
        scroll_view: ca.children[1].id,
    }
}

/// Walk the widget+layout tree to find the text‑input marker label.
pub fn find_text_input_marker(
    widget: &WidgetNode<GalleryMessage>,
    layout: &LayoutNode,
) -> Option<NodeId> {
    match widget {
        WidgetNode::Label(l) if l.text == TEXT_INPUT_MARKER => Some(layout.id),
        _ => {
            let wc = widget.children();
            for (w, l) in wc.iter().zip(layout.children.iter()) {
                if let Some(id) = find_text_input_marker(w, l) {
                    return Some(id);
                }
            }
            None
        }
    }
}

// ── Style Application ───────────────────────────────────────────────────────

pub fn apply_gallery_styles(root: &mut LayoutNode, width: f32, height: f32) {
    // Root: Row fills window
    root.style = LayoutStyle {
        kind: LayoutKind::Row,
        width: Length::px(width),
        height: Length::px(height),
        gap: 0.0,
        ..Default::default()
    };

    // Sidebar: fixed width, full height
    let sb = &mut root.children[0];
    sb.style = LayoutStyle {
        kind: LayoutKind::Column,
        width: Length::px(SIDEBAR_WIDTH),
        height: Length::px(height),
        gap: 4.0,
        padding: acme_layout::Edges {
            left: 12.0,
            right: 12.0,
            top: 16.0,
            bottom: 16.0,
        },
        ..Default::default()
    };
    // Category buttons
    for i in 2..=9 {
        sb.children[i].style.width = Length::px(SIDEBAR_WIDTH - 24.0);
        sb.children[i].style.height = Length::px(40.0);
    }

    // Content area: fills remaining width
    let cw = (width - SIDEBAR_WIDTH).max(400.0);
    let ca = &mut root.children[1];
    ca.style = LayoutStyle {
        kind: LayoutKind::Column,
        width: Length::px(cw),
        height: Length::px(height),
        gap: 0.0,
        ..Default::default()
    };

    // Toolbar
    let tb = &mut ca.children[0];
    tb.style = LayoutStyle {
        kind: LayoutKind::Row,
        width: Length::px(cw),
        height: Length::px(TOOLBAR_HEIGHT),
        gap: 8.0,
        padding: acme_layout::Edges {
            left: 16.0,
            right: 16.0,
            top: 8.0,
            bottom: 8.0,
        },
        ..Default::default()
    };
    for btn in &mut tb.children {
        btn.style.width = Length::px(130.0);
        btn.style.height = Length::px(32.0);
    }

    // Scroll view
    let sh = (height - TOOLBAR_HEIGHT).max(100.0);
    let sv = &mut ca.children[1];
    sv.style = LayoutStyle {
        kind: LayoutKind::Column,
        width: Length::px(cw),
        height: Length::px(sh),
        overflow: acme_layout::Overflow::Scroll,
        flex_grow: 1.0,
        ..Default::default()
    };

    // Page-content first child — full width
    if let Some(content) = sv.children.first_mut() {
        content.style.width = Length::px(cw);
    }
}

// ── Render Helpers ──────────────────────────────────────────────────────────

/// DFS walk of page content inside the scroll view.
#[allow(clippy::too_many_arguments)]
pub fn render_content(
    frame: &mut Frame,
    widget: &WidgetNode<GalleryMessage>,
    layout: &LayoutNode,
    snapshot: &acme_layout::LayoutSnapshot,
    theme: &Theme,
    scale: f32,
    scroll_y: f32,
    clip: [f32; 4],
    btn_idx: &mut usize,
    hovered: Option<usize>,
    pressed: Option<usize>,
    focused: usize,
    fonts: &mut FontSystem,
    atlas: &mut GlyphAtlas,
    show_focus_rings: bool,
) {
    let colors = theme.colors;
    match widget {
        WidgetNode::Label(l) => {
            if l.text == TEXT_INPUT_MARKER {
                return;
            }
            if let Some(rect) = snapshot.get(layout.id) {
                let fs = l.font_size.unwrap_or(theme.typography.body);
                let line_h = fs * theme.typography.line_height;
                let y_text = rect.y - scroll_y + (rect.height - line_h).max(0.0) * 0.5;
                let text_color = l.color.unwrap_or(colors.foreground);
                add_text(
                    fonts,
                    atlas,
                    frame,
                    &l.text,
                    ([rect.x + 4.0, y_text], fs),
                    text_color,
                    scale,
                    Some(clip),
                    theme.typography.line_height,
                );
            }
        }
        WidgetNode::Button(btn) => {
            if btn.activate().is_none() {
                return;
            }
            if let Some(rect) = snapshot.get(layout.id) {
                let y = rect.y - scroll_y;
                let idx = *btn_idx;
                *btn_idx += 1;
                let st = ButtonState {
                    hovered: hovered == Some(idx),
                    pressed: pressed == Some(idx),
                    focused: focused == idx,
                };
                let resolved = btn.resolve_style(theme, st);
                frame.quads.push(quad_rect(
                    [rect.x, y, rect.width, rect.height],
                    resolved.background,
                    theme.radii.md,
                    if st.focused && show_focus_rings {
                        2.0
                    } else {
                        1.0
                    },
                    if st.focused {
                        resolved.focus
                    } else {
                        resolved.border
                    },
                ));
                add_text(
                    fonts,
                    atlas,
                    frame,
                    &btn.label,
                    ([rect.x + 10.0, y + 8.0], theme.typography.label),
                    resolved.foreground,
                    scale,
                    Some(clip),
                    theme.typography.line_height,
                );
            }
        }
        WidgetNode::Separator(_) => {
            if let Some(rect) = snapshot.get(layout.id) {
                let y = rect.y - scroll_y;
                frame.quads.push(quad_rect(
                    [rect.x, y, rect.width, 1.0],
                    colors.border,
                    0.0,
                    0.0,
                    colors.border,
                ));
            }
        }
        WidgetNode::Card(_) => {
            let wc = widget.children();
            for (w, l) in wc.iter().zip(layout.children.iter()) {
                render_content(
                    frame,
                    w,
                    l,
                    snapshot,
                    theme,
                    scale,
                    scroll_y,
                    clip,
                    btn_idx,
                    hovered,
                    pressed,
                    focused,
                    fonts,
                    atlas,
                    show_focus_rings,
                );
            }
        }
        WidgetNode::Tooltip(t) => {
            render_content(
                frame,
                &t.child,
                layout,
                snapshot,
                theme,
                scale,
                scroll_y,
                clip,
                btn_idx,
                hovered,
                pressed,
                focused,
                fonts,
                atlas,
                show_focus_rings,
            );
        }
        WidgetNode::Popover(p) => {
            render_content(
                frame,
                &p.children[0],
                layout,
                snapshot,
                theme,
                scale,
                scroll_y,
                clip,
                btn_idx,
                hovered,
                pressed,
                focused,
                fonts,
                atlas,
                show_focus_rings,
            );
        }
        _ => {
            let wc = widget.children();
            for (w, l) in wc.iter().zip(layout.children.iter()) {
                render_content(
                    frame,
                    w,
                    l,
                    snapshot,
                    theme,
                    scale,
                    scroll_y,
                    clip,
                    btn_idx,
                    hovered,
                    pressed,
                    focused,
                    fonts,
                    atlas,
                    show_focus_rings,
                );
            }
        }
    }
}

/// Paint a Label (or nested Label-in-container) at a layout leaf position.
#[allow(clippy::too_many_arguments)]
pub fn paint_label_like(
    frame: &mut Frame,
    widget: &WidgetNode<GalleryMessage>,
    layout: &LayoutNode,
    snapshot: &acme_layout::LayoutSnapshot,
    theme: &Theme,
    scale: f32,
    scroll_y: f32,
    clip: [f32; 4],
    fonts: &mut FontSystem,
    atlas: &mut GlyphAtlas,
) {
    match widget {
        WidgetNode::Label(l) => {
            if l.text == TEXT_INPUT_MARKER {
                return;
            }
            if let Some(rect) = snapshot.get(layout.id) {
                let fs = l.font_size.unwrap_or(theme.typography.body);
                let line_h = fs * theme.typography.line_height;
                let y_text = rect.y - scroll_y + (rect.height - line_h).max(0.0) * 0.5;
                add_text(
                    fonts,
                    atlas,
                    frame,
                    &l.text,
                    ([rect.x + 4.0, y_text], fs),
                    theme.colors.foreground,
                    scale,
                    Some(clip),
                    theme.typography.line_height,
                );
            }
        }
        other => {
            let wc = other.children();
            if wc.is_empty() {
                return;
            }
            for (w, l) in wc.iter().zip(layout.children.iter()) {
                paint_label_like(
                    frame, w, l, snapshot, theme, scale, scroll_y, clip, fonts, atlas,
                );
            }
        }
    }
}

/// Walk the entire widget+layout tree and collect button hit regions (DFS order).
pub fn collect_hit_regions(
    widget: &WidgetNode<GalleryMessage>,
    layout: &LayoutNode,
    snapshot: &acme_layout::LayoutSnapshot,
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
    snapshot: &acme_layout::LayoutSnapshot,
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
            let order = crate::pages::table_display_order(table_sort_col, table_sort_asc);
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

/// Map a tree node key string to a `'static` constant used in messages.
pub fn tree_key_static(key: &str) -> Option<&'static str> {
    const ALL: &[&str] = &[
        "docs",
        "docs_readme",
        "docs_guide",
        "docs_zh",
        "docs_zh_ime",
        "docs_zh_a11y",
        "images",
        "img_logo",
        "img_banner",
        "code",
        "code_src",
        "code_main",
        "code_lib",
        "code_toml",
    ];
    ALL.iter().copied().find(|&k| k == key)
}

// ── Text Helper ─────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub fn add_text(
    fonts: &mut FontSystem,
    atlas: &mut GlyphAtlas,
    frame: &mut Frame,
    text: &str,
    geometry: ([f32; 2], f32),
    color: ThemeColor,
    scale: f32,
    clip: Option<[f32; 4]>,
    line_height_ratio: f32,
) {
    let (origin, size) = geometry;
    let style = TextStyle {
        font_size: size,
        line_height: size * line_height_ratio,
        ..TextStyle::default()
    };
    let layout = fonts.shape(text, &style, TextConstraints::default(), scale);
    let prepared = fonts.prepare(&layout, atlas);
    frame.text.push(TextRun {
        prepared,
        origin,
        color: rgba(color),
        clip,
    });
}

// ── Color / Geometry Helpers ────────────────────────────────────────────────

pub fn rgba(color: ThemeColor) -> [f32; 4] {
    [color.red, color.green, color.blue, color.alpha]
}

pub fn quad_rect(
    rect: [f32; 4],
    fill: ThemeColor,
    radius: f32,
    border_width: f32,
    border_color: ThemeColor,
) -> Quad {
    Quad {
        rect,
        color: rgba(fill),
        radius,
        border_width,
        border_color: rgba(border_color),
    }
}

pub fn point_in_rect(x: f32, y: f32, rect: [f32; 4]) -> bool {
    x >= rect[0] && x <= rect[0] + rect[2] && y >= rect[1] && y <= rect[1] + rect[3]
}

/// Map a content-space layout rect into window space using page scroll.
pub fn scrolled_hit_rect(rect: [f32; 4], scroll_y: f32) -> [f32; 4] {
    [rect[0], rect[1] - scroll_y, rect[2], rect[3]]
}