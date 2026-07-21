//! RuntimeTree shadow reconcile runner (Phase 1 — diagnostics only).
//!
//! This module converts the Gallery's declarative widget tree into `ViewNode`s
//! and feeds them to a `RetainedTree` shadow. No functional changes to layout,
//! paint, hit-testing, or accessibility — purely for diagnostic data collection.

use acme_core::{RetainedTree, ReconcileReport};
use acme_widgets::view_bridge::widget_to_views;
use acme_widgets::WidgetNode;

use crate::types::GalleryMessage;

/// Run the shadow reconcile cycle for one frame.
///
/// Converts `description` to a `ViewNode` forest, reconciles against the
/// `shadow_tree`, and logs mount/reuse/remove counts.
///
/// # Returns
/// A short diagnostics string suitable for tracing.
pub fn run_shadow_reconcile(
    shadow_tree: &mut RetainedTree,
    description: &WidgetNode<GalleryMessage>,
) -> ShadowReport {
    let views = widget_to_views(std::slice::from_ref(description));
    match shadow_tree.reconcile_roots(&views) {
        Ok(report) => {
            let summary = ShadowReport::from_report(&report);
            tracing::trace!(
                "RT shadow: {} mounted, {} reused, {} removed, {} total nodes",
                summary.mounted,
                summary.reused,
                summary.removed,
                summary.total,
            );
            summary
        }
        Err(err) => {
            tracing::warn!("RT shadow reconcile error: {err:?}");
            ShadowReport {
                mounted: 0,
                reused: 0,
                removed: 0,
                total: shadow_tree.roots().len(),
                error: Some(format!("{err:?}")),
            }
        }
    }
}

/// Summary of a single shadow reconcile cycle.
#[derive(Clone, Debug, Default)]
pub struct ShadowReport {
    /// Number of nodes newly mounted this frame.
    pub mounted: usize,
    /// Number of nodes reused from a previous frame.
    pub reused: usize,
    /// Number of nodes removed this frame.
    pub removed: usize,
    /// Total nodes currently in the shadow tree.
    pub total: usize,
    /// Error message if reconciliation failed.
    pub error: Option<String>,
}

impl ShadowReport {
    fn from_report(r: &ReconcileReport) -> Self {
        Self {
            mounted: r.mounted.len(),
            reused: r.reused.len(),
            removed: r.removed.len(),
            total: 0, // filled by caller
            error: None,
        }
    }
}
