use std::collections::{HashMap, HashSet};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(u64);
impl NodeId {
    pub fn get(self) -> u64 {
        self.0
    }
    pub fn new(value: u64) -> Self {
        Self(value)
    }
}
/// Stable widget identifier — cheap to clone via `Arc<str>`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct WidgetKey(Arc<str>);
impl WidgetKey {
    /// Create a key from any string-like type.
    pub fn new(value: impl AsRef<str>) -> Self {
        Self(Arc::from(value.as_ref()))
    }
    /// Borrow the key as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl From<&str> for WidgetKey {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}
impl From<String> for WidgetKey {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DirtyFlags(u8);
impl DirtyFlags {
    pub const NONE: Self = Self(0);
    pub const STYLE: Self = Self(1);
    pub const LAYOUT: Self = Self(2);
    pub const PAINT: Self = Self(4);
    pub const SEMANTICS: Self = Self(8);
    pub const CHILDREN: Self = Self(16);
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
    pub fn insert(&mut self, other: Self) {
        self.0 |= other.0
    }
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}
impl std::ops::BitOr for DirtyFlags {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ViewNode {
    pub key: WidgetKey,
    pub kind: String,
    pub children: Vec<ViewNode>,
    pub focusable: bool,
    pub disabled: bool,
}
impl ViewNode {
    pub fn new(key: impl Into<WidgetKey>, kind: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            kind: kind.into(),
            children: vec![],
            focusable: false,
            disabled: false,
        }
    }
    pub fn child(mut self, child: Self) -> Self {
        self.children.push(child);
        self
    }
}
#[derive(Clone, Debug)]
pub struct Node {
    pub id: NodeId,
    pub key: WidgetKey,
    pub kind: String,
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,
    pub dirty: DirtyFlags,
    pub focusable: bool,
    pub disabled: bool,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReconcileReport {
    pub mounted: Vec<NodeId>,
    pub reused: Vec<NodeId>,
    pub removed: Vec<NodeId>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TreeError {
    DuplicateSiblingKey(WidgetKey),
    UnknownParent(NodeId),
}

#[derive(Default)]
pub struct RetainedTree {
    next: u64,
    roots: Vec<NodeId>,
    nodes: HashMap<NodeId, Node>,
}
impl RetainedTree {
    pub fn new() -> Self {
        Self {
            next: 1,
            ..Self::default()
        }
    }
    pub fn roots(&self) -> &[NodeId] {
        &self.roots
    }
    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }
    pub fn contains(&self, id: NodeId) -> bool {
        self.nodes.contains_key(&id)
    }
    pub fn reconcile_roots(&mut self, views: &[ViewNode]) -> Result<ReconcileReport, TreeError> {
        validate_keys(views)?;
        let old = self.roots.clone();
        let (ids, r) = self.reconcile_children(None, &old, views)?;
        self.roots = ids;
        Ok(r)
    }
    pub fn mark_dirty(&mut self, id: NodeId, flags: DirtyFlags) -> bool {
        if !self.nodes.contains_key(&id) {
            return false;
        }
        let mut cur = Some(id);
        while let Some(nid) = cur {
            let Some(node) = self.nodes.get_mut(&nid) else {
                break;
            };
            node.dirty.insert(flags);
            cur = if flags.contains(DirtyFlags::LAYOUT) || flags.contains(DirtyFlags::CHILDREN) {
                node.parent
            } else {
                None
            };
        }
        true
    }
    pub fn clear_dirty(&mut self, id: NodeId) -> bool {
        if let Some(n) = self.nodes.get_mut(&id) {
            n.dirty = DirtyFlags::NONE;
            true
        } else {
            false
        }
    }
    fn reconcile_children(
        &mut self,
        parent: Option<NodeId>,
        old: &[NodeId],
        views: &[ViewNode],
    ) -> Result<(Vec<NodeId>, ReconcileReport), TreeError> {
        let mut seen = HashSet::new();
        for v in views {
            if !seen.insert(v.key.clone()) {
                return Err(TreeError::DuplicateSiblingKey(v.key.clone()));
            }
        }
        let old_by_key: HashMap<_, _> = old
            .iter()
            .filter_map(|id| self.nodes.get(id).map(|n| (n.key.clone(), *id)))
            .collect();
        let mut result = ReconcileReport::default();
        let mut ids = Vec::new();
        for view in views {
            let id = if let Some(id) = old_by_key.get(&view.key).copied() {
                result.reused.push(id);
                id
            } else {
                let id = NodeId(self.next);
                self.next += 1;
                self.nodes.insert(
                    id,
                    Node {
                        id,
                        key: view.key.clone(),
                        kind: view.kind.clone(),
                        parent,
                        children: vec![],
                        dirty: DirtyFlags::STYLE
                            | DirtyFlags::LAYOUT
                            | DirtyFlags::PAINT
                            | DirtyFlags::SEMANTICS
                            | DirtyFlags::CHILDREN,
                        focusable: view.focusable,
                        disabled: view.disabled,
                    },
                );
                result.mounted.push(id);
                id
            };
            let old_children = self.nodes[&id].children.clone();
            let (children, sub) =
                self.reconcile_children(Some(id), &old_children, &view.children)?;
            merge(&mut result, sub);
            let node = self
                .nodes
                .get_mut(&id)
                .expect("node just inserted or found in old_by_key");
            if node.kind != view.kind
                || node.focusable != view.focusable
                || node.disabled != view.disabled
            {
                node.dirty.insert(
                    DirtyFlags::STYLE
                        | DirtyFlags::LAYOUT
                        | DirtyFlags::PAINT
                        | DirtyFlags::SEMANTICS,
                )
            }
            node.kind = view.kind.clone();
            node.parent = parent;
            node.children = children;
            node.focusable = view.focusable;
            node.disabled = view.disabled;
            ids.push(id);
        }
        let ids_set: HashSet<NodeId> = ids.iter().copied().collect();
        for id in old {
            if !ids_set.contains(id) {
                self.remove_subtree(*id, &mut result.removed)
            }
        }
        Ok((ids, result))
    }
    fn remove_subtree(&mut self, id: NodeId, removed: &mut Vec<NodeId>) {
        if let Some(n) = self.nodes.remove(&id) {
            for c in n.children {
                self.remove_subtree(c, removed)
            }
            removed.push(id)
        }
    }
}

fn validate_keys(views: &[ViewNode]) -> Result<(), TreeError> {
    let mut seen = HashSet::new();
    for view in views {
        if !seen.insert(view.key.clone()) {
            return Err(TreeError::DuplicateSiblingKey(view.key.clone()));
        }
        validate_keys(&view.children)?;
    }
    Ok(())
}
fn merge(a: &mut ReconcileReport, b: ReconcileReport) {
    a.mounted.extend(b.mounted);
    a.reused.extend(b.reused);
    a.removed.extend(b.removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn keyed_reorder_preserves_ids_and_removes_atomically() {
        let mut t = RetainedTree::new();
        t.reconcile_roots(&[
            ViewNode::new("a", "x").child(ViewNode::new("c", "x")),
            ViewNode::new("b", "x"),
        ])
        .unwrap();
        let a = t.roots()[0];
        let c = t.get(a).unwrap().children[0];
        let b = t.roots()[1];
        let r = t.reconcile_roots(&[ViewNode::new("b", "x")]).unwrap();
        assert_eq!(t.roots(), &[b]);
        assert!(r.removed.contains(&a) && r.removed.contains(&c));
        assert!(!t.contains(c));
    }
    #[test]
    fn duplicate_keys_are_rejected() {
        let mut t = RetainedTree::new();
        assert!(matches!(
            t.reconcile_roots(&[ViewNode::new("x", "a"), ViewNode::new("x", "b")]),
            Err(TreeError::DuplicateSiblingKey(_))
        ));
    }
    #[test]
    fn nested_duplicate_failure_does_not_mutate_tree() {
        let mut t = RetainedTree::new();
        t.reconcile_roots(&[ViewNode::new("root", "old")]).unwrap();
        let root = t.roots()[0];
        let result = t.reconcile_roots(&[ViewNode::new("new", "new")
            .child(ViewNode::new("same", "x"))
            .child(ViewNode::new("same", "x"))]);
        assert!(matches!(result, Err(TreeError::DuplicateSiblingKey(_))));
        assert_eq!(t.roots(), &[root]);
        assert_eq!(t.get(root).unwrap().kind, "old");
    }
    #[test]
    fn layout_dirty_propagates_but_paint_does_not() {
        let mut t = RetainedTree::new();
        t.reconcile_roots(&[ViewNode::new("p", "x").child(ViewNode::new("c", "x"))])
            .unwrap();
        let p = t.roots()[0];
        let c = t.get(p).unwrap().children[0];
        t.clear_dirty(p);
        t.clear_dirty(c);
        t.mark_dirty(c, DirtyFlags::PAINT);
        assert!(t.get(c).unwrap().dirty.contains(DirtyFlags::PAINT));
        assert!(t.get(p).unwrap().dirty.is_empty());
        t.mark_dirty(c, DirtyFlags::LAYOUT);
        assert!(t.get(p).unwrap().dirty.contains(DirtyFlags::LAYOUT));
    }
}
