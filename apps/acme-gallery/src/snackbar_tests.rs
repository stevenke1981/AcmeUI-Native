use super::*;
use std::time::{Duration, Instant};
fn frame(g: &mut Gallery, width: f32, height: f32, scale: f32) -> acme_core::Scene {
    g.frame(FrameContext {
        window: WindowId(1),
        logical_width: width,
        logical_height: height,
        scale_factor: scale,
    })
}
fn pointer(g: &mut Gallery, x: f32, y: f32, pressed: bool) {
    g.event(PlatformEvent::PointerMoved {
        window: WindowId(1),
        x,
        y,
    });
    g.event(PlatformEvent::PointerButton {
        window: WindowId(1),
        x,
        y,
        pressed,
        button: 0,
        pointer: 0,
        shift: false,
        ctrl: false,
        alt: false,
    });
}
fn click(g: &mut Gallery, message: GalleryMessage) {
    let r = g
        .button_info
        .iter()
        .find(|h| h.message == message)
        .unwrap()
        .rect;
    let x = r[0] + r[2] / 2.0;
    let y = r[1] + r[3] / 2.0;
    pointer(g, x, y, true);
    pointer(g, x, y, false);
}
#[test]
fn snackbar_deadlines_restart_cancel_and_expire_once() {
    let mut demo = SnackbarDemo::new();
    let now = Instant::now();
    let second = Duration::from_secs(1);
    assert!(!demo.expire(now) && demo.poll_interval().is_none());
    demo.show(now, true);
    assert!(!demo.expire(now + second * 4));
    demo.show(now + second * 4, false);
    assert!(!demo.with_action && !demo.expire(now + second * 5));
    assert!(demo.expire(now + second * 9));
    assert!(!demo.expire(now + second * 10) && demo.poll_interval().is_none());
    demo.show(now, true);
    demo.close("dismissed");
    assert!(!demo.expire(now + second * 6));
}
#[test]
fn snackbar_pointer_action_plain_dismiss_and_async_expiry_use_real_host() {
    let mut g = Gallery::new();
    frame(&mut g, 1100.0, 1100.0, 1.0);
    click(&mut g, GalleryMessage::SnackbarShow(true));
    frame(&mut g, 1100.0, 1100.0, 1.0);
    click(&mut g, GalleryMessage::SnackbarAction);
    assert!(!g.snackbar.visible && g.snackbar.status == "undone");
    frame(&mut g, 1100.0, 1100.0, 1.0);
    click(&mut g, GalleryMessage::SnackbarShow(false));
    frame(&mut g, 1100.0, 1100.0, 1.0);
    assert!(
        g.button_info
            .iter()
            .all(|h| h.message != GalleryMessage::SnackbarAction)
    );
    click(&mut g, GalleryMessage::SnackbarDismiss);
    assert_eq!(g.snackbar.status, "dismissed");
    frame(&mut g, 1100.0, 1100.0, 1.0);
    click(&mut g, GalleryMessage::SnackbarShow(true));
    frame(&mut g, 1100.0, 1100.0, 1.0);
    let r = g
        .button_info
        .iter()
        .find(|h| h.message == GalleryMessage::SnackbarAction)
        .unwrap()
        .rect;
    pointer(&mut g, r[0] + 2.0, r[1] + 2.0, true);
    assert!(g.pressed.is_some());
    g.snackbar.deadline = Some(Instant::now() - Duration::from_millis(1));
    assert!(g.background_tick());
    assert!(g.pressed.is_none() && g.hovered.is_none());
    frame(&mut g, 1100.0, 1100.0, 1.0);
    pointer(&mut g, r[0] + 2.0, r[1] + 2.0, false);
    assert_eq!(g.snackbar.status, "automatically dismissed");
    assert!(
        g.button_info
            .iter()
            .all(|h| !snackbar::is_overlay_action(h.message))
    );
}
#[test]
fn snackbar_is_bottom_fixed_clipped_above_page_and_blocks_click_through() {
    let mut g = Gallery::new();
    g.snackbar.show(Instant::now(), true);
    for pack in ["default", "apple"] {
        g.theme_pack = pack.into();
        for dark in [false, true] {
            g.dark = dark;
            for (width, height) in [(500.0, 500.0), (1100.0, 1100.0)] {
                for scale in [1.0, 1.25, 2.0] {
                    g.scroll = 300.0;
                    let scene = frame(&mut g, width, height, scale);
                    scene.validate().unwrap();
                    let r = g.snackbar.bounds.unwrap();
                    assert_eq!(r[1] + r[3], height - 16.0);
                    assert!(r[0] >= SIDEBAR_WIDTH && r[0] + r[2] <= width);
                    let action = g
                        .button_info
                        .iter()
                        .find(|h| h.message == GalleryMessage::SnackbarAction)
                        .unwrap();
                    assert!(!action.scrolled && action.rect[1] >= r[1]);
                    let surface_index = scene.commands().iter().rposition(|command| {
                        matches!(command, acme_core::DrawCommand::Quad(q) if q.rect.origin.x.get() == r[0] && q.rect.origin.y.get() == r[1] && q.rect.size.width.get() == r[2])
                    }).unwrap();
                    let theme = g.current_theme();
                    let color =
                        |c: ThemeColor| acme_core::Color::rgba(c.red, c.green, c.blue, c.alpha);
                    let acme_core::DrawCommand::Quad(surface) = &scene.commands()[surface_index]
                    else {
                        unreachable!()
                    };
                    assert_eq!(surface.color, color(theme.colors.background));
                    let mut text_count = 0;
                    for command in &scene.commands()[surface_index..] {
                        if let acme_core::DrawCommand::Text(text) = command {
                            assert_eq!(text.color, color(theme.colors.foreground));
                            assert!(!text.glyphs.is_empty() && text.origin.y.get() >= r[1]);
                            text_count += 1;
                        }
                    }
                    assert_eq!(text_count, 4, "only overlay content follows its surface");
                    // The overlay's text follows all page content and retains clips.
                    let acme_core::DrawCommand::PopClip = scene.commands().last().unwrap() else {
                        panic!("overlay clip")
                    };
                    g.cursor = (r[0] + r[2] - 2.0, r[1] + 2.0);
                    g.button_info.insert(
                        0,
                        HitRegion {
                            rect: r,
                            message: GalleryMessage::FabPress(0),
                            scrolled: false,
                        },
                    );
                    assert_eq!(g.hit(), None, "opaque snackbar must block underlying hits");
                }
            }
        }
    }
}
