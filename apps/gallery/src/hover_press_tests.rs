//! ROADMAP 008 P2 evidence: pointer hover and press never outlive the hit
//! regions they were computed on.
//!
//! `hovered` and `pressed` are indices into `button_info`. A page switch (or a
//! rebuilt region list) changes what index `i` means, so a stale index would
//! highlight, or activate, a different widget. These tests drive the real
//! `Application::event` path with `frame()` between events, as the runtime does.

use acme_platform::PlatformEvent;
use acme_platform::PlatformKey;

use super::*;
use crate::events::hit_test;
use crate::render::scrolled_hit_rect;

/// Inputs / Button: the page the pointer starts on. Its index 11 is a
/// `DpiInfo` button.
const PAGE_BEFORE: (usize, usize) = (1, 0);
/// Navigation / first page: its index 11 is a `NavRailSelect` button, so the
/// same index names a different widget. Keyboard switching reaches a
/// category's first page, so this is the only page the tests switch to.
const PAGE_AFTER: (usize, usize) = (2, 0);
const AFTER_CATEGORY: usize = 2;

fn go_to(gallery: &mut Gallery, (category, page): (usize, usize)) {
    gallery.selected_category = category;
    gallery.selected_page = page;
}

fn frame_once(gallery: &mut Gallery) {
    let _ = gallery.frame(FrameContext {
        window: WindowId(0),
        logical_width: 1100.0,
        logical_height: 760.0,
        scale_factor: 1.0,
    });
}

fn key_press(gallery: &mut Gallery, key: PlatformKey) -> bool {
    gallery.event(PlatformEvent::Key {
        window: WindowId(0),
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

fn move_pointer(gallery: &mut Gallery, (x, y): (f32, f32)) {
    gallery.event(PlatformEvent::PointerMoved {
        window: WindowId(0),
        x,
        y,
    });
}

fn pointer_button(gallery: &mut Gallery, (x, y): (f32, f32), pressed: bool) -> bool {
    gallery.event(PlatformEvent::PointerButton {
        window: WindowId(0),
        pressed,
        x,
        y,
        button: 0,
        pointer: 0,
        shift: false,
        ctrl: false,
        alt: false,
    })
}

/// Tab to the sidebar button for `category` and press Enter, as a keyboard
/// user would. Each key is followed by a frame.
fn switch_category_by_keyboard(gallery: &mut Gallery, category: usize) {
    frame_once(gallery);
    let target = GalleryMessage::SelectCategory(category);
    for _ in 0..=gallery.button_info.len() {
        if gallery
            .button_info
            .get(gallery.focused)
            .is_some_and(|region| region.message == target)
        {
            break;
        }
        key_press(gallery, PlatformKey::Tab);
        frame_once(gallery);
    }
    assert!(
        gallery
            .button_info
            .get(gallery.focused)
            .is_some_and(|region| region.message == target),
        "sidebar button for category {category} is not reachable by Tab"
    );
    assert!(key_press(gallery, PlatformKey::Enter));
    frame_once(gallery);
}

/// Pointer position at the centre of a region's on-screen rect.
fn centre(rect: [f32; 4]) -> (f32, f32) {
    (rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0)
}

/// A region the pointer can hit on the current frame, at its centre.
#[derive(Clone, Copy, Debug)]
struct Spot {
    index: usize,
    message: GalleryMessage,
    at: (f32, f32),
}

/// Regions of the current frame whose centre hit-tests back to themselves.
fn visible_spots(gallery: &Gallery) -> Vec<Spot> {
    gallery
        .button_info
        .iter()
        .enumerate()
        .filter_map(|(index, region)| {
            let rect = if region.scrolled {
                scrolled_hit_rect(region.rect, gallery.scroll)
            } else {
                region.rect
            };
            let at = centre(rect);
            let hit = hit_test(
                &gallery.button_info,
                at,
                gallery.scroll,
                gallery.scroll_clip_rect,
            );
            (hit == Some(index)).then_some(Spot {
                index,
                message: region.message,
                at,
            })
        })
        .collect()
}

/// A spot on the start page and a spot on the after page that share an index
/// but are different widgets. An index that survives a page switch therefore
/// names the wrong widget.
fn stale_index_pair(gallery: &mut Gallery) -> (Spot, Spot) {
    go_to(gallery, PAGE_AFTER);
    frame_once(gallery);
    let after = visible_spots(gallery);
    go_to(gallery, PAGE_BEFORE);
    frame_once(gallery);
    let before = visible_spots(gallery);
    before
        .iter()
        .find_map(|start| {
            after
                .iter()
                .find(|end| end.index == start.index && end.message != start.message)
                .map(|end| (*start, *end))
        })
        .expect("need a visible index that names different widgets on the two pages")
}

#[test]
fn release_after_a_page_switch_does_not_activate_the_new_page() {
    let mut gallery = Gallery::new();
    let (pressed_on, released_on) = stale_index_pair(&mut gallery);

    go_to(&mut gallery, PAGE_BEFORE);
    frame_once(&mut gallery);
    move_pointer(&mut gallery, pressed_on.at);
    assert!(pointer_button(&mut gallery, pressed_on.at, true));
    assert_eq!(gallery.pressed, Some(pressed_on.index));

    // The page changes while the button is held (keyboard), then the pointer
    // is released over a different widget that shares the stale index.
    switch_category_by_keyboard(&mut gallery, AFTER_CATEGORY);
    assert_eq!(
        (gallery.selected_category, gallery.selected_page),
        PAGE_AFTER
    );
    move_pointer(&mut gallery, released_on.at);
    let handled = pointer_button(&mut gallery, released_on.at, false);
    assert!(
        !handled,
        "release over {:?} activated a widget the press never started on ({:?})",
        released_on.message, pressed_on.message
    );
    assert_eq!(
        (gallery.selected_category, gallery.selected_page),
        PAGE_AFTER,
        "release must not change the page"
    );
}

#[test]
fn hover_after_a_page_switch_is_the_region_under_the_pointer() {
    let mut gallery = Gallery::new();
    let (hovered_on, _) = stale_index_pair(&mut gallery);

    go_to(&mut gallery, PAGE_BEFORE);
    frame_once(&mut gallery);
    move_pointer(&mut gallery, hovered_on.at);
    assert_eq!(gallery.hovered, Some(hovered_on.index));

    // The pointer stays put; the page under it changes.
    switch_category_by_keyboard(&mut gallery, AFTER_CATEGORY);
    let under_pointer = hit_test(
        &gallery.button_info,
        gallery.cursor,
        gallery.scroll,
        gallery.scroll_clip_rect,
    );
    assert_eq!(
        gallery.hovered, under_pointer,
        "hover must name the region under the pointer on the current page"
    );
}

#[test]
fn press_and_release_on_the_same_region_across_a_frame_still_activates() {
    let mut gallery = Gallery::new();
    go_to(&mut gallery, PAGE_BEFORE);
    frame_once(&mut gallery);
    // The toolbar theme toggle is on every page and has an observable effect.
    let toggle = gallery
        .button_info
        .iter()
        .find(|region| region.message == GalleryMessage::ToggleTheme)
        .expect("toolbar theme toggle");
    let at = centre(toggle.rect);
    let dark_before = gallery.dark;

    move_pointer(&mut gallery, at);
    assert!(pointer_button(&mut gallery, at, true));
    assert!(gallery.pressed.is_some());
    frame_once(&mut gallery);
    assert!(pointer_button(&mut gallery, at, false));
    assert_ne!(
        gallery.dark, dark_before,
        "release must activate the toggle"
    );
}
