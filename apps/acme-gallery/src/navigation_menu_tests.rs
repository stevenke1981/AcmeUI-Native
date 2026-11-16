use super::*;
use acme_platform::PlatformKey;

fn frame(g: &mut Gallery) {
    g.frame(FrameContext {
        window: WindowId(1),
        logical_width: 1100.0,
        logical_height: 760.0,
        scale_factor: 1.0,
    });
}

fn click(g: &mut Gallery, message: GalleryMessage) {
    frame(g);
    let hit = g.button_info.iter().find(|h| h.message == message).unwrap();
    g.scroll = (hit.rect[1] - 160.0).clamp(0.0, g.max_scroll);
    frame(g);
    let hit = g.button_info.iter().find(|h| h.message == message).unwrap();
    let (x, y) = (
        hit.rect[0] + hit.rect[2] / 2.0,
        hit.rect[1] + hit.rect[3] / 2.0 - g.scroll,
    );
    g.event(PlatformEvent::PointerMoved {
        window: WindowId(1),
        x,
        y,
    });
    for pressed in [true, false] {
        assert!(
            g.event(PlatformEvent::PointerButton {
                window: WindowId(1),
                pressed,
                x,
                y,
                button: 0,
                pointer: 0,
                shift: false,
                ctrl: false,
                alt: false
            }),
            "{message:?}, pressed={pressed}, at ({x}, {y})"
        );
    }
    frame(g);
}

fn key(g: &mut Gallery, key: PlatformKey) -> bool {
    g.event(PlatformEvent::Key {
        window: WindowId(1),
        key,
        pressed: true,
        shift: false,
        ctrl: false,
        alt: false,
        meta: false,
        repeat: false,
        text: None,
        character: None,
    })
}

fn layout() -> Gallery {
    let mut g = Gallery::new();
    g.selected_category = 3;
    frame(&mut g);
    g
}

fn nav_msg(i: u8) -> GalleryMessage {
    GalleryMessage::NavMenuSelect(i)
}

fn press_at(g: &mut Gallery, x: f32, y: f32) {
    g.event(PlatformEvent::PointerMoved {
        window: WindowId(1),
        x,
        y,
    });
    g.event(PlatformEvent::PointerButton {
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
}

#[test]
fn gallery_navigation_menu_pointer_activates_items_and_children_with_real_events() {
    let mut g = layout();
    assert_eq!(g.nav_menu.item, "home");
    assert_eq!(g.nav_menu.destination, None);
    click(&mut g, nav_msg(1));
    assert_eq!(
        (g.nav_menu.item, g.nav_menu.destination),
        ("products", Some(1))
    );
    assert!(g.nav_menu.active);
    // The dropdown is open now, so its enabled children have hit regions.
    click(&mut g, nav_msg(3));
    assert_eq!(g.nav_menu.item, "products");
    assert_eq!(g.nav_menu.child, Some("mobile"));
    assert_eq!(g.nav_menu.destination, Some(3));
    click(&mut g, nav_msg(2));
    assert_eq!(g.nav_menu.child, Some("desktop"));
    click(&mut g, nav_msg(5));
    assert_eq!((g.nav_menu.item, g.nav_menu.child), ("docs", None));
    assert_eq!(g.nav_menu.destination, Some(5));
    click(&mut g, nav_msg(0));
    assert_eq!(g.nav_menu.destination, Some(0));
}

#[test]
fn gallery_navigation_menu_disabled_items_have_no_hit_region() {
    let mut g = layout();
    click(&mut g, nav_msg(1));
    let hits: Vec<_> = g
        .button_info
        .iter()
        .filter(|h| matches!(h.message, GalleryMessage::NavMenuSelect(_)))
        .map(|h| h.message)
        .collect();
    assert_eq!(
        hits,
        [nav_msg(0), nav_msg(1), nav_msg(2), nav_msg(3), nav_msg(5)]
    );
    let before = (g.nav_menu.item, g.nav_menu.child, g.nav_menu.destination);
    // Disabled (enterprise, support) and unknown/out-of-range values never apply.
    for index in [4u8, 6, 7, u8::MAX] {
        assert!(!g.nav_menu_select(index), "index {index}");
    }
    assert_eq!(
        before,
        (g.nav_menu.item, g.nav_menu.child, g.nav_menu.destination)
    );
}

#[test]
fn gallery_navigation_menu_f6_reveals_offscreen_menu_and_arrows_rove_skipping_disabled() {
    let mut g = layout();
    assert_eq!(g.scroll, 0.0);
    assert!(key(&mut g, PlatformKey::Function(6)));
    assert!(g.nav_menu.active);
    frame(&mut g);
    let hit = g
        .button_info
        .iter()
        .find(|h| h.message == nav_msg(0))
        .unwrap();
    assert!(g.scroll > 0.0, "F6 scrolls the offscreen menu into view");
    let top = hit.rect[1] - g.scroll;
    assert!(
        top >= 0.0 && top + hit.rect[3] <= 760.0,
        "menu visible at {top}"
    );
    assert_eq!(g.nav_menu.item, "home");
    for expected in ["products", "docs", "home"] {
        assert!(key(&mut g, PlatformKey::ArrowRight));
        assert_eq!(g.nav_menu.item, expected, "support is disabled and skipped");
    }
    for expected in ["docs", "products", "home"] {
        assert!(key(&mut g, PlatformKey::ArrowLeft));
        assert_eq!(g.nav_menu.item, expected);
    }
    assert_eq!(
        g.nav_menu.destination, None,
        "arrows move without confirming"
    );
    assert_eq!((g.hovered, g.pressed), (None, None));
}

#[test]
fn gallery_navigation_menu_down_enter_activates_child_and_escape_steps_out() {
    let mut g = layout();
    key(&mut g, PlatformKey::Function(6));
    // An item without children has nothing to descend into.
    assert!(!key(&mut g, PlatformKey::ArrowDown));
    key(&mut g, PlatformKey::ArrowRight);
    assert_eq!(g.nav_menu.item, "products");
    for expected in ["desktop", "mobile", "desktop"] {
        assert!(key(&mut g, PlatformKey::ArrowDown));
        assert_eq!(g.nav_menu.child, Some(expected), "enterprise is skipped");
    }
    assert!(key(&mut g, PlatformKey::ArrowUp));
    assert_eq!(g.nav_menu.child, Some("mobile"));
    assert_eq!(g.nav_menu.destination, None);
    assert!(key(&mut g, PlatformKey::Enter));
    assert_eq!(g.nav_menu.destination, Some(3));
    assert_eq!(g.nav_menu.child, Some("mobile"));
    // Escape clears child focus first, then leaves.
    assert!(key(&mut g, PlatformKey::Escape));
    assert_eq!(g.nav_menu.child, None);
    assert!(g.nav_menu.active);
    assert!(key(&mut g, PlatformKey::Space));
    assert_eq!(g.nav_menu.destination, Some(1), "Space confirms the item");
    assert!(key(&mut g, PlatformKey::Escape));
    assert!(!g.nav_menu.active);
    assert!(!key(&mut g, PlatformKey::ArrowDown));
}

#[test]
fn gallery_navigation_menu_pointer_press_off_menu_deactivates_group() {
    let mut g = layout();
    key(&mut g, PlatformKey::Function(6));
    assert!(g.nav_menu.active);
    frame(&mut g);
    press_at(&mut g, 1.0, 1.0);
    assert!(!g.nav_menu.active);
    assert!(!key(&mut g, PlatformKey::ArrowRight));
    assert_eq!(g.nav_menu.item, "home");
    // A press on another Gallery button also deactivates.
    key(&mut g, PlatformKey::Function(6));
    assert!(g.nav_menu.active);
    frame(&mut g);
    let hit = g
        .button_info
        .iter()
        .find(|h| h.message == GalleryMessage::HeaderToggleDescription)
        .unwrap();
    g.scroll = (hit.rect[1] - 160.0).clamp(0.0, g.max_scroll);
    frame(&mut g);
    let hit = g
        .button_info
        .iter()
        .find(|h| h.message == GalleryMessage::HeaderToggleDescription)
        .unwrap();
    let (x, y) = (
        hit.rect[0] + 4.0,
        hit.rect[1] + hit.rect[3] / 2.0 - g.scroll,
    );
    press_at(&mut g, x, y);
    assert!(!g.nav_menu.active);
}

#[test]
fn gallery_navigation_menu_preedit_blocks_f6_and_arrow_keys() {
    let mut g = layout();
    g.search.input.preedit = "注".into();
    assert!(!key(&mut g, PlatformKey::Function(6)));
    assert!(!g.nav_menu.active);
    g.search.input.preedit.clear();
    key(&mut g, PlatformKey::Function(6));
    assert!(g.nav_menu.active);
    g.search.input.preedit = "注".into();
    for k in [
        PlatformKey::ArrowRight,
        PlatformKey::ArrowLeft,
        PlatformKey::ArrowDown,
        PlatformKey::ArrowUp,
        PlatformKey::Enter,
        PlatformKey::Space,
        PlatformKey::Escape,
    ] {
        assert!(!key(&mut g, k), "{k:?} must not act during composition");
    }
    assert!(g.nav_menu.active);
    assert_eq!(g.nav_menu.item, "home");
    assert_eq!((g.nav_menu.child, g.nav_menu.destination), (None, None));
}

#[test]
fn gallery_navigation_menu_keys_ignored_off_page_and_after_blur_or_category_change() {
    let mut g = Gallery::new();
    g.selected_category = 0;
    frame(&mut g);
    key(&mut g, PlatformKey::Function(6));
    assert!(!g.nav_menu.active, "F6 off the Layout page is ignored");
    assert!(!key(&mut g, PlatformKey::ArrowRight));
    assert_eq!(g.nav_menu.item, "home");

    let mut g = layout();
    assert!(
        !key(&mut g, PlatformKey::ArrowRight),
        "inactive menu ignores keys"
    );
    key(&mut g, PlatformKey::Function(6));
    g.event(PlatformEvent::FocusChanged {
        window: WindowId(1),
        gained: false,
        node_id: 0,
    });
    assert!(!g.nav_menu.active);
    assert!(!key(&mut g, PlatformKey::ArrowRight));
    assert_eq!(g.nav_menu.item, "home");

    key(&mut g, PlatformKey::Function(6));
    assert!(g.nav_menu.active);
    frame(&mut g);
    let category = g
        .button_info
        .iter()
        .position(|h| h.message == GalleryMessage::SelectCategory(0))
        .unwrap();
    assert!(g.activate(category));
    assert!(!g.nav_menu.active);
}

#[test]
fn gallery_navigation_menu_hit_and_paint_slots_match_across_themes_and_scales() {
    let mut g = Gallery::new();
    for dark in [false, true] {
        g.dark = dark;
        let theme = g.current_theme();
        for scale in [1.0, 1.25, 2.0] {
            // (selected item, active child, expected hit messages)
            let cases: [(&'static str, Option<&'static str>, Vec<u8>); 4] = [
                ("home", None, vec![0, 1, 5]),
                ("products", None, vec![0, 1, 2, 3, 5]),
                ("products", Some("mobile"), vec![0, 1, 2, 3, 5]),
                ("docs", None, vec![0, 1, 5]),
            ];
            for (item, child, expected) in cases {
                g.nav_menu.item = item;
                g.nav_menu.child = child;
                let widget = g.nav_menu_examples();
                let context = WidgetLayoutContext {
                    body_font_size: theme.typography.body,
                    body_line_height: theme.typography.body * theme.typography.line_height,
                    label_font_size: theme.typography.label,
                    control_height: 36.0,
                    scale_factor: scale,
                };
                let layout = widget.to_layout_with_context(NodeId::new(1), &context);
                let snapshot = g
                    .layout
                    .compute_with_text(&layout, (600.0, 800.0), &mut g.fonts, scale)
                    .unwrap();
                let mut hits = Vec::new();
                collect_hit_regions(&widget, &layout, &snapshot, true, &mut hits);
                let messages: Vec<_> = hits.iter().map(|h| h.message).collect();
                let expected: Vec<_> = expected.into_iter().map(nav_msg).collect();
                assert_eq!(messages, expected, "{item} {child:?}");
                for hit in &hits {
                    assert!(hit.rect.iter().all(|v| v.is_finite()));
                    assert!(hit.rect[2] > 0.0 && hit.rect[3] > 0.0);
                    assert!(hit.rect[0] >= 0.0 && hit.rect[0] + hit.rect[2] <= 600.01);
                }
                for (hovered, pressed) in [(None, None), (Some(1), None), (None, Some(2))] {
                    let clip = [0.0, 5.0, 600.0, 700.0];
                    let mut paint = Frame::default();
                    let mut index = 0;
                    render_content(
                        &mut paint,
                        &widget,
                        &layout,
                        &snapshot,
                        &theme,
                        scale,
                        3.0,
                        clip,
                        &mut index,
                        hovered,
                        pressed,
                        &mut g.fonts,
                        &mut g.atlas,
                    );
                    assert_eq!(index, hits.len(), "paint slots match hit slots");
                    assert!(paint.clipped_quads.iter().all(|q| q.clip == clip));
                    assert!(paint.text.iter().all(|t| t.clip.is_some()));
                }
            }
        }
    }
}
