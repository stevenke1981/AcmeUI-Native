//! Headless focus lifecycle demo: cargo run -p acme-core --example focus_navigation
use acme_core::{FocusManager, NodeId, PointerId};

fn main() {
    let search = NodeId::new(1);
    let save = NodeId::new(2);
    let close = NodeId::new(3);
    let mut focus = FocusManager::new();
    focus.set_order([(search, true), (search, true), (save, false), (close, true)]);
    for expected in [search, close, search] {
        assert_eq!(focus.focus_next(false), Some(expected));
    }
    assert_eq!(focus.focus_next(true), Some(close));

    // Rebuilding the ring preserves live focus and applies the new order.
    focus.set_order([(close, true), (save, true), (search, true)]);
    assert_eq!(focus.focused(), Some(close));
    assert_eq!(focus.focus_next(false), Some(save));

    // Pointer capture is independent of the tab order, but not of node lifetime.
    let pointer = PointerId(1);
    focus.capture_pointer(pointer, save);
    focus.set_order([(close, true), (search, true)]);
    assert_eq!(focus.focused(), None);
    assert_eq!(focus.pointer_capture(pointer), Some(save));
    focus.remove_node(save);
    assert_eq!(focus.pointer_capture(pointer), None);
    assert_eq!(focus.focus_next(true), Some(search));
    println!("Focus traversal, rebuilding and unmount cleanup completed.");
}
