//! ROADMAP 005 D1 evidence: the Gallery keeps its logical geometry and the
//! text-input IME caret at DPI scales 1.0 to 2.0.
//!
//! Layout is computed in logical pixels. The scale factor reaches text shaping
//! only as raster placement, so logical rects should not move with scale. These
//! tests run every page through the same pipeline `frame` uses, then check the
//! logical contract. Physical pixels are covered by D2, and real IME sessions
//! remain a manual check.
//!
//! The two field-geometry regressions below failed before the ROADMAP 005 fix
//! wave and now run by default.

use acme_layout::LayoutSnapshot;

use super::*;

/// DPI matrix from ROADMAP 005. 1.75 is included because it is cheap here.
const SCALES: [f32; 5] = [1.0, 1.25, 1.5, 1.75, 2.0];
const VIEWPORT: (f32, f32) = (1100.0, 760.0);
/// Logical-px drift allowed for any layout rect against scale 1.0. Measured
/// maximum over all 32 pages and 4 non-base scales: 0.0000 (exact; shaping is
/// done in logical metrics and scale only reaches raster placement). 0.01 is a
/// display-precision margin, not a loosening: a real change moves rects by more.
const LAYOUT_TOLERANCE: f32 = 0.01;
/// Logical-px drift allowed for caret offset from the field origin against
/// scale 1.0. Measured maximum: 0.0000 on every step and scale.
const CARET_TOLERANCE: f32 = 0.01;
/// Text-input page (`Inputs` category, `TextInput` page).
const TEXT_INPUT_PAGE: (usize, usize) = (1, 1);
const PREEDIT: &str = "ㄓㄨˋ";
const COMMITTED: &str = "注音輸入 abc";

fn all_pages() -> Vec<(usize, usize)> {
    CATEGORIES
        .iter()
        .enumerate()
        .flat_map(|(category, info)| (0..info.pages.len()).map(move |page| (category, page)))
        .collect()
}

fn select(gallery: &mut Gallery, (category, page): (usize, usize)) {
    gallery.selected_category = category;
    gallery.selected_page = page;
}

fn frame_at(gallery: &mut Gallery, scale_factor: f32) -> Scene {
    gallery.frame(FrameContext {
        window: WindowId(0),
        logical_width: VIEWPORT.0,
        logical_height: VIEWPORT.1,
        scale_factor,
    })
}

/// Pre-order `[x, y, width, height]` of every layout node.
fn collect_rects(node: &LayoutNode, snapshot: &LayoutSnapshot, out: &mut Vec<[f32; 4]>) {
    let rect = snapshot.get(node.id).expect("every layout node has a rect");
    out.push([rect.x, rect.y, rect.width, rect.height]);
    for child in &node.children {
        collect_rects(child, snapshot, out);
    }
}

/// Logical layout rects of the selected page at `scale_factor`, built the way
/// `frame` builds them: reconcile, bind retained IDs, layout with the scale
/// context, apply the Gallery styles, then compute with text.
fn layout_rects(gallery: &mut Gallery, scale_factor: f32) -> Vec<[f32; 4]> {
    let description = gallery.description();
    if let Err(error) = gallery.retained.reconcile(&description) {
        panic!(
            "reconcile failed for category {} page {}: {error}",
            gallery.selected_category, gallery.selected_page
        );
    }
    let binding = bind_retained(std::slice::from_ref(&description), gallery.retained.tree());
    assert_eq!(
        binding.mismatches(),
        0,
        "binding mismatches on category {} page {}",
        gallery.selected_category,
        gallery.selected_page
    );
    let context = build_layout_context(&build_theme(gallery.dark), scale_factor);
    let mut root = layout_root(&description, Some(&binding), &context);
    apply_gallery_styles(&mut root, VIEWPORT.0, VIEWPORT.1);
    let snapshot = gallery
        .layout
        .compute_with_text(&root, VIEWPORT, &mut gallery.fonts, scale_factor)
        .expect("finite Gallery viewport");
    let mut out = Vec::new();
    collect_rects(&root, &snapshot, &mut out);
    out
}

/// Largest absolute component difference between two equal-length rect lists.
fn max_drift(base: &[[f32; 4]], other: &[[f32; 4]]) -> f32 {
    assert_eq!(
        base.len(),
        other.len(),
        "layout node count changed with scale"
    );
    base.iter()
        .zip(other)
        .flat_map(|(a, b)| a.iter().zip(b).map(|(x, y)| (x - y).abs()))
        .fold(0.0_f32, f32::max)
}

/// Scroll the TextInput page so the field sits in the middle of the viewport,
/// as a user would before clicking it. Returns the field rect after the scroll.
fn bring_text_input_into_view(gallery: &mut Gallery, scale_factor: f32) -> [f32; 4] {
    frame_at(gallery, scale_factor);
    let [_, y, _, h] = gallery.text_input_rect;
    assert!(h > 0.0, "TextInput rect missing after frame");
    gallery.scroll = (gallery.scroll + (y + h / 2.0) - VIEWPORT.1 / 2.0).max(0.0);
    frame_at(gallery, scale_factor);
    let field = gallery.text_input_rect;
    assert!(
        field[1] >= 0.0 && field[1] + field[3] <= VIEWPORT.1,
        "TextInput {field:?} is not inside the viewport after scrolling"
    );
    field
}

/// Focus the TextInput the way a pointer does: a press inside its rect. The
/// release is omitted so no button under the field is activated.
fn focus_text_input(gallery: &mut Gallery) {
    let [x, y, w, h] = gallery.text_input_rect;
    assert!(w > 0.0 && h > 0.0, "TextInput rect missing after frame");
    let (cx, cy) = (x + w / 2.0, y + h / 2.0);
    gallery.event(PlatformEvent::PointerMoved {
        window: WindowId(0),
        x: cx,
        y: cy,
    });
    let handled = gallery.event(PlatformEvent::PointerButton {
        window: WindowId(0),
        pressed: true,
        x: cx,
        y: cy,
        button: 0,
        pointer: 0,
    });
    assert!(handled, "pointer press on the TextInput was not handled");
    assert!(
        gallery.text_input.focused,
        "pointer press did not focus the TextInput"
    );
}

/// The caret rect the runtime reads through `Application::ime_cursor_area`.
fn caret_rect(gallery: &Gallery) -> [f32; 4] {
    gallery
        .ime_cursor_area(WindowId(0))
        .expect("focused TextInput reports an IME caret rect")
}

/// Caret offset from the field origin, so scale comparisons ignore the field's
/// absolute position.
fn caret_offset(caret: [f32; 4], field: [f32; 4]) -> [f32; 4] {
    [caret[0] - field[0], caret[1] - field[1], caret[2], caret[3]]
}

fn assert_finite_non_negative(caret: [f32; 4], context: &str) {
    assert!(
        caret.iter().all(|v| v.is_finite()),
        "caret not finite {caret:?} {context}"
    );
    assert!(
        caret[2] >= 0.0 && caret[3] >= 0.0,
        "caret has negative size {caret:?} {context}"
    );
}

/// Drive focus, preedit and commit through the real event path, frame after
/// each step, and return the caret and field seen after every step.
fn type_mixed_text(scale_factor: f32) -> Vec<([f32; 4], [f32; 4])> {
    let mut gallery = Gallery::new();
    select(&mut gallery, TEXT_INPUT_PAGE);
    bring_text_input_into_view(&mut gallery, scale_factor);
    focus_text_input(&mut gallery);
    frame_at(&mut gallery, scale_factor);
    let mut steps = vec![(caret_rect(&gallery), gallery.text_input_rect)];

    gallery.event(PlatformEvent::ImePreedit {
        window: WindowId(0),
        text: PREEDIT.to_owned(),
        cursor: None,
    });
    frame_at(&mut gallery, scale_factor);
    steps.push((caret_rect(&gallery), gallery.text_input_rect));

    gallery.event(PlatformEvent::ImeCommit {
        window: WindowId(0),
        text: COMMITTED.to_owned(),
    });
    frame_at(&mut gallery, scale_factor);
    assert_eq!(
        gallery.text_input.text, COMMITTED,
        "commit text at {scale_factor}"
    );
    steps.push((caret_rect(&gallery), gallery.text_input_rect));
    steps
}

#[test]
fn every_page_lays_out_finite_rects_at_every_scale() {
    let mut gallery = Gallery::new();
    for page in all_pages() {
        select(&mut gallery, page);
        for scale in SCALES {
            let scene = frame_at(&mut gallery, scale);
            assert!(
                !scene.commands().is_empty(),
                "empty scene on page {page:?} at {scale}"
            );
            for rect in layout_rects(&mut gallery, scale) {
                assert!(
                    rect.iter().all(|v| v.is_finite()),
                    "non-finite rect on page {page:?} at scale {scale}: {rect:?}"
                );
                assert!(
                    rect[2] >= 0.0 && rect[3] >= 0.0,
                    "negative size on page {page:?} at scale {scale}: {rect:?}"
                );
            }
        }
    }
}

#[test]
fn logical_layout_is_scale_invariant_on_every_page() {
    let mut gallery = Gallery::new();
    for page in all_pages() {
        select(&mut gallery, page);
        frame_at(&mut gallery, 1.0);
        let base = layout_rects(&mut gallery, 1.0);
        for scale in SCALES {
            let drift = max_drift(&base, &layout_rects(&mut gallery, scale));
            assert!(
                drift <= LAYOUT_TOLERANCE,
                "logical layout drift {drift} px > {LAYOUT_TOLERANCE} on page {page:?} at scale {scale}"
            );
        }
    }
}

#[test]
fn ime_caret_left_and_top_edges_stay_inside_the_focused_text_input_at_every_scale() {
    let base = type_mixed_text(1.0);
    for scale in SCALES {
        let steps = type_mixed_text(scale);
        for (step, ((caret, field), (base_caret, base_field))) in
            steps.iter().zip(&base).enumerate()
        {
            let context = format!("step {step} at scale {scale}");
            assert_finite_non_negative(*caret, &context);
            let [cx, cy, _, _] = *caret;
            let [fx, fy, fw, _] = *field;
            assert!(
                cx >= fx,
                "caret left edge {cx} left of field {fx} {context}"
            );
            assert!(
                cx <= fx + fw,
                "caret x {cx} past field right edge {} {context}",
                fx + fw
            );
            assert!(cy >= fy, "caret top {cy} above field top {fy} {context}");

            let drift = caret_offset(*caret, *field)
                .iter()
                .zip(caret_offset(*base_caret, *base_field))
                .map(|(a, b)| (a - b).abs())
                .fold(0.0_f32, f32::max);
            assert!(
                drift <= CARET_TOLERANCE,
                "caret offset drift {drift} px > {CARET_TOLERANCE} {context}"
            );
        }
    }
}

/// Regression: the TextInput overlay used to be drawn in a one-line (21 px)
/// marker rect, so the caret's bottom edge sat 8 px below the field. The
/// marker now reserves the theme control height.
#[test]
fn ime_caret_bottom_edge_stays_inside_the_text_input_at_every_scale() {
    for scale in SCALES {
        for (step, (caret, field)) in type_mixed_text(scale).into_iter().enumerate() {
            let [_, cy, _, ch] = caret;
            let [_, fy, _, fh] = field;
            assert!(
                cy + ch <= fy + fh,
                "caret bottom {} below field bottom {} at step {step}, scale {scale}: caret {caret:?} field {field:?}",
                cy + ch,
                fy + fh
            );
        }
    }
}

/// Regression: `scroll_offset` used to be read but never written, so overflowing
/// text pushed the caret and IME candidate rect out of the field.
/// `scroll_caret_into_view` now keeps the caret inside.
#[test]
fn ime_caret_stays_inside_the_field_when_text_overflows() {
    let long_text = "abcdefghij".repeat(30);
    for scale in SCALES {
        let mut gallery = Gallery::new();
        select(&mut gallery, TEXT_INPUT_PAGE);
        bring_text_input_into_view(&mut gallery, scale);
        focus_text_input(&mut gallery);
        gallery.event(PlatformEvent::ImeCommit {
            window: WindowId(0),
            text: long_text.clone(),
        });
        frame_at(&mut gallery, scale);
        let field = gallery.text_input_rect;
        let caret = caret_rect(&gallery);
        assert!(
            caret[0] + caret[2] <= field[0] + field[2],
            "caret x {} past field right edge {} at scale {scale}: caret {caret:?} field {field:?}",
            caret[0],
            field[0] + field[2]
        );
        assert!(
            caret[0] >= field[0],
            "caret x {} before field left edge {} at scale {scale}",
            caret[0],
            field[0]
        );
    }
}

/// Measurement report (not a gate): prints the largest logical drift per page
/// and scale, and the caret drift on the TextInput page. Run with
/// `cargo test -p acme-gallery dpi_tests::measure_drift_report -- --ignored --nocapture`.
#[test]
#[ignore = "measurement report; run with --ignored --nocapture"]
fn measure_drift_report() {
    let mut gallery = Gallery::new();
    for page in all_pages() {
        select(&mut gallery, page);
        frame_at(&mut gallery, 1.0);
        let base = layout_rects(&mut gallery, 1.0);
        let mut line = format!("{:>2}/{:<2}", page.0, page.1);
        for scale in SCALES {
            let drift = max_drift(&base, &layout_rects(&mut gallery, scale));
            line.push_str(&format!("  {scale:>4}:{drift:.4}"));
        }
        eprintln!("{line}");
    }
    let base = type_mixed_text(1.0);
    for scale in SCALES {
        let steps = type_mixed_text(scale);
        for (step, ((caret, field), (base_caret, base_field))) in
            steps.iter().zip(&base).enumerate()
        {
            let a = caret_offset(*caret, *field);
            let b = caret_offset(*base_caret, *base_field);
            eprintln!("caret step {step} scale {scale}: offset {a:?} base {b:?} field {field:?}");
        }
    }
}
