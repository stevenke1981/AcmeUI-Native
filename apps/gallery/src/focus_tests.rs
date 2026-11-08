//! ROADMAP 004 F1 evidence: keyboard focus survives page changes and never
//! indexes past the current hit regions.
//!
//! Keys go through the real `Application::event` path, with `frame()` between
//! events as the runtime does.

use super::*;

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

fn press(gallery: &mut Gallery, key: PlatformKey, shift: bool) -> bool {
    gallery.event(PlatformEvent::Key {
        window: WindowId(0),
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

/// Page with the fewest (non-zero) hit regions, and that count.
fn smallest_page(gallery: &mut Gallery) -> ((usize, usize), usize) {
    let mut best: Option<((usize, usize), usize)> = None;
    for (category, info) in CATEGORIES.iter().enumerate() {
        for page in 0..info.pages.len() {
            go_to(gallery, (category, page));
            frame_once(gallery);
            let count = gallery.button_info.len();
            if count > 0 && best.is_none_or(|(_, least)| count < least) {
                best = Some(((category, page), count));
            }
        }
    }
    best.expect("at least one page has a hit region")
}

/// Tab (forward) until the focused region carries `message`, frame between keys.
fn tab_to(gallery: &mut Gallery, message: GalleryMessage) {
    frame_once(gallery);
    for _ in 0..=gallery.button_info.len() {
        if gallery
            .button_info
            .get(gallery.focused)
            .is_some_and(|region| region.message == message)
        {
            return;
        }
        press(gallery, PlatformKey::Tab, false);
        frame_once(gallery);
    }
    panic!("no hit region carries {message:?}");
}

#[test]
fn sidebar_focus_stays_on_its_button_when_the_category_changes() {
    let mut gallery = Gallery::new();
    tab_to(&mut gallery, GalleryMessage::SelectCategory(4)); // Data
    let before = gallery.focused_target.expect("focus has a target");

    // Enter on the focused sidebar button switches category.
    assert!(press(&mut gallery, PlatformKey::Enter, false));
    assert_eq!(gallery.selected_category, 4);
    frame_once(&mut gallery);

    let region = gallery.button_info[gallery.focused];
    assert_eq!(region.message, GalleryMessage::SelectCategory(4));
    assert_eq!(region.node, before.node, "same sidebar node stays focused");
    assert_eq!(gallery.focused_target, Some(before));
}

#[test]
fn tab_and_shift_tab_wrap_around_a_page() {
    let mut gallery = Gallery::new();
    go_to(&mut gallery, (1, 0)); // Inputs / Button
    frame_once(&mut gallery);
    let count = gallery.button_info.len();
    assert!(count > 1, "page needs several hit regions");
    assert_eq!(gallery.focused, 0);

    // Shift+Tab from the first region wraps to the last.
    press(&mut gallery, PlatformKey::Tab, true);
    frame_once(&mut gallery);
    assert_eq!(gallery.focused, count - 1);
    assert_eq!(
        gallery.focused_target,
        Some(FocusTarget::of(&gallery.button_info[count - 1]))
    );

    // Tab from the last region wraps to the first.
    press(&mut gallery, PlatformKey::Tab, false);
    frame_once(&mut gallery);
    assert_eq!(gallery.focused, 0);

    // Forward through every region visits each index once, in order.
    for expected in 1..count {
        press(&mut gallery, PlatformKey::Tab, false);
        frame_once(&mut gallery);
        assert_eq!(gallery.focused, expected);
    }
}

#[test]
fn accessibility_focus_follows_keyboard_focus() {
    let mut gallery = Gallery::new();
    go_to(&mut gallery, (0, 0));
    frame_once(&mut gallery);
    for _ in 0..3 {
        press(&mut gallery, PlatformKey::Tab, false);
        frame_once(&mut gallery);
        let region = gallery.button_info[gallery.focused];
        assert!(
            !is_synthetic(region.node),
            "sidebar regions are retained, so focus must be exported"
        );
        let update = gallery
            .accessibility
            .build_tree_update()
            .expect("tree built by frame");
        assert_eq!(update.focus, acme_accessibility::to_access_id(region.node));
    }
}

#[test]
fn space_activates_the_focused_region_like_enter() {
    let mut gallery = Gallery::new();
    tab_to(&mut gallery, GalleryMessage::SelectCategory(2)); // Navigation
    assert!(press(&mut gallery, PlatformKey::Space, false));
    assert_eq!(gallery.selected_category, 2);
}

#[test]
fn enter_after_switching_to_a_smaller_page_does_not_panic() {
    let mut gallery = Gallery::new();
    go_to(&mut gallery, (7, 0)); // Stress Tests / 1000 Labels
    frame_once(&mut gallery);
    let large_count = gallery.button_info.len();
    assert!(large_count > 1, "large page needs several hit regions");

    // Move keyboard focus to the last region of the large page.
    for _ in 1..large_count {
        press(&mut gallery, PlatformKey::Tab, false);
        frame_once(&mut gallery);
    }
    assert_eq!(gallery.focused, large_count - 1);

    let (small, small_count) = smallest_page(&mut gallery);
    assert!(
        small_count < large_count,
        "need a smaller page to switch to"
    );
    go_to(&mut gallery, small);
    frame_once(&mut gallery);

    // Enter on the smaller page must not index past its hit regions.
    let handled = press(&mut gallery, PlatformKey::Enter, false);
    assert!(
        handled,
        "Enter on the smaller page should activate a region"
    );
    assert!(
        gallery.focused < gallery.button_info.len(),
        "focus index {} is outside {} regions",
        gallery.focused,
        gallery.button_info.len()
    );
}

#[test]
fn enter_with_a_stale_focus_index_and_no_regions_is_ignored() {
    // No frame has run, so there are no hit regions at all.
    let mut gallery = Gallery::new();
    assert!(gallery.button_info.is_empty());
    gallery.focused = 5;
    assert!(!press(&mut gallery, PlatformKey::Enter, false));
    assert!(!press(&mut gallery, PlatformKey::Space, false));
}

/// Every hit region's node that keyboard focus would report must exist in the
/// accessibility tree; AccessKit panics on a focus ID outside its node list.
fn assert_reported_focus_exists(gallery: &mut Gallery) {
    for index in 0..gallery.button_info.len() {
        gallery.focused = index;
        gallery.focused_target = None;
        frame_once(gallery);
        let update = gallery
            .accessibility
            .build_tree_update()
            .expect("tree update");
        assert!(
            update.nodes.iter().any(|(id, _)| *id == update.focus),
            "focus {:?} not in the tree for region {index} on page ({}, {})",
            update.focus,
            gallery.selected_category,
            gallery.selected_page
        );
    }
}

#[test]
fn reported_accessibility_focus_is_always_a_tree_node() {
    let mut gallery = Gallery::new();
    for (category, info) in CATEGORIES.iter().enumerate() {
        for page in 0..info.pages.len() {
            go_to(&mut gallery, (category, page));
            frame_once(&mut gallery);
            assert_reported_focus_exists(&mut gallery);
        }
    }
}
