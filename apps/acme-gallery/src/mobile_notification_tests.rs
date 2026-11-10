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
fn notification_actions_dismiss_show_and_expiry_clear_hits_without_replacing_other_cards() {
    let mut gallery = Gallery::new();
    frame(&mut gallery);
    let widget = gallery.description();
    let (layout, _) = gallery.last_frame_layout.as_ref().unwrap();
    let stable = find(&widget, layout, "gallery_notification_title")
        .unwrap()
        .1
        .id;
    activate(&mut gallery, GalleryMessage::MobileNotificationAction);
    assert_eq!(gallery.notification.status, "open requested");
    assert!(gallery.notification.visible);
    activate(&mut gallery, GalleryMessage::MobileNotificationDismiss);
    assert!(!gallery.notification.visible);
    assert_eq!(gallery.background_poll_interval(), None);
    frame(&mut gallery);
    assert!(!gallery.button_info.iter().any(|h| matches!(
        h.message,
        GalleryMessage::MobileNotificationAction | GalleryMessage::MobileNotificationDismiss
    )));
    activate(&mut gallery, GalleryMessage::MobileNotificationShow);
    assert!(gallery.notification.visible);
    assert!(gallery.background_poll_interval().is_some());
    gallery.notification.deadline =
        Some(std::time::Instant::now() - std::time::Duration::from_millis(1));
    assert!(gallery.background_tick());
    assert!(!gallery.notification.visible);
    assert!(!gallery.background_tick());
    assert_eq!(gallery.background_poll_interval(), None);
    assert_eq!(gallery.notification.status, "automatically dismissed");
    frame(&mut gallery);
    assert!(!gallery.button_info.iter().any(|h| matches!(
        h.message,
        GalleryMessage::MobileNotificationAction | GalleryMessage::MobileNotificationDismiss
    )));
    let widget = gallery.description();
    let (layout, _) = gallery.last_frame_layout.as_ref().unwrap();
    assert_eq!(
        find(&widget, layout, "gallery_notification_title")
            .unwrap()
            .1
            .id,
        stable
    );
    assert!(find(&widget, layout, "gallery_notification_long").is_some());
    assert_eq!(
        bind_retained(std::slice::from_ref(&widget), &gallery.retained.tree).mismatches(),
        0
    );
}
#[test]
fn host_deadline_boundaries_restart_and_manual_cancel_are_deterministic() {
    let now = std::time::Instant::now();
    let second = std::time::Duration::from_secs(1);
    let mut demo = notification_examples::NotificationDemo::new();
    assert!(!demo.expire(now + second * 100));
    assert!(demo.visible);
    demo.show_timed(now);
    assert!(!demo.expire(now + second * 4));
    demo.show_timed(now + second * 4); // An earlier deadline must not close the new notification.
    assert!(!demo.expire(now + second * 5));
    assert!(demo.expire(now + second * 9));
    assert!(!demo.expire(now + second * 10));
    assert!(demo.poll_interval().is_none());
    demo.show_timed(now + second * 10);
    demo.dismiss();
    assert!(!demo.expire(now + second * 15));
    assert_eq!(demo.status, "dismissed");
    assert!(!demo.visible);
}

#[test]
fn asynchronous_expiry_cancels_pointer_press_and_hover_before_rebuilding_hits() {
    let mut gallery = Gallery::new();
    activate(&mut gallery, GalleryMessage::MobileNotificationShow);
    frame(&mut gallery);
    let index = gallery
        .button_info
        .iter()
        .position(|h| h.message == GalleryMessage::MobileNotificationDismiss)
        .unwrap();
    let r = gallery.button_info[index].rect;
    let (x, y) = (r[0] + r[2] / 2.0, r[1] - gallery.scroll + r[3] / 2.0);
    gallery.event(PlatformEvent::PointerMoved {
        window: WindowId(1),
        x,
        y,
    });
    gallery.event(PlatformEvent::PointerButton {
        window: WindowId(1),
        pressed: true,
        x,
        y,
        button: 0,
        pointer: 0,
        shift: false,
        ctrl: false,
        alt: false,
    });
    assert_eq!(gallery.pressed, Some(index));
    gallery.notification.deadline =
        Some(std::time::Instant::now() - std::time::Duration::from_millis(1));
    assert!(gallery.background_tick());
    assert!(
        gallery.pressed.is_none(),
        "removed notification must not leave a pressed index for another action"
    );
    assert!(gallery.hovered.is_none());
    frame(&mut gallery);
    gallery.event(PlatformEvent::PointerButton {
        window: WindowId(1),
        pressed: false,
        x,
        y,
        button: 0,
        pointer: 0,
        shift: false,
        ctrl: false,
        alt: false,
    });
    assert_eq!(gallery.notification.status, "automatically dismissed");
}
#[test]
fn notification_surface_text_and_actions_keep_themed_paint_order_and_scroll_clips() {
    let mut gallery = Gallery::new();
    for pack in ["default", "apple"] {
        gallery.theme_pack = pack.into();
        for dark in [false, true] {
            gallery.dark = dark;
            let theme = gallery.current_theme();
            for scale in [1.0, 1.25, 2.0] {
                let widget = WidgetNode::from(mobile_notification("Notice · 通知").body("Long notification message that wraps. 這是一則會換行的通知訊息。請先儲存工作。").action("Open", GalleryMessage::MobileNotificationAction).on_dismiss(GalleryMessage::MobileNotificationDismiss));
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
                    .compute_with_text(&layout, (240.0, 1000.0), &mut gallery.fonts, scale)
                    .unwrap();
                let mut frame = Frame::default();
                let clip = [10.0, 10.0, 220.0, 500.0];
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
                assert_eq!(
                    frame.clipped_quads[0].quad.color,
                    rgba(theme.colors.surface)
                );
                assert!(frame.clipped_quads.iter().all(|q| q.clip == clip));
                assert_eq!(frame.text.len(), 4);
                assert!(
                    frame
                        .text
                        .iter()
                        .all(|t| t.clip == Some(clip) && !t.prepared.glyphs.is_empty())
                );
                assert_eq!(frame.text[0].color, rgba(theme.colors.foreground));
                assert_eq!(frame.text[1].color, rgba(theme.colors.foreground));
                let title = snapshot.get(layout.children[0].children[0].id).unwrap();
                let body = snapshot.get(layout.children[0].children[1].id).unwrap();
                assert!(body.y >= title.y + title.height);
                assert!(body.height > 20.0);
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
