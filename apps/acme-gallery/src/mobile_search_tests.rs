//! Automated host-event evidence, separate from physical keyboard/IME acceptance.
use super::*;
use acme_core::DrawCommand;
use acme_platform::PlatformKey;

fn frame(gallery: &mut Gallery, scale: f32) -> acme_core::Scene {
    gallery.frame(FrameContext {
        window: WindowId(1),
        logical_width: 1100.0,
        logical_height: 700.0,
        scale_factor: scale,
    })
}

fn key(
    gallery: &mut Gallery,
    key: PlatformKey,
    text: Option<&str>,
    shift: bool,
    ctrl: bool,
) -> bool {
    gallery.event(PlatformEvent::Key {
        window: WindowId(1),
        key,
        pressed: true,
        shift,
        ctrl,
        alt: false,
        meta: false,
        repeat: false,
        text: text.map(String::from),
        character: None,
    })
}

fn click(gallery: &mut Gallery, x: f32, y: f32) {
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

fn action(gallery: &mut Gallery, message: GalleryMessage) {
    frame(gallery, 1.0);
    let hit = gallery
        .button_info
        .iter()
        .find(|h| h.message == message)
        .unwrap();
    let r = hit.rect;
    click(
        gallery,
        r[0] + r[2] / 2.0,
        r[1] - gallery.scroll + r[3] / 2.0,
    );
}

#[test]
fn pointer_edit_submit_clear_and_undo_use_the_real_gallery_event_path() {
    let mut gallery = Gallery::new();
    frame(&mut gallery, 1.0);
    let r = gallery.search.rect.expect("actual retained search layout");
    click(&mut gallery, r[0] + 12.0, r[1] + 12.0);
    assert!(gallery.search.input.focused);
    assert!(key(
        &mut gallery,
        PlatformKey::Other,
        Some("找👨‍👩‍👧‍👦é"),
        false,
        false
    ));
    key(&mut gallery, PlatformKey::Backspace, None, false, false);
    assert_eq!(gallery.search.input.text, "找👨‍👩‍👧‍👦");
    key(&mut gallery, PlatformKey::Backspace, None, false, false);
    assert_eq!(gallery.search.input.text, "找");
    action(&mut gallery, GalleryMessage::MobileSearchSubmit);
    assert_eq!(gallery.search.submitted, "找");
    action(&mut gallery, GalleryMessage::MobileSearchClear);
    assert!(gallery.search.input.text.is_empty());
    assert_eq!(gallery.search.submitted, "找", "clear must not submit");
    frame(&mut gallery, 1.0);
    let r = gallery.search.rect.unwrap();
    click(&mut gallery, r[0] + 12.0, r[1] + 12.0);
    key(&mut gallery, PlatformKey::Other, Some("z"), false, true);
    assert_eq!(gallery.search.input.text, "找", "clear remains undoable");
}

#[test]
fn keyboard_focus_visits_input_search_clear_in_both_directions() {
    let mut gallery = Gallery::new();
    frame(&mut gallery, 1.0);
    key(&mut gallery, PlatformKey::Tab, None, false, false);
    assert!(gallery.search.input.focused);
    key(
        &mut gallery,
        PlatformKey::Other,
        Some("Query"),
        false,
        false,
    );
    key(&mut gallery, PlatformKey::Enter, None, false, false);
    assert_eq!(gallery.search.submitted, "Query");
    key(&mut gallery, PlatformKey::Tab, None, false, false);
    assert!(!gallery.search.input.focused);
    let scene = frame(&mut gallery, 1.0);
    assert!(
        scene
            .commands()
            .iter()
            .any(|c| matches!(c, DrawCommand::Quad(q) if q.border_width == 2.0))
    );
    key(&mut gallery, PlatformKey::Tab, None, false, false);
    key(&mut gallery, PlatformKey::Space, None, false, false);
    assert!(gallery.search.input.text.is_empty());
    key(&mut gallery, PlatformKey::Tab, None, true, false); // Clear -> Search
    key(&mut gallery, PlatformKey::Tab, None, true, false); // Search -> Input
    assert!(gallery.search.input.focused);
    key(&mut gallery, PlatformKey::Tab, None, true, false); // Input -> outside
    assert!(!gallery.search.input.focused);
    key(&mut gallery, PlatformKey::Tab, None, true, false); // outside -> Clear
    key(&mut gallery, PlatformKey::Tab, None, true, false); // Clear -> Search
    key(&mut gallery, PlatformKey::Tab, None, true, false); // Search -> Input
    assert!(gallery.search.input.focused);
}

#[test]
fn preedit_blocks_submission_and_category_changes_blur_and_reject_late_commits() {
    let mut gallery = Gallery::new();
    frame(&mut gallery, 1.0);
    key(&mut gallery, PlatformKey::Tab, None, false, false);
    gallery.event(PlatformEvent::ImePreedit {
        window: WindowId(1),
        text: "搜尋".into(),
        cursor: None,
    });
    key(&mut gallery, PlatformKey::Enter, None, false, false);
    assert_eq!(gallery.search.submitted, "—");
    assert!(!gallery.search.input.preedit.is_empty());
    key(&mut gallery, PlatformKey::Backspace, None, false, false);
    assert_eq!(gallery.search.input.preedit, "搜尋");
    assert!(gallery.search.input.text.is_empty());
    gallery.event(PlatformEvent::ImeCommit {
        window: WindowId(1),
        text: "搜尋".into(),
    });
    assert_eq!(gallery.search.input.text, "搜尋");
    key(&mut gallery, PlatformKey::Enter, None, false, false);
    assert_eq!(gallery.search.submitted, "搜尋");
    let index = gallery
        .button_info
        .iter()
        .position(|h| h.message == GalleryMessage::SelectCategory(0))
        .unwrap();
    gallery.activate(index);
    assert!(!gallery.search.input.focused);
    gallery.event(PlatformEvent::ImeCommit {
        window: WindowId(1),
        text: "late".into(),
    });
    assert_eq!(gallery.search.input.text, "搜尋");
    assert!(gallery.ime_cursor_area(WindowId(1)).is_none());
}

#[test]
fn long_query_scrolls_and_paints_inside_viewport_across_themes_and_scales() {
    let mut gallery = Gallery::new();
    for pack in ["default", "apple"] {
        gallery.theme_pack = pack.into();
        for dark in [false, true] {
            gallery.dark = dark;
            for scale in [1.0, 1.25, 2.0] {
                gallery.scroll = 0.0;
                frame(&mut gallery, scale);
                if !gallery.search.input.focused {
                    key(&mut gallery, PlatformKey::Tab, None, false, false);
                }
                gallery.search.input.select_all();
                acme_textinput::handle_text(&mut gallery.search.input, &"LongQuery搜尋".repeat(50));
                let scene = frame(&mut gallery, scale);
                assert!(scene.validate().is_ok());
                assert!(gallery.search.input.scroll_offset > 0.0);
                let r = gallery.search.rect.unwrap();
                let caret = gallery.ime_cursor_area(WindowId(1)).unwrap();
                assert!(caret[0] >= r[0] && caret[0] + caret[2] <= r[0] + r[2] + 1.0);
                let mut clips = Vec::new();
                let mut input_text_seen = false;
                for command in scene.commands() {
                    match command {
                        DrawCommand::PushClip(clip) => clips.push(*clip),
                        DrawCommand::PopClip => {
                            clips.pop();
                        }
                        DrawCommand::Text(_) if clips.len() >= 2 => {
                            let inner = clips.last().unwrap();
                            let outer = clips[clips.len() - 2];
                            if inner.origin.x.get() >= r[0]
                                && inner.origin.x.get() < r[0] + r[2]
                                && inner.size.width.get() <= r[2]
                            {
                                assert_eq!(outer.origin.y.get(), TOOLBAR_HEIGHT);
                                input_text_seen = true;
                            }
                        }
                        _ => {}
                    }
                }
                assert!(
                    input_text_seen,
                    "input paint must inherit an outer viewport clip"
                );
                gallery.scroll = gallery.max_scroll;
                frame(&mut gallery, scale);
                assert!(
                    gallery.ime_cursor_area(WindowId(1)).is_none(),
                    "offscreen field has no candidate anchor"
                );
                click(&mut gallery, r[0] + 12.0, 10.0); // toolbar must not focus a scrolled field
                assert!(!gallery.search.input.focused);
            }
        }
    }
}

#[test]
fn tab_reveals_an_offscreen_search_field() {
    let mut gallery = Gallery::new();
    frame(&mut gallery, 1.0);
    gallery.scroll = gallery.max_scroll;
    frame(&mut gallery, 1.0);
    key(&mut gallery, PlatformKey::Tab, None, false, false);
    frame(&mut gallery, 1.0);
    let rect = gallery.search.rect.unwrap();
    assert!(rect[1] - gallery.scroll >= TOOLBAR_HEIGHT);
    assert!(gallery.ime_cursor_area(WindowId(1)).is_some());
}

#[test]
fn search_action_labels_are_centered_inside_their_touch_targets() {
    let mut gallery = Gallery::new();
    for dark in [false, true] {
        gallery.dark = dark;
        let theme = gallery.current_theme();
        for scale in [1.0, 1.25, 2.0] {
            let widget = button("clear", "Clear")
                .w(Length::px(72.0))
                .min_h(Length::px(44.0))
                .on_click(GalleryMessage::MobileSearchClear);
            let context = WidgetLayoutContext {
                body_font_size: 14.0,
                body_line_height: 20.0,
                label_font_size: 13.0,
                control_height: 36.0,
                scale_factor: scale,
            };
            let layout = widget.to_layout_with_context(NodeId::new(1), &context);
            let snapshot = gallery
                .layout
                .compute_with_text(&layout, (240.0, 100.0), &mut gallery.fonts, scale)
                .unwrap();
            let rect = snapshot.get(layout.id).unwrap();
            let mut frame = Frame::default();
            render_content(
                &mut frame,
                &widget,
                &layout,
                &snapshot,
                &theme,
                scale,
                0.0,
                [0.0, 0.0, 240.0, 100.0],
                &mut 0,
                None,
                None,
                &mut gallery.fonts,
                &mut gallery.atlas,
            );
            let run = &frame.text[0];
            assert!(!run.prepared.glyphs.is_empty());
            for glyph in &run.prepared.glyphs {
                let top = run.origin[1] + glyph.y as f32 / scale;
                let bottom = top + glyph.height as f32 / scale;
                assert!(
                    top >= rect.y + 8.0,
                    "action text touches the top border: {top}"
                );
                assert!(bottom <= rect.y + rect.height - 6.0);
            }
        }
    }
}
