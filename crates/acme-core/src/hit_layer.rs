//! Spatial hit-testing layer, Z-ordering, scroll clipping, and click/drag detection.

use crate::{HitTestEntry, HitTestPath, Local, Logical, NodeId, Point, Rect};

/// Semantic Z-layer buckets for hit-testing priority.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(i32)]
pub enum HitZLayer {
    /// Standard document / widget flow.
    #[default]
    Base = 0,
    /// Popovers, dropdown menus, floating toolbars.
    Floating = 10,
    /// Non-modal overlays and banners.
    Overlay = 20,
    /// Modal dialogs and scrims.
    Modal = 30,
    /// Tooltips above modal surfaces.
    Tooltip = 40,
    /// Developer / inspector overlays.
    Debug = 50,
}

impl HitZLayer {
    /// Numeric Z-priority corresponding to this semantic layer.
    pub const fn z_value(&self) -> i32 {
        *self as i32
    }
}

/// Controls how a hit region participates in hit testing and occlusion.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum HitBehavior {
    /// Receives hits and blocks hits to regions behind it (even when disabled).
    #[default]
    Opaque,
    /// Receives hits when enabled; allows hits to pass behind if disabled or in `hit_test_all`.
    Translucent,
    /// Ignored completely by hit testing.
    Transparent,
}

/// Semantic pointer cursor hint associated with a hit region.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CursorHint {
    #[default]
    Default,
    Pointer,
    Text,
    Grab,
    Grabbing,
    ResizeEW,
    ResizeNS,
    NotAllowed,
}

/// Geometric shape of a hit-testable region in logical coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HitShape {
    /// Axis-aligned half-open rectangle.
    Rect(Rect<Logical>),
    /// Axis-aligned rectangle with uniform corner radius.
    RoundedRect { rect: Rect<Logical>, radius: f32 },
    /// Closed circular disk centered at `center` with `radius`.
    Circle { center: Point<Logical>, radius: f32 },
}

impl HitShape {
    /// Return `true` if `point` lies inside this shape.
    pub fn contains(&self, point: Point<Logical>) -> bool {
        match *self {
            Self::Rect(rect) => rect.contains(point),
            Self::RoundedRect { rect, radius } => {
                if !rect.contains(point) {
                    return false;
                }
                let w = rect.size.width.get();
                let h = rect.size.height.get();
                let r = sanitize_non_neg(radius).min(w * 0.5).min(h * 0.5);
                if r <= 0.0 {
                    return true;
                }
                let left = rect.origin.x.get();
                let top = rect.origin.y.get();
                let right = left + w;
                let bottom = top + h;
                let inner_left = left + r;
                let inner_right = right - r;
                let inner_top = top + r;
                let inner_bottom = bottom - r;

                let px = point.x.get();
                let py = point.y.get();
                let dx = if px < inner_left {
                    inner_left - px
                } else if px > inner_right {
                    px - inner_right
                } else {
                    0.0
                };
                let dy = if py < inner_top {
                    inner_top - py
                } else if py > inner_bottom {
                    py - inner_bottom
                } else {
                    0.0
                };
                dx.hypot(dy) <= r
            }
            Self::Circle { center, radius } => {
                let r = sanitize_non_neg(radius);
                if r <= 0.0 {
                    return false;
                }
                let dx = point.x.get() - center.x.get();
                let dy = point.y.get() - center.y.get();
                dx.hypot(dy) <= r
            }
        }
    }

    /// Return the axis-aligned bounding rectangle of this shape.
    pub fn bounding_rect(&self) -> Rect<Logical> {
        match *self {
            Self::Rect(rect) | Self::RoundedRect { rect, .. } => rect,
            Self::Circle { center, radius } => {
                let r = sanitize_non_neg(radius);
                if r <= 0.0 {
                    Rect::new(center.x.get(), center.y.get(), 0.0, 0.0)
                } else {
                    Rect::new(center.x.get() - r, center.y.get() - r, r * 2.0, r * 2.0)
                }
            }
        }
    }

    fn translated(self, (dx, dy): (f32, f32)) -> Self {
        match self {
            Self::Rect(rect) => Self::Rect(translate_rect(rect, (dx, dy))),
            Self::RoundedRect { rect, radius } => Self::RoundedRect {
                rect: translate_rect(rect, (dx, dy)),
                radius: sanitize_non_neg(radius),
            },
            Self::Circle { center, radius } => Self::Circle {
                center: Point::new(
                    sanitize_finite(center.x.get() + dx),
                    sanitize_finite(center.y.get() + dy),
                ),
                radius: sanitize_non_neg(radius),
            },
        }
    }
}

/// Builder specification for pushing a hit region into [`HitTestLayer`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HitRegionSpec {
    pub id: NodeId,
    pub shape: HitShape,
    pub enabled: bool,
    pub z_layer: i32,
    pub behavior: HitBehavior,
    pub cursor: CursorHint,
    pub tag: u64,
}

impl HitRegionSpec {
    /// Create an enabled opaque rectangular hit region at base Z-offset `0`.
    pub fn rect(id: NodeId, rect: Rect<Logical>) -> Self {
        Self::with_shape(id, HitShape::Rect(rect))
    }

    /// Create an enabled opaque rounded-rectangle hit region.
    pub fn rounded_rect(id: NodeId, rect: Rect<Logical>, radius: f32) -> Self {
        Self::with_shape(
            id,
            HitShape::RoundedRect {
                rect,
                radius: sanitize_non_neg(radius),
            },
        )
    }

    /// Create an enabled opaque circular hit region.
    pub fn circle(id: NodeId, center: Point<Logical>, radius: f32) -> Self {
        Self::with_shape(
            id,
            HitShape::Circle {
                center,
                radius: sanitize_non_neg(radius),
            },
        )
    }

    fn with_shape(id: NodeId, shape: HitShape) -> Self {
        Self {
            id,
            shape,
            enabled: true,
            z_layer: HitZLayer::Base.z_value(),
            behavior: HitBehavior::Opaque,
            cursor: CursorHint::Default,
            tag: 0,
        }
    }

    /// Set whether this region is interactive (`true`) or disabled (`false`).
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Set the numeric Z-layer / Z-offset for this region.
    pub fn z_layer(mut self, z_layer: i32) -> Self {
        self.z_layer = z_layer;
        self
    }

    /// Set the Z-layer from a semantic [`HitZLayer`].
    pub fn layer(mut self, layer: HitZLayer) -> Self {
        self.z_layer = layer.z_value();
        self
    }

    /// Set the hit-test occlusion [`HitBehavior`].
    pub fn behavior(mut self, behavior: HitBehavior) -> Self {
        self.behavior = behavior;
        self
    }

    /// Set the pointer [`CursorHint`] shown when hovering this region.
    pub fn cursor(mut self, cursor: CursorHint) -> Self {
        self.cursor = cursor;
        self
    }

    /// Attach an application-defined tag (e.g., handle or sub-part identifier).
    pub fn tag(mut self, tag: u64) -> Self {
        self.tag = tag;
        self
    }
}

/// Detailed hit-test match produced by [`HitTestLayer`].
#[derive(Clone, Debug, PartialEq)]
pub struct HitResult {
    pub id: NodeId,
    pub path: HitTestPath,
    pub bounds: Rect<Logical>,
    pub local_point: Point<Local>,
    pub z_layer: i32,
    pub cursor: CursorHint,
    pub tag: u64,
    pub enabled: bool,
}

#[derive(Clone, Debug)]
struct RegionEntry {
    id: NodeId,
    path: HitTestPath,
    shape: HitShape,
    clip: Option<Rect<Logical>>,
    enabled: bool,
    z_layer: i32,
    order: usize,
    behavior: HitBehavior,
    cursor: CursorHint,
    tag: u64,
}

impl RegionEntry {
    fn contains(&self, point: Point<Logical>) -> bool {
        if self.behavior == HitBehavior::Transparent {
            return false;
        }
        if let Some(clip) = self.clip
            && !clip.contains(point)
        {
            return false;
        }
        self.shape.contains(point)
    }

    fn to_result(&self, point: Point<Logical>) -> HitResult {
        let bounds = self.shape.bounding_rect();
        let local_point = Point::new(
            point.x.get() - bounds.origin.x.get(),
            point.y.get() - bounds.origin.y.get(),
        );
        HitResult {
            id: self.id,
            path: self.path.clone(),
            bounds,
            local_point,
            z_layer: self.z_layer,
            cursor: self.cursor,
            tag: self.tag,
            enabled: self.enabled,
        }
    }
}

/// Hierarchical hit-testing layer built during layout/paint with scoped offsets,
/// clips, scroll viewports, node ancestor paths, and Z-layer priorities.
#[derive(Clone, Debug, Default)]
pub struct HitTestLayer {
    entries: Vec<RegionEntry>,
    node_stack: Vec<NodeId>,
    offset_stack: Vec<(f32, f32)>,
    clip_stack: Vec<Rect<Logical>>,
    z_stack: Vec<i32>,
}

impl HitTestLayer {
    /// Create an empty [`HitTestLayer`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of recorded hit regions (including transparent or clipped entries).
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Return `true` if no hit regions have been pushed.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Clear all recorded regions and reset all scoped stacks.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.node_stack.clear();
        self.offset_stack.clear();
        self.clip_stack.clear();
        self.z_stack.clear();
    }

    // -----------------------------------------------------------------------
    // Scoped stacks
    // -----------------------------------------------------------------------

    /// Push an ancestor [`NodeId`] onto the hit-test path stack.
    pub fn push_node(&mut self, id: NodeId) {
        self.node_stack.push(id);
    }

    /// Pop the most recent [`NodeId`] from the path stack, if any.
    pub fn pop_node(&mut self) {
        self.node_stack.pop();
    }

    /// Execute `f` with `id` pushed onto the ancestor path stack.
    pub fn with_node<R>(&mut self, id: NodeId, f: impl FnOnce(&mut Self) -> R) -> R {
        struct NodeGuard<'a> {
            layer: &'a mut HitTestLayer,
        }
        impl Drop for NodeGuard<'_> {
            fn drop(&mut self) {
                self.layer.pop_node();
            }
        }
        self.push_node(id);
        let guard = NodeGuard { layer: self };
        f(guard.layer)
    }

    /// Push a relative translation offset `(dx, dy)` in logical pixels.
    pub fn push_offset(&mut self, dx: f32, dy: f32) {
        let (cx, cy) = self.current_offset();
        let nx = sanitize_finite(cx + sanitize_finite(dx));
        let ny = sanitize_finite(cy + sanitize_finite(dy));
        self.offset_stack.push((nx, ny));
    }

    /// Pop the most recent translation offset, if any.
    pub fn pop_offset(&mut self) {
        self.offset_stack.pop();
    }

    /// Return the current cumulative translation offset `(x, y)`.
    pub fn current_offset(&self) -> (f32, f32) {
        self.offset_stack.last().copied().unwrap_or((0.0, 0.0))
    }

    /// Execute `f` within a relative translation offset `(dx, dy)`.
    pub fn with_offset<R>(&mut self, dx: f32, dy: f32, f: impl FnOnce(&mut Self) -> R) -> R {
        struct OffsetGuard<'a> {
            layer: &'a mut HitTestLayer,
        }
        impl Drop for OffsetGuard<'_> {
            fn drop(&mut self) {
                self.layer.pop_offset();
            }
        }
        self.push_offset(dx, dy);
        let guard = OffsetGuard { layer: self };
        f(guard.layer)
    }

    /// Push a clip rectangle in local coordinates, translated by [`Self::current_offset`]
    /// and intersected with the active clip.
    pub fn push_clip(&mut self, rect: Rect<Logical>) {
        let translated = translate_rect(rect, self.current_offset());
        let non_empty = if translated.size.width.get() > 0.0 && translated.size.height.get() > 0.0 {
            Some(translated)
        } else {
            None
        };
        let effective = match (self.clip_stack.last(), non_empty) {
            (Some(top), Some(r)) => top
                .intersect(&r)
                .unwrap_or_else(|| Rect::new(0.0, 0.0, 0.0, 0.0)),
            (None, Some(r)) => r,
            (_, None) => Rect::new(0.0, 0.0, 0.0, 0.0),
        };
        self.clip_stack.push(effective);
    }

    /// Pop the most recent clip rectangle, if any.
    pub fn pop_clip(&mut self) {
        self.clip_stack.pop();
    }

    /// Return the current effective clip rectangle in world logical coordinates.
    pub fn current_clip(&self) -> Option<Rect<Logical>> {
        self.clip_stack.last().copied()
    }

    /// Execute `f` within a temporary clip rectangle.
    pub fn with_clip<R>(&mut self, rect: Rect<Logical>, f: impl FnOnce(&mut Self) -> R) -> R {
        struct ClipGuard<'a> {
            layer: &'a mut HitTestLayer,
        }
        impl Drop for ClipGuard<'_> {
            fn drop(&mut self) {
                self.layer.pop_clip();
            }
        }
        self.push_clip(rect);
        let guard = ClipGuard { layer: self };
        f(guard.layer)
    }

    /// Clip to `viewport` in current frame space BEFORE translating by `(-scroll_x, -scroll_y)`.
    ///
    /// Items scrolled outside `viewport` are excluded from hit testing.
    pub fn with_scroll_viewport<R>(
        &mut self,
        viewport: Rect<Logical>,
        scroll_x: f32,
        scroll_y: f32,
        f: impl FnOnce(&mut Self) -> R,
    ) -> R {
        struct ScrollGuard<'a> {
            layer: &'a mut HitTestLayer,
        }
        impl Drop for ScrollGuard<'_> {
            fn drop(&mut self) {
                self.layer.pop_offset();
                self.layer.pop_clip();
            }
        }
        self.push_clip(viewport);
        self.push_offset(-sanitize_finite(scroll_x), -sanitize_finite(scroll_y));
        let guard = ScrollGuard { layer: self };
        f(guard.layer)
    }

    /// Push a semantic [`HitZLayer`] onto the Z-layer stack.
    pub fn push_z_layer(&mut self, layer: HitZLayer) {
        self.push_z_index(layer.z_value());
    }

    /// Push an arbitrary Z-index onto the Z-layer stack.
    pub fn push_z_index(&mut self, z: i32) {
        self.z_stack.push(z);
    }

    /// Pop the most recent Z-layer/Z-index, if any.
    pub fn pop_z_layer(&mut self) {
        self.z_stack.pop();
    }

    /// Execute `f` with `layer` active on the Z-layer stack.
    pub fn with_z_layer<R>(&mut self, layer: HitZLayer, f: impl FnOnce(&mut Self) -> R) -> R {
        self.with_z_index(layer.z_value(), f)
    }

    /// Execute `f` with numeric `z` active on the Z-layer stack.
    pub fn with_z_index<R>(&mut self, z: i32, f: impl FnOnce(&mut Self) -> R) -> R {
        struct ZGuard<'a> {
            layer: &'a mut HitTestLayer,
        }
        impl Drop for ZGuard<'_> {
            fn drop(&mut self) {
                self.layer.pop_z_layer();
            }
        }
        self.push_z_index(z);
        let guard = ZGuard { layer: self };
        f(guard.layer)
    }

    // -----------------------------------------------------------------------
    // Region builders
    // -----------------------------------------------------------------------

    /// Record a hit region described by [`HitRegionSpec`], applying the current
    /// offset, clip, ancestor path, and active Z-layer.
    pub fn push_region(&mut self, spec: HitRegionSpec) {
        let shape = spec.shape.translated(self.current_offset());
        let mut path_nodes = self.node_stack.clone();
        if path_nodes.last() != Some(&spec.id) {
            path_nodes.push(spec.id);
        }
        let base_z = self
            .z_stack
            .last()
            .copied()
            .unwrap_or(HitZLayer::Base.z_value());
        let z_layer = base_z.saturating_add(spec.z_layer);
        let order = self.entries.len();
        self.entries.push(RegionEntry {
            id: spec.id,
            path: HitTestPath(path_nodes),
            shape,
            clip: self.clip_stack.last().copied(),
            enabled: spec.enabled,
            z_layer,
            order,
            behavior: spec.behavior,
            cursor: spec.cursor,
            tag: spec.tag,
        });
    }

    /// Convenience helper to add a rectangular hit region with [`CursorHint::Default`].
    pub fn add_rect(&mut self, id: NodeId, rect: Rect<Logical>, enabled: bool) {
        self.push_region(HitRegionSpec::rect(id, rect).enabled(enabled));
    }

    /// Convenience helper to add an interactive button region with [`CursorHint::Pointer`].
    pub fn add_button(&mut self, id: NodeId, rect: Rect<Logical>, enabled: bool) {
        self.push_region(
            HitRegionSpec::rect(id, rect)
                .enabled(enabled)
                .cursor(CursorHint::Pointer),
        );
    }

    /// Convenience helper to add a circular hit region.
    pub fn add_circle(&mut self, id: NodeId, center: Point<Logical>, radius: f32, enabled: bool) {
        self.push_region(HitRegionSpec::circle(id, center, radius).enabled(enabled));
    }

    /// Push a full-bounds opaque backdrop scrim on `layer`.
    ///
    /// When `dismissible` is `true`, the scrim itself returns as a hit target;
    /// when `false`, it acts as an opaque blocker returning `None` for hits behind it.
    pub fn push_scrim(
        &mut self,
        id: NodeId,
        bounds: Rect<Logical>,
        layer: HitZLayer,
        dismissible: bool,
    ) {
        self.with_z_layer(layer, |l| {
            l.push_region(
                HitRegionSpec::rect(id, bounds)
                    .enabled(dismissible)
                    .behavior(HitBehavior::Opaque),
            );
        });
    }

    // -----------------------------------------------------------------------
    // Queries
    // -----------------------------------------------------------------------

    /// Return the topmost enabled hit at `point`, respecting Z-layer order,
    /// insertion order, clip bounds, and opaque disabled blockers.
    pub fn hit_test(&self, point: Point<Logical>) -> Option<HitResult> {
        let candidates = self.matching_candidates_desc(point);
        for entry in candidates {
            if entry.enabled {
                return Some(entry.to_result(point));
            }
            if entry.behavior == HitBehavior::Opaque {
                return None;
            }
        }
        None
    }

    /// Return all matching hits at `point` from topmost down to (and including)
    /// the first [`HitBehavior::Opaque`] region.
    pub fn hit_test_all(&self, point: Point<Logical>) -> Vec<HitResult> {
        let candidates = self.matching_candidates_desc(point);
        let mut results = Vec::new();
        for entry in candidates {
            results.push(entry.to_result(point));
            if entry.behavior == HitBehavior::Opaque {
                break;
            }
        }
        results
    }

    /// Return the pointer [`CursorHint`] at `point`.
    pub fn hit_test_cursor(&self, point: Point<Logical>) -> CursorHint {
        self.hit_test(point)
            .map_or(CursorHint::Default, |hit| hit.cursor)
    }

    /// Export non-transparent, non-empty regions as flat [`HitTestEntry`] values
    /// sorted ascending by `(z_layer ASC, insertion_order ASC)` so
    /// [`crate::hit_test`] `.iter().rev()` yields highest priority first.
    pub fn to_flat_entries(&self) -> Vec<HitTestEntry> {
        let mut entries: Vec<&RegionEntry> = self
            .entries
            .iter()
            .filter(|e| e.behavior != HitBehavior::Transparent)
            .collect();
        entries.sort_by(|a, b| {
            a.z_layer
                .cmp(&b.z_layer)
                .then_with(|| a.order.cmp(&b.order))
        });

        entries
            .into_iter()
            .filter_map(|e| {
                let raw = e.shape.bounding_rect();
                let bounds = match e.clip {
                    Some(clip) => raw.intersect(&clip)?,
                    None => raw,
                };
                (bounds.size.width.get() > 0.0 && bounds.size.height.get() > 0.0).then_some(
                    HitTestEntry {
                        id: e.id,
                        bounds,
                        enabled: e.enabled,
                    },
                )
            })
            .collect()
    }

    fn matching_candidates_desc(&self, point: Point<Logical>) -> Vec<&RegionEntry> {
        let mut matches: Vec<&RegionEntry> =
            self.entries.iter().filter(|e| e.contains(point)).collect();
        matches.sort_by(|a, b| {
            b.z_layer
                .cmp(&a.z_layer)
                .then_with(|| b.order.cmp(&a.order))
        });
        matches
    }
}

// ---------------------------------------------------------------------------
// Click & Drag State Machine
// ---------------------------------------------------------------------------

/// Configuration thresholds for [`ClickDetector`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClickDetectorConfig {
    /// Minimum pointer travel in logical pixels before a press becomes a drag.
    pub drag_threshold_px: f32,
    /// Maximum elapsed time in seconds between consecutive clicks for multi-click.
    pub multi_click_interval_secs: f64,
    /// Maximum spatial distance in logical pixels between consecutive clicks for multi-click.
    pub multi_click_distance_px: f32,
}

impl Default for ClickDetectorConfig {
    fn default() -> Self {
        Self {
            drag_threshold_px: 4.0,
            multi_click_interval_secs: 0.4,
            multi_click_distance_px: 5.0,
        }
    }
}

/// Semantic pointer interaction event emitted by [`ClickDetector`].
#[derive(Clone, Debug, PartialEq)]
pub enum PointerOutcome {
    HoverEnter {
        id: NodeId,
        cursor: CursorHint,
    },
    HoverLeave {
        id: NodeId,
    },
    Pressed {
        id: NodeId,
        button: u16,
        point: Point<Logical>,
        local_point: Point<Local>,
    },
    Released {
        id: NodeId,
        button: u16,
        point: Point<Logical>,
    },
    Clicked {
        id: NodeId,
        button: u16,
        point: Point<Logical>,
        count: u32,
    },
    DragStart {
        id: NodeId,
        button: u16,
        start_point: Point<Logical>,
        current_point: Point<Logical>,
    },
    DragMove {
        id: NodeId,
        button: u16,
        delta: Point<Logical>,
        current_point: Point<Logical>,
    },
    DragEnd {
        id: NodeId,
        button: u16,
        point: Point<Logical>,
    },
    Cancelled {
        id: NodeId,
        button: u16,
    },
}

#[derive(Clone, Copy, Debug)]
struct ActivePress {
    id: NodeId,
    button: u16,
    start_point: Point<Logical>,
    last_point: Point<Logical>,
    dragging: bool,
}

#[derive(Clone, Copy, Debug)]
struct LastClick {
    id: NodeId,
    button: u16,
    point: Point<Logical>,
    time_secs: f64,
    count: u32,
}

/// Stateful pointer recognizer translating raw pointer moves/buttons against a
/// [`HitTestLayer`] into hover, press, click, multi-click, and drag outcomes.
#[derive(Clone, Debug, Default)]
pub struct ClickDetector {
    config: ClickDetectorConfig,
    hovered: Option<NodeId>,
    cursor: CursorHint,
    pressed: Option<ActivePress>,
    last_click: Option<LastClick>,
}

impl ClickDetector {
    /// Create a [`ClickDetector`] with [`ClickDetectorConfig::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a [`ClickDetector`] with a custom [`ClickDetectorConfig`].
    pub fn with_config(config: ClickDetectorConfig) -> Self {
        Self {
            config,
            ..Self::default()
        }
    }

    /// Currently hovered node, if any.
    pub fn hovered(&self) -> Option<NodeId> {
        self.hovered
    }

    /// Currently pressed node, if any.
    pub fn pressed(&self) -> Option<NodeId> {
        self.pressed.map(|p| p.id)
    }

    /// Whether the active press has crossed [`ClickDetectorConfig::drag_threshold_px`].
    pub fn is_dragging(&self) -> bool {
        self.pressed.is_some_and(|p| p.dragging)
    }

    /// Current pointer [`CursorHint`].
    pub fn cursor(&self) -> CursorHint {
        self.cursor
    }

    /// Process a pointer move at `point`, returning any hover or drag outcomes.
    pub fn on_pointer_move(
        &mut self,
        layer: &HitTestLayer,
        point: Point<Logical>,
    ) -> Vec<PointerOutcome> {
        let mut outcomes = Vec::new();

        if let Some(mut press) = self.pressed {
            let threshold = sanitize_non_neg(self.config.drag_threshold_px);
            let total_dist = distance(point, press.start_point);
            if !press.dragging && total_dist > 0.0 && total_dist >= threshold {
                press.dragging = true;
                outcomes.push(PointerOutcome::DragStart {
                    id: press.id,
                    button: press.button,
                    start_point: press.start_point,
                    current_point: point,
                });
            } else if press.dragging {
                let delta = Point::new(
                    point.x.get() - press.last_point.x.get(),
                    point.y.get() - press.last_point.y.get(),
                );
                outcomes.push(PointerOutcome::DragMove {
                    id: press.id,
                    button: press.button,
                    delta,
                    current_point: point,
                });
            }
            press.last_point = point;
            self.pressed = Some(press);
            return outcomes;
        }

        let hit = layer.hit_test(point);
        let next_id = hit.as_ref().map(|h| h.id);
        let next_cursor = hit.as_ref().map_or(CursorHint::Default, |h| h.cursor);

        if next_id != self.hovered {
            if let Some(prev) = self.hovered {
                outcomes.push(PointerOutcome::HoverLeave { id: prev });
            }
            if let Some(id) = next_id {
                outcomes.push(PointerOutcome::HoverEnter {
                    id,
                    cursor: next_cursor,
                });
            }
            self.hovered = next_id;
        }
        self.cursor = next_cursor;
        outcomes
    }

    /// Process a pointer button press or release at `point` and `time_secs`.
    pub fn on_pointer_button(
        &mut self,
        layer: &HitTestLayer,
        button: u16,
        pressed: bool,
        point: Point<Logical>,
        time_secs: f64,
    ) -> Vec<PointerOutcome> {
        let mut outcomes = Vec::new();

        if pressed {
            if let Some(prev) = self.pressed.take() {
                outcomes.push(PointerOutcome::Cancelled {
                    id: prev.id,
                    button: prev.button,
                });
            }
            if let Some(hit) = layer.hit_test(point) {
                self.pressed = Some(ActivePress {
                    id: hit.id,
                    button,
                    start_point: point,
                    last_point: point,
                    dragging: false,
                });
                outcomes.push(PointerOutcome::Pressed {
                    id: hit.id,
                    button,
                    point,
                    local_point: hit.local_point,
                });
            }
            return outcomes;
        }

        let Some(active) = self.pressed else {
            return outcomes;
        };
        if active.button != button {
            return outcomes;
        }
        self.pressed = None;

        outcomes.push(PointerOutcome::Released {
            id: active.id,
            button,
            point,
        });

        if active.dragging {
            outcomes.push(PointerOutcome::DragEnd {
                id: active.id,
                button,
                point,
            });
            self.last_click = None;
            return outcomes;
        }

        let hit_same = layer.hit_test(point).is_some_and(|h| h.id == active.id);
        if hit_same {
            let count = self.next_click_count(active.id, button, point, time_secs);
            self.last_click = if time_secs.is_finite() {
                Some(LastClick {
                    id: active.id,
                    button,
                    point,
                    time_secs,
                    count,
                })
            } else {
                None
            };
            outcomes.push(PointerOutcome::Clicked {
                id: active.id,
                button,
                point,
                count,
            });
        } else {
            self.last_click = None;
        }

        outcomes
    }

    /// Process the pointer leaving the window/surface, clearing hover state.
    pub fn on_pointer_leave(&mut self) -> Vec<PointerOutcome> {
        self.cursor = CursorHint::Default;
        if let Some(id) = self.hovered.take() {
            vec![PointerOutcome::HoverLeave { id }]
        } else {
            Vec::new()
        }
    }

    /// Cancel any active press/drag gesture and reset click streak tracking.
    pub fn cancel(&mut self) -> Vec<PointerOutcome> {
        self.last_click = None;
        if let Some(active) = self.pressed.take() {
            vec![PointerOutcome::Cancelled {
                id: active.id,
                button: active.button,
            }]
        } else {
            Vec::new()
        }
    }

    fn next_click_count(
        &self,
        id: NodeId,
        button: u16,
        point: Point<Logical>,
        time_secs: f64,
    ) -> u32 {
        let Some(prev) = self.last_click else {
            return 1;
        };
        if !time_secs.is_finite() || !prev.time_secs.is_finite() {
            return 1;
        }
        let max_dt = if self.config.multi_click_interval_secs.is_finite() {
            self.config.multi_click_interval_secs.max(0.0)
        } else {
            0.0
        };
        let max_dist = sanitize_non_neg(self.config.multi_click_distance_px);
        let dt = time_secs - prev.time_secs;
        let within_time = dt >= 0.0 && dt <= max_dt;
        let within_dist = distance(point, prev.point) <= max_dist;
        if prev.id == id && prev.button == button && within_time && within_dist {
            prev.count.saturating_add(1)
        } else {
            1
        }
    }
}

fn sanitize_finite(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

fn sanitize_non_neg(value: f32) -> f32 {
    sanitize_finite(value).max(0.0)
}

fn translate_rect(rect: Rect<Logical>, (dx, dy): (f32, f32)) -> Rect<Logical> {
    Rect::new(
        sanitize_finite(sanitize_finite(rect.origin.x.get()) + dx),
        sanitize_finite(sanitize_finite(rect.origin.y.get()) + dy),
        sanitize_non_neg(rect.size.width.get()),
        sanitize_non_neg(rect.size.height.get()),
    )
}

fn distance(a: Point<Logical>, b: Point<Logical>) -> f32 {
    (a.x.get() - b.x.get()).hypot(a.y.get() - b.y.get())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EventPhase, EventResult, ImeEvent, UiEvent, dispatch_event, hit_test};

    fn node(id: u64) -> NodeId {
        NodeId::new(id)
    }

    #[test]
    fn z_layer_numeric_values_are_ordered() {
        assert_eq!(HitZLayer::Base.z_value(), 0);
        assert_eq!(HitZLayer::Floating.z_value(), 10);
        assert_eq!(HitZLayer::Overlay.z_value(), 20);
        assert_eq!(HitZLayer::Modal.z_value(), 30);
        assert_eq!(HitZLayer::Tooltip.z_value(), 40);
        assert_eq!(HitZLayer::Debug.z_value(), 50);
    }

    #[test]
    fn hit_shape_rect_half_open_and_bounding_rect() {
        let r = Rect::new(10.0, 20.0, 30.0, 40.0);
        let shape = HitShape::Rect(r);
        assert_eq!(shape.bounding_rect(), r);
        assert!(shape.contains(Point::new(10.0, 20.0)));
        assert!(shape.contains(Point::new(39.9, 59.9)));
        assert!(!shape.contains(Point::new(40.0, 30.0)));
        assert!(!shape.contains(Point::new(20.0, 60.0)));
    }

    #[test]
    fn hit_shape_rounded_rect_excludes_cut_corners_and_includes_interior() {
        let shape = HitShape::RoundedRect {
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
            radius: 20.0,
        };
        // Extreme corner (1, 1) is outside rounded corner arc (dist to (20,20) = 26.87 > 20)
        assert!(!shape.contains(Point::new(1.0, 1.0)));
        // Far bottom-right corner inside rect bounds but outside arc
        assert!(!shape.contains(Point::new(98.0, 98.0)));
        // Edge tangent (20, 0) and (0, 20) are inside
        assert!(shape.contains(Point::new(20.0, 0.0)));
        assert!(shape.contains(Point::new(0.0, 20.0)));
        // Inside corner arc (7, 7): dist to (20,20) = sqrt(169+169) = 18.38 <= 20
        assert!(shape.contains(Point::new(7.0, 7.0)));
        // Center
        assert!(shape.contains(Point::new(50.0, 50.0)));
    }

    #[test]
    fn hit_shape_circle_containment_and_degenerate_radius() {
        let c = HitShape::Circle {
            center: Point::new(50.0, 50.0),
            radius: 10.0,
        };
        assert_eq!(c.bounding_rect(), Rect::new(40.0, 40.0, 20.0, 20.0));
        assert!(c.contains(Point::new(50.0, 50.0)));
        assert!(c.contains(Point::new(60.0, 50.0))); // on boundary
        assert!(!c.contains(Point::new(58.0, 58.0))); // sqrt(64+64) = 11.31 > 10

        let zero = HitShape::Circle {
            center: Point::new(0.0, 0.0),
            radius: -5.0,
        };
        assert!(!zero.contains(Point::new(0.0, 0.0)));
    }

    #[test]
    fn ancestor_node_stack_builds_dispatchable_hit_test_path() {
        let (root, panel, btn) = (node(1), node(2), node(3));
        let mut layer = HitTestLayer::new();
        layer.with_node(root, |l| {
            l.with_node(panel, |l| {
                l.add_button(btn, Rect::new(10.0, 10.0, 50.0, 20.0), true);
            });
        });

        let hit = layer.hit_test(Point::new(25.0, 15.0)).unwrap();
        assert_eq!(hit.id, btn);
        assert_eq!(hit.path, HitTestPath(vec![root, panel, btn]));
        assert_eq!(hit.local_point, Point::new(15.0, 5.0));

        let mut phases = Vec::new();
        dispatch_event(
            hit.path,
            &UiEvent::Ime(ImeEvent::Enabled),
            |id, phase, _| {
                phases.push((id, phase));
                EventResult::default()
            },
        );
        assert_eq!(
            phases,
            vec![
                (root, EventPhase::Capture),
                (panel, EventPhase::Capture),
                (btn, EventPhase::Target),
                (panel, EventPhase::Bubble),
                (root, EventPhase::Bubble),
            ]
        );
    }

    #[test]
    fn node_stack_avoids_duplicate_leaf_if_pushed_manually() {
        let (root, btn) = (node(1), node(2));
        let mut layer = HitTestLayer::new();
        layer.with_node(root, |l| {
            l.with_node(btn, |l| {
                l.add_button(btn, Rect::new(0.0, 0.0, 20.0, 20.0), true);
            });
        });
        let hit = layer.hit_test(Point::new(5.0, 5.0)).unwrap();
        assert_eq!(hit.path, HitTestPath(vec![root, btn]));
    }

    #[test]
    fn offsets_and_nested_clips_transform_and_clip_hits() {
        let btn = node(10);
        let mut layer = HitTestLayer::new();
        layer.with_offset(100.0, 50.0, |l| {
            l.with_clip(Rect::new(0.0, 0.0, 40.0, 40.0), |l| {
                l.add_button(btn, Rect::new(20.0, 20.0, 60.0, 60.0), true);
            });
        });

        // Button world bounds: (120, 70, 60, 60); clip world bounds: (100, 50, 40, 40) -> visible [120..140) x [70..90)
        assert!(layer.hit_test(Point::new(125.0, 75.0)).is_some());
        assert!(layer.hit_test(Point::new(145.0, 75.0)).is_none());
        assert!(layer.hit_test(Point::new(125.0, 95.0)).is_none());
    }

    #[test]
    fn scroll_viewport_clips_before_translating_so_scrolled_out_items_are_not_hit() {
        let (item1, item2, item3) = (node(1), node(2), node(3));
        let mut layer = HitTestLayer::new();
        // Viewport at (10, 20, 100, 100), scrolled down by 60px
        layer.with_scroll_viewport(Rect::new(10.0, 20.0, 100.0, 100.0), 0.0, 60.0, |l| {
            // item1 occupies content y=[20..70) -> after -60 scroll: [-40..10), completely above viewport [20..120)
            l.add_button(item1, Rect::new(10.0, 20.0, 100.0, 50.0), true);
            // item2 occupies content y=[80..130) -> after -60 scroll: [20..70), inside viewport
            l.add_button(item2, Rect::new(10.0, 80.0, 100.0, 50.0), true);
            // item3 occupies content y=[160..210) -> after -60 scroll: [100..150), partially clipped below y=120
            l.add_button(item3, Rect::new(10.0, 160.0, 100.0, 50.0), true);
        });

        // Point above viewport (where scrolled-out item1 moved to) must NOT hit item1
        assert_eq!(layer.hit_test(Point::new(50.0, 5.0)), None);
        // Inside viewport over item2
        assert_eq!(
            layer.hit_test(Point::new(50.0, 40.0)).map(|h| h.id),
            Some(item2)
        );
        // Visible portion of item3 inside viewport
        assert_eq!(
            layer.hit_test(Point::new(50.0, 110.0)).map(|h| h.id),
            Some(item3)
        );
        // Clipped-out portion of item3 below viewport
        assert_eq!(layer.hit_test(Point::new(50.0, 130.0)), None);
    }

    #[test]
    fn z_layer_and_insertion_order_resolve_overlaps() {
        let (base_early, base_late, floating, modal) = (node(1), node(2), node(3), node(4));
        let mut layer = HitTestLayer::new();
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);

        layer.with_z_layer(HitZLayer::Modal, |l| {
            l.add_rect(modal, rect, true);
        });
        layer.add_rect(base_early, rect, true);
        layer.add_rect(base_late, rect, true);
        layer.push_region(HitRegionSpec::rect(floating, rect).layer(HitZLayer::Floating));

        // Modal (30) wins even though pushed first
        assert_eq!(
            layer.hit_test(Point::new(10.0, 10.0)).map(|h| h.id),
            Some(modal)
        );

        let all = layer.hit_test_all(Point::new(10.0, 10.0));
        // Stopped at Modal because Modal is Opaque
        assert_eq!(all.iter().map(|h| h.id).collect::<Vec<_>>(), vec![modal]);
    }

    #[test]
    fn hit_behavior_opaque_translucent_and_transparent_semantics() {
        let (back, trans_disabled, transparent_top, opaque_blocker) =
            (node(1), node(2), node(3), node(4));
        let mut layer = HitTestLayer::new();
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);

        layer.add_rect(back, rect, true);
        layer.push_region(
            HitRegionSpec::rect(trans_disabled, rect)
                .enabled(false)
                .behavior(HitBehavior::Translucent),
        );
        layer.push_region(
            HitRegionSpec::rect(transparent_top, rect)
                .enabled(true)
                .behavior(HitBehavior::Transparent),
        );

        // Transparent is skipped; disabled Translucent continues searching -> hits back
        assert_eq!(
            layer.hit_test(Point::new(10.0, 10.0)).map(|h| h.id),
            Some(back)
        );

        // Add disabled Opaque blocker in between -> blocks back and returns None
        layer.push_region(
            HitRegionSpec::rect(opaque_blocker, rect)
                .layer(HitZLayer::Floating)
                .enabled(false)
                .behavior(HitBehavior::Opaque),
        );
        assert_eq!(layer.hit_test(Point::new(10.0, 10.0)), None);
    }

    #[test]
    fn hit_test_all_returns_translucent_hits_and_stops_at_opaque() {
        let (bottom, mid_translucent, top_translucent, transparent) =
            (node(1), node(2), node(3), node(4));
        let mut layer = HitTestLayer::new();
        let rect = Rect::new(0.0, 0.0, 50.0, 50.0);

        layer.add_rect(bottom, rect, true); // Opaque
        layer.push_region(
            HitRegionSpec::rect(mid_translucent, rect).behavior(HitBehavior::Translucent),
        );
        layer.push_region(
            HitRegionSpec::rect(top_translucent, rect).behavior(HitBehavior::Translucent),
        );
        layer
            .push_region(HitRegionSpec::rect(transparent, rect).behavior(HitBehavior::Transparent));

        let hits = layer.hit_test_all(Point::new(10.0, 10.0));
        assert_eq!(
            hits.iter().map(|h| h.id).collect::<Vec<_>>(),
            vec![top_translucent, mid_translucent, bottom]
        );
    }

    #[test]
    fn scrim_dismissible_vs_blocking_semantics() {
        let (base_btn, scrim, modal_btn) = (node(1), node(2), node(3));

        // Dismissible scrim
        let mut dismiss_layer = HitTestLayer::new();
        dismiss_layer.add_button(base_btn, Rect::new(0.0, 0.0, 200.0, 200.0), true);
        dismiss_layer.push_scrim(
            scrim,
            Rect::new(0.0, 0.0, 200.0, 200.0),
            HitZLayer::Modal,
            true,
        );
        dismiss_layer.with_z_layer(HitZLayer::Modal, |l| {
            l.add_button(modal_btn, Rect::new(50.0, 50.0, 40.0, 40.0), true);
        });
        assert_eq!(
            dismiss_layer.hit_test(Point::new(60.0, 60.0)).map(|h| h.id),
            Some(modal_btn)
        );
        assert_eq!(
            dismiss_layer.hit_test(Point::new(10.0, 10.0)).map(|h| h.id),
            Some(scrim)
        );

        // Non-dismissible scrim blocks base_btn and returns None
        let mut blocking_layer = HitTestLayer::new();
        blocking_layer.add_button(base_btn, Rect::new(0.0, 0.0, 200.0, 200.0), true);
        blocking_layer.push_scrim(
            scrim,
            Rect::new(0.0, 0.0, 200.0, 200.0),
            HitZLayer::Modal,
            false,
        );
        assert_eq!(blocking_layer.hit_test(Point::new(10.0, 10.0)), None);
    }

    #[test]
    fn cursor_hint_and_tags_propagate() {
        let (btn, text_box, splitter) = (node(1), node(2), node(3));
        let mut layer = HitTestLayer::new();
        layer.add_button(btn, Rect::new(0.0, 0.0, 50.0, 30.0), true);
        layer.push_region(
            HitRegionSpec::rect(text_box, Rect::new(60.0, 0.0, 80.0, 30.0))
                .cursor(CursorHint::Text)
                .tag(42),
        );
        layer.push_region(
            HitRegionSpec::rect(splitter, Rect::new(150.0, 0.0, 10.0, 30.0))
                .cursor(CursorHint::ResizeEW),
        );

        assert_eq!(
            layer.hit_test_cursor(Point::new(10.0, 10.0)),
            CursorHint::Pointer
        );
        assert_eq!(
            layer.hit_test_cursor(Point::new(70.0, 10.0)),
            CursorHint::Text
        );
        assert_eq!(layer.hit_test(Point::new(70.0, 10.0)).unwrap().tag, 42);
        assert_eq!(
            layer.hit_test_cursor(Point::new(155.0, 10.0)),
            CursorHint::ResizeEW
        );
        assert_eq!(
            layer.hit_test_cursor(Point::new(300.0, 300.0)),
            CursorHint::Default
        );
    }

    #[test]
    fn to_flat_entries_matches_acme_core_hit_test_ordering() {
        let (low, high_z, clipped_out, transparent) = (node(1), node(2), node(3), node(4));
        let mut layer = HitTestLayer::new();
        layer.push_region(
            HitRegionSpec::rect(high_z, Rect::new(0.0, 0.0, 50.0, 50.0)).layer(HitZLayer::Overlay),
        );
        layer.add_rect(low, Rect::new(0.0, 0.0, 100.0, 100.0), true);
        layer.with_clip(Rect::new(0.0, 0.0, 10.0, 10.0), |l| {
            l.add_rect(clipped_out, Rect::new(50.0, 50.0, 20.0, 20.0), true);
        });
        layer.push_region(
            HitRegionSpec::rect(transparent, Rect::new(0.0, 0.0, 100.0, 100.0))
                .behavior(HitBehavior::Transparent),
        );

        let entries = layer.to_flat_entries();
        assert_eq!(entries.len(), 2); // clipped_out & transparent omitted
        assert_eq!(entries[0].id, low);
        assert_eq!(entries[1].id, high_z);

        let pt_overlap = Point::new(25.0, 25.0);
        assert_eq!(
            hit_test(&entries, pt_overlap),
            layer.hit_test(pt_overlap).map(|h| h.id)
        );
        let pt_low = Point::new(75.0, 75.0);
        assert_eq!(
            hit_test(&entries, pt_low),
            layer.hit_test(pt_low).map(|h| h.id)
        );
    }

    #[test]
    fn clear_and_empty_stack_pops_are_safe() {
        let mut layer = HitTestLayer::new();
        assert!(layer.is_empty());
        layer.pop_node();
        layer.pop_offset();
        layer.pop_clip();
        layer.pop_z_layer();

        layer.add_rect(node(1), Rect::new(0.0, 0.0, 10.0, 10.0), true);
        assert_eq!(layer.len(), 1);
        assert!(!layer.is_empty());

        layer.clear();
        assert!(layer.is_empty());
        assert_eq!(layer.len(), 0);
        assert_eq!(layer.current_offset(), (0.0, 0.0));
        assert_eq!(layer.current_clip(), None);
    }

    #[test]
    fn click_detector_hover_enter_leave_and_cursor() {
        let (b1, b2) = (node(1), node(2));
        let mut layer = HitTestLayer::new();
        layer.add_button(b1, Rect::new(0.0, 0.0, 40.0, 40.0), true);
        layer.push_region(
            HitRegionSpec::rect(b2, Rect::new(50.0, 0.0, 40.0, 40.0)).cursor(CursorHint::Text),
        );

        let mut det = ClickDetector::new();
        assert_eq!(det.hovered(), None);
        assert_eq!(det.cursor(), CursorHint::Default);

        let o1 = det.on_pointer_move(&layer, Point::new(10.0, 10.0));
        assert_eq!(
            o1,
            vec![PointerOutcome::HoverEnter {
                id: b1,
                cursor: CursorHint::Pointer,
            }]
        );
        assert_eq!(det.hovered(), Some(b1));
        assert_eq!(det.cursor(), CursorHint::Pointer);

        // Move within same node emits no redundant hover events
        assert!(
            det.on_pointer_move(&layer, Point::new(20.0, 20.0))
                .is_empty()
        );

        let o2 = det.on_pointer_move(&layer, Point::new(60.0, 10.0));
        assert_eq!(
            o2,
            vec![
                PointerOutcome::HoverLeave { id: b1 },
                PointerOutcome::HoverEnter {
                    id: b2,
                    cursor: CursorHint::Text,
                },
            ]
        );
        assert_eq!(det.hovered(), Some(b2));
        assert_eq!(det.cursor(), CursorHint::Text);

        let o3 = det.on_pointer_leave();
        assert_eq!(o3, vec![PointerOutcome::HoverLeave { id: b2 }]);
        assert_eq!(det.hovered(), None);
        assert_eq!(det.cursor(), CursorHint::Default);
    }

    #[test]
    fn click_detector_multi_click_and_reset_on_distance_or_timeout() {
        let btn = node(1);
        let mut layer = HitTestLayer::new();
        layer.add_button(btn, Rect::new(0.0, 0.0, 100.0, 100.0), true);
        let mut det = ClickDetector::new();

        // Click 1 at t=1.0
        let down1 = det.on_pointer_button(&layer, 0, true, Point::new(10.0, 10.0), 1.0);
        assert!(matches!(down1.as_slice(), [PointerOutcome::Pressed { id, .. }] if *id == btn));
        let up1 = det.on_pointer_button(&layer, 0, false, Point::new(11.0, 10.0), 1.05);
        assert_eq!(
            up1,
            vec![
                PointerOutcome::Released {
                    id: btn,
                    button: 0,
                    point: Point::new(11.0, 10.0),
                },
                PointerOutcome::Clicked {
                    id: btn,
                    button: 0,
                    point: Point::new(11.0, 10.0),
                    count: 1,
                },
            ]
        );

        // Click 2 within 0.4s and 5px -> double click (count = 2)
        det.on_pointer_button(&layer, 0, true, Point::new(12.0, 12.0), 1.2);
        let up2 = det.on_pointer_button(&layer, 0, false, Point::new(12.0, 12.0), 1.25);
        assert!(matches!(
            up2.as_slice(),
            [_, PointerOutcome::Clicked { count: 2, .. }]
        ));

        // Click 3 within time/distance -> triple click (count = 3)
        det.on_pointer_button(&layer, 0, true, Point::new(13.0, 12.0), 1.4);
        let up3 = det.on_pointer_button(&layer, 0, false, Point::new(13.0, 12.0), 1.45);
        assert!(matches!(
            up3.as_slice(),
            [_, PointerOutcome::Clicked { count: 3, .. }]
        ));

        // Click 4 far away (> 5px) -> resets to count = 1
        det.on_pointer_button(&layer, 0, true, Point::new(50.0, 50.0), 1.5);
        let up4 = det.on_pointer_button(&layer, 0, false, Point::new(50.0, 50.0), 1.55);
        assert!(matches!(
            up4.as_slice(),
            [_, PointerOutcome::Clicked { count: 1, .. }]
        ));

        // Click 5 after interval (> 0.4s) -> resets to count = 1
        det.on_pointer_button(&layer, 0, true, Point::new(50.0, 50.0), 2.5);
        let up5 = det.on_pointer_button(&layer, 0, false, Point::new(50.0, 50.0), 2.55);
        assert!(matches!(
            up5.as_slice(),
            [_, PointerOutcome::Clicked { count: 1, .. }]
        ));
    }

    #[test]
    fn click_detector_drag_lifecycle_suppresses_click_and_tracks_deltas() {
        let slider = node(1);
        let mut layer = HitTestLayer::new();
        layer.add_button(slider, Rect::new(0.0, 0.0, 100.0, 40.0), true);
        let mut det = ClickDetector::with_config(ClickDetectorConfig {
            drag_threshold_px: 4.0,
            ..ClickDetectorConfig::default()
        });

        det.on_pointer_button(&layer, 0, true, Point::new(10.0, 10.0), 0.1);
        assert_eq!(det.pressed(), Some(slider));
        assert!(!det.is_dragging());

        // Move 2px (< threshold 4px): no drag yet
        assert!(
            det.on_pointer_move(&layer, Point::new(12.0, 10.0))
                .is_empty()
        );
        assert!(!det.is_dragging());

        // Move to (15, 10) (dist 5px >= 4px): DragStart
        let start = det.on_pointer_move(&layer, Point::new(15.0, 10.0));
        assert_eq!(
            start,
            vec![PointerOutcome::DragStart {
                id: slider,
                button: 0,
                start_point: Point::new(10.0, 10.0),
                current_point: Point::new(15.0, 10.0),
            }]
        );
        assert!(det.is_dragging());

        // Move to (25, 13): DragMove with delta (10, 3)
        let mv = det.on_pointer_move(&layer, Point::new(25.0, 13.0));
        assert_eq!(
            mv,
            vec![PointerOutcome::DragMove {
                id: slider,
                button: 0,
                delta: Point::new(10.0, 3.0),
                current_point: Point::new(25.0, 13.0),
            }]
        );

        // Release ends drag without firing Clicked
        let end = det.on_pointer_button(&layer, 0, false, Point::new(25.0, 13.0), 0.3);
        assert_eq!(
            end,
            vec![
                PointerOutcome::Released {
                    id: slider,
                    button: 0,
                    point: Point::new(25.0, 13.0),
                },
                PointerOutcome::DragEnd {
                    id: slider,
                    button: 0,
                    point: Point::new(25.0, 13.0),
                },
            ]
        );
        assert!(!det.is_dragging());
        assert_eq!(det.pressed(), None);
    }

    #[test]
    fn click_detector_release_outside_target_does_not_click_and_cancel_resets() {
        let btn = node(1);
        let mut layer = HitTestLayer::new();
        layer.add_button(btn, Rect::new(0.0, 0.0, 10.0, 10.0), true);
        let mut det = ClickDetector::with_config(ClickDetectorConfig {
            drag_threshold_px: 20.0,
            ..ClickDetectorConfig::default()
        });

        // Press near edge at (9, 5), release outside at (12, 5) (< 20px drag threshold, outside btn)
        det.on_pointer_button(&layer, 0, true, Point::new(9.0, 5.0), 0.1);
        let up = det.on_pointer_button(&layer, 0, false, Point::new(12.0, 5.0), 0.2);
        assert_eq!(
            up,
            vec![PointerOutcome::Released {
                id: btn,
                button: 0,
                point: Point::new(12.0, 5.0),
            }]
        );

        // Press and cancel
        det.on_pointer_button(&layer, 1, true, Point::new(5.0, 5.0), 0.5);
        assert_eq!(det.pressed(), Some(btn));
        let cancelled = det.cancel();
        assert!(cancelled == vec![PointerOutcome::Cancelled { id: btn, button: 1 }]);
        assert_eq!(det.pressed(), None);
        assert!(!det.is_dragging());
        assert!(det.cancel().is_empty());
    }

    #[test]
    fn circle_regions_respect_offsets_clips_and_non_finite_scroll() {
        let orb = node(9);
        let mut layer = HitTestLayer::new();
        layer.with_scroll_viewport(
            Rect::new(0.0, 0.0, 100.0, 100.0),
            f32::NAN,
            f32::INFINITY,
            |l| {
                l.add_circle(orb, Point::new(50.0, 50.0), 25.0, true);
            },
        );
        // Inside circle & clip
        assert_eq!(
            layer.hit_test(Point::new(50.0, 50.0)).map(|h| h.id),
            Some(orb)
        );
        // Inside bounding rect (25..75, 25..75) at corner (27, 27), but outside circle
        assert_eq!(layer.hit_test(Point::new(27.0, 27.0)), None);
    }
}
