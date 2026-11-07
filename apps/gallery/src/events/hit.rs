//! Layer 1 — Hit testing: pure function, no mutation.
//!
//! Given hit-region data and current cursor/scroll state, returns the
//! top-most hit-region index under the cursor, or `None`.

use crate::render::{point_in_rect, scrolled_hit_rect};
use crate::types::HitRegion;

/// Whether two region lists name the same widgets in the same order.
///
/// Only identity (node and action) is compared, not geometry: a layout move
/// keeps a press valid, while a rebuilt list with different widgets does not.
pub fn same_hit_list(before: &[HitRegion], after: &[HitRegion]) -> bool {
    before.len() == after.len()
        && before
            .iter()
            .zip(after)
            .all(|(a, b)| a.node == b.node && a.message == b.message)
}

/// Reverse-iterate hit regions to find the top-most match under cursor.
///
/// * `button_info` — hit regions collected during the last frame.
/// * `cursor` — current pointer coordinates `(x, y)`.
/// * `scroll` — current content scroll offset.
/// * `scroll_clip_rect` — window-space clip rect of the scroll viewport.
pub fn hit_test(
    button_info: &[HitRegion],
    cursor: (f32, f32),
    scroll: f32,
    scroll_clip_rect: [f32; 4],
) -> Option<usize> {
    button_info.iter().enumerate().rev().find_map(|(i, hr)| {
        let r = if hr.scrolled {
            if scroll_clip_rect[2] > 0.0 && !point_in_rect(cursor.0, cursor.1, scroll_clip_rect) {
                return None;
            }
            scrolled_hit_rect(hr.rect, scroll)
        } else {
            hr.rect
        };
        if point_in_rect(cursor.0, cursor.1, r) {
            Some(i)
        } else {
            None
        }
    })
}
