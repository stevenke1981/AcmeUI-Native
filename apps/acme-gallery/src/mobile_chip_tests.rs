//! Automated chip acceptance; no physical touch/screen-reader claims.
use super::*;

fn frame(gallery: &mut Gallery) {
    gallery.frame(FrameContext {
        window: WindowId(1),
        logical_width: 1100.0,
        logical_height: 700.0,
        scale_factor: 1.0,
    });
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

fn activate(gallery: &mut Gallery, message: GalleryMessage) {
    frame(gallery);
    let r = gallery
        .button_info
        .iter()
        .find(|h| h.message == message)
        .expect("action hit")
        .rect;
    gallery.scroll = (r[1] - 80.0).max(0.0).min(gallery.max_scroll);
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

#[test]
fn chip_tap_dismiss_restore_and_filter_use_actual_pointer_dispatch() {
    let mut gallery = Gallery::new();
    activate(&mut gallery, GalleryMessage::MobileChipTap);
    assert_eq!(gallery.chip_action, "tag tapped");
    assert!(gallery.chip_visible);
    activate(&mut gallery, GalleryMessage::MobileChipToggle);
    assert!(!gallery.chip_selected);
    activate(&mut gallery, GalleryMessage::MobileChipDismiss);
    assert!(!gallery.chip_visible);
    assert_eq!(
        gallery.chip_action, "tag dismissed",
        "dismiss must not also tap"
    );
    frame(&mut gallery);
    assert!(!gallery.button_info.iter().any(|h| matches!(
        h.message,
        GalleryMessage::MobileChipTap | GalleryMessage::MobileChipDismiss
    )));
    activate(&mut gallery, GalleryMessage::MobileChipRestore);
    assert!(gallery.chip_visible);
    frame(&mut gallery);
    assert!(
        gallery
            .button_info
            .iter()
            .any(|h| h.message == GalleryMessage::MobileChipDismiss)
    );
}

#[test]
fn chip_examples_have_complete_binding_and_filter_identity_survives_other_chip_removal() {
    let mut gallery = Gallery::new();
    let mut ids = None;
    for message in [
        None,
        Some(GalleryMessage::MobileChipToggle),
        Some(GalleryMessage::MobileChipDismiss),
        Some(GalleryMessage::MobileChipRestore),
    ] {
        if let Some(message) = message {
            activate(&mut gallery, message);
        }
        frame(&mut gallery);
        let widget = gallery.description();
        let (layout, _) = gallery.last_frame_layout.as_ref().unwrap();
        let (filter, filter_layout) = find(&widget, layout, "gallery_chip_filter").unwrap();
        let current = (filter_layout.id, filter_layout.children[0].id);
        if let Some(ids) = ids {
            assert_eq!(current, ids);
        } else {
            ids = Some(current);
        }
        let WidgetNode::Button(body) = &filter.children()[0] else {
            panic!("filter action")
        };
        assert_eq!(body.label.starts_with('✓'), gallery.chip_selected);
        let binding = bind_retained(std::slice::from_ref(&widget), &gallery.retained.tree);
        assert_eq!(binding.mismatches(), 0);
        let (static_tag, _) = find(&widget, layout, "gallery_chip_static").unwrap();
        assert!(matches!(static_tag.children()[0], WidgetNode::Label(_)));
    }
}

#[test]
fn chip_surface_precedes_actions_and_keeps_theme_and_scroll_clips() {
    let mut gallery = Gallery::new();
    for pack in ["default", "apple"] {
        gallery.theme_pack = pack.into();
        for dark in [false, true] {
            gallery.dark = dark;
            let theme = gallery.current_theme();
            for selected in [false, true] {
                for scale in [1.0, 1.25, 2.0] {
                    let widget = WidgetNode::from(
                        mobile_chip("Tag · 標籤")
                            .selected(selected)
                            .on_tap(GalleryMessage::MobileChipTap)
                            .on_dismiss(GalleryMessage::MobileChipDismiss),
                    );
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
                        .compute_with_text(&layout, (320.0, 100.0), &mut gallery.fonts, scale)
                        .unwrap();
                    let mut frame = Frame::default();
                    let clip = [10.0, 10.0, 300.0, 70.0];
                    render_content(
                        &mut frame,
                        &widget,
                        &layout,
                        &snapshot,
                        &theme,
                        scale,
                        12.0,
                        clip,
                        &mut 0,
                        None,
                        None,
                        &mut gallery.fonts,
                        &mut gallery.atlas,
                    );
                    assert_eq!(frame.clipped_quads.len(), 3);
                    let root = &frame.clipped_quads[0];
                    assert_eq!(
                        root.quad.color,
                        rgba(if selected {
                            theme.colors.accent
                        } else {
                            theme.colors.muted
                        })
                    );
                    assert!(root.quad.radius > 0.0);
                    assert!(frame.clipped_quads.iter().all(|q| q.clip == clip));
                    assert_eq!(frame.text.len(), 2);
                    assert!(
                        frame
                            .text
                            .iter()
                            .all(|t| t.clip == Some(clip) && !t.prepared.glyphs.is_empty())
                    );
                }
            }
        }
    }
}
