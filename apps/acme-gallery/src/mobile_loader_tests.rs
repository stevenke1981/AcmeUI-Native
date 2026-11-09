//! Automated Gallery evidence; no animation or human/device acceptance claims.
use super::*;
use acme_ui::mobile::mobile_loader;
use acme_widgets::LabelWrap;

fn context(theme: &Theme) -> WidgetLayoutContext {
    WidgetLayoutContext {
        body_font_size: theme.typography.body,
        body_line_height: theme.typography.body * theme.typography.line_height,
        label_font_size: theme.typography.label,
        control_height: theme.control_heights.md,
        scale_factor: 1.0,
    }
}

fn find<'a>(
    widget: &'a WidgetNode<GalleryMessage>,
    key: &str,
) -> Option<&'a WidgetNode<GalleryMessage>> {
    if widget.key().is_some_and(|k| k.as_str() == key) {
        Some(widget)
    } else {
        widget.children().iter().find_map(|child| find(child, key))
    }
}

#[test]
fn mobile_page_has_distinct_indicator_short_and_long_message_examples() {
    let mut gallery = Gallery::new();
    let widget = gallery.mobile_page();
    for (key, count) in [
        ("gallery_loader_indicator", 1),
        ("gallery_loader_message", 2),
        ("gallery_loader_long", 2),
    ] {
        let loader = find(&widget, key).expect("Gallery loader example");
        assert_eq!(loader.children().len(), count);
        let mut hits = Vec::new();
        let layout =
            loader.to_layout_with_context(NodeId::new(1), &context(&gallery.current_theme()));
        let snapshot = LayoutEngine::new()
            .compute_with_text(&layout, (240.0, 1000.0), &mut gallery.fonts, 1.0)
            .unwrap();
        collect_hit_regions(loader, &layout, &snapshot, false, &mut hits);
        assert!(hits.is_empty(), "a loader must not be an action");
        if count == 2 {
            let WidgetNode::Label(message) = &loader.children()[1] else {
                panic!("message label")
            };
            assert!(!message.text.is_empty());
        }
    }
    // Also exercise the actual page's retained layout path with all examples.
    let description = gallery.description();
    assert!(
        gallery
            .retained
            .tree
            .reconcile_roots(&widget_to_views(std::slice::from_ref(&description)))
            .is_ok()
    );
    let binding = bind_retained(std::slice::from_ref(&description), &gallery.retained.tree);
    assert_eq!(binding.mismatches(), 0);
}

#[test]
fn loader_messages_paint_multiple_lines_with_themed_foreground_and_scroll_clips() {
    let mut gallery = Gallery::new();
    for pack in ["default", "apple"] {
        gallery.theme_pack = pack.into();
        for dark in [false, true] {
            gallery.dark = dark;
            let theme = gallery.current_theme();
            for scale in [1.0, 1.25, 2.0] {
                let widget = WidgetNode::from(
                    mobile_loader()
                        .message("LoadingWithoutSpacesLoadingWithoutSpacesLoadingWithoutSpaces"),
                );
                let layout = widget.to_layout_with_context(NodeId::new(1), &context(&theme));
                let snapshot = LayoutEngine::new()
                    .compute_with_text(&layout, (180.0, 1000.0), &mut gallery.fonts, scale)
                    .unwrap();
                let message = snapshot.get(layout.children[1].id).unwrap();
                for scroll in [0.0, 1000.0] {
                    let clip = [10.0, 10.0, 160.0, 300.0];
                    let mut frame = Frame::default();
                    render_content(
                        &mut frame,
                        &widget,
                        &layout,
                        &snapshot,
                        &theme,
                        scale,
                        scroll,
                        clip,
                        &mut 0,
                        None,
                        None,
                        &mut gallery.fonts,
                        &mut gallery.atlas,
                    );
                    assert!(frame.quads.is_empty() && frame.clipped_quads.is_empty());
                    assert_eq!(frame.text.len(), 2);
                    for run in &frame.text {
                        assert_eq!(run.color, rgba(theme.colors.foreground));
                        assert_eq!(run.clip, Some(clip));
                        assert!(!run.prepared.glyphs.is_empty());
                    }
                    let run = frame.text.last().unwrap();
                    let min_y = run.prepared.glyphs.iter().map(|g| g.y).min().unwrap();
                    let max_y = run.prepared.glyphs.iter().map(|g| g.y).max().unwrap();
                    assert!(
                        max_y - min_y > (theme.typography.body * scale) as i32,
                        "long message must be painted on multiple lines"
                    );
                    for glyph in &run.prepared.glyphs {
                        let x = run.origin[0] + glyph.x as f32 / scale;
                        assert!(
                            x >= message.x - 2.0 && x < message.x + message.width + 2.0,
                            "message ink escapes the layout width: {x}, {message:?}"
                        );
                        let y = run.origin[1] + glyph.y as f32 / scale + scroll;
                        assert!(y >= message.y - 2.0 && y < message.y + message.height + 2.0);
                    }
                }
            }
        }
    }
}

#[test]
fn gallery_label_paint_respects_existing_wrap_modes_and_line_height() {
    let mut gallery = Gallery::new();
    let theme = gallery.current_theme();
    for wrap in [LabelWrap::None, LabelWrap::Word, LabelWrap::Character] {
        let text = acme_widgets::label_builder("AAAAAAAA AAAA AAAAAAAA AAAA AAAAAAAA")
            .wrap(wrap)
            .line_height(25.0)
            .build();
        // Match the Gallery: labels stretch in a width-constrained column.
        let widget = column().width(120.0).child(text).build();
        let layout = widget.to_layout_with_context(NodeId::new(1), &context(&theme));
        let snapshot = LayoutEngine::new()
            .compute_with_text(&layout, (120.0, 1000.0), &mut gallery.fonts, 1.0)
            .unwrap();
        let mut frame = Frame::default();
        render_content(
            &mut frame,
            &widget,
            &layout,
            &snapshot,
            &theme,
            1.0,
            0.0,
            [0.0, 0.0, 120.0, 1000.0],
            &mut 0,
            None,
            None,
            &mut gallery.fonts,
            &mut gallery.atlas,
        );
        let run = &frame.text[0];
        let low = run.prepared.glyphs.iter().map(|g| g.y).min().unwrap();
        let high = run.prepared.glyphs.iter().map(|g| g.y).max().unwrap();
        // Identical glyph bearings isolate the explicit 25 px line advance.
        assert!(run.prepared.glyphs.iter().all(|g| (g.y - low) % 25 == 0));
        if wrap == LabelWrap::None {
            assert!(high - low < 25);
        } else {
            assert!(high - low >= 50);
            let r = snapshot.get(layout.children[0].id).unwrap();
            assert!(
                run.prepared
                    .glyphs
                    .iter()
                    .all(|g| run.origin[1] + (g.y as f32) < r.height + 2.0)
            );
        }
    }
}
