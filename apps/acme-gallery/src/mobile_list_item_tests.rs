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

fn key(gallery: &mut Gallery, key: acme_platform::PlatformKey, shift: bool) -> bool {
    gallery.event(PlatformEvent::Key {
        window: WindowId(1),
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
fn keyboard_scrolled_page_text_retains_the_viewport_clip_in_the_actual_scene() {
    let mut gallery = Gallery::new();
    frame(&mut gallery);
    key(&mut gallery, acme_platform::PlatformKey::Function(6), false);
    key(&mut gallery, acme_platform::PlatformKey::ArrowDown, false);
    let scene = gallery.frame(FrameContext {
        window: WindowId(1),
        logical_width: 1100.0,
        logical_height: 1100.0,
        scale_factor: 1.0,
    });
    let (layout, snapshot) = gallery.last_frame_layout.as_ref().unwrap();
    let r = snapshot
        .get(extract_gallery_ids(layout).scroll_view)
        .unwrap();
    let expected = [r.x, r.y, r.width, r.height];
    assert_eq!(r.y, TOOLBAR_HEIGHT);
    let mut clips = Vec::new();
    let mut checked = 0;
    for command in scene.commands() {
        match command {
            acme_core::DrawCommand::PushClip(r) => clips.push(*r),
            acme_core::DrawCommand::PopClip => {
                clips.pop().unwrap();
            }
            acme_core::DrawCommand::Text(t)
                if t.origin.x.get() >= SIDEBAR_WIDTH && t.origin.y.get() < 0.0 =>
            {
                assert!(
                    fab_tests::clip_within(
                        clips.last().map(|c| [
                            c.origin.x.get(),
                            c.origin.y.get(),
                            c.size.width.get(),
                            c.size.height.get()
                        ]),
                        expected
                    ),
                    "offscreen content must not escape the scroll viewport"
                );
                checked += 1;
            }
            _ => {}
        }
    }
    assert!(checked > 0);
    assert!(clips.is_empty());
}
#[test]
fn list_rows_dispatch_pointer_taps_and_keep_identity_after_detail_changes_and_reordering() {
    let mut gallery = Gallery::new();
    frame(&mut gallery);
    let ids = |gallery: &Gallery| {
        let widget = gallery.description();
        let (layout, _) = gallery.last_frame_layout.as_ref().unwrap();
        ["gallery_list_wifi", "gallery_list_notifications"].map(|key| {
            let (_, l) = find(&widget, layout, key).unwrap();
            (l.id, l.children[0].id, l.children[0].children[0].id)
        })
    };
    let saved = ids(&gallery);
    for message in [
        GalleryMessage::MobileListItemTap(0),
        GalleryMessage::MobileListItemTap(1),
    ] {
        activate(&mut gallery, message);
        let GalleryMessage::MobileListItemTap(index) = message else {
            unreachable!()
        };
        assert_eq!(gallery.last_list_tap, Some(index));
    }
    activate(&mut gallery, GalleryMessage::MobileListItemDetails);
    assert!(!gallery.list_details);
    activate(&mut gallery, GalleryMessage::MobileListItemReverse);
    assert!(gallery.list_reverse);
    frame(&mut gallery);
    assert_eq!(ids(&gallery), saved);
    assert_eq!(
        gallery
            .button_info
            .iter()
            .filter(|h| matches!(h.message, GalleryMessage::MobileListItemTap(_)))
            .count(),
        2
    );
    activate(&mut gallery, GalleryMessage::MobileListItemTap(0));
    assert_eq!(gallery.last_list_tap, Some(0));
    frame(&mut gallery);
    let widget = gallery.description();
    let (layout, snapshot) = gallery.last_frame_layout.as_ref().unwrap();
    let (_, passive) = find(&widget, layout, "gallery_list_passive").unwrap();
    let r = *snapshot.get(passive.id).unwrap();
    gallery.scroll = (r.y - 100.0).max(0.0).min(gallery.max_scroll);
    frame(&mut gallery);
    let (x, y) = (r.x + r.width / 2.0, r.y - gallery.scroll + r.height / 2.0);
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
    assert_eq!(
        gallery.last_list_tap,
        Some(0),
        "passive row must not open another item"
    );
    assert_eq!(
        bind_retained(
            std::slice::from_ref(&gallery.description()),
            &gallery.retained.tree
        )
        .mismatches(),
        0
    );
}
#[test]
fn gallery_list_keyboard_navigation_activation_reorder_and_composition_guard_use_real_events() {
    use acme_platform::PlatformKey::*;
    let mut gallery = Gallery::new();
    frame(&mut gallery);
    assert!(key(&mut gallery, Function(6), false));
    assert_eq!(gallery.list_focus, Some(0));
    assert!(!gallery.search.input.focused);
    frame(&mut gallery);
    assert!(key(&mut gallery, Enter, false));
    assert_eq!(gallery.last_list_tap, Some(0));
    key(&mut gallery, ArrowDown, false);
    frame(&mut gallery);
    assert!(key(&mut gallery, Space, false));
    assert_eq!(gallery.last_list_tap, Some(1));
    key(&mut gallery, Tab, true);
    assert_eq!(gallery.list_focus, Some(0));
    key(&mut gallery, Tab, false);
    assert_eq!(gallery.list_focus, Some(1));
    key(&mut gallery, Tab, false);
    assert_eq!(gallery.list_focus, None);
    activate(&mut gallery, GalleryMessage::MobileListItemReverse);
    frame(&mut gallery);
    key(&mut gallery, Function(6), false);
    assert_eq!(gallery.list_focus, Some(1));
    frame(&mut gallery);
    key(&mut gallery, Enter, false);
    assert_eq!(gallery.last_list_tap, Some(1));
    key(&mut gallery, Escape, false);
    assert_eq!(gallery.list_focus, None);
    key(&mut gallery, Tab, false);
    assert!(gallery.search.input.focused);
    gallery.event(PlatformEvent::ImePreedit {
        window: WindowId(1),
        text: "注".into(),
        cursor: None,
    });
    assert!(!key(&mut gallery, Function(6), false));
    assert_eq!(gallery.list_focus, None);
    assert_eq!(gallery.search.input.preedit, "注");
}
#[test]
fn list_row_hits_and_paint_indices_preserve_following_buttons_theme_and_scroll_clips() {
    let mut gallery = Gallery::new();
    for pack in ["default", "apple"] {
        gallery.theme_pack = pack.into();
        for dark in [false, true] {
            gallery.dark = dark;
            let theme = gallery.current_theme();
            for scale in [1.0, 1.25, 2.0] {
                let widget = column()
                    .width(320.0)
                    .child(mobile_list_item("Passive", None::<&str>))
                    .child(
                        mobile_list_item("Wi-Fi · 無線網路", Some("Connected · 已連線"))
                            .key("active")
                            .trailing_icon(acme_ui::IconName::ChevronRight)
                            .on_tap(GalleryMessage::MobileListItemTap(0)),
                    )
                    .child(button("after", "After").on_click(GalleryMessage::DpiInfo))
                    .build();
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
                    .compute_with_text(&layout, (320.0, 1000.0), &mut gallery.fonts, scale)
                    .unwrap();
                let mut hits = Vec::new();
                collect_hit_regions(&widget, &layout, &snapshot, true, &mut hits);
                assert_eq!(hits.len(), 2);
                assert_eq!(hits[0].message, GalleryMessage::MobileListItemTap(0));
                assert_eq!(hits[1].message, GalleryMessage::DpiInfo);
                assert_eq!(hits[0].rect[2], 320.0);
                for (hovered, pressed) in [(None, None), (Some(1), None), (None, Some(0))] {
                    let mut frame = Frame::default();
                    let clip = [8.0, 8.0, 300.0, 600.0];
                    let mut index = 0;
                    render_content(
                        &mut frame,
                        &widget,
                        &layout,
                        &snapshot,
                        &theme,
                        scale,
                        5.0,
                        clip,
                        &mut index,
                        hovered,
                        pressed,
                        &mut gallery.fonts,
                        &mut gallery.atlas,
                    );
                    assert_eq!(index, 2);
                    assert_eq!(frame.clipped_quads.len(), 3);
                    assert!(frame.clipped_quads.iter().all(|q| q.clip == clip));
                    assert_eq!(
                        frame.clipped_quads[0].quad.color,
                        rgba(theme.colors.surface)
                    );
                    assert_eq!(
                        frame.clipped_quads[1].quad.color,
                        rgba(if pressed == Some(0) {
                            theme.colors.ghost_pressed
                        } else {
                            theme.colors.surface
                        })
                    );
                    let WidgetNode::Button(after) = &widget.children()[2] else {
                        panic!("after")
                    };
                    assert_eq!(
                        frame.clipped_quads[2].quad.color,
                        rgba(
                            after
                                .resolve_style(
                                    &theme,
                                    ButtonState {
                                        hovered: hovered == Some(1),
                                        ..Default::default()
                                    }
                                )
                                .background
                        )
                    );
                    assert!(
                        frame
                            .text
                            .iter()
                            .all(|t| fab_tests::clip_within(t.clip, clip)
                                && !t.prepared.glyphs.is_empty())
                    );
                }
            }
        }
    }
    let search: WidgetNode<GalleryMessage> = mobile_search("Search")
        .on_submit(GalleryMessage::MobileSearchSubmit)
        .into();
    assert!(
        list_examples::row_action(&search).is_none(),
        "the search host owns input/submit routing"
    );
}
