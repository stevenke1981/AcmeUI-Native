//! Keyboard focus keyed by stable identity (ROADMAP 004 F1).
//!
//! `Gallery::focused` stays an index into the hit regions, because the render
//! pipeline counts regions in the same order. An index is only valid for the
//! frame that produced it, so the durable part of focus is a [`FocusTarget`].
//! After each frame rebuilds the regions, [`resolve_focus`] maps the target
//! back to an index in the new list.

use acme_core::NodeId;
use acme_widgets::retained_binding::is_synthetic;

use crate::types::{GalleryMessage, HitRegion};

/// Identity of a focused hit region: its layout node and its action.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FocusTarget {
    pub node: NodeId,
    pub message: GalleryMessage,
}

impl FocusTarget {
    /// The target that identifies `region`.
    pub fn of(region: &HitRegion) -> Self {
        Self {
            node: region.node,
            message: region.message,
        }
    }
}

/// Target of the region at `index`, or `None` when `index` is out of range.
pub fn target_at(regions: &[HitRegion], index: usize) -> Option<FocusTarget> {
    regions.get(index).map(FocusTarget::of)
}

/// Map the previous focus onto the current hit regions.
///
/// Returns the focused index and the target of the region at that index.
///
/// Rules, in order:
/// 1. Empty `regions`: index `0`, target `None`.
/// 2. A non-synthetic target node with a region of the same node *and* the same
///    message: that region. Synthetic IDs are per frame, so they are not
///    identity. The message check matters because one node can carry several
///    regions, such as a tree row's chevron and its label.
/// 3. A region with an equal message: that region. Messages such as
///    `TableSelectRow(orig)` are stable across table sorts.
/// 4. Otherwise `previous`, clamped into range.
pub fn resolve_focus(
    regions: &[HitRegion],
    target: Option<FocusTarget>,
    previous: usize,
) -> (usize, Option<FocusTarget>) {
    let Some(last) = regions.len().checked_sub(1) else {
        return (0, None);
    };
    let index = find_by_target(regions, target).unwrap_or_else(|| previous.min(last));
    (index, target_at(regions, index))
}

fn find_by_target(regions: &[HitRegion], target: Option<FocusTarget>) -> Option<usize> {
    let target = target?;
    if !is_synthetic(target.node) {
        let same_node = regions
            .iter()
            .position(|r| r.node == target.node && r.message == target.message);
        if same_node.is_some() {
            return same_node;
        }
    }
    regions.iter().position(|r| r.message == target.message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use acme_widgets::retained_binding::SYNTHETIC_ID_BASE;

    fn region(node: u64, message: GalleryMessage) -> HitRegion {
        HitRegion {
            rect: [0.0; 4],
            message,
            scrolled: false,
            node: NodeId::new(node),
        }
    }

    fn target(node: u64, message: GalleryMessage) -> Option<FocusTarget> {
        Some(FocusTarget {
            node: NodeId::new(node),
            message,
        })
    }

    #[test]
    fn empty_regions_resolve_to_zero_without_target() {
        let stale = target(3, GalleryMessage::SelectPage(0));
        assert_eq!(resolve_focus(&[], stale, 7), (0, None));
        assert_eq!(resolve_focus(&[], None, 0), (0, None));
    }

    #[test]
    fn non_synthetic_node_match_wins_even_when_the_index_moved() {
        let regions = [
            region(10, GalleryMessage::SelectPage(0)),
            region(11, GalleryMessage::SelectPage(1)),
            region(12, GalleryMessage::SelectPage(2)),
        ];
        let (index, resolved) =
            resolve_focus(&regions, target(12, GalleryMessage::SelectPage(2)), 0);
        assert_eq!(index, 2);
        assert_eq!(resolved, Some(FocusTarget::of(&regions[2])));
    }

    #[test]
    fn synthetic_node_is_not_identity_so_message_decides() {
        // Same synthetic node on a different action must not match.
        let synthetic = SYNTHETIC_ID_BASE + 5;
        let regions = [
            region(synthetic, GalleryMessage::SelectPage(0)),
            region(20, GalleryMessage::TableSelectRow(4)),
        ];
        let (index, _) = resolve_focus(
            &regions,
            target(synthetic, GalleryMessage::TableSelectRow(4)),
            0,
        );
        assert_eq!(index, 1);
    }

    #[test]
    fn shared_node_is_disambiguated_by_message() {
        // A tree row: chevron (toggle) and label (select) share one node.
        let regions = [
            region(5, GalleryMessage::TreeToggleKey("docs")),
            region(5, GalleryMessage::TreeSelectKey("docs")),
        ];
        let (index, resolved) = resolve_focus(
            &regions,
            target(5, GalleryMessage::TreeSelectKey("docs")),
            0,
        );
        assert_eq!(index, 1);
        assert_eq!(resolved, Some(FocusTarget::of(&regions[1])));
    }

    #[test]
    fn message_fallback_when_the_node_is_gone() {
        let regions = [
            region(30, GalleryMessage::SelectPage(1)),
            region(31, GalleryMessage::SelectCategory(4)),
        ];
        let (index, _) = resolve_focus(&regions, target(99, GalleryMessage::SelectCategory(4)), 0);
        assert_eq!(index, 1);
    }

    #[test]
    fn clamps_previous_index_when_the_target_is_gone() {
        let regions = [
            region(1, GalleryMessage::SelectPage(0)),
            region(2, GalleryMessage::SelectPage(1)),
            region(3, GalleryMessage::SelectPage(2)),
        ];
        let (index, resolved) = resolve_focus(&regions, target(99, GalleryMessage::ToggleTheme), 5);
        assert_eq!(index, 2);
        assert_eq!(resolved, Some(FocusTarget::of(&regions[2])));
    }

    #[test]
    fn no_target_keeps_an_in_range_previous_index() {
        let regions = [
            region(1, GalleryMessage::SelectPage(0)),
            region(2, GalleryMessage::SelectPage(1)),
            region(3, GalleryMessage::SelectPage(2)),
        ];
        let (index, resolved) = resolve_focus(&regions, None, 1);
        assert_eq!(index, 1);
        assert_eq!(resolved, Some(FocusTarget::of(&regions[1])));
    }

    #[test]
    fn target_at_is_none_out_of_range() {
        let regions = [region(1, GalleryMessage::ToggleTheme)];
        assert!(target_at(&regions, 0).is_some());
        assert_eq!(target_at(&regions, 1), None);
    }
}
