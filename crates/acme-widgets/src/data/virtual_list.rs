use crate::WidgetNode;
use acme_core::WidgetKey;

// ============================================================================
// VariableHeightCache — tracks per-item heights for variable-height lists
// ============================================================================

/// Tracks measured/per-item heights for variable-height virtual lists.
/// When an item's height is unknown, `default_height` is used.
#[derive(Clone, Debug)]
pub struct VariableHeightCache {
    heights: Vec<f32>,
    default_height: f32,
    total: f32,
}

impl VariableHeightCache {
    pub fn new(default_height: f32) -> Self {
        Self {
            heights: Vec::new(),
            default_height,
            total: 0.0,
        }
    }

    /// Get the height for item at `index`. Returns `default_height` if unknown.
    pub fn get(&self, index: usize) -> f32 {
        self.heights
            .get(index)
            .copied()
            .unwrap_or(self.default_height)
    }

    /// Set the height for item at `index`. Extends the internal vec if needed.
    pub fn set(&mut self, index: usize, height: f32) {
        let h = if height.is_finite() && height >= 0.0 {
            height
        } else {
            self.default_height
        };
        if index >= self.heights.len() {
            let added = index + 1 - self.heights.len();
            self.heights.resize(index + 1, self.default_height);
            self.total += added as f32 * self.default_height;
        }
        let old = self.heights[index];
        self.heights[index] = h;
        self.total = self.total + h - old;
    }

    /// Total estimated content height.
    pub fn total(&self) -> f32 {
        self.total.max(0.0)
    }

    /// Total number of tracked items.
    pub fn len(&self) -> usize {
        self.heights.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heights.is_empty()
    }

    /// Reset the cache.
    pub fn clear(&mut self) {
        self.heights.clear();
        self.total = 0.0;
    }
}

// ============================================================================
// VirtualList
// ============================================================================

/// A virtual scrolling list that only lays out visible children.
///
/// - Supply **all** children up front; the list computes which are visible
///   based on `scroll_offset` / `viewport_height` and only creates layout
///   nodes for the visible range (+ overscan).
/// - When `item_height` is `Some(fixed_height)`, visible-range arithmetic uses
///   that constant.  When `None`, a `VariableHeightCache` is required.
/// - Scroll anchoring keeps the first visible item stable when items are added
///   or removed above the current viewport.
pub struct VirtualList<M> {
    pub key: WidgetKey,
    /// All children of the list.  Only the visible subset enters the layout tree.
    pub children: Vec<WidgetNode<M>>,
    /// Fixed item height.  `None` enables variable-height mode (requires cache).
    pub item_height: Option<f32>,
    /// Number of extra items to render above/below the visible window.
    pub overscan: usize,
    /// Current scroll offset in logical pixels.
    pub scroll_offset: f32,
    /// Viewport height in logical pixels.
    pub viewport_height: f32,
    /// Variable height cache (only used when `item_height` is `None`).
    pub height_cache: Option<VariableHeightCache>,
    // — Scroll anchoring state —
    /// Index of the first visible item used as an anchor during rebuild.
    pub anchor_item: usize,
    /// Pixel offset of the anchor item's top relative to the viewport top.
    pub anchor_offset: f32,
}

// Manual impls for Clone / Debug / PartialEq that skip height_cache and
// anchoring state in equality checks.

impl<M: Clone> std::clone::Clone for VirtualList<M> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            children: self.children.clone(),
            item_height: self.item_height,
            overscan: self.overscan,
            scroll_offset: self.scroll_offset,
            viewport_height: self.viewport_height,
            height_cache: self.height_cache.clone(),
            anchor_item: self.anchor_item,
            anchor_offset: self.anchor_offset,
        }
    }
}

impl<M: std::fmt::Debug> std::fmt::Debug for VirtualList<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualList")
            .field("key", &self.key)
            .field("children", &self.children)
            .field("item_height", &self.item_height)
            .field("overscan", &self.overscan)
            .field("scroll_offset", &self.scroll_offset)
            .field("viewport_height", &self.viewport_height)
            .field("height_cache", &self.height_cache)
            .field("anchor_item", &self.anchor_item)
            .field("anchor_offset", &self.anchor_offset)
            .finish()
    }
}

impl<M: PartialEq> PartialEq for VirtualList<M> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.children == other.children
            && self.item_height == other.item_height
            && self.overscan == other.overscan
            && self.scroll_offset == other.scroll_offset
            && self.viewport_height == other.viewport_height
    }
}

// ── Builder / factory ───────────────────────────────────────────────────────

/// Create a new `VirtualList` builder with fixed item height.
pub fn virtual_list<M>(key: impl Into<WidgetKey>) -> VirtualList<M> {
    VirtualList {
        key: key.into(),
        children: Vec::new(),
        item_height: None,
        overscan: 3,
        scroll_offset: 0.0,
        viewport_height: 0.0,
        height_cache: None,
        anchor_item: 0,
        anchor_offset: 0.0,
    }
}

// ── Fluent methods ───────────────────────────────────────────────────────────

impl<M> VirtualList<M> {
    /// Add a child to the list.
    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Set fixed item height (fast path).  Pass `None` for variable heights.
    pub fn item_height(mut self, value: Option<f32>) -> Self {
        self.item_height = value.map(crate::finite);
        self
    }

    /// Set overscan count.
    pub fn overscan(mut self, value: usize) -> Self {
        self.overscan = value;
        self
    }

    /// Set viewport height.
    pub fn viewport_height(mut self, value: f32) -> Self {
        self.viewport_height = crate::finite(value);
        self
    }

    /// Set scroll offset.
    pub fn scroll_offset(mut self, value: f32) -> Self {
        self.scroll_offset = value.max(0.0);
        self
    }

    /// Set the variable-height cache.
    pub fn height_cache(mut self, cache: VariableHeightCache) -> Self {
        self.height_cache = Some(cache);
        self
    }

    /// Build into a `WidgetNode`.
    pub fn build(self) -> WidgetNode<M> {
        WidgetNode::VirtualList(self)
    }
}

// ── Core methods ─────────────────────────────────────────────────────────────

impl<M> VirtualList<M> {
    /// Calculate the range of visible item indices.
    ///
    /// Returns `(first_visible, one_past_last_visible)`.
    /// When `item_height` is `Some(fixed)`, uses fixed arithmetic.
    /// When `None`, uses `height_cache` for per-item heights (falls back to
    /// `default_height` from the cache, or 32 px if no cache exists).
    pub fn visible_range(&self) -> (usize, usize) {
        if self.children.is_empty() || self.viewport_height <= 0.0 {
            return (0, 0);
        }

        let default_h = self
            .height_cache
            .as_ref()
            .map(|c| c.default_height)
            .unwrap_or(32.0);

        match self.item_height {
            Some(fixed) if fixed > 0.0 => {
                let first = (self.scroll_offset / fixed).floor() as usize;
                let count = (self.viewport_height / fixed).ceil() as usize + self.overscan * 2;
                let end = (first + count).min(self.children.len());
                (first, end)
            }
            _ => {
                // Variable height: walk from top to find first visible item,
                // then collect items until past the viewport.
                let n = self.children.len();
                if n == 0 {
                    return (0, 0);
                }
                let mut y = 0.0_f32;
                let mut first: usize = 0;
                let cache = self.height_cache.as_ref();
                while first < n {
                    let h = cache.map(|c| c.get(first)).unwrap_or(default_h);
                    if y + h > self.scroll_offset {
                        break;
                    }
                    y += h;
                    first += 1;
                }
                let overscan_first = first.saturating_sub(self.overscan);

                let mut last = first;
                let mut vy = y;
                while last < n && vy < self.scroll_offset + self.viewport_height {
                    let h = cache.map(|c| c.get(last)).unwrap_or(default_h);
                    vy += h;
                    last += 1;
                }
                let overscan_last = (last + self.overscan).min(n);

                (overscan_first, overscan_last)
            }
        }
    }

    /// Total content height (estimated).
    pub fn content_height(&self) -> f32 {
        match self.item_height {
            Some(fixed) => self.children.len() as f32 * fixed,
            None => self.height_cache.as_ref().map_or_else(
                || self.children.len() as f32 * 32.0,
                |c| {
                    // Use cached total if all items have been measured
                    if c.len() >= self.children.len() {
                        c.total()
                    } else {
                        // Mix of cached and default
                        let default_h = c.default_height;
                        let total = c.total();
                        let uncached = self.children.len() - c.len();
                        total + uncached as f32 * default_h
                    }
                },
            ),
        }
    }

    /// Scroll to bring `index` into view.
    pub fn scroll_to_item(&mut self, index: usize) {
        if index >= self.children.len() {
            return;
        }
        let (_first, _) = self.visible_range();
        let item_top = self.item_offset(index);
        let item_bottom = item_top + self.item_height_at(index);

        if item_top < self.scroll_offset {
            self.scroll_offset = item_top;
        } else if item_bottom > self.scroll_offset + self.viewport_height {
            self.scroll_offset = item_bottom - self.viewport_height;
        }
    }

    /// Pixel offset of item `index` from the top of the content.
    pub fn item_offset(&self, index: usize) -> f32 {
        let n = self.children.len();
        if index >= n {
            return self.content_height();
        }
        match self.item_height {
            Some(fixed) => index as f32 * fixed,
            None => {
                let default_h = self
                    .height_cache
                    .as_ref()
                    .map(|c| c.default_height)
                    .unwrap_or(32.0);
                let cache = match self.height_cache.as_ref() {
                    Some(c) => c,
                    None => return index as f32 * default_h,
                };
                let mut y = 0.0_f32;
                for i in 0..index.min(n) {
                    y += cache.get(i);
                }
                y
            }
        }
    }

    /// Height of item at `index`.
    pub fn item_height_at(&self, index: usize) -> f32 {
        match self.item_height {
            Some(fixed) => fixed,
            None => self
                .height_cache
                .as_ref()
                .map(|c| c.get(index))
                .unwrap_or(32.0),
        }
    }

    // ── Scroll anchoring ─────────────────────────────────────────────────

    /// Save the current scroll anchor state (call after layout, before rebuild).
    pub fn save_anchor(&mut self) {
        let (first, _) = self.visible_range();
        if first < self.children.len() {
            self.anchor_item = first;
            self.anchor_offset = self.item_offset(first) - self.scroll_offset;
        }
    }

    /// Restore scroll offset from the saved anchor.
    /// Call after items above the anchor may have been inserted/removed.
    pub fn restore_anchor(&mut self) {
        if self.anchor_item >= self.children.len() {
            // Anchor item was removed; clamp to end
            self.anchor_item = self.children.len().saturating_sub(1);
            self.anchor_offset = 0.0;
        }
        if !self.children.is_empty() {
            let new_top = self.item_offset(self.anchor_item);
            self.scroll_offset = (new_top - self.anchor_offset).max(0.0);
        }
    }

    /// Clear scroll anchoring.
    pub fn clear_anchor(&mut self) {
        self.anchor_item = 0;
        self.anchor_offset = 0.0;
    }
}

// ── From conversion ──────────────────────────────────────────────────────────

impl<M> From<VirtualList<M>> for WidgetNode<M> {
    fn from(value: VirtualList<M>) -> Self {
        WidgetNode::VirtualList(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item<M>(i: usize) -> WidgetNode<M> {
        crate::label(format!("Item {i}"))
    }

    fn msg_list() -> VirtualList<crate::label::Label> {
        // Use Label as the message type for testing
        type M = crate::label::Label;
        let mut list = virtual_list::<M>("test");
        for i in 0..50 {
            list = list.child(item::<M>(i));
        }
        list.viewport_height(200.0).item_height(Some(40.0))
    }

    #[test]
    fn visible_range_at_start() {
        let list = msg_list().scroll_offset(0.0);
        let (first, last) = list.visible_range();
        // fixed 40px, viewport 200px → 5 visible + 6 overscan (3 each side) = up to 11
        assert_eq!(first, 0);
        assert!((5..=11).contains(&last));
    }

    #[test]
    fn visible_range_scrolled() {
        let list = msg_list().scroll_offset(100.0);
        let (first, last) = list.visible_range();
        // scroll_offset=100 → item 2 at top; 5 visible + overscan
        assert!(first <= 2);
        assert!(last > first);
    }

    #[test]
    fn visible_range_at_end() {
        let list = msg_list().scroll_offset(1920.0);
        let (first, last) = list.visible_range();
        assert!(first >= 45);
        assert_eq!(last, 50);
    }

    #[test]
    fn visible_range_empty() {
        let list = virtual_list::<()>("empty").viewport_height(200.0);
        let (first, last) = list.visible_range();
        assert_eq!(first, 0);
        assert_eq!(last, 0);
    }

    #[test]
    fn content_height_fixed() {
        let list = msg_list();
        assert!((list.content_height() - 2000.0).abs() < f32::EPSILON);
    }

    #[test]
    fn content_height_empty() {
        let list = virtual_list::<()>("e").viewport_height(200.0);
        assert!((list.content_height() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn scroll_to_item_brings_it_into_view() {
        let mut list = msg_list().scroll_offset(0.0);
        list.scroll_to_item(45);
        assert!(list.scroll_offset > 0.0);
        let (first, _) = list.visible_range();
        assert!(first <= 45);
    }

    #[test]
    fn item_offset_fixed() {
        let list = msg_list();
        assert!((list.item_offset(0) - 0.0).abs() < f32::EPSILON);
        assert!((list.item_offset(1) - 40.0).abs() < f32::EPSILON);
        assert!((list.item_offset(5) - 200.0).abs() < f32::EPSILON);
    }

    // ── VariableHeightCache ──────────────────────────────────────────────

    #[test]
    fn variable_cache_default_get() {
        let cache = VariableHeightCache::new(48.0);
        assert!((cache.get(0) - 48.0).abs() < f32::EPSILON);
        assert!((cache.get(100) - 48.0).abs() < f32::EPSILON);
    }

    #[test]
    fn variable_cache_set_and_get() {
        let mut cache = VariableHeightCache::new(48.0);
        cache.set(0, 60.0);
        cache.set(5, 30.0);
        assert!((cache.get(0) - 60.0).abs() < f32::EPSILON);
        assert!((cache.get(1) - 48.0).abs() < f32::EPSILON);
        assert!((cache.get(5) - 30.0).abs() < f32::EPSILON);
    }

    #[test]
    fn variable_cache_total() {
        let mut cache = VariableHeightCache::new(48.0);
        cache.set(0, 60.0);
        cache.set(1, 40.0);
        assert!((cache.total() - 100.0).abs() < f32::EPSILON);
    }

    #[test]
    fn variable_cache_clear() {
        let mut cache = VariableHeightCache::new(48.0);
        cache.set(0, 60.0);
        cache.clear();
        assert!((cache.total() - 0.0).abs() < f32::EPSILON);
        assert!(cache.is_empty());
    }

    #[test]
    fn variable_cache_len() {
        let mut cache = VariableHeightCache::new(48.0);
        assert_eq!(cache.len(), 0);
        cache.set(2, 50.0);
        // resize to 3 (indices 0,1,2)
        assert_eq!(cache.len(), 3);
    }

    // ── Scroll anchoring ─────────────────────────────────────────────────

    #[test]
    fn save_and_restore_anchor() {
        let mut list = msg_list().scroll_offset(80.0);
        list.save_anchor();
        let _saved_off = list.anchor_offset;
        // Simulate removing items above anchor
        list.children.drain(0..2);
        // Adjust anchor_item to reflect removed items above it.
        // Anchor was at index 2 (the 3rd item); after removing 2 items,
        // that same item is now at index 0, so scroll_offset becomes 0.
        list.anchor_item = list.anchor_item.saturating_sub(2);
        list.restore_anchor();
        assert!((list.scroll_offset).abs() < f32::EPSILON);
    }

    #[test]
    fn clear_anchor_resets() {
        let mut list = msg_list();
        list.save_anchor();
        assert!(list.anchor_item < 50 || list.children.is_empty());
        list.clear_anchor();
        assert_eq!(list.anchor_item, 0);
        assert_eq!(list.anchor_offset, 0.0);
    }

    // ── Variable height visible range ────────────────────────────────────

    #[test]
    fn visible_range_variable_height() {
        let mut cache = VariableHeightCache::new(40.0);
        cache.set(0, 80.0); // item 0 is 80px
        cache.set(1, 20.0); // item 1 is 20px
        cache.set(2, 60.0); // item 2 is 60px
        let mut list = virtual_list::<()>("var");
        for i in 0..10 {
            list = list.child(crate::label(format!("Item {i}")));
        }
        list = list.viewport_height(100.0).height_cache(cache);
        // Without scroll, start at 0
        let (first, _last) = list.visible_range();
        assert_eq!(first, 0);
    }
}
