use std::collections::HashSet;

use crate::WidgetNode;
use acme_core::WidgetKey;

// ============================================================================
// TreeNode
// ============================================================================

/// A single node in a tree widget.
///
/// `content` is the widget displayed for this node (typically a `Label` or
/// a `Row` with an expand chevron + label).  Children form the subtree.
#[derive(Clone, Debug, PartialEq)]
pub struct TreeNode<M> {
    pub key: WidgetKey,
    /// The widget rendered for this node (label, row, etc.).
    pub content: WidgetNode<M>,
    /// Child nodes.
    pub children: Vec<TreeNode<M>>,
    /// Whether this node is expanded by default (used when no explicit
    /// `expanded` set is maintained in the parent `Tree`).
    pub expanded: bool,
}

impl<M> TreeNode<M> {
    /// Create a new tree node with a content widget.
    pub fn new(key: impl Into<WidgetKey>, content: impl Into<WidgetNode<M>>) -> Self {
        Self {
            key: key.into(),
            content: content.into(),
            children: Vec::new(),
            expanded: false,
        }
    }

    /// Add a child node.
    pub fn child(mut self, node: TreeNode<M>) -> Self {
        self.children.push(node);
        self
    }

    /// Set default expanded state.
    pub fn expanded(mut self, value: bool) -> Self {
        self.expanded = value;
        self
    }
}

// ============================================================================
// Tree
// ============================================================================

/// A hierarchical tree widget with expand/collapse, selection, and keyboard
/// navigation.
///
/// # Virtualization
/// Only visible (expanded + within viewport) nodes enter the layout tree.
/// Non-visible nodes are skipped during `to_layout`.
///
/// # Keyboard navigation (when focused)
/// - ArrowDown / ArrowUp → next / previous visible node
/// - ArrowRight → expand selected node
/// - ArrowLeft → collapse selected node
/// - Home → first visible node
/// - End → last visible node
/// - Typeahead: letter keys jump to next matching visible node
pub struct Tree<M> {
    pub key: WidgetKey,
    /// Top-level tree nodes.
    pub children: Vec<TreeNode<M>>,
    /// Set of expanded node keys (overrides `TreeNode::expanded`).
    pub expanded: HashSet<WidgetKey>,
    /// Currently selected node key.
    pub selected: Option<WidgetKey>,
    /// Indent (in logical pixels) per depth level.  Default 20.
    pub indent: f32,
    /// Current scroll offset.
    pub scroll_offset: f32,
    /// Viewport height for virtualization.
    pub viewport_height: f32,
    /// Overscan node count.
    pub overscan: usize,
}

// Manual Clone / Debug / PartialEq

impl<M: Clone> Clone for Tree<M> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            children: self.children.clone(),
            expanded: self.expanded.clone(),
            selected: self.selected.clone(),
            indent: self.indent,
            scroll_offset: self.scroll_offset,
            viewport_height: self.viewport_height,
            overscan: self.overscan,
        }
    }
}

impl<M: std::fmt::Debug> std::fmt::Debug for Tree<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tree")
            .field("key", &self.key)
            .field("children", &self.children)
            .field("expanded", &self.expanded)
            .field("selected", &self.selected)
            .field("indent", &self.indent)
            .field("scroll_offset", &self.scroll_offset)
            .field("viewport_height", &self.viewport_height)
            .field("overscan", &self.overscan)
            .finish()
    }
}

impl<M: PartialEq> PartialEq for Tree<M> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.children == other.children
            && self.expanded == other.expanded
            && self.selected == other.selected
            && self.indent == other.indent
            && self.scroll_offset == other.scroll_offset
            && self.viewport_height == other.viewport_height
            && self.overscan == other.overscan
    }
}

// ── Builder / factory ───────────────────────────────────────────────────────

/// Create a new `Tree` builder.
pub fn tree<M>(key: impl Into<WidgetKey>) -> Tree<M> {
    Tree {
        key: key.into(),
        children: Vec::new(),
        expanded: HashSet::new(),
        selected: None,
        indent: 20.0,
        scroll_offset: 0.0,
        viewport_height: 0.0,
        overscan: 3,
    }
}

impl<M> Tree<M> {
    /// Add a top-level tree node.
    pub fn child(mut self, node: TreeNode<M>) -> Self {
        self.children.push(node);
        self
    }

    /// Set the indent per depth level.
    pub fn indent(mut self, value: f32) -> Self {
        self.indent = crate::finite(value);
        self
    }

    /// Set viewport height.
    pub fn viewport_height(mut self, value: f32) -> Self {
        self.viewport_height = crate::finite(value);
        self
    }

    /// Set scroll offset.
    pub fn scroll_offset(mut self, value: f32) -> Self {
        self.scroll_offset = value.max(0.0);
        self
    }

    /// Build into a `WidgetNode`.
    pub fn build(self) -> WidgetNode<M> {
        WidgetNode::Tree(self)
    }
}

// ── Core methods ─────────────────────────────────────────────────────────────

/// A flattened visible node entry used for layout and navigation.
#[derive(Clone, Debug)]
pub struct VisibleNode<M> {
    pub key: WidgetKey,
    pub content: WidgetNode<M>,
    pub depth: usize,
    pub has_children: bool,
    pub expanded: bool,
}

impl<M: Clone> Tree<M> {
    /// Walk the tree and collect all visible (expanded + in-viewport) nodes.
    /// Returns a flat list of `VisibleNode` entries suitable for layout.
    pub fn visible_nodes(&self) -> Vec<VisibleNode<M>> {
        let mut result = Vec::new();
        let mut y: f32 = 0.0;
        let row_height = 24.0; // standard row height for estimation

        for node in &self.children {
            self.collect_visible(node, 0, &mut y, row_height, &mut result);
        }

        result
    }

    fn collect_visible(
        &self,
        node: &TreeNode<M>,
        depth: usize,
        y: &mut f32,
        row_height: f32,
        out: &mut Vec<VisibleNode<M>>,
    ) {
        let is_expanded = self.expanded.contains(&node.key) || node.expanded;
        let has_children = !node.children.is_empty();

        // Skip nodes strictly after the viewport + overscan window
        let end_y = *y + row_height;
        let max_y = self.scroll_offset + self.viewport_height + (self.overscan as f32 * row_height);
        let min_y = (self.scroll_offset - (self.overscan as f32 * row_height)).max(0.0);

        if end_y >= min_y && *y <= max_y {
            out.push(VisibleNode {
                key: node.key.clone(),
                content: node.content.clone(),
                depth,
                has_children,
                expanded: is_expanded,
            });
        }

        *y = end_y;

        // Recurse into children if expanded
        if is_expanded {
            for child in &node.children {
                self.collect_visible(child, depth + 1, y, row_height, out);
            }
        }
    }

    /// Count total visible nodes (for navigation bounds).
    pub fn visible_count(&self) -> usize {
        self.visible_nodes().len()
    }

    /// Get the index of `key` in the visible nodes list.
    pub fn visible_index(&self, key: &WidgetKey) -> Option<usize> {
        self.visible_nodes().iter().position(|vn| &vn.key == key)
    }

    /// Total content height estimate (visible rows * row height).
    pub fn content_height(&self) -> f32 {
        self.visible_count() as f32 * 24.0
    }

    // ── Keyboard navigation ──────────────────────────────────────────────

    /// Move selection to the next visible node.  Wraps at end.
    pub fn select_next(&mut self) {
        let nodes = self.visible_nodes();
        if nodes.is_empty() {
            return;
        }
        let current = self
            .selected
            .as_ref()
            .and_then(|k| nodes.iter().position(|n| &n.key == k));
        let next = match current {
            Some(i) if i + 1 < nodes.len() => i + 1,
            Some(_) => 0, // wrap
            None => 0,
        };
        self.selected = Some(nodes[next].key.clone());
    }

    /// Move selection to the previous visible node.  Wraps at start.
    pub fn select_prev(&mut self) {
        let nodes = self.visible_nodes();
        if nodes.is_empty() {
            return;
        }
        let current = self
            .selected
            .as_ref()
            .and_then(|k| nodes.iter().position(|n| &n.key == k));
        let prev = match current {
            Some(0) => nodes.len() - 1, // wrap
            Some(i) => i - 1,
            None => nodes.len() - 1,
        };
        self.selected = Some(nodes[prev].key.clone());
    }

    /// Select the first visible node (Home key).
    pub fn select_first(&mut self) {
        if let Some(first) = self.visible_nodes().first() {
            self.selected = Some(first.key.clone());
        }
    }

    /// Select the last visible node (End key).
    pub fn select_last(&mut self) {
        if let Some(last) = self.visible_nodes().last() {
            self.selected = Some(last.key.clone());
        }
    }

    /// Expand the selected node.
    #[allow(clippy::collapsible_if)]
    pub fn expand_selected(&mut self) {
        if let Some(ref key) = self.selected {
            if let Some(node) = self.find_node(key) {
                if !node.children.is_empty() {
                    self.expanded.insert(key.clone());
                }
            }
        }
    }

    /// Collapse the selected node.
    pub fn collapse_selected(&mut self) {
        if let Some(ref key) = self.selected {
            self.expanded.remove(key);
        }
    }

    /// Toggle expand/collapse on the selected node.
    pub fn toggle_selected(&mut self) {
        if let Some(ref key) = self.selected.clone() {
            if self.expanded.contains(key) {
                self.expanded.remove(key);
            } else {
                self.expanded.insert(key.clone());
            }
        }
    }

    /// Typeahead: jump to the next visible node whose content starts with `ch`.
    /// Searches from the current selection onward (wrapping).
    pub fn typeahead(&mut self, ch: char) {
        let nodes = self.visible_nodes();
        if nodes.is_empty() {
            return;
        }
        let start = self
            .selected
            .as_ref()
            .and_then(|k| nodes.iter().position(|n| &n.key == k))
            .map(|i| i + 1)
            .unwrap_or(0);

        for i in 0..nodes.len() {
            let idx = (start + i) % nodes.len();
            // Check the content widget for a Label match
            if Self::node_matches_first_char(&nodes[idx].content, ch) {
                self.selected = Some(nodes[idx].key.clone());
                return;
            }
        }
    }

    /// Check if a widget node's display text starts with the given character.
    fn node_matches_first_char(node: &WidgetNode<M>, ch: char) -> bool {
        match node {
            WidgetNode::Label(l) => {
                l.text.chars().next().map(|c| c.eq_ignore_ascii_case(&ch)) == Some(true)
            }
            _ => false,
        }
    }

    // ── Helpers ──────────────────────────────────────────────────────────

    /// Find a node by key (depth-first).
    pub fn find_node(&self, key: &WidgetKey) -> Option<&TreeNode<M>> {
        find_in_children(&self.children, key)
    }

    /// Find a node by key (mutable).
    pub fn find_node_mut(&mut self, key: &WidgetKey) -> Option<&mut TreeNode<M>> {
        find_in_children_mut(&mut self.children, key)
    }
}

// Standalone recursive helpers (avoids borrow conflicts when Tree is &mut self)
fn find_in_children<'a, M>(
    children: &'a [TreeNode<M>],
    key: &WidgetKey,
) -> Option<&'a TreeNode<M>> {
    for child in children {
        if &child.key == key {
            return Some(child);
        }
        if let Some(found) = find_in_children(&child.children, key) {
            return Some(found);
        }
    }
    None
}

fn find_in_children_mut<'a, M>(
    children: &'a mut [TreeNode<M>],
    key: &WidgetKey,
) -> Option<&'a mut TreeNode<M>> {
    for child in children.iter_mut() {
        if &child.key == key {
            return Some(child);
        }
        if let Some(found) = find_in_children_mut(&mut child.children, key) {
            return Some(found);
        }
    }
    None
}

// ── From conversion ──────────────────────────────────────────────────────────

impl<M> From<Tree<M>> for WidgetNode<M> {
    fn from(value: Tree<M>) -> Self {
        WidgetNode::Tree(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::label;

    fn sample_tree() -> Tree<()> {
        tree::<()>("t")
            .child(
                TreeNode::new("n1", label("Root"))
                    .child(TreeNode::new("n1a", label("Child A")))
                    .child(TreeNode::new("n1b", label("Child B"))),
            )
            .child(TreeNode::new("n2", label("Sibling")))
    }

    #[test]
    fn tree_creates_nodes() {
        let t = tree::<()>("t")
            .child(TreeNode::new("a", label("A")))
            .child(TreeNode::new("b", label("B")));
        assert_eq!(t.children.len(), 2);
        assert_eq!(t.key.as_str(), "t");
    }

    #[test]
    fn visible_nodes_shows_only_expanded() {
        let t = sample_tree().viewport_height(500.0);
        let visible = t.visible_nodes();
        // Initially nothing expanded → only 2 top-level nodes
        assert_eq!(visible.len(), 2);
        assert_eq!(visible[0].depth, 0);
        assert_eq!(visible[1].depth, 0);
    }

    #[test]
    fn visible_nodes_with_expansion() {
        let mut t = sample_tree().viewport_height(500.0);
        t.expanded.insert(WidgetKey::new("n1"));
        let visible = t.visible_nodes();
        // Root (n1) + Child A + Child B + Sibling (n2) = 4
        assert_eq!(visible.len(), 4);
        assert_eq!(visible[0].depth, 0);
        assert_eq!(visible[1].depth, 1);
        assert_eq!(visible[2].depth, 1);
        assert_eq!(visible[3].depth, 0);
    }

    #[test]
    fn select_next_and_prev() {
        let mut t = sample_tree().viewport_height(500.0);
        t.expanded.insert(WidgetKey::new("n1"));

        t.select_next();
        assert_eq!(t.selected.as_ref().map(|k| k.as_str()), Some("n1"));

        t.select_next();
        assert_eq!(t.selected.as_ref().map(|k| k.as_str()), Some("n1a"));

        t.select_prev();
        assert_eq!(t.selected.as_ref().map(|k| k.as_str()), Some("n1"));
    }

    #[test]
    fn select_first_and_last() {
        let mut t = sample_tree().viewport_height(500.0);
        t.expanded.insert(WidgetKey::new("n1"));

        t.select_first();
        assert_eq!(t.selected.as_ref().map(|k| k.as_str()), Some("n1"));

        t.select_last();
        assert_eq!(t.selected.as_ref().map(|k| k.as_str()), Some("n2"));
    }

    #[test]
    fn expand_and_collapse_selected() {
        let mut t = sample_tree().viewport_height(500.0);
        t.selected = Some(WidgetKey::new("n1"));

        t.expand_selected();
        assert!(t.expanded.contains(&WidgetKey::new("n1")));

        t.collapse_selected();
        assert!(!t.expanded.contains(&WidgetKey::new("n1")));
    }

    #[test]
    fn toggle_selected() {
        let mut t = sample_tree().viewport_height(500.0);
        t.selected = Some(WidgetKey::new("n1"));

        t.toggle_selected();
        assert!(t.expanded.contains(&WidgetKey::new("n1")));

        t.toggle_selected();
        assert!(!t.expanded.contains(&WidgetKey::new("n1")));
    }

    #[test]
    fn typeahead_matches_first_char() {
        let mut t = tree::<()>("t")
            .child(TreeNode::new("apple", label("Apple")))
            .child(TreeNode::new("banana", label("Banana")))
            .child(TreeNode::new("avocado", label("Avocado")))
            .child(TreeNode::new("cherry", label("Cherry")))
            .viewport_height(500.0);

        // Type 'b' → should select banana
        t.typeahead('b');
        assert_eq!(t.selected.as_ref().map(|k| k.as_str()), Some("banana"));

        // Type 'a' (starting from banana, wraps) → avocado
        // (search starts after current selection, so avocado at index 2 is found before apple at index 0)
        t.typeahead('a');
        assert_eq!(t.selected.as_ref().map(|k| k.as_str()), Some("avocado"));

        // Type 'c' → cherry
        t.typeahead('c');
        assert_eq!(t.selected.as_ref().map(|k| k.as_str()), Some("cherry"));
    }

    #[test]
    fn find_node_works() {
        let t = sample_tree();
        assert!(t.find_node(&WidgetKey::new("n1")).is_some());
        assert!(t.find_node(&WidgetKey::new("n1a")).is_some());
        assert!(t.find_node(&WidgetKey::new("n2")).is_some());
        assert!(t.find_node(&WidgetKey::new("nonexistent")).is_none());
    }

    #[test]
    fn visible_count_matches() {
        let t = sample_tree().viewport_height(500.0);
        assert_eq!(t.visible_count(), 2);

        let mut t2 = sample_tree().viewport_height(500.0);
        t2.expanded.insert(WidgetKey::new("n1"));
        assert_eq!(t2.visible_count(), 4);
    }
}
