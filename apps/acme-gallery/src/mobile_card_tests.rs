//! Headless evidence for the real Gallery Mobile page and card painter.
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
        .find_map(|(widget, layout)| find(widget, layout, key))
}

#[test]
fn mobile_page_has_all_card_variants_and_media_before_text() {
    let gallery = Gallery::new();
    let widget = gallery.mobile_page();
    let layout = widget.to_layout_with_context(NodeId::new(1), &context());
    let snapshot = LayoutEngine::new()
        .compute(&layout, (700.0, 2400.0))
        .unwrap();
    for (key, variant, children) in [
        ("gallery_card_title", CardVariant::Elevated, 1),
        ("gallery_card_outlined", CardVariant::Outlined, 1),
        ("gallery_card_media", CardVariant::Elevated, 2),
    ] {
        let (WidgetNode::Card(card), card_layout) = find(&widget, &layout, key).unwrap() else {
            panic!("Gallery example must use the mobile Card builder");
        };
        assert_eq!(card.variant, variant);
        assert_eq!(card.children.len(), children);
        let rect = snapshot.get(card_layout.id).unwrap();
        assert!(rect.width > 0.0 && rect.height > 0.0);
        if children == 2 {
            let media = snapshot.get(card_layout.children[0].id).unwrap();
            let text = snapshot.get(card_layout.children[1].id).unwrap();
            assert_eq!(media.height, 80.0);
            assert!(text.y >= media.y + media.height);
        }
    }
}

#[test]
fn a_card_surface_is_painted_before_its_interactive_media() {
    let mut gallery = Gallery::new();
    let theme = gallery.current_theme();
    let widget = WidgetNode::from(
        mobile_card("Title").media(button("media", "Open").on_click(GalleryMessage::DpiInfo)),
    );
    let layout = widget.to_layout_with_context(NodeId::new(1), &context());
    let snapshot = LayoutEngine::new()
        .compute(&layout, (400.0, 300.0))
        .unwrap();
    let mut frame = Frame::default();
    let clip = [0.0, 0.0, 400.0, 300.0];
    render_content(
        &mut frame,
        &widget,
        &layout,
        &snapshot,
        &theme,
        1.0,
        0.0,
        clip,
        &mut 0,
        None,
        None,
        &mut gallery.fonts,
        &mut gallery.atlas,
    );
    let rect = |layout: &LayoutNode| {
        let r = snapshot.get(layout.id).unwrap();
        acme_core::Rect::new(r.x, r.y, r.width, r.height)
    };
    let scene = scene_from_frame(&frame);
    let position = |rect| {
        scene.commands().iter().position(|command| {
        matches!(command, acme_core::DrawCommand::Quad(quad) if quad.rect == rect)
    }).unwrap()
    };
    assert!(
        position(rect(&layout)) < position(rect(&layout.children[0])),
        "the card surface must not cover its media button"
    );
    assert!(
        frame.quads.is_empty(),
        "all card content must share the ordered clipped path"
    );
    assert!(frame.clipped_quads.iter().all(|quad| quad.clip == clip));
}

#[test]
fn card_surfaces_use_theme_tokens_and_keep_the_viewport_clip_when_scrolled() {
    let mut gallery = Gallery::new();
    for dark in [false, true] {
        gallery.dark = dark;
        gallery.theme_pack = "default".into();
        let theme = gallery.current_theme();
        let widget = gallery.mobile_page();
        let layout = widget.to_layout_with_context(NodeId::new(1), &context());
        let snapshot = LayoutEngine::new()
            .compute(&layout, (700.0, 2400.0))
            .unwrap();
        for key in [
            "gallery_card_title",
            "gallery_card_outlined",
            "gallery_card_media",
        ] {
            let (WidgetNode::Card(card), card_layout) = find(&widget, &layout, key).unwrap() else {
                panic!("expected card");
            };
            let rect = snapshot.get(card_layout.id).unwrap();
            let clip = [200.0, 40.0, 700.0, 300.0];
            // Partial intersection, then fully scrolled past the viewport.
            for scroll in [rect.y, rect.y + rect.height + 100.0] {
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
                let expected_rect = [rect.x, rect.y - scroll, rect.width, rect.height];
                let painted = frame
                    .clipped_quads
                    .iter()
                    .find(|q| q.quad.rect == expected_rect)
                    .expect("card surface must carry the viewport clip");
                assert_eq!(painted.clip, clip);
                assert_eq!(
                    painted.quad.color,
                    rgba(if card.variant == CardVariant::Outlined {
                        theme.colors.background
                    } else {
                        theme.colors.surface
                    })
                );
                assert_eq!(painted.quad.border_color, rgba(theme.colors.border));
                assert!(
                    !frame.quads.iter().any(|q| q.rect == expected_rect),
                    "an unclipped duplicate would bleed above the viewport"
                );
            }
        }
    }
}
