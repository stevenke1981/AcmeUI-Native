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

fn inputs() -> Gallery {
    let mut g = Gallery::new();
    g.selected_category = 2;
    frame(&mut g);
    g
}

fn cb(i: u8) -> GalleryMessage {
    GalleryMessage::CheckboxCardToggle(i)
}

fn rc(i: u8) -> GalleryMessage {
    GalleryMessage::RadioCardSelect(i)
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

/// Scrolls the target toward the top of the viewport and returns its center.
fn bring_into_view(g: &mut Gallery, message: GalleryMessage) -> (f32, f32) {
    frame(g);
    let y = g
        .button_info
        .iter()
        .find(|h| h.message == message)
        .unwrap()
        .rect[1];
    g.scroll = (y - 160.0).clamp(0.0, g.max_scroll);
    frame(g);
    center(g, message)
}

/// Scrolls to the target and clicks its hit region with real pointer events.
fn click(g: &mut Gallery, message: GalleryMessage) {
    let (x, y) = bring_into_view(g, message);
    press_at(g, x, y);
    assert!(release_at(g, x, y), "{message:?} at ({x}, {y})");
    frame(g);
}

/// Presses (without releasing) on the target, as a pointer-off probe.
fn press_on(g: &mut Gallery, message: GalleryMessage) {
    let (x, y) = bring_into_view(g, message);
    press_at(g, x, y);
}

/// Moves the page 20px so every target is still fully in view but the page no
/// longer sits at the reveal anchor; a later unchanged scroll then proves no
/// reveal ran. Scrolls up when possible, otherwise down.
fn nudge_keeping_in_view(g: &mut Gallery, messages: &[GalleryMessage]) -> f32 {
    frame(g);
    let anchor = g.scroll;
    g.scroll = if anchor >= 20.0 {
        anchor - 20.0
    } else {
        anchor + 20.0
    };
    frame(g);
    for m in messages {
        assert_eq!(g.hit_is_hidden(*m), Some(false), "{m:?} in view");
    }
    assert_ne!(g.scroll, anchor, "page moved off the anchor");
    g.scroll
}

fn focus_lost(g: &mut Gallery) {
    g.event(PlatformEvent::FocusChanged {
        window: WindowId(1),
        gained: false,
        node_id: 0,
    });
}

fn checkbox_hits(g: &Gallery) -> Vec<GalleryMessage> {
    g.button_info
        .iter()
        .map(|h| h.message)
        .filter(|m| matches!(m, GalleryMessage::CheckboxCardToggle(_)))
        .collect()
}

#[test]
fn gallery_checkbox_cards_pointer_toggles_each_option_with_real_events() {
    let mut g = inputs();
    assert_eq!(checkbox_hits(&g), [cb(0), cb(1), cb(2), cb(3)]);
    assert_eq!(g.checkbox_cards.checked, [false; 4]);
    for index in [2u8, 0, 3, 1] {
        click(&mut g, cb(index));
        assert!(g.checkbox_cards.checked[index as usize], "option {index}");
        assert_eq!(g.checkbox_cards.focus, Some(index));
        assert!(g.checkbox_cards.active);
    }
    assert_eq!(g.checkbox_cards.checked, [true; 4]);
    // A second click on the same card unchecks only that card.
    click(&mut g, cb(2));
    assert_eq!(g.checkbox_cards.checked, [true, true, false, true]);
    click(&mut g, cb(2));
    assert_eq!(g.checkbox_cards.checked, [true; 4]);
}

#[test]
fn gallery_checkbox_cards_f7_enters_and_reveals_only_when_hidden() {
    let mut g = inputs();
    assert_eq!(g.scroll, 0.0);
    assert!(
        (0..4).any(|i| g.hit_is_hidden(cb(i)) == Some(true)),
        "group starts at least partly off-screen"
    );
    assert!(!g.checkbox_cards.active);
    assert_eq!(g.checkbox_cards.focus, None);
    assert!(key(&mut g, PlatformKey::Function(7)));
    frame(&mut g);
    assert!(g.checkbox_cards.active);
    assert_eq!(g.checkbox_cards.focus, Some(0), "focus defaults to first");
    assert!(g.scroll > 0.0, "F7 reveals a hidden group");
    for i in 0..4 {
        assert_eq!(g.hit_is_hidden(cb(i)), Some(false), "card {i} visible");
    }
    // Visible but off the anchor: neither a second F7 nor arrows scroll.
    let scroll = nudge_keeping_in_view(&mut g, &[cb(0), cb(1), cb(2), cb(3)]);
    key(&mut g, PlatformKey::Function(7));
    frame(&mut g);
    assert_eq!(g.scroll, scroll, "F7 on a visible group");
    for k in [
        PlatformKey::ArrowDown,
        PlatformKey::ArrowDown,
        PlatformKey::ArrowUp,
        PlatformKey::ArrowLeft,
        PlatformKey::ArrowRight,
        PlatformKey::Space,
        PlatformKey::Space,
    ] {
        assert!(key(&mut g, k));
        frame(&mut g);
        assert_eq!(g.scroll, scroll, "{k:?} on a visible group");
    }
    // An existing focus is kept when re-entering.
    g.checkbox_cards.active = false;
    g.checkbox_cards.focus = Some(2);
    key(&mut g, PlatformKey::Function(7));
    assert_eq!(g.checkbox_cards.focus, Some(2));
}

#[test]
fn gallery_checkbox_cards_arrows_move_focus_without_toggling() {
    let mut g = inputs();
    assert!(
        !key(&mut g, PlatformKey::ArrowDown),
        "inactive ignores keys"
    );
    assert_eq!(g.checkbox_cards.focus, None);
    key(&mut g, PlatformKey::Function(7));
    assert_eq!(g.checkbox_cards.focus, Some(0));
    for expected in [1, 2, 3, 0] {
        assert!(key(&mut g, PlatformKey::ArrowDown));
        assert_eq!(g.checkbox_cards.focus, Some(expected));
    }
    for expected in [3, 2] {
        assert!(key(&mut g, PlatformKey::ArrowUp));
        assert_eq!(g.checkbox_cards.focus, Some(expected));
    }
    assert!(key(&mut g, PlatformKey::ArrowRight));
    assert_eq!(g.checkbox_cards.focus, Some(3));
    assert!(key(&mut g, PlatformKey::ArrowLeft));
    assert_eq!(g.checkbox_cards.focus, Some(2));
    assert_eq!(g.checkbox_cards.checked, [false; 4], "arrows never toggle");
    // Every keyboard change clears hover / press state.
    g.pressed = Some(1);
    g.hovered = Some(1);
    assert!(key(&mut g, PlatformKey::ArrowDown));
    assert_eq!((g.pressed, g.hovered), (None, None));
}

#[test]
fn gallery_checkbox_cards_space_toggles_enter_passes_through() {
    let mut g = inputs();
    key(&mut g, PlatformKey::Function(7));
    assert!(key(&mut g, PlatformKey::Space));
    assert_eq!(g.checkbox_cards.checked, [true, false, false, false]);
    assert!(key(&mut g, PlatformKey::Space));
    assert_eq!(g.checkbox_cards.checked, [false; 4]);
    key(&mut g, PlatformKey::ArrowDown);
    key(&mut g, PlatformKey::ArrowDown);
    g.pressed = Some(1);
    g.hovered = Some(1);
    assert!(key(&mut g, PlatformKey::Space));
    assert_eq!(g.checkbox_cards.checked, [false, false, true, false]);
    assert_eq!(
        g.checkbox_cards.focus,
        Some(2),
        "focus stays on the toggled"
    );
    assert_eq!((g.pressed, g.hovered), (None, None));
    // Enter is unhandled: the hook returns None and nothing changes.
    assert_eq!(g.checkbox_cards_event(&key_event(PlatformKey::Enter)), None);
    assert_eq!(
        g.inputs_keyboard_event(&key_event(PlatformKey::Enter)),
        None
    );
    key(&mut g, PlatformKey::Enter);
    assert_eq!(g.checkbox_cards.checked, [false, false, true, false]);
    assert!(g.checkbox_cards.active, "Enter keeps the group active");
}

#[test]
fn gallery_checkbox_cards_escape_blur_category_and_pointer_off_deactivate() {
    let mut g = inputs();
    key(&mut g, PlatformKey::Function(7));
    assert!(key(&mut g, PlatformKey::Escape));
    assert!(!g.checkbox_cards.active);
    assert!(
        !key(&mut g, PlatformKey::ArrowDown),
        "inactive ignores keys"
    );
    assert_eq!(g.checkbox_cards.focus, Some(0));

    key(&mut g, PlatformKey::Function(7));
    focus_lost(&mut g);
    assert!(!g.checkbox_cards.active);
    assert!(!key(&mut g, PlatformKey::Space));
    assert_eq!(g.checkbox_cards.checked, [false; 4]);

    // A pointer press off the cards deactivates; on a card it does not.
    key(&mut g, PlatformKey::Function(7));
    press_on(&mut g, cb(1));
    assert!(g.checkbox_cards.active, "press on a card keeps the group");
    press_on(&mut g, rc(0));
    assert!(!g.checkbox_cards.active, "press off the cards deactivates");

    // Leaving the page deactivates the group.
    key(&mut g, PlatformKey::Function(7));
    assert!(g.checkbox_cards.active);
    let category = g
        .button_info
        .iter()
        .position(|h| h.message == GalleryMessage::SelectCategory(0))
        .unwrap();
    assert!(g.activate(category));
    assert!(!g.checkbox_cards.active);
}

#[test]
fn gallery_checkbox_cards_preedit_guard_passes_unhandled_keys() {
    let mut g = inputs();
    key(&mut g, PlatformKey::Function(7));
    g.checkbox_cards.focus = Some(1);
    g.search.input.preedit = "注".into();
    // Handled keys are ignored during composition.
    for k in [
        PlatformKey::ArrowDown,
        PlatformKey::ArrowUp,
        PlatformKey::ArrowLeft,
        PlatformKey::ArrowRight,
        PlatformKey::Space,
        PlatformKey::Escape,
    ] {
        assert_eq!(
            g.checkbox_cards_event(&key_event(k)),
            Some(false),
            "{k:?} blocked"
        );
    }
    assert_eq!(g.checkbox_cards.focus, Some(1));
    assert_eq!(g.checkbox_cards.checked, [false; 4]);
    assert!(g.checkbox_cards.active, "Escape did not leave");
    // Unhandled keys pass through to the rest of the Gallery.
    for k in [
        PlatformKey::Enter,
        PlatformKey::Tab,
        PlatformKey::Function(1),
    ] {
        assert_eq!(g.checkbox_cards_event(&key_event(k)), None, "{k:?}");
    }
    // F7 during composition is ignored and does not activate.
    g.checkbox_cards.active = false;
    assert!(!key(&mut g, PlatformKey::Function(7)));
    assert!(!g.checkbox_cards.active);
    // F6 during composition leaves an active group untouched.
    g.checkbox_cards.active = true;
    assert!(!key(&mut g, PlatformKey::Function(6)));
    assert!(g.checkbox_cards.active);
    assert!(!g.radio_cards.active);
    g.search.input.preedit.clear();
    assert!(key(&mut g, PlatformKey::ArrowDown));
    assert_eq!(g.checkbox_cards.focus, Some(2));
}

#[test]
fn gallery_checkbox_cards_f6_f7_mutually_exclusive_with_radio_cards() {
    let mut g = inputs();
    assert!(key(&mut g, PlatformKey::Function(6)));
    assert!(g.radio_cards.active && !g.checkbox_cards.active);
    assert!(key(&mut g, PlatformKey::Function(7)));
    assert!(g.checkbox_cards.active && !g.radio_cards.active);
    // Keys now belong to CheckboxCards only.
    assert!(key(&mut g, PlatformKey::ArrowDown));
    assert_eq!(g.checkbox_cards.focus, Some(1));
    assert_eq!(g.radio_cards.selected, None);
    assert!(key(&mut g, PlatformKey::Function(6)));
    assert!(g.radio_cards.active && !g.checkbox_cards.active);
    assert!(key(&mut g, PlatformKey::ArrowDown));
    assert_eq!(g.radio_cards.selected, Some(0));
    assert_eq!(g.checkbox_cards.focus, Some(1), "unchanged");
    // Pointer and focus events never claim the page.
    for event in [
        PlatformEvent::PointerMoved {
            window: WindowId(1),
            x: 5.0,
            y: 5.0,
        },
        PlatformEvent::FocusChanged {
            window: WindowId(1),
            gained: true,
            node_id: 0,
        },
    ] {
        assert_eq!(g.inputs_keyboard_event(&event), None);
    }
    // Pointer activation moves the group too.
    click(&mut g, cb(3));
    assert!(g.checkbox_cards.active && !g.radio_cards.active);
    click(&mut g, rc(1));
    assert!(g.radio_cards.active && !g.checkbox_cards.active);
}

#[test]
fn gallery_checkbox_cards_unknown_value_is_rejected() {
    assert_eq!(checkbox_cards::checkbox_message("email"), cb(0));
    assert_eq!(checkbox_cards::checkbox_message("digest"), cb(3));
    let unknown = checkbox_cards::checkbox_message("fax");
    assert_eq!(unknown, cb(u8::MAX), "unknown maps to the sentinel, not 0");
    let mut g = inputs();
    g.checkbox_cards.focus = Some(1);
    assert!(!g.checkbox_card_toggle(u8::MAX));
    assert!(!g.checkbox_card_toggle(4));
    assert_eq!(g.checkbox_cards.checked, [false; 4]);
    assert_eq!(g.checkbox_cards.focus, Some(1));
    assert!(!g.checkbox_cards.active);
}

#[test]
fn gallery_checkbox_cards_hit_and_paint_slots_match_across_themes_and_scales() {
    let mut g = Gallery::new();
    for dark in [false, true] {
        g.dark = dark;
        let theme = g.current_theme();
        for scale in [1.0, 1.25, 2.0] {
            for (checked, focus, active) in [
                ([false; 4], None, false),
                ([true, false, true, false], Some(1u8), true),
                ([true; 4], Some(3), true),
            ] {
                g.checkbox_cards.checked = checked;
                g.checkbox_cards.focus = focus;
                g.checkbox_cards.active = active;
                let widget = g.checkbox_cards_examples();
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
                    .compute_with_text(&layout, (400.0, 800.0), &mut g.fonts, scale)
                    .unwrap();
                let mut hits = Vec::new();
                collect_hit_regions(&widget, &layout, &snapshot, true, &mut hits);
                assert_eq!(hits.len(), 4);
                for (i, hit) in hits.iter().enumerate() {
                    assert_eq!(hit.message, cb(i as u8));
                    assert!(hit.rect.iter().all(|v| v.is_finite()));
                    assert!(hit.rect[2] > 0.0 && hit.rect[3] > 0.0);
                    assert!(hit.rect[0] >= 0.0 && hit.rect[0] + hit.rect[2] <= 400.01);
                }
                for (hovered, pressed) in [(None, None), (Some(1), None), (None, Some(2))] {
                    let clip = [0.0, 5.0, 400.0, 700.0];
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

// --- ROADMAP 055 P2 fold-ins for RadioCards (radio_cards_tests.rs is frozen) ---

#[test]
fn gallery_radio_cards_f6_reveals_only_when_hidden() {
    let mut g = inputs();
    let radio = [rc(0), rc(1), rc(2)];
    // Scroll the RadioCards group out of view, then F6 reveals it.
    g.scroll = g.max_scroll;
    frame(&mut g);
    assert_eq!(g.hit_is_hidden(rc(0)), Some(true), "group scrolled away");
    assert!(key(&mut g, PlatformKey::Function(6)));
    frame(&mut g);
    assert!(g.radio_cards.active);
    assert!(g.scroll < g.max_scroll, "F6 reveals a hidden group");
    for m in radio {
        assert_eq!(g.hit_is_hidden(m), Some(false), "{m:?} visible");
    }
    // Visible but off the anchor: F6 and arrows must not move the page.
    let scroll = nudge_keeping_in_view(&mut g, &radio);
    key(&mut g, PlatformKey::Function(6));
    frame(&mut g);
    assert_eq!(g.scroll, scroll, "F6 on a visible group");
    for k in [
        PlatformKey::ArrowDown,
        PlatformKey::ArrowDown,
        PlatformKey::ArrowUp,
        PlatformKey::Enter,
    ] {
        key(&mut g, k);
        frame(&mut g);
        assert_eq!(g.scroll, scroll, "{k:?} on a visible group");
    }
}

#[test]
fn gallery_radio_cards_pointer_off_card_deactivates() {
    let mut g = inputs();
    key(&mut g, PlatformKey::Function(6));
    assert!(g.radio_cards.active);
    // A press on one of its own cards keeps the group active.
    press_on(&mut g, rc(1));
    assert!(g.radio_cards.active);
    // A press elsewhere (a CheckboxCards card) deactivates it.
    press_on(&mut g, cb(0));
    assert!(!g.radio_cards.active);
    assert!(!key(&mut g, PlatformKey::ArrowDown), "keys now ignored");
    assert_eq!(g.radio_cards.selected, None);
    // Keyboard clears hover / press state (D7 #7).
    key(&mut g, PlatformKey::Function(6));
    g.pressed = Some(1);
    g.hovered = Some(1);
    assert!(key(&mut g, PlatformKey::ArrowDown));
    assert_eq!((g.pressed, g.hovered), (None, None));
}

#[test]
fn gallery_radio_cards_preedit_blocks_arrows() {
    let mut g = inputs();
    key(&mut g, PlatformKey::Function(6));
    assert!(key(&mut g, PlatformKey::ArrowDown));
    assert_eq!(g.radio_cards.selected, Some(0));
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
        g.radio_cards_event(&key_event(k));
        assert_eq!(g.radio_cards.selected, Some(0), "{k:?} blocked");
        assert_eq!(g.radio_cards.confirmed, None, "{k:?} did not confirm");
        assert!(g.radio_cards.active, "{k:?} did not leave");
    }
    g.search.input.preedit.clear();
    assert!(key(&mut g, PlatformKey::ArrowDown));
    assert_eq!(g.radio_cards.selected, Some(1));
}

// --- ROADMAP 058 review fold-ins ---

fn key_with(key: PlatformKey, ctrl: bool, alt: bool, meta: bool, repeat: bool) -> PlatformEvent {
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

/// Scrolls the page to whichever extreme hides `message`.
fn scroll_away(g: &mut Gallery, message: GalleryMessage) {
    for scroll in [0.0, g.max_scroll] {
        g.scroll = scroll;
        frame(g);
        if g.hit_is_hidden(message) == Some(true) {
            return;
        }
    }
    panic!("cannot scroll {message:?} out of view");
}

#[test]
fn gallery_checkbox_cards_space_ignores_os_auto_repeat() {
    let mut g = inputs();
    key(&mut g, PlatformKey::Function(7));
    assert!(g.event(key_with(PlatformKey::Space, false, false, false, false)));
    assert_eq!(g.checkbox_cards.checked, [true, false, false, false]);
    for _ in 0..3 {
        assert!(!g.event(key_with(PlatformKey::Space, false, false, false, true)));
        assert_eq!(g.checkbox_cards.checked, [true, false, false, false]);
    }
    assert_eq!(
        g.checkbox_cards_event(&key_with(PlatformKey::Space, false, false, false, true)),
        Some(false)
    );
    // Auto-repeat still moves focus with arrows.
    assert!(g.event(key_with(PlatformKey::ArrowDown, false, false, false, true)));
    assert_eq!(g.checkbox_cards.focus, Some(1));
}

#[test]
fn gallery_checkbox_cards_ignores_ctrl_alt_meta_chords() {
    for (ctrl, alt, meta) in [
        (true, false, false),
        (false, true, false),
        (false, false, true),
    ] {
        let mut g = inputs();
        // An F7 chord does not enter the group.
        let f7 = key_with(PlatformKey::Function(7), ctrl, alt, meta, false);
        assert_eq!(g.checkbox_cards_event(&f7), None);
        assert!(!g.checkbox_cards.active);
        assert!(!g.event(f7));
        // Active group: arrow, Space and Esc chords are all passed through.
        key(&mut g, PlatformKey::Function(7));
        for k in [
            PlatformKey::ArrowUp,
            PlatformKey::ArrowDown,
            PlatformKey::ArrowLeft,
            PlatformKey::ArrowRight,
            PlatformKey::Space,
            PlatformKey::Escape,
        ] {
            let e = key_with(k, ctrl, alt, meta, false);
            assert_eq!(g.checkbox_cards_event(&e), None, "{k:?} {ctrl}{alt}{meta}");
            assert_eq!(g.inputs_keyboard_event(&e), None);
        }
        assert_eq!(g.checkbox_cards.focus, Some(0));
        assert_eq!(g.checkbox_cards.checked, [false; 4]);
        assert!(g.checkbox_cards.active);
        // RadioCards follows the same rule.
        assert_eq!(
            g.radio_cards_event(&key_with(PlatformKey::Function(6), ctrl, alt, meta, false)),
            None
        );
        assert!(!g.radio_cards.active);
        key(&mut g, PlatformKey::Function(6));
        for k in [
            PlatformKey::ArrowDown,
            PlatformKey::Space,
            PlatformKey::Escape,
        ] {
            assert_eq!(
                g.radio_cards_event(&key_with(k, ctrl, alt, meta, false)),
                None,
                "{k:?}"
            );
        }
        assert_eq!(g.radio_cards.selected, None);
        assert!(g.radio_cards.active);
    }
}

#[test]
fn gallery_checkbox_cards_space_reveals_hidden_focus() {
    let mut g = inputs();
    key(&mut g, PlatformKey::Function(7));
    assert!(key(&mut g, PlatformKey::ArrowUp));
    assert_eq!(g.checkbox_cards.focus, Some(3));
    scroll_away(&mut g, cb(3));
    assert!(key(&mut g, PlatformKey::Space));
    frame(&mut g);
    assert_eq!(g.checkbox_cards.checked, [false, false, false, true]);
    assert_eq!(g.hit_is_hidden(cb(3)), Some(false), "toggled card revealed");
}

#[test]
fn gallery_checkbox_cards_arrow_reveals_hidden_focus() {
    let mut g = inputs();
    key(&mut g, PlatformKey::Function(7));
    // Focus 0 -> 1; scroll so that card 1 is hidden before the arrow.
    scroll_away(&mut g, cb(1));
    assert!(key(&mut g, PlatformKey::ArrowDown));
    frame(&mut g);
    assert_eq!(g.checkbox_cards.focus, Some(1));
    assert_eq!(g.hit_is_hidden(cb(1)), Some(false), "focus revealed");
    assert_eq!(g.checkbox_cards.checked, [false; 4]);
}

#[test]
fn gallery_checkbox_cards_f7_reveals_focus_in_short_viewport() {
    let mut g = Gallery::new();
    g.selected_category = 2;
    let short = |g: &mut Gallery| {
        g.frame(FrameContext {
            window: WindowId(1),
            logical_width: 1100.0,
            logical_height: 330.0,
            scale_factor: 1.0,
        })
    };
    short(&mut g);
    g.checkbox_cards.focus = Some(3); // 每週摘要
    g.scroll = 0.0;
    short(&mut g);
    assert!(key(&mut g, PlatformKey::Function(7)));
    short(&mut g);
    assert_eq!(g.checkbox_cards.focus, Some(3), "existing focus kept");
    assert_eq!(g.hit_is_hidden(cb(3)), Some(false), "focused card visible");
    assert!(
        (0..4).any(|i| g.hit_is_hidden(cb(i)) == Some(true)),
        "group is taller than the viewport"
    );
}

#[test]
fn gallery_cards_keyboard_resets_hover_and_press_everywhere() {
    fn seed(g: &mut Gallery) {
        g.pressed = Some(1);
        g.hovered = Some(1);
    }
    fn clear(g: &Gallery, what: &str) {
        assert_eq!((g.pressed, g.hovered), (None, None), "{what}");
    }
    // RadioCards: F6, then (with a selection) Enter, Space, Esc.
    let mut g = inputs();
    seed(&mut g);
    assert!(key(&mut g, PlatformKey::Function(6)));
    clear(&g, "radio F6");
    assert!(key(&mut g, PlatformKey::ArrowDown));
    for k in [PlatformKey::Enter, PlatformKey::Space, PlatformKey::Escape] {
        seed(&mut g);
        assert!(key(&mut g, k), "{k:?}");
        clear(&g, &format!("radio {k:?}"));
    }
    // CheckboxCards: F7, Space, Esc.
    let mut g = inputs();
    seed(&mut g);
    assert!(key(&mut g, PlatformKey::Function(7)));
    clear(&g, "checkbox F7");
    seed(&mut g);
    assert!(key(&mut g, PlatformKey::Space));
    clear(&g, "checkbox Space");
    seed(&mut g);
    assert!(key(&mut g, PlatformKey::Escape));
    clear(&g, "checkbox Esc");
}

#[test]
fn gallery_radio_cards_enter_space_without_selection_keeps_hover() {
    let mut g = inputs();
    key(&mut g, PlatformKey::Function(6));
    for k in [PlatformKey::Enter, PlatformKey::Space] {
        g.pressed = Some(1);
        g.hovered = Some(1);
        assert!(!key(&mut g, k), "{k:?} without selection");
        assert_eq!((g.pressed, g.hovered), (Some(1), Some(1)), "{k:?}");
        assert_eq!(g.radio_cards.confirmed, None);
    }
}

#[test]
fn gallery_checkbox_cards_pointer_press_and_focus_loss_return_none() {
    let mut g = inputs();
    key(&mut g, PlatformKey::Function(7));
    let press = PlatformEvent::PointerButton {
        window: WindowId(1),
        pressed: true,
        x: 5.0,
        y: 5.0,
        button: 0,
        pointer: 0,
        shift: false,
        ctrl: false,
        alt: false,
    };
    assert_eq!(g.checkbox_cards_event(&press), None);
    assert_eq!(g.inputs_keyboard_event(&press), None);
    assert!(!g.checkbox_cards.active, "press off the cards deactivates");
    key(&mut g, PlatformKey::Function(7));
    let lost = PlatformEvent::FocusChanged {
        window: WindowId(1),
        gained: false,
        node_id: 0,
    };
    assert_eq!(g.checkbox_cards_event(&lost), None);
    assert_eq!(g.inputs_keyboard_event(&lost), None);
    assert!(!g.checkbox_cards.active);
    assert_eq!(g.radio_cards_event(&press), None);
    assert_eq!(g.radio_cards_event(&lost), None);
}
