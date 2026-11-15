//! Frame dirty-gating tracker combining [`RetainedTree`] dirty flags,
//! reconcile reports, viewport size changes, and [`Theme`] equality.

use acme_core::{DirtyFlags, Logical, NodeId, ReconcileReport, RetainedTree, Size};
use acme_theme::Theme;
use std::collections::HashSet;

/// Work required for the current frame after evaluating retained and external changes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DirtyState {
    pub needs_layout: bool,
    pub needs_paint: bool,
    pub needs_hit_rebuild: bool,
    pub needs_accessibility: bool,
}

impl DirtyState {
    /// No stages need work.
    pub const fn clean() -> Self {
        Self {
            needs_layout: false,
            needs_paint: false,
            needs_hit_rebuild: false,
            needs_accessibility: false,
        }
    }

    /// All stages need work.
    pub const fn all() -> Self {
        Self {
            needs_layout: true,
            needs_paint: true,
            needs_hit_rebuild: true,
            needs_accessibility: true,
        }
    }

    /// Returns `true` when every stage is clean.
    pub const fn is_clean(&self) -> bool {
        !self.needs_layout
            && !self.needs_paint
            && !self.needs_hit_rebuild
            && !self.needs_accessibility
    }

    /// Returns `true` when at least one stage needs work.
    pub const fn any(&self) -> bool {
        !self.is_clean()
    }

    /// Merge another [`DirtyState`] using bitwise OR.
    pub fn merge(&mut self, other: Self) {
        self.needs_layout |= other.needs_layout;
        self.needs_paint |= other.needs_paint;
        self.needs_hit_rebuild |= other.needs_hit_rebuild;
        self.needs_accessibility |= other.needs_accessibility;
    }
}

/// Stateful per-frame dirty tracker wrapping retained-tree invalidation,
/// viewport resizing, and [`Theme`] changes.
#[derive(Clone, Debug)]
pub struct FrameDirtyTracker {
    last_viewport: Option<Size<Logical>>,
    last_theme: Option<Theme>,
    force_all: bool,
    pending: DirtyState,
}

impl Default for FrameDirtyTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameDirtyTracker {
    /// Create a tracker primed for the initial frame (`DirtyState::all()`).
    pub fn new() -> Self {
        Self {
            last_viewport: None,
            last_theme: None,
            force_all: true,
            pending: DirtyState::clean(),
        }
    }

    /// Return the last committed viewport size, if any.
    pub fn last_viewport(&self) -> Option<Size<Logical>> {
        self.last_viewport
    }

    /// Return the last committed [`Theme`], if any.
    pub fn last_theme(&self) -> Option<&Theme> {
        self.last_theme.as_ref()
    }

    /// Force all frame stages to run on the next [`Self::evaluate`] call.
    pub fn invalidate_all(&mut self) {
        self.force_all = true;
    }

    /// Mark layout (plus paint and hit-layer rebuild) dirty for the next frame.
    pub fn mark_layout_dirty(&mut self) {
        self.pending.needs_layout = true;
        self.pending.needs_paint = true;
        self.pending.needs_hit_rebuild = true;
    }

    /// Mark paint dirty for the next frame.
    pub fn mark_paint_dirty(&mut self) {
        self.pending.needs_paint = true;
    }

    /// Mark hit-layer rebuild dirty for the next frame.
    pub fn mark_hit_dirty(&mut self) {
        self.pending.needs_hit_rebuild = true;
    }

    /// Mark accessibility tree rebuild dirty for the next frame.
    pub fn mark_accessibility_dirty(&mut self) {
        self.pending.needs_accessibility = true;
    }

    /// Evaluate which frame stages must execute given `tree`, optional `report`,
    /// `viewport`, and `theme`.
    pub fn evaluate(
        &self,
        tree: &RetainedTree,
        report: Option<&ReconcileReport>,
        viewport: Size<Logical>,
        theme: &Theme,
    ) -> DirtyState {
        if self.force_all {
            return DirtyState::all();
        }

        let mut state = self.pending;

        if self.last_viewport != Some(viewport) {
            state.needs_layout = true;
            state.needs_paint = true;
            state.needs_hit_rebuild = true;
        }

        if let Some(prev_theme) = &self.last_theme {
            if prev_theme != theme {
                state.needs_paint = true;
                if prev_theme.spacing != theme.spacing || prev_theme.typography != theme.typography
                {
                    state.needs_layout = true;
                    state.needs_hit_rebuild = true;
                }
            }
        } else {
            state.merge(DirtyState::all());
        }

        if let Some(rep) = report
            && (!rep.mounted.is_empty() || !rep.removed.is_empty())
        {
            state.merge(DirtyState::all());
        }

        let flags = collect_tree_dirty_flags(tree);
        if flags.contains(DirtyFlags::LAYOUT) || flags.contains(DirtyFlags::CHILDREN) {
            state.needs_layout = true;
            state.needs_paint = true;
            state.needs_hit_rebuild = true;
            state.needs_accessibility = true;
        }
        if flags.contains(DirtyFlags::STYLE) || flags.contains(DirtyFlags::PAINT) {
            state.needs_paint = true;
        }
        if flags.contains(DirtyFlags::SEMANTICS) {
            state.needs_accessibility = true;
        }

        state
    }

    /// Record `viewport` and `theme` as committed and clear dirty flags on `tree`.
    pub fn commit(&mut self, tree: &mut RetainedTree, viewport: Size<Logical>, theme: &Theme) {
        self.last_viewport = Some(viewport);
        self.last_theme = Some(theme.clone());
        self.force_all = false;
        self.pending = DirtyState::clean();
        clear_tree_dirty_flags(tree);
    }

    /// Evaluate the current frame's [`DirtyState`] and commit `viewport` / `theme` / cleared flags.
    pub fn evaluate_and_commit(
        &mut self,
        tree: &mut RetainedTree,
        report: Option<&ReconcileReport>,
        viewport: Size<Logical>,
        theme: &Theme,
    ) -> DirtyState {
        let state = self.evaluate(tree, report, viewport, theme);
        self.commit(tree, viewport, theme);
        state
    }
}

fn collect_tree_dirty_flags(tree: &RetainedTree) -> DirtyFlags {
    let mut stack: Vec<NodeId> = tree.roots().to_vec();
    let mut visited = HashSet::new();
    let mut combined = DirtyFlags::NONE;

    while let Some(id) = stack.pop() {
        if !visited.insert(id) {
            continue;
        }
        if let Some(node) = tree.get(id) {
            combined.insert(node.dirty);
            stack.extend(node.children.iter().copied());
        }
    }

    combined
}

fn clear_tree_dirty_flags(tree: &mut RetainedTree) {
    let mut stack: Vec<NodeId> = tree.roots().to_vec();
    let mut visited = HashSet::new();

    while let Some(id) = stack.pop() {
        if !visited.insert(id) {
            continue;
        }
        if let Some(node) = tree.get(id) {
            stack.extend(node.children.iter().copied());
        }
        tree.clear_dirty(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WidgetNode;
    use crate::view_bridge::widget_to_views;
    use crate::{button, column, label};

    fn reconcile_frame<M>(
        tree: &mut RetainedTree,
        widgets: &[WidgetNode<M>],
    ) -> Result<ReconcileReport, acme_core::TreeError> {
        let views = widget_to_views(widgets);
        tree.reconcile_roots(&views)
    }

    #[test]
    fn initial_frame_is_all_dirty_then_unchanged_frame_is_clean() {
        let mut tree = RetainedTree::new();
        let mut tracker = FrameDirtyTracker::new();
        let viewport = Size::new(800.0, 600.0);
        let theme = Theme::dark();

        let widgets = vec![
            column::<()>()
                .key("root")
                .child(button::<()>("ok", "OK"))
                .build(),
        ];
        let rep1 = reconcile_frame(&mut tree, &widgets).unwrap();
        let s1 = tracker.evaluate_and_commit(&mut tree, Some(&rep1), viewport, &theme);
        assert_eq!(s1, DirtyState::all());

        // Second frame with identical tree, viewport, and theme => clean!
        let rep2 = reconcile_frame(&mut tree, &widgets).unwrap();
        let s2 = tracker.evaluate_and_commit(&mut tree, Some(&rep2), viewport, &theme);
        assert_eq!(s2, DirtyState::clean());
        assert!(s2.is_clean());
    }

    #[test]
    fn viewport_change_triggers_layout_paint_and_hit_rebuild() {
        let mut tree = RetainedTree::new();
        let mut tracker = FrameDirtyTracker::new();
        let theme = Theme::light();
        let widgets = vec![label::<()>("hello")];

        let rep = reconcile_frame(&mut tree, &widgets).unwrap();
        tracker.commit(&mut tree, Size::new(800.0, 600.0), &theme);

        let s = tracker.evaluate(&tree, Some(&rep), Size::new(1024.0, 768.0), &theme);
        assert!(s.needs_layout);
        assert!(s.needs_paint);
        assert!(s.needs_hit_rebuild);
    }

    #[test]
    fn theme_color_switch_triggers_paint_without_layout() {
        let mut tree = RetainedTree::new();
        let mut tracker = FrameDirtyTracker::new();
        let viewport = Size::new(800.0, 600.0);
        let light = Theme::light();
        let dark = Theme::dark();
        let widgets = vec![label::<()>("hello")];

        let rep = reconcile_frame(&mut tree, &widgets).unwrap();
        tracker.commit(&mut tree, viewport, &light);

        let s = tracker.evaluate(&tree, None, viewport, &dark);
        assert!(s.needs_paint);
        assert!(!s.needs_layout);
        assert!(!s.needs_hit_rebuild);
        assert!(rep.removed.is_empty());
    }

    #[test]
    fn paint_only_dirty_flag_gates_layout_and_hit_rebuild() {
        let mut tree = RetainedTree::new();
        let mut tracker = FrameDirtyTracker::new();
        let viewport = Size::new(800.0, 600.0);
        let theme = Theme::dark();
        let widgets = vec![WidgetNode::from(button::<()>("ok", "OK"))];

        let _ = reconcile_frame(&mut tree, &widgets).unwrap();
        tracker.commit(&mut tree, viewport, &theme);

        let root_id = tree.roots()[0];
        assert!(tree.mark_dirty(root_id, DirtyFlags::PAINT));

        let s = tracker.evaluate(&tree, None, viewport, &theme);
        assert!(s.needs_paint);
        assert!(!s.needs_layout);
        assert!(!s.needs_hit_rebuild);
        assert!(!s.needs_accessibility);
    }

    #[test]
    fn external_invalidation_methods_work_and_clear_on_commit() {
        let mut tree = RetainedTree::new();
        let mut tracker = FrameDirtyTracker::new();
        let viewport = Size::new(640.0, 480.0);
        let theme = Theme::dark();
        let widgets = vec![label::<()>("a")];

        let _ = reconcile_frame(&mut tree, &widgets).unwrap();
        tracker.commit(&mut tree, viewport, &theme);

        tracker.mark_hit_dirty();
        let s = tracker.evaluate(&tree, None, viewport, &theme);
        assert!(s.needs_hit_rebuild);
        assert!(!s.needs_layout);
        assert!(!s.needs_paint);

        tracker.commit(&mut tree, viewport, &theme);
        assert_eq!(
            tracker.evaluate(&tree, None, viewport, &theme),
            DirtyState::clean()
        );
    }
}
