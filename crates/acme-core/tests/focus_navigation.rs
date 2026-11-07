//! Public-API regressions for keyboard focus and pointer-capture independence.
use acme_core::{FocusManager, NodeId, PointerId};

fn ids() -> [NodeId; 3] {
    [NodeId::new(1), NodeId::new(2), NodeId::new(3)]
}

#[test]
fn duplicate_ids_do_not_trap_forward_or_reverse_navigation() {
    let [a, b, c] = ids();
    let mut focus = FocusManager::new();
    focus.set_order([
        (a, true),
        (a, true),
        (b, true),
        (b, true),
        (c, true),
        (a, true),
    ]);
    for expected in [a, b, c, a, b, c] {
        assert_eq!(focus.focus_next(false), Some(expected));
    }
    focus.clear_focus();
    for expected in [c, b, a, c, b, a] {
        assert_eq!(focus.focus_next(true), Some(expected));
    }
}

#[test]
fn mixed_duplicates_use_first_eligible_occurrence() {
    let [a, b, c] = ids();
    let mut focus = FocusManager::new();
    focus.set_order([(a, false), (b, true), (a, true), (c, false), (b, false)]);
    assert!(!focus.request_focus(c));
    for expected in [b, a, b, a] {
        assert_eq!(focus.focus_next(false), Some(expected));
    }
    assert!(focus.request_focus(b));
}

#[test]
fn rebuilding_the_order_preserves_eligible_focus_and_uses_new_neighbors() {
    let [a, b, c] = ids();
    let mut focus = FocusManager::new();
    focus.set_order([(a, true), (b, true), (c, true)]);
    assert!(focus.request_focus(b));
    focus.set_order([(c, true), (b, true), (a, true)]);
    assert_eq!(focus.focused(), Some(b));
    assert_eq!(focus.focus_next(false), Some(a));
    assert_eq!(focus.focus_next(true), Some(b));
    focus.set_order([(c, true), (b, true), (a, true)]);
    assert_eq!(focus.focused(), Some(b));
}

#[test]
fn disabling_the_focused_node_resets_navigation() {
    let [a, b, c] = ids();
    let mut focus = FocusManager::new();
    focus.set_order([(a, true), (b, true), (c, true)]);
    assert!(focus.request_focus(b));
    focus.set_order([(a, true), (b, false), (c, true)]);
    assert_eq!(focus.focused(), None);
    assert!(!focus.request_focus(b));
    assert_eq!(focus.focus_next(false), Some(a));
    focus.clear_focus();
    assert_eq!(focus.focus_next(true), Some(c));
}

#[test]
fn rebuilding_the_order_drops_old_eligibility() {
    let [a, b, c] = ids();
    let mut focus = FocusManager::new();
    focus.set_order([(a, true), (b, true)]);
    assert!(focus.request_focus(a));
    focus.set_order([(c, true)]);
    assert_eq!(focus.focused(), None);
    assert!(!focus.request_focus(a));
    assert!(!focus.request_focus(b));
    assert_eq!(focus.focus_next(false), Some(c));
    assert_eq!(focus.focus_next(true), Some(c));
}

#[test]
fn ineligible_and_unknown_focus_requests_preserve_current_focus() {
    let [a, b, c] = ids();
    let mut focus = FocusManager::new();
    focus.set_order([(a, true), (b, false)]);
    assert!(focus.request_focus(a));
    assert!(!focus.request_focus(b));
    assert!(!focus.request_focus(c));
    assert_eq!(focus.focused(), Some(a));
}

#[test]
fn empty_disabled_and_singleton_orders_are_safe() {
    let [a, b, _] = ids();
    let mut focus = FocusManager::new();
    assert_eq!(focus.focus_next(false), None);
    assert_eq!(focus.focus_next(true), None);
    focus.set_order([(a, true), (a, true), (b, false)]);
    for reverse in [false, true, false, true] {
        assert_eq!(focus.focus_next(reverse), Some(a));
    }
    focus.set_order([(a, false), (b, false)]);
    assert_eq!(focus.focused(), None);
    assert_eq!(focus.focus_next(false), None);
    assert_eq!(focus.focus_next(true), None);
    focus.set_order([]);
    assert_eq!(focus.focus_next(false), None);
}

#[test]
fn tab_order_changes_do_not_release_nonfocusable_pointer_capture() {
    let [a, b, _] = ids();
    let pointer = PointerId(7);
    let mut focus = FocusManager::new();
    focus.set_order([(a, true), (b, false)]);
    focus.capture_pointer(pointer, b);
    assert!(focus.request_focus(a));
    focus.clear_focus();
    assert_eq!(focus.pointer_capture(pointer), Some(b));
    focus.set_order([]);
    assert_eq!(focus.pointer_capture(pointer), Some(b));
    focus.remove_node(b);
    assert_eq!(focus.pointer_capture(pointer), None);
}

#[test]
fn removing_a_focused_node_cleans_all_of_its_captures_only() {
    let [a, b, c] = ids();
    let mut focus = FocusManager::new();
    focus.set_order([(a, true), (b, true), (b, true), (c, true)]);
    assert!(focus.request_focus(b));
    focus.capture_pointer(PointerId(1), b);
    focus.capture_pointer(PointerId(2), b);
    focus.capture_pointer(PointerId(3), c);
    focus.remove_node(b);
    focus.remove_node(b);
    assert_eq!(focus.focused(), None);
    assert!(!focus.request_focus(b));
    assert_eq!(focus.pointer_capture(PointerId(1)), None);
    assert_eq!(focus.pointer_capture(PointerId(2)), None);
    assert_eq!(focus.pointer_capture(PointerId(3)), Some(c));
    assert_eq!(focus.focus_next(false), Some(a));
    assert_eq!(focus.focus_next(false), Some(c));
    focus.release_pointer(PointerId(3));
    assert_eq!(focus.pointer_capture(PointerId(3)), None);
}

#[test]
fn removing_a_different_node_keeps_the_current_focus() {
    let [a, b, c] = ids();
    let mut focus = FocusManager::new();
    focus.set_order([(a, true), (b, true), (c, true)]);
    assert!(focus.request_focus(b));
    focus.remove_node(a);
    assert_eq!(focus.focused(), Some(b));
    assert_eq!(focus.focus_next(false), Some(c));
    assert_eq!(focus.focus_next(false), Some(b));
    focus.remove_node(NodeId::new(99));
    assert_eq!(focus.focused(), Some(b));
}

#[test]
fn unique_orders_match_the_reference_for_every_eligibility_mask() {
    let nodes = [1, 2, 3, 4, 5].map(NodeId::new);
    for mask in 0_u32..32 {
        let items: Vec<_> = nodes
            .iter()
            .enumerate()
            .map(|(index, &id)| (id, mask & (1 << index) != 0))
            .collect();
        let eligible: Vec<_> = items
            .iter()
            .filter_map(|&(id, enabled)| enabled.then_some(id))
            .collect();
        for initial in std::iter::once(None).chain(nodes.map(Some)) {
            for reverse in [false, true] {
                let mut focus = FocusManager::new();
                focus.set_order(items.iter().copied());
                if let Some(id) = initial {
                    assert_eq!(focus.request_focus(id), eligible.contains(&id));
                }
                let mut expected = initial.filter(|id| eligible.contains(id));
                for _ in 0..12 {
                    expected = reference_step(&eligible, expected, reverse);
                    assert_eq!(focus.focus_next(reverse), expected, "mask={mask}");
                    assert_eq!(focus.focused(), expected);
                }
            }
        }
    }
}

fn reference_step(order: &[NodeId], current: Option<NodeId>, reverse: bool) -> Option<NodeId> {
    let mut traversal = order.to_vec();
    if reverse {
        traversal.reverse();
    }
    let first = traversal.first().copied()?;
    let mut after_current = false;
    for id in traversal {
        if after_current {
            return Some(id);
        }
        after_current = Some(id) == current;
    }
    Some(first)
}
