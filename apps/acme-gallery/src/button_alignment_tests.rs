//! ROADMAP 063 evidence: Gallery button painters honour `Button::label_x`.
//!
//! Unaligned buttons keep today's centring (checked against the centred
//! formula across every page); Sidebar / Sidenav handler items and the shell
//! category list start at the left padding / just after the marker gutter.
//! All positions are read back from the painted `Scene`.
use super::*;
use acme_core::DrawCommand;
use acme_text::TextWrap;

type Snapshot = acme_layout::LayoutSnapshot;

fn context(theme: &Theme) -> WidgetLayoutContext {
    WidgetLayoutContext {
        body_font_size: theme.typography.body,
        body_line_height: theme.typography.body * theme.typography.line_height,
        label_font_size: theme.typography.label,
        control_height: 36.0,
        scale_factor: 1.0,
    }
}

/// Lay out and paint `widget` on its own (no scroll).
fn paint(
    g: &mut Gallery,
    widget: &WidgetNode<GalleryMessage>,
) -> (LayoutNode, Snapshot, acme_core::Scene) {
    let theme = g.current_theme();
    let layout = widget.to_layout_with_context(NodeId::new(1), &context(&theme));
    let snapshot = g
        .layout
        .compute_with_text(&layout, (600.0, 4000.0), &mut g.fonts, 1.0)
        .unwrap();
    let mut painter = Painter::with_clear(theme_color(theme.colors.background));
    let mut index = 0;
    render_content_painter(
        &mut painter,
        widget,
        &layout,
        &snapshot,
        &theme,
        1.0,
        0.0,
        [0.0, 0.0, 600.0, 4000.0],
        &mut index,
        None,
        None,
        &mut g.fonts,
        &mut g.atlas,
    );
    (layout, snapshot, painter.finish())
}

fn texts(scene: &acme_core::Scene) -> Vec<acme_core::TextPrimitive> {
    scene
        .commands()
        .iter()
        .filter_map(|c| match c {
            DrawCommand::Text(t) => Some(t.clone()),
            _ => None,
        })
        .collect()
}

/// Activatable buttons with no explicit alignment: (label, rect).
fn unaligned_buttons(
    widget: &WidgetNode<GalleryMessage>,
    layout: &LayoutNode,
    snapshot: &Snapshot,
    out: &mut Vec<(String, [f32; 4])>,
) {
    match widget {
        WidgetNode::Button(btn) => {
            if btn.style.text_align.is_none() && btn.activate().is_some() && !btn.label.is_empty() {
                if let Some(r) = snapshot.get(layout.id) {
                    out.push((btn.label.clone(), [r.x, r.y, r.width, r.height]));
                }
            }
        }
        WidgetNode::Tooltip(t) => unaligned_buttons(&t.child, layout, snapshot, out),
        WidgetNode::Popover(p) => unaligned_buttons(&p.children[0], layout, snapshot, out),
        _ => {
            for (w, l) in widget.children().iter().zip(layout.children.iter()) {
                unaligned_buttons(w, l, snapshot, out);
            }
        }
    }
}

fn shape(g: &mut Gallery, text: &str, size: f32) -> acme_text::TextLayout {
    let theme = g.current_theme();
    g.fonts.shape(
        text,
        &TextStyle {
            font_size: size,
            line_height: size * theme.typography.line_height,
            ..TextStyle::default()
        },
        TextConstraints {
            max_width: None,
            wrap: TextWrap::None,
        },
        1.0,
    )
}

#[test]
fn gallery_unaligned_buttons_paint_identically() {
    let mut checked = 0;
    for dark in [false, true] {
        for category in 0..CATEGORIES.len() {
            let mut g = Gallery::new();
            g.theme_pack = "default".into();
            g.dark = dark;
            g.selected_category = category;
            let theme = g.current_theme();
            let widget = g.render_page();
            let (layout, snapshot, scene) = paint(&mut g, &widget);
            let mut buttons = Vec::new();
            unaligned_buttons(&widget, &layout, &snapshot, &mut buttons);
            let runs = texts(&scene);
            for (label, rect) in buttons {
                let shaped = shape(&mut g, &label, theme.typography.label);
                let x = rect[0] + (rect[2] - shaped.width).max(0.0) / 2.0;
                let y = rect[1] + (rect[3] - shaped.height).max(0.0) / 2.0 + shaped.baseline;
                assert!(
                    runs.iter().any(|t| (t.origin.x.get() - x).abs() < 1e-3
                        && (t.origin.y.get() - y).abs() < 1e-3),
                    "category {category} dark {dark}: button {label:?} not centred at ({x}, {y})"
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 20, "only {checked} unaligned buttons checked");
}

#[test]
fn gallery_toolbar_unaligned_buttons_paint_identically() {
    let mut g = Gallery::new();
    g.theme_pack = "default".into();
    let scene = g.frame(FrameContext {
        window: WindowId(0),
        logical_width: 1100.0,
        logical_height: 760.0,
        scale_factor: 1.0,
    });
    let (root, snapshot) = g.last_frame_layout.clone().unwrap();
    let ids = extract_gallery_ids(&root);
    let r = snapshot.get(ids.toolbar_buttons[2]).unwrap();
    let shaped = shape(&mut g, "ℹ Info", 13.0);
    let x = r.x + (r.width - shaped.width).max(0.0) / 2.0;
    let y = r.y + (r.height - shaped.height) / 2.0 + shaped.baseline;
    assert!(
        texts(&scene)
            .iter()
            .any(|t| (t.origin.x.get() - x).abs() < 1e-3 && (t.origin.y.get() - y).abs() < 1e-3),
        "toolbar Info label not centred at ({x}, {y})"
    );
}

/// Ink-start x (window space) of each glyph in a run, sorted.
fn glyph_xs(t: &acme_core::TextPrimitive) -> Vec<f32> {
    let mut xs: Vec<f32> = t.glyphs.iter().map(|g| t.origin.x.get() + g.x).collect();
    xs.sort_by(f32::total_cmp);
    xs
}

fn inside(t: &acme_core::TextPrimitive, rect: [f32; 4]) -> bool {
    let (x, y) = (t.origin.x.get(), t.origin.y.get());
    x >= rect[0] && x < rect[0] + rect[2] && y > rect[1] && y <= rect[1] + rect[3]
}

fn demo_hits(
    g: &mut Gallery,
    widget: &WidgetNode<GalleryMessage>,
) -> (acme_core::Scene, Vec<HitRegion>) {
    let (layout, snapshot, scene) = paint(g, widget);
    let mut hits = Vec::new();
    collect_hit_regions(widget, &layout, &snapshot, false, &mut hits);
    (scene, hits)
}

fn assert_same_label_column(
    g: &mut Gallery,
    build: fn(&Gallery) -> WidgetNode<GalleryMessage>,
    set_current: fn(&mut Gallery, u8),
    message: fn(u8) -> GalleryMessage,
    name: &str,
) {
    // Two enabled items (hit regions only exist for enabled ones).
    set_current(g, 0);
    let widget = build(g);
    let (_, hits) = demo_hits(g, &widget);
    let ids: Vec<u8> = (0..16u8)
        .filter(|&i| hits.iter().any(|h| h.message == message(i)))
        .take(2)
        .collect();
    assert_eq!(ids.len(), 2, "{name}: two enabled items");
    let (a, b) = (ids[0], ids[1]);
    // (run origin x, label ink start, rect) of `item` while `current` is selected.
    let mut column = |current: u8, item: u8| -> (f32, f32, [f32; 4]) {
        set_current(g, current);
        let widget = build(g);
        let (scene, hits) = demo_hits(g, &widget);
        let rect = hits
            .iter()
            .find(|h| h.message == message(item))
            .unwrap()
            .rect;
        let run = texts(&scene)
            .into_iter()
            .find(|t| inside(t, rect))
            .unwrap_or_else(|| panic!("{name}: no run in {rect:?}"));
        // Selected runs start with the marker glyph; the label follows it.
        let ink = glyph_xs(&run)[usize::from(current == item)];
        (run.origin.x.get(), ink, rect)
    };
    for item in [a, b] {
        let other = if item == a { b } else { a };
        let (sel_origin, sel_ink, rect) = column(item, item);
        let (un_origin, un_ink, _) = column(other, item);
        // Left padding: the run starts at rect.x + 12 selected or not.
        assert!(
            (sel_origin - (rect[0] + 12.0)).abs() < 0.01,
            "{name}: selected origin {sel_origin} vs rect {rect:?}"
        );
        assert!(
            (un_origin - (rect[0] + 12.0)).abs() < 0.01,
            "{name}: unselected origin {un_origin} vs rect {rect:?}"
        );
        // The label text (after the marker) starts in the same column (<= 2px).
        // Font-dependent: verified for the Windows default font only.
        if cfg!(target_os = "windows") {
            assert!(
                (sel_ink - un_ink).abs() <= 2.0,
                "{name} item {item}: selected label ink {sel_ink} vs unselected {un_ink}"
            );
        } else {
            eprintln!(
                "063 {name} item {item}: label ink offset {}",
                sel_ink - un_ink
            );
        }
    }
}

#[test]
fn gallery_sidebar_and_sidenav_labels_start_at_left_padding() {
    let mut g = Gallery::new();
    g.theme_pack = "default".into();
    assert_same_label_column(
        &mut g,
        Gallery::sidebar_examples,
        |g, i| g.side_nav.current = i,
        GalleryMessage::SidebarSelect,
        "sidebar",
    );
    assert_same_label_column(
        &mut g,
        Gallery::sidenav_examples,
        |g, i| g.sidenav.current = i,
        GalleryMessage::SidenavSelect,
        "sidenav",
    );
}

#[test]
fn gallery_shell_category_labels_left_aligned_after_marker() {
    let mut label_offsets = Vec::new();
    for selected in [0usize, 3, CATEGORIES.len() - 1] {
        let mut g = Gallery::new();
        g.theme_pack = "default".into();
        g.selected_category = selected;
        let scene = g.frame(FrameContext {
            window: WindowId(0),
            logical_width: 1100.0,
            logical_height: 760.0,
            scale_factor: 1.0,
        });
        for category in 0..CATEGORIES.len() {
            let rect = g
                .button_info
                .iter()
                .find(|h| h.message == GalleryMessage::SelectCategory(category))
                .unwrap()
                .rect;
            let in_rect: Vec<_> = texts(&scene)
                .into_iter()
                .filter(|t| inside(t, rect))
                .collect();
            let label = in_rect.last().expect("label painted");
            let offset = label.origin.x.get() - rect[0];
            label_offsets.push(offset);
            // Left-aligned just after the 8px marker gutter, not centred.
            assert!(
                (16.0..=32.0).contains(&offset),
                "category {category} (selected {selected}): label offset {offset}"
            );
            if category == selected {
                assert_eq!(in_rect.len(), 2, "marker plus label");
                let marker = &in_rect[0];
                let marker_hi = marker.origin.x.get()
                    + marker
                        .glyphs
                        .iter()
                        .map(|g| g.x + g.width)
                        .fold(f32::MIN, f32::max);
                assert!(marker.origin.x.get() >= rect[0]);
                assert!(
                    marker_hi <= glyph_xs(label)[0],
                    "marker ink ends {marker_hi} before label ink {}",
                    glyph_xs(label)[0]
                );
            }
        }
    }
    // Same x for every category and selection state.
    assert!(label_offsets.windows(2).all(|w| (w[0] - w[1]).abs() < 0.01));
}
