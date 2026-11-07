//! Binding between a frame's [`WidgetNode`] description and the stable node
//! identities of an [`acme_core::RetainedTree`].
//!
//! The retained tree is reconciled from [`crate::view_bridge::widget_to_views`]
//! every frame. [`bind_retained`] then walks the *same* borrowed description in
//! the same order and records which retained [`NodeId`] belongs to each widget.
//! Layout ([`WidgetNode::to_layout_with_binding`]) and accessibility consume the
//! binding so all of them share one stable ID per widget across frames.
//!
//! Widgets are identified by their address inside the borrowed description.
//! The binding's lifetime is tied to that borrow, so it cannot outlive or be
//! applied to a description that was dropped or mutated. Widgets that are not
//! part of the description (for example rows synthesised by data widgets during
//! layout) have no binding and receive IDs from [`SyntheticIds`].

use std::collections::HashMap;
use std::marker::PhantomData;

use acme_core::{NodeId, RetainedTree};

use crate::WidgetNode;
use crate::view_bridge::view_child_widgets;

/// First ID of the per-frame synthetic range. Retained IDs are allocated from 1
/// upward and must stay below this value; synthetic IDs never collide with them.
pub const SYNTHETIC_ID_BASE: u64 = 1 << 48;

/// Stable retained IDs for the widgets of one borrowed description.
#[derive(Debug)]
pub struct RetainedBinding<'a> {
    ids: HashMap<usize, NodeId>,
    mismatches: usize,
    _description: PhantomData<&'a ()>,
}

impl<'a> RetainedBinding<'a> {
    /// The retained ID bound to `widget`, if `widget` belongs to the bound
    /// description.
    pub fn id_for<M>(&self, widget: &WidgetNode<M>) -> Option<NodeId> {
        self.ids.get(&widget_address(widget)).copied()
    }

    /// Number of widgets that received a retained ID.
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// `true` when no widget is bound.
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// Number of structural disagreements found while binding (missing root,
    /// child-count difference, or a retained ID inside the synthetic range).
    /// Zero means every widget of the description was paired with its node.
    pub fn mismatches(&self) -> usize {
        self.mismatches
    }
}

/// Bind every widget of `roots` to the node that `tree` holds for it.
///
/// Call this right after `tree.reconcile_roots(&widget_to_views(roots))`
/// succeeded for the same `roots`. When the shapes disagree (for example the
/// reconcile failed and the tree still holds an older frame), unpaired widgets
/// are left unbound and counted in [`RetainedBinding::mismatches`].
pub fn bind_retained<'a, M>(
    roots: &'a [WidgetNode<M>],
    tree: &RetainedTree,
) -> RetainedBinding<'a> {
    let mut binding = RetainedBinding {
        ids: HashMap::new(),
        mismatches: 0,
        _description: PhantomData,
    };
    let tree_roots = tree.roots();
    if tree_roots.len() != roots.len() {
        binding.mismatches += 1;
    }
    for (widget, &id) in roots.iter().zip(tree_roots) {
        bind_node(widget, id, tree, &mut binding);
    }
    binding
}

fn bind_node<M>(
    widget: &WidgetNode<M>,
    id: NodeId,
    tree: &RetainedTree,
    binding: &mut RetainedBinding<'_>,
) {
    let Some(node) = tree.get(id) else {
        binding.mismatches += 1;
        return;
    };
    if id.get() >= SYNTHETIC_ID_BASE {
        binding.mismatches += 1;
        return;
    }
    binding.ids.insert(widget_address(widget), id);
    let children = view_child_widgets(widget);
    if children.len() != node.children.len() {
        binding.mismatches += 1;
    }
    for (child, &child_id) in children.into_iter().zip(&node.children) {
        bind_node(child, child_id, tree, binding);
    }
}

fn widget_address<M>(widget: &WidgetNode<M>) -> usize {
    std::ptr::from_ref(widget) as usize
}

/// Per-frame allocator for IDs of nodes that have no retained identity.
///
/// IDs start at [`SYNTHETIC_ID_BASE`] and increase in traversal order, so they
/// are deterministic for a given description but not stable across shape
/// changes.
#[derive(Debug)]
pub struct SyntheticIds {
    next: u64,
}

impl SyntheticIds {
    /// A fresh allocator starting at [`SYNTHETIC_ID_BASE`].
    pub fn new() -> Self {
        Self {
            next: SYNTHETIC_ID_BASE,
        }
    }

    /// Allocate the next synthetic ID.
    pub fn allocate(&mut self) -> NodeId {
        let id = NodeId::new(self.next);
        self.next += 1;
        id
    }
}

impl Default for SyntheticIds {
    fn default() -> Self {
        Self::new()
    }
}

/// `true` when `id` belongs to the per-frame synthetic range.
pub fn is_synthetic(id: NodeId) -> bool {
    id.get() >= SYNTHETIC_ID_BASE
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_bridge::widget_to_views;
    use crate::{button, column, label, row, tooltip};

    fn reconcile<M>(tree: &mut RetainedTree, roots: &[WidgetNode<M>]) {
        tree.reconcile_roots(&widget_to_views(roots))
            .expect("reconcile");
    }

    fn keyed_row(order: &[&str]) -> WidgetNode<()> {
        let mut r = row::<()>();
        for key in order {
            r = r.child(button::<()>(*key, *key));
        }
        r.into()
    }

    fn button_ids(binding: &RetainedBinding<'_>, root: &WidgetNode<()>) -> Vec<NodeId> {
        let WidgetNode::Row(r) = root else {
            panic!("expected row");
        };
        r.children
            .iter()
            .map(|c| binding.id_for(c).expect("bound"))
            .collect()
    }

    #[test]
    fn binds_every_view_node_without_mismatch() {
        let mut tree = RetainedTree::new();
        let roots = vec![
            column::<()>()
                .child(label::<()>("Title"))
                .child(tooltip("tip", button::<()>("ok", "OK"), "help"))
                .into(),
        ];
        reconcile(&mut tree, &roots);
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0);
        // column, label, tooltip, button
        assert_eq!(binding.len(), 4);
    }

    #[test]
    fn keyed_reorder_keeps_ids_with_their_widgets() {
        let mut tree = RetainedTree::new();
        let first = vec![keyed_row(&["a", "b", "c"])];
        reconcile(&mut tree, &first);
        let before = button_ids(&bind_retained(&first, &tree), &first[0]);

        let second = vec![keyed_row(&["c", "a", "b"])];
        reconcile(&mut tree, &second);
        let after = button_ids(&bind_retained(&second, &tree), &second[0]);

        assert_eq!(after, vec![before[2], before[0], before[1]]);
    }

    #[test]
    fn insertion_before_existing_keys_does_not_shift_their_ids() {
        let mut tree = RetainedTree::new();
        let first = vec![keyed_row(&["a", "b"])];
        reconcile(&mut tree, &first);
        let before = button_ids(&bind_retained(&first, &tree), &first[0]);

        let second = vec![keyed_row(&["new", "a", "b"])];
        reconcile(&mut tree, &second);
        let after = button_ids(&bind_retained(&second, &tree), &second[0]);

        assert_eq!(&after[1..], &before[..]);
        assert!(!before.contains(&after[0]));
    }

    #[test]
    fn stale_tree_is_reported_as_mismatch() {
        let mut tree = RetainedTree::new();
        let first = vec![keyed_row(&["a"])];
        reconcile(&mut tree, &first);
        let changed = vec![keyed_row(&["a", "b"])];
        let binding = bind_retained(&changed, &tree);
        assert!(binding.mismatches() > 0);
    }

    #[test]
    fn synthetic_ids_never_collide_with_retained_ids() {
        let mut ids = SyntheticIds::new();
        let a = ids.allocate();
        let b = ids.allocate();
        assert!(is_synthetic(a) && is_synthetic(b));
        assert_ne!(a, b);
        assert!(!is_synthetic(NodeId::new(SYNTHETIC_ID_BASE - 1)));
    }

    #[test]
    fn view_child_widgets_matches_view_bridge_child_counts() {
        fn check(widget: &WidgetNode<()>, view: &acme_core::ViewNode) {
            let children = view_child_widgets(widget);
            assert_eq!(children.len(), view.children.len(), "kind {}", view.kind);
            for (c, v) in children.into_iter().zip(&view.children) {
                check(c, v);
            }
        }
        let root: WidgetNode<()> = column::<()>()
            .child(label::<()>("x"))
            .child(keyed_row(&["a", "b"]))
            .child(tooltip("tip", button::<()>("ok", "OK"), "help"))
            .into();
        let views = widget_to_views(std::slice::from_ref(&root));
        check(&root, &views[0]);
    }
}
