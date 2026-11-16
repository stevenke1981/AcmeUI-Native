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

fn inputs() -> Gallery {
    let mut g = Gallery::new();
    g.selected_category = 2;
    frame(&mut g);
    g
}

#[test]
fn gallery_radio_cards_pointer_selects_each_option_with_real_events() {
    let mut g = inputs();
    assert_eq!(g.radio_cards.selected, None);
    for index in [1u8, 0, 2, 1] {
        click(&mut g, GalleryMessage::RadioCardSelect(index));
        assert_eq!(g.radio_cards.selected, Some(index));
        assert_eq!(g.radio_cards.confirmed, Some(index));
        assert!(g.radio_cards.active);
    }
    assert_eq!(
        g.button_info
            .iter()
            .filter(|h| matches!(h.message, GalleryMessage::RadioCardSelect(_)))
            .count(),
        3
    );
}

#[test]
fn gallery_radio_cards_f6_arrow_keys_wrap_and_enter_space_confirm() {
    let mut g = inputs();
    assert!(key(&mut g, PlatformKey::Function(6)));
    assert!(g.radio_cards.active);
    assert_eq!(g.radio_cards.selected, None);
    // No selection: forward starts at the first option.
    assert!(key(&mut g, PlatformKey::ArrowDown));
    assert_eq!(g.radio_cards.selected, Some(0));
    for expected in [1, 2, 0] {
        assert!(key(&mut g, PlatformKey::ArrowRight));
        assert_eq!(g.radio_cards.selected, Some(expected));
    }
    assert!(key(&mut g, PlatformKey::ArrowUp));
    assert_eq!(g.radio_cards.selected, Some(2));
    assert!(key(&mut g, PlatformKey::ArrowLeft));
    assert_eq!(g.radio_cards.selected, Some(1));
    assert_eq!(
        g.radio_cards.confirmed, None,
        "arrows move without confirming"
    );
    assert!(key(&mut g, PlatformKey::Enter));
    assert_eq!(g.radio_cards.confirmed, Some(1));
    key(&mut g, PlatformKey::ArrowDown);
    assert!(key(&mut g, PlatformKey::Space));
    assert_eq!(g.radio_cards.confirmed, Some(2));
    // No selection: backward starts at the last option.
    g.radio_cards.selected = None;
    assert!(key(&mut g, PlatformKey::ArrowUp));
    assert_eq!(g.radio_cards.selected, Some(2));
}

#[test]
fn gallery_radio_cards_keys_ignored_off_page_and_after_escape_or_blur() {
    let mut g = Gallery::new();
    g.selected_category = 0;
    frame(&mut g);
    key(&mut g, PlatformKey::Function(6));
    assert!(!g.radio_cards.active, "F6 off the Inputs page is ignored");
    assert!(!key(&mut g, PlatformKey::ArrowDown));
    assert_eq!(g.radio_cards.selected, None);

    let mut g = inputs();
    assert!(
        !key(&mut g, PlatformKey::ArrowDown),
        "inactive group ignores keys"
    );
    assert_eq!(g.radio_cards.selected, None);
    key(&mut g, PlatformKey::Function(6));
    key(&mut g, PlatformKey::ArrowDown);
    assert_eq!(g.radio_cards.selected, Some(0));
    assert!(key(&mut g, PlatformKey::Escape));
    assert!(!g.radio_cards.active);
    assert!(!key(&mut g, PlatformKey::ArrowDown));
    assert_eq!(g.radio_cards.selected, Some(0));

    key(&mut g, PlatformKey::Function(6));
    g.event(PlatformEvent::FocusChanged {
        window: WindowId(1),
        gained: false,
        node_id: 0,
    });
    assert!(!g.radio_cards.active);
    assert!(!key(&mut g, PlatformKey::ArrowDown));
    assert_eq!(g.radio_cards.selected, Some(0));

    // Leaving the page deactivates the group.
    key(&mut g, PlatformKey::Function(6));
    assert!(g.radio_cards.active);
    let category = g
        .button_info
        .iter()
        .position(|h| h.message == GalleryMessage::SelectCategory(0))
        .unwrap();
    assert!(g.activate(category));
    assert!(!g.radio_cards.active);

    // The IME guard keeps F6 from stealing a composition.
    let mut g = inputs();
    g.search.input.preedit = "注".into();
    assert!(!key(&mut g, PlatformKey::Function(6)));
    assert!(!g.radio_cards.active);
}

#[test]
fn gallery_radio_cards_hit_and_paint_slots_match_across_themes_and_scales() {
    let mut g = Gallery::new();
    for dark in [false, true] {
        g.dark = dark;
        let theme = g.current_theme();
        for scale in [1.0, 1.25, 2.0] {
            for selected in [None, Some(0u8), Some(2)] {
                g.radio_cards.selected = selected;
                let widget = g.radio_cards_examples();
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
                assert_eq!(hits.len(), 3);
                for (i, hit) in hits.iter().enumerate() {
                    assert_eq!(hit.message, GalleryMessage::RadioCardSelect(i as u8));
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
