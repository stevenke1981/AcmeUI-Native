//! Layer 1 — Hit testing: pure function backed by [`acme_core::HitTestLayer`].
//!
//! Given hit-region data and current cursor/scroll state, returns the
//! top-most hit-region index under the cursor, or `None`.

#[cfg(test)]
use acme_core::Point;

#[cfg(test)]
use crate::render::build_hit_layer;
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

/// Reverse-iterate hit regions via [`acme_core::HitTestLayer`] to find the
/// top-most match under cursor.
///
/// * `button_info` — hit regions collected during the last frame.
/// * `cursor` — current pointer coordinates `(x, y)`.
/// * `scroll` — current content scroll offset.
/// * `scroll_clip_rect` — window-space clip rect of the scroll viewport.
#[cfg(test)]
pub fn hit_test(
    button_info: &[HitRegion],
    cursor: (f32, f32),
    scroll: f32,
    scroll_clip_rect: [f32; 4],
) -> Option<usize> {
    build_hit_layer(button_info, scroll, scroll_clip_rect)
        .hit_test(Point::new(cursor.0, cursor.1))
        .map(|hit| hit.tag as usize)
}
