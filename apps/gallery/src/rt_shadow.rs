//! Retained-tree step of the Gallery frame pipeline (ROADMAP 003).
//!
//! Every frame the declarative widget tree is converted into `ViewNode`s and
//! reconciled into the Gallery's `RetainedTree`. On success the frame binds the
//! description to the retained node IDs (`acme_widgets::retained_binding`), so
//! layout, hit regions and accessibility share one stable `NodeId` per widget
//! across frames. On a reconcile error the frame falls back to the sequential
//! layout IDs; the error is logged once until a later reconcile succeeds.

use acme_core::{ReconcileReport, RetainedTree};
use acme_widgets::WidgetNode;
use acme_widgets::view_bridge::widget_to_views;

use crate::types::GalleryMessage;

/// Owns the retained tree and the "error already logged" latch.
#[derive(Default)]
pub struct RetainedStep {
    tree: RetainedTree,
    error_logged: bool,
    mismatch_logged: bool,
}

impl RetainedStep {
    pub fn new() -> Self {
        Self {
            tree: RetainedTree::new(),
            error_logged: false,
            mismatch_logged: false,
        }
    }

    /// The retained tree, reconciled from the most recent successful frame.
    pub fn tree(&self) -> &RetainedTree {
        &self.tree
    }

    /// Decide whether a binding with `mismatches` structural disagreements may
    /// drive the frame. Returns `false` (and logs once per run of failing
    /// frames) when the binding is partial.
    pub fn accept_binding(&mut self, mismatches: usize) -> bool {
        if mismatches == 0 {
            self.mismatch_logged = false;
            return true;
        }
        if !self.mismatch_logged {
            tracing::warn!(
                "retained binding has {mismatches} mismatches, using sequential layout IDs"
            );
            self.mismatch_logged = true;
        }
        false
    }

    /// Reconcile `description` into the retained tree.
    ///
    /// Returns the counts on success. On failure, returns the error text and
    /// logs it only the first time in a run of failing frames.
    pub fn reconcile(
        &mut self,
        description: &WidgetNode<GalleryMessage>,
    ) -> Result<ReconcileCounts, String> {
        let views = widget_to_views(std::slice::from_ref(description));
        match self.tree.reconcile_roots(&views) {
            Ok(report) => {
                self.error_logged = false;
                let counts = ReconcileCounts::from_report(&report);
                tracing::trace!(
                    "retained tree: {} mounted, {} reused, {} removed",
                    counts.mounted,
                    counts.reused,
                    counts.removed,
                );
                Ok(counts)
            }
            Err(error) => {
                let message = format!("{error:?}");
                if !self.error_logged {
                    tracing::warn!(
                        "retained reconcile failed, using sequential layout IDs: {message}"
                    );
                    self.error_logged = true;
                }
                Err(message)
            }
        }
    }
}

/// Mount/reuse/remove counts of one reconcile.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ReconcileCounts {
    pub mounted: usize,
    pub reused: usize,
    pub removed: usize,
}

impl ReconcileCounts {
    fn from_report(report: &ReconcileReport) -> Self {
        Self {
            mounted: report.mounted.len(),
            reused: report.reused.len(),
            removed: report.removed.len(),
        }
    }
}
