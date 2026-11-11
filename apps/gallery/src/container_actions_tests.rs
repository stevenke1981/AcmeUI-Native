use super::*;
use acme_core::{DrawCommand, Rect};

fn frame(g: &mut Gallery) -> Scene {
    g.frame(FrameContext {
        window: WindowId(0),
        logical_width: 1100.0,
        logical_height: 760.0,
        scale_factor: 1.0,
    })
}
fn key(g: &mut Gallery, key: PlatformKey, shift: bool) -> bool {
    g.event(PlatformEvent::Key {
        window: WindowId(0),
        key,
        pressed: true,
        shift,
        ctrl: false,
        alt: false,
        meta: false,
        repeat: false,
        text: None,
        character: None,
    })
}

#[test]
fn container_focus_ring_and_following_button_use_the_actual_hit_indices() {
    let mut g = Gallery::new();
    g.selected_category = 6;
    g.selected_page = 0;
    g.show_focus_rings = true;
    frame(&mut g);
    let target = *g
        .button_info
        .iter()
        .find(|h| h.scrolled && h.message == GalleryMessage::ToggleDensity)
        .unwrap();
    for _ in 0..g.button_info.len() {
        if g.focused_target.is_some_and(|f| f.node == target.node) {
            break;
        }
        key(&mut g, PlatformKey::Tab, false);
        frame(&mut g);
    }
    assert_eq!(g.focused_target.unwrap().node, target.node);
    let scene = frame(&mut g);
    let theme = build_theme(g.dark);
    let ring = scene
        .commands()
        .iter()
        .find_map(|cmd| match cmd {
            DrawCommand::Quad(q)
                if q.rect
                    == Rect::new(
                        target.rect[0],
                        target.rect[1] - g.scroll,
                        target.rect[2],
                        target.rect[3],
                    )
                    && q.border_width == 2.0 =>
            {
                Some(q)
            }
            _ => None,
        })
        .expect("focused container ring must use its own hit index");
    assert_eq!(
        ring.border_color,
        Color::rgba(
            theme.colors.ring.red,
            theme.colors.ring.green,
            theme.colors.ring.blue,
            theme.colors.ring.alpha
        )
    );
    let previous = g.focused_target;
    let before = g.density;
    assert!(key(&mut g, PlatformKey::Enter, false));
    assert_ne!(g.density, before);
    frame(&mut g);
    assert_eq!(g.focused_target, previous);
    assert!(g.retained.tree().get(target.node).unwrap().focusable);
    assert_eq!(
        g.accessibility.build_tree_update().unwrap().focus,
        acme_accessibility::to_access_id(target.node)
    );
    key(&mut g, PlatformKey::Tab, false);
    frame(&mut g);
    let child = g.button_info[g.focused];
    assert_eq!(child.message, GalleryMessage::ToggleFocusRings);
    assert_ne!(child.node, target.node);
    key(&mut g, PlatformKey::Tab, true);
    frame(&mut g);
    assert_eq!(g.focused_target, previous);
}

#[test]
fn focus_demo_keyboard_resolves_current_typed_action_instead_of_cached_hit_message() {
    let mut g = Gallery::new();
    g.selected_category = 6;
    frame(&mut g);
    let index = g
        .button_info
        .iter()
        .position(|h| h.scrolled && h.message == GalleryMessage::ToggleDensity)
        .unwrap();
    for _ in 0..index {
        key(&mut g, PlatformKey::Tab, false);
        frame(&mut g);
    }
    let before = g.density;
    // An observable poison message proves the keyboard path uses the current
    // retained description, not the old coordinate/hit-message path.
    g.button_info[index].message = GalleryMessage::ToggleTheme;
    let dark = g.dark;
    assert!(key(&mut g, PlatformKey::Space, false));
    assert_ne!(g.density, before);
    assert_eq!(g.dark, dark);
    g.focused_target = Some(FocusTarget {
        node: NodeId::new(u64::MAX),
        ..g.focused_target.unwrap()
    });
    assert!(!key(&mut g, PlatformKey::Enter, false));
    assert_eq!(g.dark, dark);
}

#[test]
fn all_container_and_child_rings_follow_theme_scale_and_scroll_clip() {
    for dark in [false, true] {
        for scale in [1.0, 1.25, 2.0] {
            let mut g = Gallery::new();
            g.selected_category = 6;
            g.dark = dark;
            g.show_focus_rings = true;
            let context = FrameContext {
                window: WindowId(0),
                logical_width: 1100.0,
                logical_height: 760.0,
                scale_factor: scale,
            };
            g.frame(context.clone());
            let targets: Vec<_> = g
                .button_info
                .iter()
                .copied()
                .filter(|h| h.scrolled)
                .collect();
            assert_eq!(
                targets.len(),
                5,
                "three active containers, child and following Button; passive row is skipped"
            );
            for target in targets {
                for _ in 0..g.button_info.len() {
                    if g.focused_target.is_some_and(|f| f.node == target.node) {
                        break;
                    }
                    key(&mut g, PlatformKey::Tab, false);
                    g.frame(context.clone());
                }
                g.scroll = 60.0;
                let scene = g.frame(context.clone());
                let rect = Rect::new(
                    target.rect[0],
                    target.rect[1] - g.scroll,
                    target.rect[2],
                    target.rect[3],
                );
                let clip = g.scroll_clip_rect;
                let viewport = Rect::new(clip[0], clip[1], clip[2], clip[3]);
                let mut clips = Vec::new();
                let mut found = false;
                for command in scene.commands() {
                    match command {
                        DrawCommand::PushClip(r) => clips.push(*r),
                        DrawCommand::PopClip => {
                            clips.pop().expect("balanced clip");
                        }
                        DrawCommand::Quad(q) if q.rect == rect && q.border_width == 2.0 => {
                            assert!(
                                clips.contains(&viewport),
                                "focused content must be clipped to page viewport"
                            );
                            let ring = build_theme(dark).colors.ring;
                            assert_eq!(
                                q.border_color,
                                Color::rgba(ring.red, ring.green, ring.blue, ring.alpha)
                            );
                            found = true;
                        }
                        _ => {}
                    }
                }
                assert!(found, "missing ring for {:?} at {scale}", target.message);
                assert!(clips.is_empty());
                // Return to the target's original window geometry for the next pass.
                g.scroll = 0.0;
                g.frame(context.clone());
            }
        }
    }
}

#[test]
fn nested_button_pointer_action_does_not_activate_parent_container() {
    let mut g = Gallery::new();
    g.selected_category = 6;
    frame(&mut g);
    let child = *g
        .button_info
        .iter()
        .find(|h| h.scrolled && h.message == GalleryMessage::ToggleFocusRings)
        .unwrap();
    let (x, y) = (
        child.rect[0] + child.rect[2] / 2.0,
        child.rect[1] + child.rect[3] / 2.0,
    );
    let index = hit_test(&g.button_info, (x, y), g.scroll, g.scroll_clip_rect).unwrap();
    assert_eq!(g.button_info[index].node, child.node);
    let density = g.density;
    let rings = g.show_focus_rings;
    g.event(PlatformEvent::PointerMoved {
        window: WindowId(0),
        x,
        y,
    });
    for pressed in [true, false] {
        assert!(g.event(PlatformEvent::PointerButton {
            window: WindowId(0),
            pressed,
            x,
            y,
            button: 0,
            pointer: 0,
            shift: false,
            ctrl: false,
            alt: false
        }));
        frame(&mut g);
    }
    assert_eq!(g.density, density);
    assert_ne!(g.show_focus_rings, rings);
}
