//! Automated CPU/layout/event evidence; manual device acceptance is separate.
use super::*;

fn frame(gallery: &mut Gallery) {
    gallery.frame(FrameContext {
        window: WindowId(1),
        logical_width: 1100.0,
        logical_height: 1100.0,
        scale_factor: 1.0,
    });
}

fn activate(gallery: &mut Gallery, message: GalleryMessage) {
    frame(gallery);
    let r = gallery
        .button_info
        .iter()
        .find(|h| h.message == message)
        .expect("action hit")
        .rect;
    gallery.scroll = (r[1] - 100.0).max(0.0).min(gallery.max_scroll);
    frame(gallery);
    let (x, y) = (r[0] + r[2] / 2.0, r[1] - gallery.scroll + r[3] / 2.0);
    gallery.event(PlatformEvent::PointerMoved {
        window: WindowId(1),
        x,
        y,
    });
    for pressed in [true, false] {
        gallery.event(PlatformEvent::PointerButton {
            window: WindowId(1),
            pressed,
            x,
            y,
            button: 0,
            pointer: 0,
            shift: false,
            ctrl: false,
            alt: false,
        });
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
fn banner_action_dismiss_and_restore_use_actual_pointer_dispatch_and_keep_other_tones() {
    let mut gallery = Gallery::new();
    frame(&mut gallery);
    let widget = gallery.description();
    let (layout, _) = gallery.last_frame_layout.as_ref().unwrap();
    let stable = find(&widget, layout, "gallery_banner_success")
        .unwrap()
        .1
        .id;
    activate(&mut gallery, GalleryMessage::MobileBannerAction);
    assert_eq!(gallery.banner_action, "retry requested");
    assert!(gallery.banner_visible);
    activate(&mut gallery, GalleryMessage::MobileBannerDismiss);
    assert_eq!(gallery.banner_action, "dismissed");
    assert!(!gallery.banner_visible);
    frame(&mut gallery);
    assert!(!gallery.button_info.iter().any(|h| matches!(
        h.message,
        GalleryMessage::MobileBannerAction | GalleryMessage::MobileBannerDismiss
    )));
    let widget = gallery.description();
    let (layout, _) = gallery.last_frame_layout.as_ref().unwrap();
    assert_eq!(
        find(&widget, layout, "gallery_banner_success")
            .unwrap()
            .1
            .id,
        stable
    );
    assert!(find(&widget, layout, "gallery_banner_info").is_none());
    activate(&mut gallery, GalleryMessage::MobileBannerRestore);
    assert!(gallery.banner_visible);
    assert_eq!(gallery.banner_action, "restored");
    frame(&mut gallery);
    let widget = gallery.description();
    let binding = bind_retained(std::slice::from_ref(&widget), &gallery.retained.tree);
    assert_eq!(binding.mismatches(), 0);
}

#[test]
fn all_banner_tones_paint_before_text_with_theme_tokens_and_viewport_clips() {
    let mut gallery = Gallery::new();
    for pack in ["default", "apple"] {
        gallery.theme_pack = pack.into();
        for dark in [false, true] {
            gallery.dark = dark;
            let theme = gallery.current_theme();
            for (tone, color) in [
                (MobileBannerTone::Info, theme.colors.info_soft),
                (MobileBannerTone::Success, theme.colors.success_soft),
                (MobileBannerTone::Warning, theme.colors.warning_soft),
                (MobileBannerTone::Danger, theme.colors.danger_soft),
            ] {
                for scale in [1.0, 1.25, 2.0] {
                    let widget = WidgetNode::from(
                        mobile_banner("Alert · 警示")
                            .tone(tone)
                            .action("Retry", GalleryMessage::MobileBannerAction)
                            .on_dismiss(GalleryMessage::MobileBannerDismiss),
                    );
                    let context = WidgetLayoutContext {
                        body_font_size: theme.typography.body,
                        body_line_height: theme.typography.body * theme.typography.line_height,
                        label_font_size: theme.typography.label,
                        control_height: 36.0,
                        scale_factor: scale,
                    };
                    let layout = widget.to_layout_with_context(NodeId::new(1), &context);
                    let snapshot = gallery
                        .layout
                        .compute_with_text(&layout, (400.0, 600.0), &mut gallery.fonts, scale)
                        .unwrap();
                    let mut frame = Frame::default();
                    let clip = [10.0, 10.0, 380.0, 500.0];
                    render_content(
                        &mut frame,
                        &widget,
                        &layout,
                        &snapshot,
                        &theme,
                        scale,
                        5.0,
                        clip,
                        &mut 0,
                        None,
                        None,
                        &mut gallery.fonts,
                        &mut gallery.atlas,
                    );
                    assert_eq!(frame.clipped_quads.len(), 3);
                    let surface = &frame.clipped_quads[0];
                    assert_eq!(surface.quad.color, rgba(color));
                    assert_eq!(
                        surface.quad.rect,
                        [0.0, -5.0, 400.0, snapshot.get(layout.id).unwrap().height]
                    );
                    assert!(frame.clipped_quads.iter().all(|q| q.clip == clip));
                    assert_eq!(frame.text.len(), 4);
                    assert!(
                        frame
                            .text
                            .iter()
                            .all(|t| t.clip == Some(clip) && !t.prepared.glyphs.is_empty())
                    );
                    assert_eq!(frame.text[1].color, rgba(theme.colors.foreground));
                    let scene = scene_from_frame(&frame);
                    let surface = scene
                        .commands()
                        .iter()
                        .position(|c| matches!(c, acme_core::DrawCommand::Quad(_)))
                        .unwrap();
                    let ink = scene
                        .commands()
                        .iter()
                        .position(|c| matches!(c, acme_core::DrawCommand::Text(_)))
                        .unwrap();
                    assert!(surface < ink);
                }
            }
        }
    }
}
