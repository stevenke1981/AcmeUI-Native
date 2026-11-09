//! Automated Gallery page, dispatch and paint evidence; no human validation.
use super::*;

fn context() -> WidgetLayoutContext {
    WidgetLayoutContext {
        body_font_size: 14.0,
        body_line_height: 20.0,
        label_font_size: 13.0,
        control_height: 36.0,
        scale_factor: 1.0,
    }
}

fn find<'a>(
    widget: &'a WidgetNode<GalleryMessage>,
    layout: &'a LayoutNode,
    key: &str,
) -> Option<(&'a WidgetNode<GalleryMessage>, &'a LayoutNode)> {
    if widget.key().is_some_and(|k| k.as_str() == key) {
        return Some((widget, layout));
    }
    widget
        .children()
        .iter()
        .zip(&layout.children)
        .find_map(|(w, l)| find(w, l, key))
}

#[test]
fn mobile_page_exposes_empty_partial_and_complete_progress_with_readable_labels() {
    let gallery = Gallery::new();
    let widget = gallery.mobile_page();
    let layout = widget.to_layout_with_context(NodeId::new(1), &context());
    let snapshot = LayoutEngine::new()
        .compute(&layout, (700.0, 2800.0))
        .unwrap();
    for (key, fraction, description) in [
        ("gallery_progress_empty", 0.0, false),
        (
            "gallery_progress_quantity",
            gallery.mobile_value as f32 / 10.0,
            true,
        ),
        ("gallery_progress_complete", 1.0, true),
    ] {
        let (progress, _) = find(&widget, &layout, key).expect("Gallery progress example");
        assert_eq!(progress.children().len(), 2 + usize::from(description));
        let (_, track) = find(&widget, &layout, &format!("{key}__track")).unwrap();
        let rect = snapshot.get(track.id).unwrap();
        let fill = snapshot.get(track.children[0].id).unwrap();
        assert!(rect.width > 0.0);
        assert_eq!(rect.height, 6.0);
        // The page's intrinsic width varies with its examples; Taffy rounds
        // final edges to logical pixels, as in the component layout tests.
        assert_eq!(fill.width, (rect.width * fraction).round());
        let WidgetNode::Label(percent) = progress.children().last().unwrap() else {
            panic!("progress percentage must be readable text");
        };
        assert_eq!(percent.text, format!("{}%", (fraction * 100.0) as u32));
    }
}

#[test]
fn stepper_activation_rebuilds_quantity_progress() {
    let mut gallery = Gallery::new();
    let mut widths = Vec::new();
    for value in [2, 4] {
        let widget = gallery.mobile_page();
        let layout = widget.to_layout_with_context(NodeId::new(1), &context());
        let snapshot = LayoutEngine::new()
            .compute(&layout, (700.0, 2800.0))
            .unwrap();
        let (progress, _) = find(&widget, &layout, "gallery_progress_quantity").unwrap();
        let WidgetNode::Label(percent) = progress.children().last().unwrap() else {
            panic!("expected percentage");
        };
        assert_eq!(percent.text, format!("{}%", value * 10));
        let (_, fill) = find(&widget, &layout, "gallery_progress_quantity__fill").unwrap();
        widths.push(snapshot.get(fill.id).unwrap().width);
        if value == 2 {
            let mut hits = Vec::new();
            collect_hit_regions(&widget, &layout, &snapshot, false, &mut hits);
            gallery.button_info = hits;
            let index = gallery
                .button_info
                .iter()
                .position(|h| h.message == GalleryMessage::MobileValue(4))
                .unwrap();
            let [x, y, w, h] = gallery.button_info[index].rect;
            gallery.cursor = (x + w / 2.0, y + h / 2.0);
            assert_eq!(gallery.hit(), Some(index));
            assert!(gallery.activate(index));
        }
    }
    assert!(widths[1] > widths[0]);
    assert!((widths[1] / widths[0] - 2.0).abs() < 0.001);
}

#[test]
fn progress_paints_track_before_fill_with_semantic_colours_and_viewport_clips() {
    let mut gallery = Gallery::new();
    for pack in ["default", "apple"] {
        gallery.theme_pack = pack.into();
        for dark in [false, true] {
            gallery.dark = dark;
            let theme = gallery.current_theme();
            for value in [0.0, 25.0, 100.0] {
                let widget = WidgetNode::from(mobile_progress(value, 100.0).key("job"));
                let layout = widget.to_layout_with_context(NodeId::new(1), &context());
                let snapshot = LayoutEngine::new()
                    .compute(&layout, (400.0, 200.0))
                    .unwrap();
                let (_, track) = find(&widget, &layout, "job__track").unwrap();
                let (_, fill) = find(&widget, &layout, "job__fill").unwrap();
                // Partial clip, then entirely outside the viewport after scrolling.
                let clip = [20.0, 10.0, 300.0, 100.0];
                for scroll in [0.0, 200.0] {
                    let mut frame = Frame::default();
                    render_content(
                        &mut frame,
                        &widget,
                        &layout,
                        &snapshot,
                        &theme,
                        1.25,
                        scroll,
                        clip,
                        &mut 0,
                        None,
                        None,
                        &mut gallery.fonts,
                        &mut gallery.atlas,
                    );
                    assert!(frame.quads.is_empty());
                    assert_eq!(frame.clipped_quads.len(), 2);
                    for (painted, part, colour) in [
                        (&frame.clipped_quads[0], track, theme.colors.muted),
                        (&frame.clipped_quads[1], fill, theme.colors.primary),
                    ] {
                        let r = snapshot.get(part.id).unwrap();
                        assert_eq!(painted.quad.rect, [r.x, r.y - scroll, r.width, r.height]);
                        assert_eq!(painted.quad.color, rgba(colour));
                        assert_eq!(painted.quad.radius, 3.0);
                        assert_eq!(painted.clip, clip);
                    }
                    let scene = scene_from_frame(&frame);
                    let widths: Vec<_> = scene
                        .commands()
                        .iter()
                        .filter_map(|cmd| {
                            if let acme_core::DrawCommand::Quad(q) = cmd {
                                Some(q.rect.size.width.get())
                            } else {
                                None
                            }
                        })
                        .collect();
                    assert_eq!(widths, vec![384.0, 384.0 * value / 100.0]);
                }
            }
        }
    }
}

#[test]
fn shaped_percentage_glyphs_stay_below_the_progress_track() {
    let mut gallery = Gallery::new();
    let theme = gallery.current_theme();
    for scale in [1.0, 1.25, 2.0] {
        let widget = WidgetNode::from(mobile_progress(25.0, 100.0).key("job").label("Download"));
        let layout = widget.to_layout_with_context(NodeId::new(1), &context());
        let snapshot = LayoutEngine::new()
            .compute_with_text(&layout, (400.0, 200.0), &mut gallery.fonts, scale)
            .unwrap();
        let (_, track) = find(&widget, &layout, "job__track").unwrap();
        let track = snapshot.get(track.id).unwrap();
        let mut frame = Frame::default();
        render_content(
            &mut frame,
            &widget,
            &layout,
            &snapshot,
            &theme,
            scale,
            0.0,
            [0.0, 0.0, 400.0, 200.0],
            &mut 0,
            None,
            None,
            &mut gallery.fonts,
            &mut gallery.atlas,
        );
        let percent = frame.text.last().unwrap();
        assert!(!percent.prepared.glyphs.is_empty());
        for glyph in &percent.prepared.glyphs {
            let top = percent.origin[1] + glyph.y as f32 / scale;
            assert!(
                top >= track.y + track.height + 4.0,
                "percentage ink overlaps the track: {top}, track={track:?}"
            );
        }
    }
}
