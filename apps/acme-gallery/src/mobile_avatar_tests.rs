//! Automated geometry/paint evidence, not human/physical-DPI acceptance.
use super::*;
use acme_widgets::style::{Styled, TextAlign};

fn context(theme: &Theme) -> WidgetLayoutContext {
    WidgetLayoutContext {
        body_font_size: theme.typography.body,
        body_line_height: theme.typography.body * theme.typography.line_height,
        label_font_size: theme.typography.label,
        control_height: 36.0,
        scale_factor: 1.0,
    }
}

#[test]
fn mobile_page_has_all_avatar_sizes_placeholder_and_complete_binding() {
    let mut gallery = Gallery::new();
    let widget = gallery.description();
    let theme = gallery.current_theme();
    let layout = retained_layout_root(&widget, &mut gallery.retained, &context(&theme));
    let binding = bind_retained(std::slice::from_ref(&widget), &gallery.retained.tree);
    assert_eq!(binding.mismatches(), 0);
    fn find<'a>(
        widget: &'a WidgetNode<GalleryMessage>,
        key: &str,
    ) -> Option<&'a WidgetNode<GalleryMessage>> {
        if widget.key().is_some_and(|k| k.as_str() == key) {
            Some(widget)
        } else {
            widget.children().iter().find_map(|c| find(c, key))
        }
    }
    for (key, d, text) in [
        ("gallery_avatar_sm", 32.0, "AB"),
        ("gallery_avatar_md", 40.0, "陳"),
        ("gallery_avatar_lg", 56.0, "UI"),
        ("gallery_avatar_placeholder", 40.0, "?"),
    ] {
        let avatar = find(&widget, key).unwrap();
        let WidgetNode::Stack(stack) = avatar else {
            panic!("avatar")
        };
        assert_eq!(stack.width, Some(d));
        let WidgetNode::Label(label) = &avatar.children()[0] else {
            panic!("initials")
        };
        assert_eq!(label.text, text);
        let l = avatar.to_layout_with_context(NodeId::new(1), &context(&gallery.current_theme()));
        let snapshot = gallery
            .layout
            .compute_with_text(&l, (240.0, 100.0), &mut gallery.fonts, 1.0)
            .unwrap();
        let mut hits = Vec::new();
        collect_hit_regions(avatar, &l, &snapshot, false, &mut hits);
        assert!(hits.is_empty());
    }
    assert!(!layout.children.is_empty());
}

#[test]
fn avatars_paint_themed_circles_before_centered_clipped_initials() {
    let mut gallery = Gallery::new();
    for pack in ["default", "apple"] {
        gallery.theme_pack = pack.into();
        for dark in [false, true] {
            gallery.dark = dark;
            let theme = gallery.current_theme();
            for size in [
                MobileAvatarSize::Sm,
                MobileAvatarSize::Md,
                MobileAvatarSize::Lg,
            ] {
                for scale in [1.0, 1.25, 2.0] {
                    let widget = WidgetNode::from(mobile_avatar("AB").size(size));
                    let layout = widget.to_layout_with_context(NodeId::new(1), &context(&theme));
                    let snapshot = gallery
                        .layout
                        .compute_with_text(&layout, (240.0, 100.0), &mut gallery.fonts, scale)
                        .unwrap();
                    let r = snapshot.get(layout.id).unwrap();
                    let clip = [0.0, 0.0, 240.0, 100.0];
                    let mut frame = Frame::default();
                    render_content(
                        &mut frame,
                        &widget,
                        &layout,
                        &snapshot,
                        &theme,
                        scale,
                        0.0,
                        clip,
                        &mut 0,
                        None,
                        None,
                        &mut gallery.fonts,
                        &mut gallery.atlas,
                    );
                    assert_eq!(
                        frame.clipped_quads.len(),
                        1,
                        "avatar circle must be painted"
                    );
                    let q = &frame.clipped_quads[0];
                    assert_eq!(q.quad.radius, size.diameter() / 2.0);
                    assert_eq!(q.quad.color, rgba(theme.colors.muted));
                    assert_eq!(q.clip, clip);
                    let text = &frame.text[0];
                    assert_eq!(text.color, rgba(theme.colors.foreground));
                    let text_clip = text.clip.unwrap();
                    assert!(text_clip[2] < r.width && text_clip[3] < r.height);
                    let mut low = [f32::INFINITY; 2];
                    let mut high = [f32::NEG_INFINITY; 2];
                    for g in &text.prepared.glyphs {
                        let x = text.origin[0] + g.x as f32 / scale;
                        let y = text.origin[1] + g.y as f32 / scale;
                        low[0] = low[0].min(x);
                        low[1] = low[1].min(y);
                        high[0] = high[0].max(x + g.width as f32 / scale);
                        high[1] = high[1].max(y + g.height as f32 / scale);
                    }
                    assert!((low[0] + high[0] - 2.0 * r.x - r.width).abs() < r.width * 0.25);
                    assert!((low[1] + high[1] - 2.0 * r.y - r.height).abs() < r.height * 0.35);
                    let scene = scene_from_frame(&frame);
                    let circle = scene
                        .commands()
                        .iter()
                        .position(|c| matches!(c, acme_core::DrawCommand::Quad(_)))
                        .unwrap();
                    let ink = scene
                        .commands()
                        .iter()
                        .position(|c| matches!(c, acme_core::DrawCommand::Text(_)))
                        .unwrap();
                    assert!(circle < ink);
                }
            }
        }
    }
}

#[test]
fn explicit_single_line_alignment_uses_existing_style_and_bounds_long_text() {
    let mut gallery = Gallery::new();
    let theme = gallery.current_theme();
    for align in [TextAlign::Center, TextAlign::Right] {
        for text in ["AB", "VeryLongInitialsThatMustBeClipped"] {
            let widget = acme_widgets::label_builder(text)
                .w(Length::px(40.0))
                .h(Length::px(32.0))
                .line_height(32.0)
                .text_align(align)
                .build();
            let layout = widget.to_layout_with_context(NodeId::new(1), &context(&theme));
            let snapshot = gallery
                .layout
                .compute_with_text(&layout, (240.0, 100.0), &mut gallery.fonts, 1.0)
                .unwrap();
            let rect = snapshot.get(layout.id).unwrap();
            let mut frame = Frame::default();
            render_content(
                &mut frame,
                &widget,
                &layout,
                &snapshot,
                &theme,
                1.0,
                5.0,
                [0.0, 0.0, 240.0, 100.0],
                &mut 0,
                None,
                None,
                &mut gallery.fonts,
                &mut gallery.atlas,
            );
            let run = &frame.text[0];
            let clip = run.clip.unwrap();
            assert_eq!(clip[0], rect.x);
            assert_eq!(clip[2], rect.width);
            assert!(clip[1] >= 0.0 && clip[3] <= rect.height);
            if text == "AB" {
                assert!(run.origin[0] > rect.x + 4.0);
            }
        }
    }
}
