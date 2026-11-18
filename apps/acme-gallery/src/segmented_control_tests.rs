use super::*;
use acme_platform::PlatformKey;

fn frame(gallery: &mut Gallery) {
    gallery.frame(FrameContext {
        window: WindowId(1),
        logical_width: 1100.0,
        logical_height: 760.0,
        scale_factor: 1.0,
    });
}

fn key(gallery: &mut Gallery, key: PlatformKey, ctrl: bool) -> bool {
    gallery.event(PlatformEvent::Key {
        window: WindowId(1),
        key,
        pressed: true,
        shift: false,
        ctrl,
        alt: false,
        meta: false,
        repeat: false,
        text: None,
        character: None,
    })
}

fn inputs() -> Gallery {
    let mut gallery = Gallery::new();
    gallery.selected_category = 2;
    frame(&mut gallery);
    gallery
}

#[test]
fn gallery_segmented_control_click_updates_period_and_omits_disabled_hit() {
    let mut gallery = inputs();
    let hits: Vec<_> = gallery
        .button_info
        .iter()
        .filter(|hit| matches!(hit.message, GalleryMessage::SegmentSelect(_)))
        .collect();
    assert_eq!(hits.len(), 2);
    assert_eq!(hits[0].message, GalleryMessage::SegmentSelect(0));
    assert_eq!(hits[1].message, GalleryMessage::SegmentSelect(2));
    let [x, y, width, height] = hits[1].rect;
    let (x, y) = (x + width / 2.0, y + height / 2.0 - gallery.scroll);
    gallery.event(PlatformEvent::PointerMoved {
        window: WindowId(1),
        x,
        y,
    });
    for pressed in [true, false] {
        assert!(gallery.event(PlatformEvent::PointerButton {
            window: WindowId(1),
            pressed,
            x,
            y,
            button: 0,
            pointer: 0,
            shift: false,
            ctrl: false,
            alt: false,
        }));
    }
    assert_eq!(gallery.segments.selected, 2);
    assert!(gallery.segments.active);
    assert!(gallery.segmented_control_example().children().iter().any(
        |child| matches!(child, WidgetNode::Label(label) if label.text == "報表期間：本月 · 已完成 37 件")
    ));
    assert!(!gallery.select_segment(1));
    assert!(!gallery.select_segment(usize::MAX));
    assert_eq!(gallery.segments.selected, 2);
    gallery.scroll = gallery.max_scroll;
    frame(&mut gallery);
    assert!(key(&mut gallery, PlatformKey::Function(8), false));
    assert_eq!(
        gallery.hit_is_hidden(GalleryMessage::SegmentSelect(2)),
        Some(false)
    );
}

#[test]
fn gallery_segmented_control_keys_wrap_and_remain_page_and_focus_scoped() {
    let mut gallery = inputs();
    assert!(!key(&mut gallery, PlatformKey::ArrowRight, false));
    assert!(key(&mut gallery, PlatformKey::Function(8), false));
    assert!(gallery.segments.active);
    assert!(!key(&mut gallery, PlatformKey::ArrowRight, true));
    assert_eq!(gallery.segments.selected, 0);
    assert!(key(&mut gallery, PlatformKey::ArrowRight, false));
    assert_eq!(gallery.segments.selected, 2);
    assert!(key(&mut gallery, PlatformKey::ArrowRight, false));
    assert_eq!(gallery.segments.selected, 0);
    assert!(key(&mut gallery, PlatformKey::ArrowLeft, false));
    assert_eq!(gallery.segments.selected, 2);
    assert!(key(&mut gallery, PlatformKey::Tab, false));
    assert!(!gallery.segments.active);
    assert!(!key(&mut gallery, PlatformKey::ArrowLeft, false));
    assert_eq!(gallery.segments.selected, 2);

    assert!(key(&mut gallery, PlatformKey::Function(6), false));
    assert!(gallery.radio_cards.active);
    assert!(key(&mut gallery, PlatformKey::Function(8), false));
    assert!(gallery.segments.active);
    assert!(!gallery.radio_cards.active);
    assert!(key(&mut gallery, PlatformKey::Function(7), false));
    assert!(gallery.checkbox_cards.active);
    assert!(!gallery.segments.active);
    assert!(key(&mut gallery, PlatformKey::Function(8), false));
    assert!(!gallery.checkbox_cards.active);
    assert!(gallery.event(PlatformEvent::FocusChanged {
        window: WindowId(1),
        gained: false,
        node_id: 0,
    }));
    assert!(!gallery.segments.active);

    gallery.selected_category = 0;
    assert!(!key(&mut gallery, PlatformKey::Function(8), false));
    assert!(!gallery.segments.active);
}
