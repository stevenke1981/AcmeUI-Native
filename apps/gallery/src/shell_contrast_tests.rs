//! ROADMAP 060 evidence: the selected category label in the Gallery sidebar is
//! readable (>= 4.5:1) in light and dark, carries a non-colour marker, and the
//! hit rects do not move with the selection.
//!
//! Colours are read back from the painted `Scene`: the quad behind the category
//! button and the text run drawn on top of it.

use acme_core::{DrawCommand, TextPrimitive};
use acme_theme::{ThemeColor, contrast_ratio};

use super::*;

const AA: f32 = 4.5;

fn tc(c: Color) -> ThemeColor {
    ThemeColor::rgba(c.r, c.g, c.b, c.a)
}

fn render(gallery: &mut Gallery) -> Scene {
    gallery.frame(FrameContext {
        window: WindowId(0),
        logical_width: 1100.0,
        logical_height: 760.0,
        scale_factor: 1.0,
    })
}

fn category_rect(gallery: &Gallery, category: usize) -> [f32; 4] {
    gallery
        .button_info
        .iter()
        .find(|h| h.message == GalleryMessage::SelectCategory(category))
        .unwrap_or_else(|| panic!("category {category} has a hit region"))
        .rect
}

struct Painted {
    bg: ThemeColor,
    fg: ThemeColor,
    label: TextPrimitive,
    marker: Option<TextPrimitive>,
}

/// Horizontal ink extent (min, max) of a painted text run, in window space.
fn ink_x(t: &TextPrimitive) -> (f32, f32) {
    let ox = t.origin.x.get();
    let lo = t.glyphs.iter().map(|g| ox + g.x).fold(f32::MAX, f32::min);
    let hi = t
        .glyphs
        .iter()
        .map(|g| ox + g.x + g.width)
        .fold(f32::MIN, f32::max);
    (lo, hi)
}

/// The quad that fills the category button and the text runs painted after it
/// (before the next quad): the label is the last run, a marker (if any) the first.
fn painted(scene: &Scene, rect: [f32; 4], _context: &str) -> Painted {
    let near = |a: f32, b: f32| (a - b).abs() < 0.01;
    let commands = scene.commands();
    let quad_at = commands
        .iter()
        .position(|c| match c {
            DrawCommand::Quad(q) => {
                near(q.rect.origin.x.get(), rect[0])
                    && near(q.rect.origin.y.get(), rect[1])
                    && near(q.rect.size.width.get(), rect[2])
                    && near(q.rect.size.height.get(), rect[3])
            }
            _ => false,
        })
        .unwrap_or_else(|| panic!("{_context}: no painted quad at the category rect"));
    let DrawCommand::Quad(quad) = &commands[quad_at] else {
        unreachable!()
    };
    let mut texts: Vec<_> = commands[quad_at + 1..]
        .iter()
        .take_while(|c| !matches!(c, DrawCommand::Quad(_)))
        .filter_map(|c| match c {
            DrawCommand::Text(t) => Some(t.clone()),
            _ => None,
        })
        .collect();
    let label = texts
        .pop()
        .unwrap_or_else(|| panic!("{_context}: no text painted after the category quad"));
    assert!(
        texts.len() <= 1,
        "{_context}: at most a marker plus a label"
    );
    Painted {
        bg: tc(quad.color),
        fg: tc(label.color),
        label,
        marker: texts.pop(),
    }
}

#[test]
fn gallery_selected_category_text_contrast_meets_aa() {
    let mut failures = Vec::new();
    for dark in [false, true] {
        let mut gallery = Gallery::new();
        gallery.dark = dark;
        gallery.selected_category = 2;
        let scene = render(&mut gallery);
        let Painted { bg, fg, .. } = painted(&scene, category_rect(&gallery, 2), "selected");
        let ratio = contrast_ratio(fg, bg);
        println!(
            "selected default/{}: {ratio:.2}",
            if dark { "dark" } else { "light" }
        );
        if ratio < AA {
            failures.push(format!("dark={dark} {ratio:.2}:1"));
        }
    }
    assert!(
        failures.is_empty(),
        "selected category text below {AA}:1: {failures:?}"
    );
}

#[test]
fn gallery_selected_category_has_marker_and_stable_hit_rects() {
    let mut base = Gallery::new();
    let base_scene = render(&mut base);
    let base_rects = (0..CATEGORIES.len())
        .map(|i| category_rect(&base, i))
        .collect::<Vec<_>>();
    let unselected = painted(&base_scene, base_rects[3], "base");
    assert!(unselected.marker.is_none(), "unselected paints no marker");

    let mut moved = Gallery::new();
    moved.selected_category = 3;
    let moved_scene = render(&mut moved);
    assert_eq!(
        (0..CATEGORIES.len())
            .map(|i| category_rect(&moved, i))
            .collect::<Vec<_>>(),
        base_rects,
        "changing the selection must not move any category hit rect"
    );
    let selected = painted(&moved_scene, base_rects[3], "moved");
    let marker = selected.marker.as_ref().expect("selected paints a marker");
    assert!(!marker.glyphs.is_empty(), "marker has glyphs");

    // The label text must not jump when selection changes.
    assert_eq!(
        selected.label.origin.x.get(),
        unselected.label.origin.x.get(),
        "label x origin is identical selected vs unselected"
    );
    assert_eq!(
        ink_x(&selected.label),
        ink_x(&unselected.label),
        "label ink extent is identical selected vs unselected"
    );
    // The marker sits in its own gutter, left of the label and inside the rect.
    let rect = base_rects[3];
    let (m_lo, m_hi) = ink_x(marker);
    let (l_lo, _) = ink_x(&selected.label);
    assert!(
        m_lo >= rect[0] && m_hi <= l_lo,
        "marker {m_lo}..{m_hi} must precede label ink {l_lo} inside rect"
    );
}

#[test]
fn gallery_longest_selected_category_label_fits_inside_button() {
    let (longest, _) = CATEGORIES
        .iter()
        .enumerate()
        .max_by_key(|(_, c)| c.name.chars().count())
        .unwrap();
    for dark in [false, true] {
        let mut gallery = Gallery::new();
        gallery.dark = dark;
        gallery.selected_category = longest;
        let scene = render(&mut gallery);
        let rect = category_rect(&gallery, longest);
        let p = painted(&scene, rect, "longest");
        let (lo, hi) = ink_x(&p.label);
        let (right, bottom_ok) = (
            rect[0] + rect[2],
            p.label.origin.y.get() <= rect[1] + rect[3],
        );
        assert!(bottom_ok);
        assert!(
            lo >= rect[0] && hi <= right,
            "label ink {lo}..{hi} must lie inside button {}..{right}",
            rect[0]
        );
        let (m_lo, m_hi) = ink_x(p.marker.as_ref().expect("marker"));
        assert!(
            m_lo >= rect[0] && m_hi <= lo,
            "marker {m_lo}..{m_hi} before label {lo}"
        );
    }
}

// ── ROADMAP 063: left-aligned labels ────────────────────────────────────────

fn text_runs(scene: &Scene) -> Vec<TextPrimitive> {
    scene
        .commands()
        .iter()
        .filter_map(|c| match c {
            DrawCommand::Text(t) => Some(t.clone()),
            _ => None,
        })
        .collect()
}

fn unaligned_content_buttons(
    widget: &WidgetNode<GalleryMessage>,
    layout: &LayoutNode,
    snapshot: &acme_layout::LayoutSnapshot,
    out: &mut Vec<[f32; 2]>,
) {
    match widget {
        WidgetNode::Button(btn) => {
            if btn.style.text_align.is_none() && btn.activate().is_some() && !btn.label.is_empty() {
                if let Some(r) = snapshot.get(layout.id) {
                    out.push([r.x, r.y]);
                }
            }
        }
        // Only descend through containers `render_content` itself walks (other
        // composites, e.g. NavRail, are not painted by this renderer).
        WidgetNode::Row(_)
        | WidgetNode::Column(_)
        | WidgetNode::Stack(_)
        | WidgetNode::Card(_)
        | WidgetNode::ScrollView(_) => {
            for (w, l) in widget.children().iter().zip(layout.children.iter()) {
                unaligned_content_buttons(w, l, snapshot, out);
            }
        }
        _ => {}
    }
}

/// Unaligned content buttons keep the fixed `rect.x + 10, y + 8` label origin
/// they had before `Button::label_x` existed.
#[test]
fn gallery_unaligned_buttons_paint_identically() {
    let mut checked = 0;
    for category in 0..CATEGORIES.len() {
        let mut gallery = Gallery::new();
        gallery.selected_category = category;
        let scene = render(&mut gallery);
        let theme = build_theme(gallery.dark);
        let context = build_layout_context(&theme, 1.0);
        let description = gallery.description();
        let mut root = layout_root(&description, None, &context);
        apply_gallery_styles(&mut root, 1100.0, 760.0);
        let snapshot = gallery
            .layout
            .compute_with_text(&root, (1100.0, 760.0), &mut gallery.fonts, 1.0)
            .unwrap();
        let mut buttons = Vec::new();
        unaligned_content_buttons(&description, &root, &snapshot, &mut buttons);
        let runs = text_runs(&scene);
        for [x, y] in buttons {
            // Skip shell chrome (category list and toolbar): different painters.
            if x < SIDEBAR_WIDTH || y < TOOLBAR_HEIGHT {
                continue;
            }
            let (ex, ey) = (x + 10.0, y - gallery.scroll + 8.0);
            assert!(
                runs.iter()
                    .any(|t| t.origin.x.get() == ex && (t.origin.y.get() - ey).abs() < 1e-3),
                "category {category}: unaligned button at ({x}, {y}) not painted at ({ex}, {ey})"
            );
            checked += 1;
        }
    }
    assert!(checked > 10, "only {checked} content buttons checked");
}

/// The shell category list already left-aligns: marker gutter at +8 and the
/// label at a fixed inset, the same x selected or not, never centred.
#[test]
fn gallery_shell_category_labels_left_aligned_after_marker() {
    for selected in [0, 3, CATEGORIES.len() - 1] {
        let mut gallery = Gallery::new();
        gallery.selected_category = selected;
        let scene = render(&mut gallery);
        for category in 0..CATEGORIES.len() {
            let rect = category_rect(&gallery, category);
            let p = painted(&scene, rect, "left");
            assert_eq!(
                p.label.origin.x.get(),
                rect[0] + 24.0,
                "category {category} (selected {selected}) label x"
            );
            if let Some(marker) = &p.marker {
                let (_, m_hi) = ink_x(marker);
                let (l_lo, _) = ink_x(&p.label);
                assert!(m_hi <= l_lo, "marker ink {m_hi} before label ink {l_lo}");
            }
        }
    }
}
