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

fn key_event(key: PlatformKey) -> PlatformEvent {
    PlatformEvent::Key {
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
    }
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

fn sb(i: u8) -> GalleryMessage {
    GalleryMessage::SidebarSelect(i)
}

fn nav(i: u8) -> GalleryMessage {
    GalleryMessage::NavMenuSelect(i)
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

/// Moves the page 20px so `message` is still fully in view but no longer sits at
/// the reveal anchor; a later unchanged scroll then proves no reveal ran.
fn nudge_keeping_in_view(g: &mut Gallery, message: GalleryMessage) -> f32 {
    frame(g);
    assert!(g.scroll >= 20.0, "page is scrolled far enough to nudge");
    let anchor = g.scroll;
    g.scroll -= 20.0;
    frame(g);
    assert_eq!(g.hit_is_hidden(message), Some(false), "{message:?} in view");
    assert_ne!(g.scroll, anchor);
    g.scroll
}

fn focus_lost(g: &mut Gallery) {
    g.event(PlatformEvent::FocusChanged {
        window: WindowId(1),
        gained: false,
        node_id: 0,
    });
}

fn sidebar_hits(g: &Gallery) -> Vec<GalleryMessage> {
    g.button_info
        .iter()
        .map(|h| h.message)
        .filter(|m| matches!(m, GalleryMessage::SidebarSelect(_)))
        .collect()
}

#[test]
fn gallery_sidebar_pointer_selects_each_enabled_item_with_real_events() {
    let mut g = layout();
    assert_eq!((g.side_nav.current, g.side_nav.confirmed), (0, None));
    click(&mut g, sb(1));
    assert_eq!((g.side_nav.current, g.side_nav.confirmed), (1, Some(1)));
    assert!(g.side_nav.active);
    click(&mut g, sb(3));
    assert_eq!((g.side_nav.current, g.side_nav.confirmed), (3, Some(3)));
    click(&mut g, sb(0));
    assert_eq!((g.side_nav.current, g.side_nav.confirmed), (0, Some(0)));
    assert!(!g.nav_menu.active);
}

#[test]
fn gallery_sidebar_disabled_item_has_no_hit_region() {
    let mut g = layout();
    assert_eq!(sidebar_hits(&g), [sb(0), sb(1), sb(3)]);
    let before = (g.side_nav.current, g.side_nav.confirmed);
    // Disabled (報表), out-of-range and the unknown sentinel never apply.
    for index in [2u8, 4, 9, u8::MAX] {
        assert!(!g.sidebar_select(index), "index {index}");
    }
    assert_eq!(before, (g.side_nav.current, g.side_nav.confirmed));
    assert_eq!(crate::sidebar::sidebar_message("nope"), sb(u8::MAX));
    assert!(!g.side_nav.active);
}

#[test]
fn gallery_sidebar_f7_reveals_group_into_view() {
    let mut g = layout();
    assert_eq!(g.scroll, 0.0);
    assert!(key(&mut g, PlatformKey::Function(7)));
    assert!(g.side_nav.active);
    frame(&mut g);
    assert!(g.scroll > 0.0, "F7 scrolls the offscreen group into view");
    let hit = g.button_info.iter().find(|h| h.message == sb(0)).unwrap();
    let top = hit.rect[1] - g.scroll;
    assert!(
        top >= 0.0 && top + hit.rect[3] <= 760.0,
        "group visible at {top}"
    );
    // Already visible but off the anchor: neither a second F7 nor arrows scroll.
    let scroll = nudge_keeping_in_view(&mut g, sb(0));
    assert_eq!(g.hit_is_hidden(sb(3)), Some(false));
    key(&mut g, PlatformKey::Function(7));
    frame(&mut g);
    assert_eq!(g.scroll, scroll, "F7 on a visible group");
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
}

#[test]
fn gallery_sidebar_f7_with_late_current_item_reveals_whole_group() {
    let mut g = layout();
    g.side_nav.current = 3;
    assert_eq!(g.scroll, 0.0);
    key(&mut g, PlatformKey::Function(7));
    frame(&mut g);
    assert!(g.scroll > 0.0);
    assert_eq!(g.hit_is_hidden(sb(0)), Some(false), "first item visible");
    assert_eq!(g.hit_is_hidden(sb(3)), Some(false), "current item visible");
    let first = g.button_info.iter().find(|h| h.message == sb(0)).unwrap();
    assert!(
        first.rect[1] - g.scroll >= TOOLBAR_HEIGHT + 60.0,
        "room left for the section title"
    );
}

#[test]
fn gallery_sidebar_up_down_wrap_skip_disabled_and_enter_space_confirm() {
    let mut g = layout();
    key(&mut g, PlatformKey::Function(7));
    frame(&mut g);
    for expected in [1, 3, 0, 1] {
        assert!(key(&mut g, PlatformKey::ArrowDown));
        assert_eq!(g.side_nav.current, expected, "報表 is disabled and skipped");
    }
    for expected in [0, 3, 1] {
        assert!(key(&mut g, PlatformKey::ArrowUp));
        assert_eq!(g.side_nav.current, expected);
    }
    assert_eq!(g.side_nav.confirmed, None, "arrows move without confirming");
    assert_eq!((g.hovered, g.pressed), (None, None));
    assert!(key(&mut g, PlatformKey::Enter));
    assert_eq!(g.side_nav.confirmed, Some(1));
    key(&mut g, PlatformKey::ArrowDown);
    assert!(key(&mut g, PlatformKey::Space));
    assert_eq!(g.side_nav.confirmed, Some(3));
    assert!(g.side_nav.active);
}

#[test]
fn gallery_sidebar_left_right_collapse_expand_keep_selection_and_node_ids() {
    let mut g = layout();
    key(&mut g, PlatformKey::Function(7));
    key(&mut g, PlatformKey::ArrowDown);
    key(&mut g, PlatformKey::Enter);
    frame(&mut g);
    let before = (g.side_nav.current, g.side_nav.confirmed);
    let nodes = g.hit_nodes.clone();
    assert!(!g.side_nav.collapsed);
    assert!(key(&mut g, PlatformKey::ArrowLeft));
    assert!(g.side_nav.collapsed);
    frame(&mut g);
    assert_eq!(before, (g.side_nav.current, g.side_nav.confirmed));
    assert_eq!(sidebar_hits(&g), [sb(0), sb(1), sb(3)]);
    assert_eq!(nodes, g.hit_nodes, "item node ids do not depend on state");
    assert!(
        !key(&mut g, PlatformKey::ArrowLeft),
        "already collapsed: handled, no change"
    );
    assert!(key(&mut g, PlatformKey::ArrowRight));
    assert!(!g.side_nav.collapsed);
    frame(&mut g);
    assert_eq!(before, (g.side_nav.current, g.side_nav.confirmed));
    assert_eq!(nodes, g.hit_nodes);
    assert!(g.side_nav.active);
}

#[test]
fn gallery_sidebar_pointer_press_off_group_deactivates() {
    let mut g = layout();
    key(&mut g, PlatformKey::Function(7));
    assert!(g.side_nav.active);
    frame(&mut g);
    press_at(&mut g, 1.0, 1.0);
    assert!(!g.side_nav.active);
    assert!(!key(&mut g, PlatformKey::ArrowDown));
    assert_eq!(g.side_nav.current, 0);
    // A press on another Gallery button also deactivates.
    key(&mut g, PlatformKey::Function(7));
    assert!(g.side_nav.active);
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
    assert!(!g.side_nav.active);
    // A press on a sidebar item keeps the group active.
    click(&mut g, sb(1));
    assert!(g.side_nav.active);
}

#[test]
fn gallery_sidebar_keys_ignored_off_page_and_after_escape_or_blur() {
    let mut g = Gallery::new();
    g.selected_category = 0;
    frame(&mut g);
    key(&mut g, PlatformKey::Function(7));
    assert!(!g.side_nav.active, "F7 off the Layout page is ignored");
    assert!(!key(&mut g, PlatformKey::ArrowDown));

    let mut g = layout();
    assert!(
        !key(&mut g, PlatformKey::ArrowDown),
        "inactive ignores keys"
    );
    assert!(!key(&mut g, PlatformKey::Escape));
    assert_eq!(g.side_nav.current, 0);

    key(&mut g, PlatformKey::Function(7));
    frame(&mut g);
    let scroll = g.scroll;
    assert!(key(&mut g, PlatformKey::Escape));
    assert!(!g.side_nav.active);
    assert_eq!(g.scroll, scroll, "the leaving Esc never scrolls");
    assert!(!key(&mut g, PlatformKey::ArrowDown));
    assert_eq!(g.side_nav.current, 0);

    key(&mut g, PlatformKey::Function(7));
    focus_lost(&mut g);
    assert!(!g.side_nav.active);
    assert!(!key(&mut g, PlatformKey::ArrowDown));

    key(&mut g, PlatformKey::Function(7));
    assert!(g.side_nav.active);
    frame(&mut g);
    let category = g
        .button_info
        .iter()
        .position(|h| h.message == GalleryMessage::SelectCategory(0))
        .unwrap();
    assert!(g.activate(category));
    assert!(!g.side_nav.active);

    // Unhandled keys are never consumed, and never change the group.
    let mut g = layout();
    key(&mut g, PlatformKey::Function(7));
    assert_eq!(g.sidebar_event(&key_event(PlatformKey::Tab)), None);
    assert_eq!(g.sidebar_event(&key_event(PlatformKey::Home)), None);
    assert!(g.side_nav.active);
}

#[test]
fn gallery_sidebar_keys_ignored_during_preedit() {
    let mut g = layout();
    g.search.input.preedit = "注".into();
    assert!(!key(&mut g, PlatformKey::Function(7)));
    assert!(!g.side_nav.active);
    g.search.input.preedit.clear();
    key(&mut g, PlatformKey::Function(7));
    assert!(g.side_nav.active);
    g.search.input.preedit = "注".into();
    for k in [
        PlatformKey::ArrowDown,
        PlatformKey::ArrowUp,
        PlatformKey::ArrowLeft,
        PlatformKey::ArrowRight,
        PlatformKey::Enter,
        PlatformKey::Space,
        PlatformKey::Escape,
    ] {
        assert!(!key(&mut g, k), "{k:?} must not act during composition");
    }
    assert!(g.side_nav.active);
    assert_eq!(g.side_nav.current, 0);
    assert!(!g.side_nav.collapsed);
    assert_eq!(g.side_nav.confirmed, None);
    // Keys the host does not handle pass through even during composition.
    assert_eq!(g.sidebar_event(&key_event(PlatformKey::Tab)), None);
}

#[test]
fn gallery_sidebar_f7_and_navigation_menu_f6_are_mutually_exclusive() {
    let mut g = layout();
    key(&mut g, PlatformKey::Function(6));
    assert!((g.nav_menu.active, g.side_nav.active) == (true, false));
    key(&mut g, PlatformKey::Function(7));
    assert!((g.nav_menu.active, g.side_nav.active) == (false, true));
    // Keys now belong to the sidebar only.
    assert!(key(&mut g, PlatformKey::ArrowLeft));
    assert!(g.side_nav.collapsed);
    assert_eq!(g.nav_menu.item, "home");
    key(&mut g, PlatformKey::Function(6));
    assert!((g.nav_menu.active, g.side_nav.active) == (true, false));
    assert!(key(&mut g, PlatformKey::ArrowRight));
    assert_eq!(g.nav_menu.item, "products");
    assert!(g.side_nav.collapsed, "inactive sidebar ignores arrows");
    // Pointer: pressing a sidebar item hands the group over.
    click(&mut g, sb(1));
    assert!((g.nav_menu.active, g.side_nav.active) == (false, true));
    // Pressing a menu item hands it back.
    click(&mut g, nav(0));
    assert!((g.nav_menu.active, g.side_nav.active) == (true, false));
    // Focus loss and an off-group press clear both.
    key(&mut g, PlatformKey::Function(7));
    focus_lost(&mut g);
    assert!((g.nav_menu.active, g.side_nav.active) == (false, false));
    assert_eq!(g.sidebar_event(&key_event(PlatformKey::ArrowDown)), None);
    key(&mut g, PlatformKey::Function(7));
    frame(&mut g);
    press_at(&mut g, 1.0, 1.0);
    assert!((g.nav_menu.active, g.side_nav.active) == (false, false));
}

#[test]
fn gallery_sidebar_hit_and_paint_slots_match_across_themes_scales_and_states() {
    let mut g = Gallery::new();
    for dark in [false, true] {
        g.dark = dark;
        let theme = g.current_theme();
        for scale in [1.0, 1.25, 2.0] {
            for collapsed in [false, true] {
                for current in [0u8, 1, 3] {
                    g.side_nav.collapsed = collapsed;
                    g.side_nav.current = current;
                    let widget = g.sidebar_examples();
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
                    assert_eq!(messages, [sb(0), sb(1), sb(3)], "{collapsed} {current}");
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
}

// ── R1 / R2 for the NavigationMenu host ─────────────────────────────────

#[test]
fn gallery_navigation_menu_reveal_only_when_out_of_view_and_not_on_leaving_escape() {
    let mut g = layout();
    assert_eq!(g.scroll, 0.0);
    key(&mut g, PlatformKey::Function(6));
    frame(&mut g);
    assert!(g.scroll > 0.0, "F6 reveals an out-of-view menu");
    // In view but off the anchor: keys must not move the page.
    let revealed = nudge_keeping_in_view(&mut g, nav(0));
    for k in [
        PlatformKey::ArrowRight,
        PlatformKey::ArrowDown,
        PlatformKey::ArrowDown,
        PlatformKey::ArrowUp,
        PlatformKey::ArrowLeft,
        PlatformKey::ArrowLeft,
    ] {
        key(&mut g, k);
        frame(&mut g);
        assert_eq!(g.scroll, revealed, "{k:?} on a visible menu");
    }
    // F6 again while visible does not scroll either.
    key(&mut g, PlatformKey::Function(6));
    frame(&mut g);
    assert_eq!(g.scroll, revealed, "F6 on a visible menu");
    // Out of view again: a navigation key reveals it.
    g.scroll = 0.0;
    frame(&mut g);
    assert!(key(&mut g, PlatformKey::ArrowRight));
    frame(&mut g);
    assert!(g.scroll > 0.0, "an out-of-view menu is revealed by arrows");
    // The Esc that leaves never scrolls, even when the menu is out of view.
    g.nav_menu.child = None;
    g.scroll = 0.0;
    frame(&mut g);
    assert!(key(&mut g, PlatformKey::Escape));
    assert!(!g.nav_menu.active);
    assert_eq!(g.scroll, 0.0, "leaving Esc must not scroll");
}

#[test]
fn gallery_navigation_menu_pointer_click_does_not_reveal_when_content_height_is_stable() {
    // Known limitation (056 GUI finding, not fixed here): clicking an item
    // while the in-flow 產品 dropdown is open closes it, the content shrinks,
    // and the shared `scroll.clamp(0, max_scroll)` in `frame` can move the
    // page. This test only proves that the host itself does not reveal on a
    // pointer click; it uses items whose click leaves the content height alone.
    let mut g = layout();
    key(&mut g, PlatformKey::Function(6));
    frame(&mut g);
    let scroll = nudge_keeping_in_view(&mut g, nav(0));
    // Real events at the live hit region of an already-visible item.
    for message in [nav(5), nav(0), nav(5)] {
        let (x, y) = center(&g, message);
        press_at(&mut g, x, y);
        assert!(release_at(&mut g, x, y), "{message:?}");
        frame(&mut g);
        assert_eq!(g.scroll, scroll, "{message:?} click on a visible menu");
    }
    assert_eq!(g.nav_menu.destination, Some(5));
}

#[test]
fn gallery_navigation_menu_reveals_focused_child_below_the_fold() {
    let mut g = layout();
    key(&mut g, PlatformKey::Function(6));
    key(&mut g, PlatformKey::ArrowRight);
    frame(&mut g);
    assert_eq!(g.nav_menu.item, "products");
    // Park the products row at the very bottom of the viewport.
    let hit = g.button_info.iter().find(|h| h.message == nav(1)).unwrap();
    let [_, view_top, _, view_height] = g.viewport_clip;
    g.scroll = (hit.rect[1] + hit.rect[3] - (view_top + view_height)).clamp(0.0, g.max_scroll);
    frame(&mut g);
    assert_eq!(g.hit_is_hidden(nav(1)), Some(false));
    assert_eq!(
        g.hit_is_hidden(nav(2)),
        Some(true),
        "child is below the fold"
    );
    assert!(key(&mut g, PlatformKey::ArrowDown));
    frame(&mut g);
    assert_eq!(g.nav_menu.child, Some("desktop"));
    assert_eq!(
        g.hit_is_hidden(nav(2)),
        Some(false),
        "focused child revealed"
    );
}

#[test]
fn gallery_sidebar_f6_during_preedit_keeps_sidebar_active() {
    let mut g = layout();
    key(&mut g, PlatformKey::Function(7));
    assert!(g.side_nav.active);
    g.search.input.preedit = "注".into();
    let scroll = g.scroll;
    assert_eq!(g.sidebar_event(&key_event(PlatformKey::Function(6))), None);
    assert!(g.side_nav.active);
    assert!(!key(&mut g, PlatformKey::Function(6)), "F6 is ignored");
    assert!((g.side_nav.active, g.nav_menu.active) == (true, false));
    assert_eq!(g.scroll, scroll);
    assert_eq!((g.side_nav.current, g.side_nav.confirmed), (0, None));
}

#[test]
fn gallery_navigation_menu_activation_deactivates_sidebar() {
    let mut g = layout();
    key(&mut g, PlatformKey::Function(7));
    assert!(g.nav_menu_select(1));
    assert!((g.nav_menu.active, g.side_nav.active) == (true, false));
    g.side_nav.active = true;
    g.nav_menu.active = false;
    // Direct F6 hook (bypassing the owner) still keeps the groups exclusive.
    assert_eq!(
        g.nav_menu_event(&key_event(PlatformKey::Function(6))),
        Some(true)
    );
    assert!((g.nav_menu.active, g.side_nav.active) == (true, false));
}

#[test]
fn gallery_navigation_menu_preedit_passes_unhandled_keys_through() {
    let mut g = layout();
    key(&mut g, PlatformKey::Function(6));
    assert!(g.nav_menu.active);
    g.search.input.preedit = "注".into();
    for k in [
        PlatformKey::Tab,
        PlatformKey::Home,
        PlatformKey::Backspace,
        PlatformKey::Function(2),
    ] {
        assert_eq!(
            g.nav_menu_event(&key_event(k)),
            None,
            "{k:?} is not a menu key and must pass through"
        );
    }
    // Handled keys are still blocked during composition.
    for k in [
        PlatformKey::ArrowRight,
        PlatformKey::Enter,
        PlatformKey::Escape,
    ] {
        assert_eq!(g.nav_menu_event(&key_event(k)), Some(false), "{k:?}");
    }
    assert!(g.nav_menu.active);
    assert_eq!(g.nav_menu.item, "home");
}

// ── ROADMAP 059 ─────────────────────────────────────────────────────────

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

fn find_widget<'a>(
    node: &'a WidgetNode<GalleryMessage>,
    key: &str,
) -> Option<&'a WidgetNode<GalleryMessage>> {
    if node.key().is_some_and(|k| k.as_str() == key) {
        return Some(node);
    }
    node.children().iter().find_map(|c| find_widget(c, key))
}

fn frame_variant(node: &WidgetNode<GalleryMessage>, key: &str) -> (CardVariant, f32, usize) {
    let Some(WidgetNode::Card(card)) = find_widget(node, key) else {
        panic!("{key} must be an always-present Card");
    };
    (card.variant, card.padding.top, card.children.len())
}

#[test]
fn gallery_layout_hosts_snackbar_press_deactivates_menu_and_sidebar() {
    for (nav_active, side_key) in [
        (true, PlatformKey::Function(6)),
        (false, PlatformKey::Function(7)),
    ] {
        let mut g = layout();
        key(&mut g, side_key);
        assert_eq!(
            (g.nav_menu.active, g.side_nav.active),
            (nav_active, !nav_active)
        );
        g.snackbar.show(std::time::Instant::now(), true);
        frame(&mut g);
        let sb = g.snackbar.bounds.expect("snackbar bounds painted");
        let (item, current) = (g.nav_menu.item, g.side_nav.current);
        let before = (
            g.nav_menu.destination,
            g.side_nav.confirmed,
            g.nav_menu.child,
        );
        press_at(&mut g, sb[0] + 4.0, sb[1] + 4.0);
        assert_eq!((g.nav_menu.active, g.side_nav.active), (false, false));
        assert_eq!((g.nav_menu.item, g.side_nav.current), (item, current));
        assert_eq!(
            before,
            (
                g.nav_menu.destination,
                g.side_nav.confirmed,
                g.nav_menu.child
            )
        );
    }
}

#[test]
fn gallery_sidebar_ignores_ctrl_alt_meta_chords() {
    for chord in [
        (true, false, false),
        (false, true, false),
        (false, false, true),
    ] {
        let mut g = layout();
        // Entry chords are not ours while inactive.
        for k in [PlatformKey::Function(6), PlatformKey::Function(7)] {
            assert_eq!(g.sidebar_event(&chord_event(k, chord, false)), None);
        }
        assert!(!g.side_nav.active, "{chord:?}");
        key(&mut g, PlatformKey::Function(7));
        let scroll = g.scroll;
        for k in [
            PlatformKey::ArrowDown,
            PlatformKey::ArrowUp,
            PlatformKey::ArrowLeft,
            PlatformKey::ArrowRight,
            PlatformKey::Enter,
            PlatformKey::Space,
            PlatformKey::Escape,
            PlatformKey::Function(6),
            PlatformKey::Function(7),
        ] {
            assert_eq!(
                g.sidebar_event(&chord_event(k, chord, false)),
                None,
                "{k:?} with {chord:?}"
            );
        }
        assert!(
            g.side_nav.active,
            "chords never leave the group ({chord:?})"
        );
        assert_eq!(
            (
                g.side_nav.current,
                g.side_nav.confirmed,
                g.side_nav.collapsed
            ),
            (0, None, false)
        );
        assert_eq!(g.scroll, scroll);
    }
    // Shift stays allowed.
    let mut g = layout();
    key(&mut g, PlatformKey::Function(7));
    let shift = PlatformEvent::Key {
        window: WindowId(1),
        key: PlatformKey::ArrowDown,
        pressed: true,
        shift: true,
        ctrl: false,
        alt: false,
        meta: false,
        repeat: false,
        text: None,
        character: None,
    };
    assert_eq!(g.sidebar_event(&shift), Some(true));
    assert_eq!(g.side_nav.current, 1);
}

#[test]
fn gallery_sidebar_enter_space_ignore_os_auto_repeat() {
    let mut g = layout();
    key(&mut g, PlatformKey::Function(7));
    for k in [PlatformKey::Enter, PlatformKey::Space] {
        assert_eq!(
            g.sidebar_event(&chord_event(k, (false, false, false), true)),
            Some(false),
            "{k:?} repeat"
        );
        assert_eq!(g.side_nav.confirmed, None);
        assert!(g.side_nav.active);
    }
    // A fresh press still confirms; arrow repeat still roves.
    assert_eq!(
        g.sidebar_event(&chord_event(
            PlatformKey::ArrowDown,
            (false, false, false),
            true
        )),
        Some(true)
    );
    assert_eq!(g.side_nav.current, 1);
    assert!(key(&mut g, PlatformKey::Enter));
    assert_eq!(g.side_nav.confirmed, Some(1));
}

#[test]
fn gallery_layout_keyboard_owner_frame_is_interactive_only_while_active() {
    let nav_key = "gallery_nav_menu_focus";
    let side_key = "gallery_layout_sidebar_focus";
    let mut g = layout();
    let idle_nav = frame_variant(&g.nav_menu_examples(), nav_key);
    let idle_side = frame_variant(&g.sidebar_examples(), side_key);
    assert_eq!(idle_nav.0, CardVariant::Plain);
    assert_eq!(idle_side.0, CardVariant::Plain);
    key(&mut g, PlatformKey::Function(6));
    let nav_on = frame_variant(&g.nav_menu_examples(), nav_key);
    let side_off = frame_variant(&g.sidebar_examples(), side_key);
    assert_eq!(nav_on.0, CardVariant::Interactive);
    assert_eq!(side_off.0, CardVariant::Plain);
    key(&mut g, PlatformKey::Function(7));
    assert_eq!(
        frame_variant(&g.nav_menu_examples(), nav_key).0,
        CardVariant::Plain
    );
    let side_on = frame_variant(&g.sidebar_examples(), side_key);
    assert_eq!(side_on.0, CardVariant::Interactive);
    // Padding and child count are constant, so layout and IDs never shift.
    assert_eq!((idle_nav.1, idle_nav.2), (nav_on.1, nav_on.2));
    assert_eq!((idle_side.1, idle_side.2), (side_on.1, side_on.2));
    assert!(idle_nav.1 > 0.0 && idle_side.1 > 0.0);
}

#[test]
fn gallery_keyboard_status_labels_are_zh_tw() {
    let mut g = layout();
    let dump = |n: WidgetNode<GalleryMessage>| format!("{n:?}");
    let idle = dump(g.nav_menu_examples());
    assert!(idle.contains("鍵盤選單：未使用"));
    assert!(dump(g.sidebar_examples()).contains("鍵盤群組：未使用"));
    key(&mut g, PlatformKey::Function(6));
    assert!(dump(g.nav_menu_examples()).contains("鍵盤選單：使用中"));
    key(&mut g, PlatformKey::Function(7));
    assert!(dump(g.sidebar_examples()).contains("鍵盤群組：使用中"));
    for text in [
        dump(g.nav_menu_examples()),
        dump(g.sidebar_examples()),
        dump(g.radio_cards_examples()),
        dump(g.checkbox_cards_examples()),
    ] {
        assert!(!text.contains("Keyboard group") && !text.contains("Keyboard menu"));
    }
    assert!(dump(g.radio_cards_examples()).contains("鍵盤群組：未使用"));
    assert!(dump(g.checkbox_cards_examples()).contains("鍵盤群組：未使用"));
    g.radio_cards.active = true;
    g.checkbox_cards.active = true;
    assert!(dump(g.radio_cards_examples()).contains("鍵盤群組：使用中"));
    assert!(dump(g.checkbox_cards_examples()).contains("鍵盤群組：使用中"));
}

// ── 059 P1: keyboard-owner ring contrast ────────────────────────────────

fn paint_widget(g: &mut Gallery, widget: &WidgetNode<GalleryMessage>) -> Frame {
    let theme = g.current_theme();
    let context = WidgetLayoutContext {
        body_font_size: theme.typography.body,
        body_line_height: theme.typography.body * theme.typography.line_height,
        label_font_size: theme.typography.label,
        control_height: 36.0,
        scale_factor: 1.0,
    };
    let layout = widget.to_layout_with_context(NodeId::new(1), &context);
    let snapshot = g
        .layout
        .compute_with_text(&layout, (600.0, 800.0), &mut g.fonts, 1.0)
        .unwrap();
    let mut paint = Frame::default();
    let mut index = 0;
    render_content(
        &mut paint,
        widget,
        &layout,
        &snapshot,
        &theme,
        1.0,
        0.0,
        [0.0, 0.0, 600.0, 800.0],
        &mut index,
        None,
        None,
        &mut g.fonts,
        &mut g.atlas,
    );
    paint
}

#[test]
fn gallery_layout_owner_frame_border_is_ring_with_3_to_1_contrast() {
    for dark in [false, true] {
        let mut g = layout();
        g.dark = dark;
        let theme = g.current_theme();
        for host in [6u8, 7] {
            key(&mut g, PlatformKey::Function(host));
            let widget = if host == 6 {
                g.nav_menu_examples()
            } else {
                g.sidebar_examples()
            };
            let paint = paint_widget(&mut g, &widget);
            let ring = rgba(theme.colors.ring);
            let frames: Vec<_> = paint
                .clipped_quads
                .iter()
                .filter(|q| q.quad.border_color == ring && q.quad.border_width == 2.0)
                .collect();
            assert_eq!(frames.len(), 1, "one 2px ring frame (dark={dark}, F{host})");
            assert!(
                acme_theme::contrast_ratio(theme.colors.ring, theme.colors.background) >= 3.0,
                "ring vs background, dark={dark}"
            );
            let accent = rgba(theme.colors.accent);
            assert!(
                paint
                    .clipped_quads
                    .iter()
                    .all(|q| q.quad.border_color != accent),
                "no accent-coloured border"
            );
        }
    }
}

#[test]
fn gallery_theme_packs_ring_contrast_against_surface_and_background() {
    let mut report = Vec::new();
    let mut failing = Vec::new();
    let mut names = vec!["default".to_string()];
    names.extend(available_themes().iter().map(|s| s.to_string()));
    for name in &names {
        for mode in [ThemeMode::Light, ThemeMode::Dark] {
            let theme = if name == "default" {
                match mode {
                    ThemeMode::Dark => Theme::dark(),
                    ThemeMode::Light => Theme::light(),
                }
            } else {
                match theme_by_name(name, mode) {
                    Some(t) => t,
                    None => continue,
                }
            };
            let s = acme_theme::contrast_ratio(theme.colors.ring, theme.colors.surface);
            let b = acme_theme::contrast_ratio(theme.colors.ring, theme.colors.background);
            report.push(format!("{name} {mode:?}: surface {s:.2} background {b:.2}"));
            if s < 3.0 || b < 3.0 {
                failing.push(format!("{name} {mode:?}"));
            }
            if name == "default" {
                assert!(s >= 3.0 && b >= 3.0, "default {mode:?}: {s} {b}");
            }
        }
    }
    println!(
        "{}",
        report.join(
            "
"
        )
    );
    println!("FAILING PACKS: {failing:?}");
}

#[test]
fn gallery_layout_hit_rects_are_stable_across_f6_f7_escape() {
    let mut g = layout();
    let rects = |g: &Gallery| -> Vec<(GalleryMessage, [f32; 4])> {
        g.button_info
            .iter()
            .filter(|h| {
                matches!(
                    h.message,
                    GalleryMessage::NavMenuSelect(_) | GalleryMessage::SidebarSelect(_)
                )
            })
            .map(|h| (h.message, h.rect))
            .collect()
    };
    let before = rects(&g);
    assert!(!before.is_empty());
    for step in [
        PlatformKey::Function(6),
        PlatformKey::Function(7),
        PlatformKey::Escape,
    ] {
        key(&mut g, step);
        frame(&mut g);
        assert_eq!(rects(&g), before, "hit rects shifted after {step:?}");
    }
}
