//! Widget-content rendering — DFS dispatch that paints each widget variant.

use acme_core::{DrawCommand, NodeId, Scene};
use acme_layout::{LayoutNode, LayoutSnapshot};
use acme_text::{FontSystem, GlyphAtlas};
use acme_theme::Theme;
use acme_widgets::ButtonState;
use acme_widgets::WidgetNode;

use crate::render::geometry::quad_rect;
use crate::render::style::push_widget_style;
use crate::render::text::add_text;
use crate::types::{GalleryMessage, TEXT_INPUT_MARKER};

/// DFS walk of page content inside the scroll view.
#[allow(clippy::too_many_arguments)]
pub fn render_content(
    scene: &mut Scene,
    widget: &WidgetNode<GalleryMessage>,
    layout: &LayoutNode,
    snapshot: &LayoutSnapshot,
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
                let fs = l
                    .style
                    .font_size
                    .or(l.font_size)
                    .unwrap_or(theme.typography.body);
                let line_h = l
                    .style
                    .line_height
                    .or(l.line_height)
                    .unwrap_or(fs * theme.typography.line_height);
                let y_text = rect.y - scroll_y + (rect.height - line_h).max(0.0) * 0.5;
                let text_color = l.color.unwrap_or_else(|| {
                    let s = &l.style;
                    s.text_color
                        .as_ref()
                        .map(|t| t.resolve(theme))
                        .unwrap_or(colors.foreground)
                });
                add_text(
                    fonts,
                    atlas,
                    scene,
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
                scene.push(DrawCommand::Quad(quad_rect(
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
                )));
                add_text(
                    fonts,
                    atlas,
                    scene,
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
                scene.push(DrawCommand::Quad(quad_rect(
                    [rect.x, y, rect.width, 1.0],
                    colors.border,
                    0.0,
                    0.0,
                    colors.border,
                )));
            }
        }
        WidgetNode::Tooltip(t) => {
            render_content(
                scene,
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
                scene,
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
        // Tree: children() is empty; layout leaves pair with visible_nodes().
        WidgetNode::Tree(t) => {
            let visible = t.visible_nodes();
            for (node, child_layout) in visible.iter().zip(layout.children.iter()) {
                if let Some(rect) = snapshot.get(child_layout.id) {
                    let y = rect.y - scroll_y;
                    let selected = t.selected.as_ref() == Some(&node.key);
                    if selected {
                        scene.push(DrawCommand::Quad(quad_rect(
                            [rect.x, y, rect.width.max(1.0), rect.height.max(1.0)],
                            colors.ghost_hover,
                            0.0,
                            0.0,
                            colors.ghost_hover,
                        )));
                    }
                    if node.has_children {
                        let mark = if node.expanded { "▾" } else { "▸" };
                        add_text(
                            fonts,
                            atlas,
                            scene,
                            mark,
                            ([rect.x + 2.0, y + 2.0], theme.typography.body),
                            colors.muted_foreground,
                            scale,
                            Some(clip),
                            theme.typography.line_height,
                        );
                    }
                }
                paint_label_like(
                    scene,
                    &node.content,
                    child_layout,
                    snapshot,
                    theme,
                    scale,
                    scroll_y,
                    clip,
                    fonts,
                    atlas,
                );
            }
        }
        // Table: layout is header-row + data-row containers.
        WidgetNode::Table(t) => {
            render_table_content(
                scene, t, layout, snapshot, theme, scale, scroll_y, clip, fonts, atlas,
            );
        }
        // DataGrid: same row/column container layout as Table.
        WidgetNode::DataGrid(g) => {
            render_datagrid_content(
                scene, g, layout, snapshot, theme, scale, scroll_y, clip, fonts, atlas,
            );
        }
        // VirtualList: to_layout emits an empty container; paint the visible window.
        WidgetNode::VirtualList(v) => {
            render_vlist_content(
                scene, v, layout, snapshot, theme, scale, scroll_y, clip, fonts, atlas,
            );
        }
        _ => {
            // Render style background / shadow for containers that carry `Style`.
            if let Some(rect) = snapshot.get(layout.id) {
                let style = match widget {
                    WidgetNode::Row(c)
                    | WidgetNode::Column(c)
                    | WidgetNode::Stack(c) => &c.style,
                    WidgetNode::Card(c) => &c.style,
                    WidgetNode::ScrollView(s) => &s.style,
                    _ => return,
                };
                let y = rect.y - scroll_y;
                let r = [rect.x, y, rect.width, rect.height];
                push_widget_style(scene, style, r, theme);
            }
            let wc = widget.children();
            for (w, l) in wc.iter().zip(layout.children.iter()) {
                render_content(
                    scene,
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
    scene: &mut Scene,
    widget: &WidgetNode<GalleryMessage>,
    layout: &LayoutNode,
    snapshot: &LayoutSnapshot,
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
                    scene,
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
                    scene, w, l, snapshot, theme, scale, scroll_y, clip, fonts, atlas,
                );
            }
        }
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

/// --- Table rendering sub-function ---

use acme_widgets::{DataGrid, Table, VirtualList};

#[allow(clippy::too_many_arguments)]
fn render_table_content(
    scene: &mut Scene,
    t: &Table<GalleryMessage>,
    layout: &LayoutNode,
    snapshot: &LayoutSnapshot,
    theme: &Theme,
    scale: f32,
    scroll_y: f32,
    clip: [f32; 4],
    fonts: &mut FontSystem,
    atlas: &mut GlyphAtlas,
) {
    let colors = theme.colors;
    let mut row_i = 0usize;
    if !t.columns.is_empty() {
        if let Some(header_row) = layout.children.get(row_i) {
            if let Some(hr) = snapshot.get(header_row.id) {
                let y = hr.y - scroll_y;
                scene.push(DrawCommand::Quad(quad_rect(
                    [hr.x, y, hr.width.max(1.0), hr.height.max(1.0)],
                    colors.surface,
                    0.0,
                    1.0,
                    colors.border,
                )));
            }
            for (col, cell_layout) in t.columns.iter().zip(header_row.children.iter()) {
                paint_label_like(
                    scene,
                    &col.header,
                    cell_layout,
                    snapshot,
                    theme,
                    scale,
                    scroll_y,
                    clip,
                    fonts,
                    atlas,
                );
            }
        }
        row_i += 1;
    }
    for (display_i, data_row) in t.rows.iter().enumerate() {
        let Some(row_layout) = layout.children.get(row_i) else {
            break;
        };
        if t.selected_row == Some(display_i)
            && let Some(rr) = snapshot.get(row_layout.id)
        {
            let y = rr.y - scroll_y;
            scene.push(DrawCommand::Quad(quad_rect(
                [rr.x, y, rr.width.max(1.0), rr.height.max(1.0)],
                colors.ghost_hover,
                0.0,
                0.0,
                colors.ghost_hover,
            )));
        }
        for (cell, cell_layout) in data_row.cells.iter().zip(row_layout.children.iter()) {
            paint_label_like(
                scene,
                cell,
                cell_layout,
                snapshot,
                theme,
                scale,
                scroll_y,
                clip,
                fonts,
                atlas,
            );
        }
        row_i += 1;
    }
}

/// --- DataGrid rendering sub-function ---

#[allow(clippy::too_many_arguments)]
fn render_datagrid_content(
    scene: &mut Scene,
    g: &DataGrid<GalleryMessage>,
    layout: &LayoutNode,
    snapshot: &LayoutSnapshot,
    theme: &Theme,
    scale: f32,
    scroll_y: f32,
    clip: [f32; 4],
    fonts: &mut FontSystem,
    atlas: &mut GlyphAtlas,
) {
    let mut row_i = 0usize;
    if !g.columns.is_empty() {
        if let Some(header_row) = layout.children.get(row_i) {
            for (col, cell_layout) in g.columns.iter().zip(header_row.children.iter()) {
                paint_label_like(
                    scene,
                    &col.header,
                    cell_layout,
                    snapshot,
                    theme,
                    scale,
                    scroll_y,
                    clip,
                    fonts,
                    atlas,
                );
            }
        }
        row_i += 1;
    }
    for data_row in &g.rows {
        let Some(row_layout) = layout.children.get(row_i) else {
            break;
        };
        for (cell, cell_layout) in data_row.cells.iter().zip(row_layout.children.iter()) {
            paint_label_like(
                scene,
                cell,
                cell_layout,
                snapshot,
                theme,
                scale,
                scroll_y,
                clip,
                fonts,
                atlas,
            );
        }
        row_i += 1;
    }
}

/// --- VirtualList rendering sub-function ---

#[allow(clippy::too_many_arguments)]
fn render_vlist_content(
    scene: &mut Scene,
    v: &VirtualList<GalleryMessage>,
    layout: &LayoutNode,
    snapshot: &LayoutSnapshot,
    theme: &Theme,
    scale: f32,
    scroll_y: f32,
    clip: [f32; 4],
    fonts: &mut FontSystem,
    atlas: &mut GlyphAtlas,
) {
    let colors = theme.colors;
    if let Some(rect) = snapshot.get(layout.id) {
        let (first, last) = v.visible_range();
        let item_h = v.item_height.unwrap_or(32.0).max(1.0);
        let list_clip = [
            rect.x.max(clip[0]),
            (rect.y - scroll_y).max(clip[1]),
            rect.width.min(clip[2]),
            rect.height.min(clip[3]),
        ];
        for i in first..last {
            let Some(child) = v.children.get(i) else {
                break;
            };
            let y = rect.y + (i as f32 * item_h) - v.scroll_offset - scroll_y;
            if let WidgetNode::Label(l) = child {
                add_text(
                    fonts,
                    atlas,
                    scene,
                    &l.text,
                    (
                        [rect.x + 4.0, y + 2.0],
                        l.font_size.unwrap_or(theme.typography.body),
                    ),
                    colors.foreground,
                    scale,
                    Some(list_clip),
                    theme.typography.line_height,
                );
            }
        }
    }
}

/// --- tree_key_static (shared with hit_test) ---

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
