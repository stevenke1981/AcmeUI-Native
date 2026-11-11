//! Retained identities through the showcase's actual builders and frame path.

use std::collections::{HashMap, HashSet};

use acme_layout::LayoutSnapshot;
use acme_widgets::retained_binding::is_synthetic;

use super::*;

const VIEWPORT: (f32, f32) = (1100.0, 700.0);

fn gallery() -> Gallery {
    let mut gallery = Gallery::new();
    // Keep tests independent of the saved preference without writing it.
    gallery.theme_pack = "default".into();
    gallery.dark = false;
    gallery
}

fn context(gallery: &Gallery, scale: f32) -> WidgetLayoutContext {
    let theme = gallery.current_theme();
    WidgetLayoutContext {
        body_font_size: theme.typography.body,
        body_line_height: theme.typography.body * theme.typography.line_height,
        label_font_size: theme.typography.label,
        control_height: 36.0,
        scale_factor: scale,
    }
}

fn frame_context(scale: f32) -> FrameContext {
    FrameContext {
        window: WindowId(0),
        logical_width: VIEWPORT.0,
        logical_height: VIEWPORT.1,
        scale_factor: scale,
    }
}

fn layout_ids(root: &LayoutNode) -> Vec<NodeId> {
    let mut ids = vec![root.id];
    for child in &root.children {
        ids.extend(layout_ids(child));
    }
    ids
}

fn assert_bound(
    widget: &WidgetNode<GalleryMessage>,
    layout: &LayoutNode,
    binding: &RetainedBinding<'_>,
) -> usize {
    assert_eq!(binding.id_for(widget), Some(layout.id));
    assert!(!is_synthetic(layout.id));
    assert_eq!(widget.children().len(), layout.children.len());
    1 + widget
        .children()
        .iter()
        .zip(&layout.children)
        .map(|(widget, layout)| assert_bound(widget, layout, binding))
        .sum::<usize>()
}

fn assert_same_shape(bound: &LayoutNode, sequential: &LayoutNode) {
    assert_eq!(bound.style, sequential.style);
    assert_eq!(bound.measure, sequential.measure);
    assert_eq!(bound.children.len(), sequential.children.len());
    for (bound, sequential) in bound.children.iter().zip(&sequential.children) {
        assert_same_shape(bound, sequential);
    }
}

fn assert_same_geometry(
    bound: &LayoutNode,
    bound_snapshot: &LayoutSnapshot,
    sequential: &LayoutNode,
    sequential_snapshot: &LayoutSnapshot,
) {
    assert_eq!(
        bound_snapshot.get(bound.id),
        sequential_snapshot.get(sequential.id)
    );
    assert!(bound_snapshot.get(bound.id).is_some());
    assert_eq!(
        bound_snapshot.scroll_metrics(bound.id),
        sequential_snapshot.scroll_metrics(sequential.id)
    );
    for (bound, sequential) in bound.children.iter().zip(&sequential.children) {
        assert_same_geometry(bound, bound_snapshot, sequential, sequential_snapshot);
    }
}

fn compute(gallery: &mut Gallery, root: &mut LayoutNode, scale: f32) -> LayoutSnapshot {
    apply_gallery_styles(root, VIEWPORT.0, VIEWPORT.1);
    gallery
        .layout
        .compute_with_text(root, VIEWPORT, &mut gallery.fonts, scale)
        .expect("finite test viewport")
}

fn chrome_ids(root: &LayoutNode) -> Vec<NodeId> {
    let mut ids = vec![root.id, root.children[1].id];
    ids.extend(layout_ids(&root.children[0]));
    ids.extend(layout_ids(&root.children[1].children[0]));
    ids.push(root.children[1].children[1].id);
    ids
}

#[test]
fn every_category_has_complete_binding_and_unique_retained_layout_ids() {
    let mut gallery = gallery();
    // Offset the allocator so sequential fallback cannot masquerade as success.
    let seed = column::<GalleryMessage>().key("seed").build();
    gallery
        .retained
        .tree
        .reconcile_roots(&widget_to_views(&[seed]))
        .unwrap();

    for category in 0..CATEGORIES.len() {
        gallery.selected_category = category;
        let description = gallery.description();
        let context = context(&gallery, 1.0);
        let root = retained_layout_root(&description, &mut gallery.retained, &context);
        let binding = bind_retained(std::slice::from_ref(&description), &gallery.retained.tree);
        assert_eq!(binding.mismatches(), 0, "category {category}");
        assert_eq!(assert_bound(&description, &root, &binding), binding.len());
        assert_ne!(root.id, NodeId::new(1));
        let ids = layout_ids(&root);
        assert_eq!(ids.iter().copied().collect::<HashSet<_>>().len(), ids.len());
    }
}

#[test]
fn retained_layout_preserves_computed_geometry_in_all_categories_and_modes() {
    let mut gallery = gallery();
    for dark in [false, true] {
        gallery.dark = dark;
        for scale in [1.0, 1.5] {
            for category in 0..CATEGORIES.len() {
                gallery.selected_category = category;
                let description = gallery.description();
                let context = context(&gallery, scale);
                let mut bound = retained_layout_root(&description, &mut gallery.retained, &context);
                let mut sequential = description.to_layout_with_context(NodeId::new(1), &context);
                assert_same_shape(&bound, &sequential);
                let bound_snapshot = compute(&mut gallery, &mut bound, scale);
                let sequential_snapshot = compute(&mut gallery, &mut sequential, scale);
                assert_same_shape(&bound, &sequential);
                assert_same_geometry(&bound, &bound_snapshot, &sequential, &sequential_snapshot);
            }
        }
    }
}

#[test]
fn repeated_frames_and_theme_pack_transitions_preserve_all_ids() {
    let mut gallery = gallery();
    let packs = std::iter::once("default").chain(available_themes().iter().copied());
    let packs: Vec<_> = packs.collect();
    for category in 0..CATEGORIES.len() {
        gallery.selected_category = category;
        let mut previous = None;
        for pack in &packs {
            gallery.theme_pack = (*pack).into();
            for dark in [false, true, false] {
                gallery.dark = dark;
                let description = gallery.description();
                let context = context(&gallery, 1.0);
                let root = retained_layout_root(&description, &mut gallery.retained, &context);
                let ids = layout_ids(&root);
                if let Some(previous) = &previous {
                    assert_eq!(
                        &ids, previous,
                        "category {category}, pack {pack}, dark {dark}"
                    );
                }
                previous = Some(ids);
            }
        }
    }
}

#[test]
fn category_replacement_keeps_chrome_ids_and_replaces_page_identity() {
    let mut gallery = gallery();
    let mut before = None;
    let mut page_ids = HashSet::new();
    for category in 0..CATEGORIES.len() {
        gallery.selected_category = category;
        let description = gallery.description();
        let context = context(&gallery, 1.0);
        let root = retained_layout_root(&description, &mut gallery.retained, &context);
        let ids = chrome_ids(&root);
        if let Some(before) = &before {
            assert_eq!(&ids, before, "chrome changed in category {category}");
        }
        before = Some(ids);
        let (_, page) = gallery_page_nodes(&description, &root).unwrap();
        assert!(
            page_ids.insert(page.id),
            "category reused another page's identity"
        );
    }
}

fn sidebar_button_ids(root: &LayoutNode, tree: &RetainedTree) -> HashMap<String, NodeId> {
    root.children[0]
        .children
        .iter()
        .skip(2)
        .map(|button| {
            (
                tree.get(button.id).unwrap().key.as_str().to_string(),
                button.id,
            )
        })
        .collect()
}

#[test]
fn actual_sidebar_keys_survive_insertion_and_reorder() {
    let mut gallery = gallery();
    let first = gallery.description();
    let context = context(&gallery, 1.0);
    let root = retained_layout_root(&first, &mut gallery.retained, &context);
    let before = sidebar_button_ids(&root, &gallery.retained.tree);
    assert_eq!(before.len(), CATEGORIES.len());
    assert!(before.keys().all(|key| !key.is_empty()));

    let mut second = gallery.description();
    let WidgetNode::Row(root) = &mut second else {
        panic!("root row")
    };
    let WidgetNode::Column(sidebar) = &mut root.children[0] else {
        panic!("sidebar column")
    };
    sidebar.children[2..].reverse();
    sidebar
        .children
        .insert(2, button("extra_category", "Extra").into());
    let root = retained_layout_root(&second, &mut gallery.retained, &context);
    let after = sidebar_button_ids(&root, &gallery.retained.tree);
    for (key, id) in &before {
        assert_eq!(after.get(key), Some(id));
    }
    assert!(!before.values().any(|id| *id == after["extra_category"]));
}

#[test]
fn duplicate_sidebar_key_falls_back_and_next_valid_frame_recovers() {
    let mut gallery = gallery();
    let context = context(&gallery, 1.0);
    let valid = gallery.description();
    let before = retained_layout_root(&valid, &mut gallery.retained, &context);
    assert!(!gallery.retained.failure_logged);

    let mut invalid = gallery.description();
    let WidgetNode::Row(root) = &mut invalid else {
        panic!("root row")
    };
    let WidgetNode::Column(sidebar) = &mut root.children[0] else {
        panic!("sidebar column")
    };
    let WidgetNode::Button(first) = &sidebar.children[2] else {
        panic!("category button")
    };
    let duplicate = first.key.clone();
    let WidgetNode::Button(second) = &mut sidebar.children[3] else {
        panic!("category button")
    };
    second.key = duplicate;
    assert!(
        gallery
            .retained
            .tree
            .reconcile_roots(&widget_to_views(std::slice::from_ref(&invalid)))
            .is_err()
    );
    let fallback = retained_layout_root(&invalid, &mut gallery.retained, &context);
    assert!(gallery.retained.failure_logged);
    assert_eq!(
        fallback,
        invalid.to_layout_with_context(NodeId::new(1), &context)
    );
    let repeated = retained_layout_root(&invalid, &mut gallery.retained, &context);
    assert!(gallery.retained.failure_logged);
    assert_eq!(repeated, fallback);

    let valid = gallery.description();
    let recovered = retained_layout_root(&valid, &mut gallery.retained, &context);
    assert!(!gallery.retained.failure_logged);
    assert_eq!(layout_ids(&before), layout_ids(&recovered));

    retained_layout_root(&invalid, &mut gallery.retained, &context);
    assert!(gallery.retained.failure_logged);
}

#[test]
fn partial_binding_uses_sequential_ids_for_the_entire_layout() {
    let mut gallery = gallery();
    let description = gallery.description();
    let stale = column::<GalleryMessage>().key("stale").build();
    gallery
        .retained
        .tree
        .reconcile_roots(&widget_to_views(&[stale]))
        .unwrap();
    let binding = bind_retained(std::slice::from_ref(&description), &gallery.retained.tree);
    assert!(binding.mismatches() > 0);
    assert!(!gallery.retained.accept_binding(binding.mismatches()));
    assert!(gallery.retained.failure_logged);
    let context = context(&gallery, 1.0);
    let fallback = layout_root(&description, Some(&binding), &context);
    assert_eq!(
        fallback,
        description.to_layout_with_context(NodeId::new(1), &context)
    );
    retained_layout_root(&description, &mut gallery.retained, &context);
    assert!(!gallery.retained.failure_logged);
}

#[test]
fn real_frames_keep_hits_and_drawn_buttons_on_the_retained_geometry() {
    let mut gallery = gallery();
    let seed = column::<GalleryMessage>().key("seed").build();
    gallery
        .retained
        .tree
        .reconcile_roots(&widget_to_views(&[seed]))
        .unwrap();
    // Warm the actual frame before replacing Mobile with the first category.
    gallery.frame(frame_context(1.25));
    for category in 0..CATEGORIES.len() {
        gallery.selected_category = category;
        gallery.dark = category % 2 != 0;
        gallery.scroll = 20.0;
        let scene = gallery.frame(frame_context(1.25));
        assert!(!scene.commands().is_empty());
        let (actual_root, actual_snapshot) = gallery
            .last_frame_layout
            .take()
            .expect("frame captured its computed layout");
        let description = gallery.description();
        let binding = bind_retained(std::slice::from_ref(&description), &gallery.retained.tree);
        assert_eq!(binding.mismatches(), 0);
        let mut root = layout_root(&description, Some(&binding), &context(&gallery, 1.25));
        assert_bound(&description, &root, &binding);
        assert_ne!(root.id, NodeId::new(1));
        assert_ne!(
            layout_ids(&root),
            layout_ids(
                &description.to_layout_with_context(NodeId::new(1), &context(&gallery, 1.25))
            )
        );
        let snapshot = compute(&mut gallery, &mut root, 1.25);
        assert_eq!(actual_root, root, "frame did not use the retained layout");
        assert_eq!(
            actual_snapshot, snapshot,
            "frame snapshot differs from the retained layout"
        );
        let mut expected = Vec::new();
        collect_hit_regions(&description, &root, &snapshot, false, &mut expected);
        assert_eq!(gallery.button_info.len(), expected.len());
        for (actual, expected) in gallery.button_info.iter().zip(&expected) {
            assert_eq!(actual.message, expected.message);
            assert_eq!(actual.rect, expected.rect);
            assert_eq!(actual.scrolled, expected.scrolled);
            let [x, mut y, width, height] = expected.rect;
            if expected.scrolled {
                y -= gallery.scroll;
            }
            let rect = acme_core::Rect::new(x, y, width, height);
            if expected.message == GalleryMessage::CollapsibleToggle {
                // This header preserves a transparent Row: the chevron and
                // title are its paint evidence, rather than a button surface.
                let text: Vec<_> = scene
                    .commands()
                    .iter()
                    .filter_map(|command| match command {
                        acme_core::DrawCommand::Text(text)
                            if (x..x + width).contains(&text.origin.x.get())
                                && (y..y + height).contains(&text.origin.y.get()) =>
                        {
                            Some(text)
                        }
                        _ => None,
                    })
                    .collect();
                assert_eq!(text.len(), 2, "header chevron and title must be painted");
                for text in text {
                    assert!(!text.glyphs.is_empty());
                    for glyph in &text.glyphs {
                        let left = text.origin.x.get() * 1.25 + glyph.x;
                        let top = text.origin.y.get() * 1.25 + glyph.y;
                        assert!(left >= x * 1.25 && left + glyph.width <= (x + width) * 1.25);
                        assert!(top >= y * 1.25 && top + glyph.height <= (y + height) * 1.25);
                    }
                }
                continue;
            }
            assert!(
                scene.commands().iter().any(|command| matches!(command,
                    acme_core::DrawCommand::Quad(quad) if quad.rect == rect
                )),
                "button {:?} missing from scene",
                expected.message
            );
        }
        let scroll_id = extract_gallery_ids(&root).scroll_view;
        let metrics = snapshot.scroll_metrics(scroll_id).unwrap();
        assert_eq!(
            gallery.max_scroll,
            (metrics.content_height - metrics.viewport_height).max(0.0)
        );
    }
}

#[test]
fn mobile_message_updates_keep_ids_and_disabled_button_out_of_hits() {
    let mut gallery = gallery();
    gallery.selected_category = CATEGORIES.len() - 1;
    gallery.frame(frame_context(1.0));
    let description = gallery.description();
    let context = context(&gallery, 1.0);
    let before = retained_layout_root(&description, &mut gallery.retained, &context);
    let presses: Vec<_> = gallery
        .button_info
        .iter()
        .filter_map(|hit| match hit.message {
            GalleryMessage::MobilePressed(size) => Some(size),
            _ => None,
        })
        .collect();
    assert_eq!(
        presses,
        [
            MobileButtonSize::Sm,
            MobileButtonSize::Md,
            MobileButtonSize::Lg
        ]
    );
    let index = gallery
        .button_info
        .iter()
        .position(|hit| hit.message == GalleryMessage::MobilePressed(MobileButtonSize::Md))
        .unwrap();
    assert!(gallery.activate(index));
    assert_eq!(gallery.last_mobile_press, Some(MobileButtonSize::Md));
    gallery.frame(frame_context(1.0));
    let description = gallery.description();
    let after = retained_layout_root(&description, &mut gallery.retained, &context);
    assert_eq!(layout_ids(&before), layout_ids(&after));
}
