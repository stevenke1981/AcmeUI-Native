//! Headless retained-tree update cycle: cargo run -p acme-core --example reconcile_dirty
use acme_core::{DirtyFlags, RetainedTree, ViewNode};

fn main() {
    let view = |reversed: bool| {
        let keys = if reversed { ["b", "a"] } else { ["a", "b"] };
        ViewNode::new("list", "column")
            .child(ViewNode::new(keys[0], "button"))
            .child(ViewNode::new(keys[1], "button"))
    };
    let mut tree = RetainedTree::new();
    let first = tree.reconcile_roots(&[view(false)]).unwrap();
    let root = tree.roots()[0];
    let original = tree.get(root).unwrap().children.clone();
    for id in first.mounted {
        tree.clear_dirty(id);
    }

    let reordered = tree.reconcile_roots(&[view(true)]).unwrap();
    assert!(reordered.mounted.is_empty());
    assert!(reordered.removed.is_empty());
    assert_eq!(
        tree.get(root).unwrap().children,
        vec![original[1], original[0]]
    );
    assert!(tree.get(root).unwrap().dirty.contains(
        DirtyFlags::CHILDREN | DirtyFlags::LAYOUT | DirtyFlags::PAINT | DirtyFlags::SEMANTICS
    ));
    for id in reordered.reused {
        tree.clear_dirty(id);
    }

    let repeated = tree.reconcile_roots(&[view(true)]).unwrap();
    for id in repeated.reused {
        assert!(tree.get(id).unwrap().dirty.is_empty());
    }
    println!(
        "Reorder invalidated the list, preserved child IDs, and the repeated frame stayed clean."
    );
}
