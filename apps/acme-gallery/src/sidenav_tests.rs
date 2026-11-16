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

fn chord_event(
    key: PlatformKey,
    (ctrl, alt, meta): (bool, bool, bool),
    repeat: bool,
) -> PlatformEvent {
    PlatformEvent::Key {
        window: WindowId(1),
        key,
        pressed: true,
        shift: false,
        ctrl,
        alt,
        meta,
        repeat,
        text: None,
        character: None,
    }
}

fn key_event(key: PlatformKey) -> PlatformEvent {
    chord_event(key, (false, false, false), false)
}

fn key(g: &mut Gallery, key: PlatformKey) -> bool {
    g.event(key_event(key))
}

fn layout() -> Gallery {
    let mut g = Gallery::new();
    g.selected_category = 3;
    frame(&mut g);
    g
}

fn sn(i: u8) -> GalleryMessage {
    GalleryMessage::SidenavSelect(i)
}

fn center(g: &Gallery, message: GalleryMessage) -> (f32, f32) {
    let hit = g
        .button_info
        .iter()
        .find(|h| h.message == message)
        .unwrap_or_else(|| panic!("no hit region for {message:?}"));
    (
        hit.rect[0] + hit.rect[2] / 2.0,
        hit.rect[1] + hit.rect[3] / 2.0 - g.scroll,
    )
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

fn release_at(g: &mut Gallery, x: f32, y: f32) -> bool {
    g.event(PlatformEvent::PointerButton {
        window: WindowId(1),
        pressed: false,
        x,
        y,
        button: 0,
        pointer: 0,
        shift: false,
        ctrl: false,
        alt: false,
    })
}

/// Scrolls the target toward the top of the viewport and clicks its hit region.
fn click(g: &mut Gallery, message: GalleryMessage) {
    frame(g);
    let y = g
        .button_info
        .iter()
        .find(|h| h.message == message)
        .unwrap()
        .rect[1];
    g.scroll = (y - 160.0).clamp(0.0, g.max_scroll);
    frame(g);
    let (x, y) = center(g, message);
    press_at(g, x, y);
    assert!(release_at(g, x, y), "{message:?} at ({x}, {y})");
    frame(g);
}

fn focus_lost(g: &mut Gallery) {
    g.event(PlatformEvent::FocusChanged {
        window: WindowId(1),
        gained: false,
        node_id: 0,
    });
}

fn sidenav_hits(g: &Gallery) -> Vec<GalleryMessage> {
    g.button_info
        .iter()
        .map(|h| h.message)
        .filter(|m| matches!(m, GalleryMessage::SidenavSelect(_)))
        .collect()
}

fn groups(g: &Gallery) -> (bool, bool, bool) {
    (g.nav_menu.active, g.side_nav.active, g.sidenav.active)
}

#[test]
fn gallery_sidenav_pointer_selects_each_enabled_item_with_real_events() {
    let mut g = layout();
    assert_eq!((g.sidenav.current, g.sidenav.confirmed), (0, None));
    click(&mut g, sn(1));
    assert_eq!((g.sidenav.current, g.sidenav.confirmed), (1, Some(1)));
    assert!(g.sidenav.active);
    click(&mut g, sn(2));
    assert_eq!((g.sidenav.current, g.sidenav.confirmed), (2, Some(2)));
    click(&mut g, sn(0));
    assert_eq!((g.sidenav.current, g.sidenav.confirmed), (0, Some(0)));
    assert_eq!(groups(&g), (false, false, true));
}

#[test]
fn gallery_sidenav_disabled_item_has_no_hit_region() {
    let mut g = layout();
    assert_eq!(sidenav_hits(&g), [sn(0), sn(1), sn(2)]);
    let before = (g.sidenav.current, g.sidenav.confirmed);
    // Disabled (Security), out-of-range and the unknown sentinel never apply.
    for index in [3u8, 4, 9, u8::MAX] {
        assert!(!g.sidenav_select(index), "index {index}");
    }
    assert_eq!(before, (g.sidenav.current, g.sidenav.confirmed));
    assert_eq!(crate::sidenav::sidenav_message("nope"), sn(u8::MAX));
    assert!(!g.sidenav.active);
}

#[test]
fn gallery_sidenav_f8_reveals_only_when_hidden() {
    let mut g = layout();
    assert_eq!(g.scroll, 0.0);
    assert!(key(&mut g, PlatformKey::Function(8)));
    assert!(g.sidenav.active);
    frame(&mut g);
    assert!(g.scroll > 0.0, "F8 scrolls the offscreen group into view");
    for i in [0, 1, 2] {
        assert_eq!(g.hit_is_hidden(sn(i)), Some(false), "item {i} visible");
    }
    // In view but off the anchor: neither a second F8 nor arrows scroll.
    assert!(g.scroll >= 20.0);
    g.scroll -= 20.0;
    frame(&mut g);
    assert_eq!(g.hit_is_hidden(sn(0)), Some(false));
    let scroll = g.scroll;
    key(&mut g, PlatformKey::Function(8));
    frame(&mut g);
    assert_eq!(g.scroll, scroll, "F8 on a visible group");
    for k in [
        PlatformKey::ArrowDown,
        PlatformKey::ArrowDown,
        PlatformKey::ArrowUp,
        PlatformKey::ArrowUp,
    ] {
        assert!(key(&mut g, k));
        frame(&mut g);
        assert_eq!(g.scroll, scroll, "{k:?} on a visible group");
    }
    // Out of view again: a navigation key reveals the focused item.
    g.scroll = 0.0;
    frame(&mut g);
    assert_eq!(g.hit_is_hidden(sn(g.sidenav.current)), Some(true));
    assert!(key(&mut g, PlatformKey::ArrowDown));
    frame(&mut g);
    assert!(g.scroll > 0.0);
    assert_eq!(g.hit_is_hidden(sn(g.sidenav.current)), Some(false));
}

#[test]
fn gallery_sidenav_up_down_cross_sections_wrap_skip_disabled_and_enter_space_confirm() {
    let mut g = layout();
    key(&mut g, PlatformKey::Function(8));
    frame(&mut g);
    // Appearance -> Privacy crosses the section; Security is skipped; wraps.
    for expected in [1, 2, 0, 1] {
        assert!(key(&mut g, PlatformKey::ArrowDown));
        assert_eq!(g.sidenav.current, expected, "Security is disabled");
    }
    for expected in [0, 2, 1] {
        assert!(key(&mut g, PlatformKey::ArrowUp));
        assert_eq!(g.sidenav.current, expected);
    }
    assert_eq!(g.sidenav.confirmed, None, "arrows move without confirming");
    assert_eq!((g.hovered, g.pressed), (None, None));
    assert!(key(&mut g, PlatformKey::Enter));
    assert_eq!(g.sidenav.confirmed, Some(1));
    key(&mut g, PlatformKey::ArrowDown);
    assert!(key(&mut g, PlatformKey::Space));
    assert_eq!(g.sidenav.confirmed, Some(2));
    assert!(g.sidenav.active);
    let text = format!("{:?}", g.sidenav_examples());
    assert!(text.contains("Sidenav selected: Privacy"));
    assert!(text.contains("已確認：Privacy"));
}

#[test]
fn gallery_sidenav_repeat_and_modifier_chords_ignored() {
    for chord in [
        (true, false, false),
        (false, true, false),
        (false, false, true),
    ] {
        let mut g = layout();
        assert!(
            !g.event(chord_event(PlatformKey::Function(8), chord, false)),
            "{chord:?}"
        );
        assert!(!g.sidenav.active, "{chord:?}");
        key(&mut g, PlatformKey::Function(8));
        let scroll = g.scroll;
        for k in [
            PlatformKey::ArrowDown,
            PlatformKey::ArrowUp,
            PlatformKey::Enter,
            PlatformKey::Space,
            PlatformKey::Escape,
            PlatformKey::Function(8),
        ] {
            assert!(
                !g.event(chord_event(k, chord, false)),
                "{k:?} with {chord:?}"
            );
        }
        assert!(g.sidenav.active, "chords never leave the group ({chord:?})");
        assert_eq!((g.sidenav.current, g.sidenav.confirmed), (0, None));
        assert_eq!(g.scroll, scroll);
    }
    // Enter / Space ignore OS auto-repeat; arrow repeat still roves.
    let mut g = layout();
    key(&mut g, PlatformKey::Function(8));
    for k in [PlatformKey::Enter, PlatformKey::Space] {
        assert_eq!(
            g.sidenav_event(&chord_event(k, (false, false, false), true)),
            Some(false),
            "{k:?} repeat"
        );
        assert_eq!(g.sidenav.confirmed, None);
    }
    assert_eq!(
        g.sidenav_event(&chord_event(
            PlatformKey::ArrowDown,
            (false, false, false),
            true
        )),
        Some(true)
    );
    assert_eq!(g.sidenav.current, 1);
    assert!(key(&mut g, PlatformKey::Enter));
    assert_eq!(g.sidenav.confirmed, Some(1));
}

#[test]
fn gallery_sidenav_pointer_press_off_group_deactivates() {
    let mut g = layout();
    key(&mut g, PlatformKey::Function(8));
    assert!(g.sidenav.active);
    frame(&mut g);
    press_at(&mut g, 1.0, 1.0);
    assert!(!g.sidenav.active);
    assert!(!key(&mut g, PlatformKey::ArrowDown));
    assert_eq!(g.sidenav.current, 0);
    // A press on another Gallery button also deactivates.
    key(&mut g, PlatformKey::Function(8));
    assert!(g.sidenav.active);
    frame(&mut g);
    let y = g
        .button_info
        .iter()
        .find(|h| h.message == GalleryMessage::HeaderToggleDescription)
        .unwrap()
        .rect[1];
    g.scroll = (y - 160.0).clamp(0.0, g.max_scroll);
    frame(&mut g);
    let (x, y) = center(&g, GalleryMessage::HeaderToggleDescription);
    press_at(&mut g, x, y);
    assert!(!g.sidenav.active);
    // A press on a sidenav item keeps the group active.
    click(&mut g, sn(1));
    assert!(g.sidenav.active);
    // Snackbar press releases every Layout group.
    g.snackbar.show(std::time::Instant::now(), true);
    frame(&mut g);
    let b = g.snackbar.bounds.expect("snackbar bounds painted");
    press_at(&mut g, b[0] + 4.0, b[1] + 4.0);
    assert_eq!(groups(&g), (false, false, false));
    assert_eq!((g.sidenav.current, g.sidenav.confirmed), (1, Some(1)));
}

#[test]
fn gallery_sidenav_keys_ignored_off_page_after_escape_blur_and_preedit() {
    let mut g = Gallery::new();
    g.selected_category = 0;
    frame(&mut g);
    key(&mut g, PlatformKey::Function(8));
    assert!(!g.sidenav.active, "F8 off the Layout page is ignored");
    assert!(!key(&mut g, PlatformKey::ArrowDown));

    let mut g = layout();
    assert!(
        !key(&mut g, PlatformKey::ArrowDown),
        "inactive ignores keys"
    );
    assert!(!key(&mut g, PlatformKey::Escape));
    assert_eq!(g.sidenav.current, 0);

    key(&mut g, PlatformKey::Function(8));
    frame(&mut g);
    let scroll = g.scroll;
    assert!(key(&mut g, PlatformKey::Escape));
    assert!(!g.sidenav.active);
    assert_eq!(g.scroll, scroll, "the leaving Esc never scrolls");
    assert!(!key(&mut g, PlatformKey::ArrowDown));
    assert_eq!(g.sidenav.current, 0);

    key(&mut g, PlatformKey::Function(8));
    focus_lost(&mut g);
    assert!(!g.sidenav.active);
    assert!(!key(&mut g, PlatformKey::ArrowDown));

    // A category change deactivates the group.
    key(&mut g, PlatformKey::Function(8));
    assert!(g.sidenav.active);
    frame(&mut g);
    // Category buttons live in the fixed side rail, outside the page scroll.
    let rect = g
        .button_info
        .iter()
        .find(|h| h.message == GalleryMessage::SelectCategory(0))
        .unwrap()
        .rect;
    let (x, y) = (rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0);
    press_at(&mut g, x, y);
    assert!(release_at(&mut g, x, y));
    assert_eq!(g.selected_category, 0);
    assert!(!g.sidenav.active);

    // Unhandled keys are never consumed, and never change the group.
    let mut g = layout();
    key(&mut g, PlatformKey::Function(8));
    assert_eq!(g.sidenav_event(&key_event(PlatformKey::Tab)), None);
    assert_eq!(g.sidenav_event(&key_event(PlatformKey::ArrowLeft)), None);
    assert!(g.sidenav.active);

    // Composition: nothing is acted on, handled keys report "unchanged".
    let mut g = layout();
    g.search.input.preedit = "注".into();
    assert!(!key(&mut g, PlatformKey::Function(8)));
    assert!(!g.sidenav.active);
    g.search.input.preedit.clear();
    key(&mut g, PlatformKey::Function(8));
    g.search.input.preedit = "注".into();
    for k in [
        PlatformKey::ArrowDown,
        PlatformKey::ArrowUp,
        PlatformKey::Enter,
        PlatformKey::Space,
        PlatformKey::Escape,
    ] {
        assert!(!key(&mut g, k), "{k:?} must not act during composition");
    }
    assert!(g.sidenav.active);
    assert_eq!((g.sidenav.current, g.sidenav.confirmed), (0, None));
    assert_eq!(g.sidenav_event(&key_event(PlatformKey::Tab)), None);
}

#[test]
fn gallery_layout_f6_f7_f8_groups_are_mutually_exclusive() {
    let mut g = layout();
    for (f, expected) in [
        (6, (true, false, false)),
        (7, (false, true, false)),
        (8, (false, false, true)),
        (6, (true, false, false)),
        (8, (false, false, true)),
        (7, (false, true, false)),
        (6, (true, false, false)),
    ] {
        key(&mut g, PlatformKey::Function(f));
        assert_eq!(groups(&g), expected, "after F{f}");
    }
    // Keys belong to the active group only.
    key(&mut g, PlatformKey::Function(8));
    assert!(key(&mut g, PlatformKey::ArrowDown));
    assert_eq!((g.sidenav.current, g.side_nav.current), (1, 0));
    assert_eq!(g.nav_menu.item, "home");
    // Pointer activation hands the group over.
    click(&mut g, GalleryMessage::SidebarSelect(1));
    assert_eq!(groups(&g), (false, true, false));
    click(&mut g, sn(0));
    assert_eq!(groups(&g), (false, false, true));
    click(&mut g, GalleryMessage::NavMenuSelect(0));
    assert_eq!(groups(&g), (true, false, false));
    // Direct hook calls (bypassing the owner) stay exclusive too.
    assert_eq!(
        g.sidenav_event(&key_event(PlatformKey::Function(8))),
        Some(true)
    );
    assert_eq!(groups(&g), (false, false, true));
    assert_eq!(
        g.sidebar_event(&key_event(PlatformKey::Function(7))),
        Some(true)
    );
    assert_eq!(groups(&g), (false, true, false));
    assert_eq!(
        g.nav_menu_event(&key_event(PlatformKey::Function(6))),
        Some(true)
    );
    assert_eq!(groups(&g), (true, false, false));
    // Release, focus loss and an off-group press clear all three.
    key(&mut g, PlatformKey::Function(8));
    g.layout_keyboard_release();
    assert_eq!(groups(&g), (false, false, false));
    key(&mut g, PlatformKey::Function(8));
    focus_lost(&mut g);
    assert_eq!(groups(&g), (false, false, false));
    key(&mut g, PlatformKey::Function(8));
    frame(&mut g);
    press_at(&mut g, 1.0, 1.0);
    assert_eq!(groups(&g), (false, false, false));
    // F6 / F7 during composition keep the sidenav active.
    key(&mut g, PlatformKey::Function(8));
    g.search.input.preedit = "注".into();
    assert!(!key(&mut g, PlatformKey::Function(6)));
    assert!(!key(&mut g, PlatformKey::Function(7)));
    assert_eq!(groups(&g), (false, false, true));
}

fn find_widget<'a>(
    node: &'a WidgetNode<GalleryMessage>,
    key: &str,
) -> Option<&'a WidgetNode<GalleryMessage>> {
    if node.key().is_some_and(|k| k.as_str() == key) {
        return Some(node);
    }
    node.children().iter().find_map(|c| find_widget(c, key))
}

#[test]
fn gallery_layout_sidenav_owner_frame_and_status_labels() {
    let key_name = "gallery_layout_sidenav_focus";
    let variant = |g: &Gallery| {
        let widget = g.sidenav_examples();
        let Some(WidgetNode::Card(card)) = find_widget(&widget, key_name) else {
            panic!("{key_name} must be an always-present Card");
        };
        (card.variant, card.padding.top, card.children.len())
    };
    let mut g = layout();
    let idle = variant(&g);
    assert_eq!(idle.0, CardVariant::Plain);
    assert!(format!("{:?}", g.sidenav_examples()).contains("鍵盤導覽：未使用"));
    key(&mut g, PlatformKey::Function(8));
    let on = variant(&g);
    assert_eq!(on.0, CardVariant::Interactive);
    assert_eq!((idle.1, idle.2), (on.1, on.2), "layout never shifts");
    assert!(format!("{:?}", g.sidenav_examples()).contains("鍵盤導覽：使用中"));
    key(&mut g, PlatformKey::Function(7));
    assert_eq!(variant(&g).0, CardVariant::Plain);
    let page = format!("{:?}", g.sidenav_examples());
    assert!(page.contains("F8 進入側邊導覽"));
    assert!(find_widget(&g.sidenav_examples(), "gallery_layout_sidenav").is_some());
}

#[test]
fn gallery_sidenav_hit_and_paint_slots_match_across_themes_and_scales() {
    let mut g = Gallery::new();
    for dark in [false, true] {
        g.dark = dark;
        let theme = g.current_theme();
        for scale in [1.0, 1.25, 2.0] {
            for current in [0u8, 1, 2] {
                g.sidenav.current = current;
                let widget = g.sidenav_examples();
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
                assert_eq!(messages, [sn(0), sn(1), sn(2)], "{dark} {scale} {current}");
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

fn frame_h(g: &mut Gallery, height: f32) {
    g.frame(FrameContext {
        window: WindowId(1),
        logical_width: 1100.0,
        logical_height: height,
        scale_factor: 1.0,
    });
}

#[test]
fn gallery_sidenav_pointer_press_on_disabled_row_changes_nothing_and_deactivates() {
    let mut g = layout();
    key(&mut g, PlatformKey::Function(8));
    frame(&mut g);
    click(&mut g, sn(1));
    assert!(g.sidenav.active);
    assert!(g.hit_is_hidden(sn(3)).is_none(), "disabled row has no hit");
    // Locate the Security row from the enabled Privacy row (same section).
    let (x, y) = center(&g, sn(2));
    let (px, py) = (x, y + 36.0); // Security sits directly below Privacy
    let before = (g.sidenav.current, g.sidenav.confirmed);
    press_at(&mut g, px, py);
    g.event(PlatformEvent::PointerButton {
        window: WindowId(1),
        pressed: false,
        x: px,
        y: py,
        button: 0,
        pointer: 0,
        shift: false,
        ctrl: false,
        alt: false,
    });
    assert_eq!((g.sidenav.current, g.sidenav.confirmed), before);
    assert!(!g.sidenav.active, "a press off any sidenav hit deactivates");
}

#[test]
fn gallery_sidenav_f8_repeat_is_ignored() {
    let mut g = layout();
    assert_eq!(
        g.sidenav_event(&chord_event(
            PlatformKey::Function(8),
            (false, false, false),
            true
        )),
        Some(false)
    );
    assert!(!g.sidenav.active, "a repeat never enters the group");
    key(&mut g, PlatformKey::Function(8));
    frame(&mut g);
    g.hovered = Some(0);
    g.pressed = Some(0);
    g.scroll = 0.0;
    assert_eq!(
        g.sidenav_event(&chord_event(
            PlatformKey::Function(8),
            (false, false, false),
            true
        )),
        Some(false)
    );
    assert_eq!((g.hovered, g.pressed), (Some(0), Some(0)));
    assert_eq!(g.scroll, 0.0, "a repeat never reveals again");
    assert!(g.sidenav.active);
}

#[test]
fn gallery_sidenav_f8_short_viewport_falls_back_to_current_item() {
    let mut found = false;
    for height in (120..=240).step_by(20) {
        let mut g = layout();
        frame_h(&mut g, height as f32);
        assert!(g.sidenav_select(2));
        g.sidenav.active = false;
        g.scroll = 0.0;
        frame_h(&mut g, height as f32);
        assert!(key(&mut g, PlatformKey::Function(8)));
        frame_h(&mut g, height as f32);
        assert_eq!(
            g.hit_is_hidden(sn(2)),
            Some(false),
            "current Privacy visible at height {height}"
        );
        // Short enough that the group anchor cannot show the first item too.
        found |= g.hit_is_hidden(sn(0)) == Some(true);
    }
    assert!(found, "some height exercises the fallback");
}

#[test]
fn gallery_sidebar_and_nav_menu_selection_deactivate_sidenav() {
    let mut g = layout();
    assert!(g.sidenav_select(1));
    assert!(g.sidenav.active);
    assert!(g.sidebar_select(1));
    assert_eq!(groups(&g), (false, true, false));
    assert!(g.sidenav_select(1));
    assert!(g.sidenav.active);
    assert!(g.nav_menu_select(0));
    assert_eq!(groups(&g), (true, false, false));
}
