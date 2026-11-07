//! ROADMAP 003 evidence: the Gallery frame path uses retained `NodeId`s.
//!
//! These tests build the real Gallery description for every page, reconcile it
//! into the retained tree exactly as `frame` does, and check that the binding is
//! complete, that layout IDs are stable across frames and category switches,
//! and that the bound layout has the same shape as the sequential one.

use acme_layout::LayoutSnapshot;

use super::*;

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

/// Layout roots (bound, sequential) and the binding mismatch count for the
/// gallery's current page. Panics with the page when reconcile fails.
fn build_layouts(gallery: &mut Gallery) -> (LayoutNode, LayoutNode, usize) {
    let description = gallery.description();
    if let Err(error) = gallery.retained.reconcile(&description) {
        panic!(
            "reconcile failed for category {} page {}: {error}",
            gallery.selected_category, gallery.selected_page
        );
    }
    let binding = bind_retained(std::slice::from_ref(&description), gallery.retained.tree());
    let context = build_layout_context(&build_theme(gallery.dark), 1.0);
    let bound = layout_root(&description, Some(&binding), &context);
    let sequential = layout_root(&description, None, &context);
    (bound, sequential, binding.mismatches())
}

const VIEWPORT: (f32, f32) = (1100.0, 760.0);

/// Apply the Gallery styles to `root` and compute its snapshot, as `frame` does.
fn computed(gallery: &mut Gallery, mut root: LayoutNode) -> (LayoutNode, LayoutSnapshot) {
    apply_gallery_styles(&mut root, VIEWPORT.0, VIEWPORT.1);
    let snapshot = gallery
        .layout
        .compute_with_text(&root, VIEWPORT, &mut gallery.fonts, 1.0)
        .expect("finite viewport");
    (root, snapshot)
}

/// Pre-order `[x, y, width, height]` of every layout node.
fn rects(node: &LayoutNode, snapshot: &LayoutSnapshot, out: &mut Vec<[f32; 4]>) {
    let rect = snapshot.get(node.id).expect("every layout node has a rect");
    out.push([rect.x, rect.y, rect.width, rect.height]);
    for child in &node.children {
        rects(child, snapshot, out);
    }
}

fn ids(node: &LayoutNode, out: &mut Vec<NodeId>) {
    out.push(node.id);
    for child in &node.children {
        ids(child, out);
    }
}

fn assert_same_shape(bound: &LayoutNode, sequential: &LayoutNode, path: &str) {
    assert_eq!(
        format!("{:?}", bound.style),
        format!("{:?}", sequential.style),
        "style differs at {path}"
    );
    assert_eq!(
        format!("{:?}", bound.measure),
        format!("{:?}", sequential.measure),
        "measure differs at {path}"
    );
    assert_eq!(
        bound.children.len(),
        sequential.children.len(),
        "child count differs at {path}"
    );
    for (index, (b, s)) in bound.children.iter().zip(&sequential.children).enumerate() {
        assert_same_shape(b, s, &format!("{path}/{index}"));
    }
}

#[test]
fn every_page_reconciles_and_binds_without_mismatch() {
    let mut gallery = Gallery::new();
    for page in all_pages() {
        select(&mut gallery, page);
        let (_, _, mismatches) = build_layouts(&mut gallery);
        assert_eq!(mismatches, 0, "binding mismatches on page {page:?}");
    }
}

#[test]
fn bound_layout_has_the_same_shape_as_sequential_layout_on_every_page() {
    let mut gallery = Gallery::new();
    for page in all_pages() {
        select(&mut gallery, page);
        let (bound, sequential, _) = build_layouts(&mut gallery);
        assert_same_shape(&bound, &sequential, &format!("{page:?}"));
        let (bound, bound_snapshot) = computed(&mut gallery, bound);
        let (sequential, sequential_snapshot) = computed(&mut gallery, sequential);
        let (mut a, mut b) = (Vec::new(), Vec::new());
        rects(&bound, &bound_snapshot, &mut a);
        rects(&sequential, &sequential_snapshot, &mut b);
        assert_eq!(a, b, "computed rects differ on page {page:?}");
    }
}

#[test]
fn bound_layout_ids_are_unique_on_every_page() {
    let mut gallery = Gallery::new();
    for page in all_pages() {
        select(&mut gallery, page);
        let (bound, _, _) = build_layouts(&mut gallery);
        let mut all = Vec::new();
        ids(&bound, &mut all);
        let unique: std::collections::HashSet<_> = all.iter().copied().collect();
        assert_eq!(
            unique.len(),
            all.len(),
            "duplicate layout IDs on page {page:?}"
        );
    }
}

#[test]
fn repeated_frame_keeps_every_layout_id() {
    let mut gallery = Gallery::new();
    for page in all_pages() {
        select(&mut gallery, page);
        let (first, _, _) = build_layouts(&mut gallery);
        let (second, _, _) = build_layouts(&mut gallery);
        let (mut a, mut b) = (Vec::new(), Vec::new());
        ids(&first, &mut a);
        ids(&second, &mut b);
        assert_eq!(a, b, "layout IDs changed between frames on page {page:?}");
    }
}

#[test]
fn sidebar_keeps_its_ids_when_the_category_changes() {
    let mut gallery = Gallery::new();
    select(&mut gallery, (0, 0));
    let (first, _, _) = build_layouts(&mut gallery);
    let sidebar_before = first.children[0].id;
    let mut before = Vec::new();
    ids(&first.children[0], &mut before);

    select(&mut gallery, (1, 0));
    let (second, _, _) = build_layouts(&mut gallery);
    let mut after = Vec::new();
    ids(&second.children[0], &mut after);

    assert_eq!(second.children[0].id, sidebar_before);
    assert_eq!(
        before, after,
        "sidebar IDs changed after switching category"
    );
}

#[test]
fn frame_uses_retained_ids_for_layout_and_accessibility() {
    let mut gallery = Gallery::new();
    let context = FrameContext {
        window: WindowId(0),
        logical_width: 1100.0,
        logical_height: 760.0,
        scale_factor: 1.0,
    };
    for page in all_pages() {
        select(&mut gallery, page);
        let scene = gallery.frame(context.clone());
        assert!(!scene.commands().is_empty(), "empty scene on page {page:?}");
        let tree = gallery
            .accessibility
            .tree_ref()
            .expect("accessibility tree built by frame");
        let root = gallery.retained.tree().roots()[0];
        assert_eq!(
            tree.root,
            acme_accessibility::to_access_id(root),
            "accessibility root is not the retained root on page {page:?}"
        );
    }
}

#[test]
fn accessibility_bounds_match_the_bound_layout_rects() {
    let mut gallery = Gallery::new();
    let context = FrameContext {
        window: WindowId(0),
        logical_width: VIEWPORT.0,
        logical_height: VIEWPORT.1,
        scale_factor: 1.0,
    };
    for page in all_pages() {
        select(&mut gallery, page);
        gallery.frame(context.clone());
        // Same description and retained tree, so the bound IDs are the frame's.
        let (bound, _, _) = build_layouts(&mut gallery);
        let (_, snapshot) = computed(&mut gallery, bound);
        let tree = gallery.accessibility.tree_ref().expect("tree");
        let mut checked = 0;
        for (id, node) in &tree.nodes {
            let Some(rect) = snapshot.get(NodeId::new(id.0)) else {
                continue;
            };
            let bounds = node
                .bounds()
                .unwrap_or_else(|| panic!("a11y node {id:?} lacks bounds on page {page:?}"));
            let expected = [rect.x, rect.y, rect.x + rect.width, rect.y + rect.height];
            let actual = [bounds.x0, bounds.y0, bounds.x1, bounds.y1];
            for (e, a) in expected.iter().zip(actual) {
                assert!(
                    (f64::from(*e) - a).abs() < 0.01,
                    "a11y bounds {actual:?} != layout {expected:?} for {id:?} on page {page:?}"
                );
            }
            checked += 1;
        }
        // Root, sidebar, its 8 category buttons and page content at least.
        assert!(
            checked > 10,
            "only {checked} a11y nodes matched layout on page {page:?}"
        );
    }
}
