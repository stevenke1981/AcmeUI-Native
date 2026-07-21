//! AccessKit integration for AcmeUI Native.
//!
//! Provides functions to build [`TreeUpdate`] values from the widget tree
//! and layout snapshot used by the AcmeUI framework. Also provides a
//! per-window [`AccessibilityAdapter`] that bridges the accessibility tree
//! with the platform event system.

#![forbid(unsafe_op_in_unsafe_fn)]

use accesskit::{Action, Node as AccessNode, NodeId as AccessNodeId, Rect, Role, Tree, TreeUpdate};
use acme_core::NodeId;
use acme_layout::{LayoutRect, LayoutSnapshot};
use acme_platform::{PlatformEvent, WindowId};
use acme_widgets::WidgetNode;

/// Re-export AccessKit types used in the public API.
pub use accesskit::{Node as AccNode, NodeId as AccNodeId};

/// Converts an AcmeUI [`NodeId`] to an AccessKit [`AccessNodeId`].
#[inline]
pub fn to_access_id(id: NodeId) -> AccessNodeId {
    AccessNodeId::from(id.get())
}

/// A snapshot of the accessibility tree for one window.
///
/// Contains the flat node list and the root node ID, mirroring the
/// structure of AccessKit's [`TreeUpdate`] without the focus or
/// tree-identity fields.
#[derive(Clone, Debug)]
pub struct AccessibilityTree {
    pub nodes: Vec<(AccessNodeId, AccessNode)>,
    pub root: AccessNodeId,
}

/// Actions that accessibility tools (screen readers, etc.) can trigger
/// on a widget. Each variant carries the target [`NodeId`] within the
/// accessibility tree.
#[derive(Debug, Clone, PartialEq)]
pub enum AccessibilityAction {
    /// Move keyboard focus to the widget.
    Focus(NodeId),
    /// Simulate a click on the widget.
    Click(NodeId),
    /// Set the accessible value of a widget (e.g. a text field).
    SetValue(NodeId, String),
    /// Scroll the widget into the visible viewport.
    ScrollIntoView(NodeId),
    /// Generic "activate" – press a button, toggle a switch, etc.
    Activate(NodeId),
}

/// Bridges the accessibility tree with one window.
///
/// Holds the last-built [`AccessibilityTree`], its [`LayoutSnapshot`],
/// and can route [`AccessibilityAction`] values into [`PlatformEvent`]
/// values for the windowing system.
pub struct AccessibilityAdapter {
    window_id: u64,
    tree: Option<AccessibilityTree>,
    /// Cache of the last built tree root for diffing.
    last_root: Option<AccessNodeId>,
    focused_node: Option<NodeId>,
    /// Layout snapshot stored during the last [`AccessibilityAdapter::update`]
    /// call. Used by [`route_action`] to resolve node bounding-box geometry
    /// (e.g. Click/Activate at the node center) instead of emitting (0,0)
    /// pointer coordinates.
    snapshot: Option<LayoutSnapshot>,
}

impl AccessibilityAdapter {
    /// Create a new adapter for the given window.
    pub fn new(window_id: u64) -> Self {
        Self {
            window_id,
            tree: None,
            last_root: None,
            focused_node: None,
            snapshot: None,
        }
    }

    /// Rebuild the accessibility tree from the widget tree and layout snapshot.
    ///
    /// Walks `root` in DFS order (matching [`WidgetNode::to_layout`]), assigns
    /// sequential IDs, and stores the result as the current tree.
    pub fn update<M>(&mut self, root: &WidgetNode<M>, snapshot: &LayoutSnapshot)
    where
        M: Clone + 'static,
    {
        // TODO: use retained NodeId when RuntimeTree is integrated (P0-002).
        // Currently IDs are reassigned starting from 1 each frame, which means
        // accessibility nodes are not stable across rebuilds.
        let mut next = 1u64;
        let mut nodes = Vec::new();
        let root_id = walk_node(root, snapshot, &mut next, None, &mut nodes);
        self.last_root = Some(root_id);
        self.tree = Some(AccessibilityTree {
            nodes,
            root: root_id,
        });
        self.snapshot = Some(snapshot.clone());
    }

    /// Get a reference to the current accessibility tree, if one has been built.
    pub fn tree_ref(&self) -> Option<&AccessibilityTree> {
        self.tree.as_ref()
    }

    /// Resolve the center point of a node from the stored layout snapshot.
    ///
    /// Returns `(x, y)` in logical pixels, or `None` if the node is not
    /// present in the snapshot.
    fn resolve_node_center(&self, id: NodeId) -> Option<(f32, f32)> {
        let rect = self.snapshot.as_ref()?.get(id)?;
        Some((rect.x + rect.width * 0.5, rect.y + rect.height * 0.5))
    }

    /// Route an accessibility action into corresponding platform event(s).
    ///
    /// Returns zero, one, or more [`PlatformEvent`] values.  The caller should
    /// dispatch all returned events in order through the normal application
    /// event pipeline.
    ///
    /// # Coordinate resolution
    ///
    /// `Click` and `Activate` resolve the target node's bounding‑box center
    /// from the stored [`LayoutSnapshot`] (populated during [`update`]).  If
    /// the snapshot is missing or the node is absent, coordinates fall back
    /// to `(0, 0)`.
    ///
    /// `SetValue` emits a [`PlatformEvent::ImeCommit`] because there is no
    /// dedicated value‑set variant in the current event set — the framework
    /// layer should route this to the target text field identified by the
    /// enclosed node ID.
    ///
    /// `ScrollIntoView` emits a dedicated [`PlatformEvent::AccessibilityScrollIntoView`]
    /// that the framework layer can handle to bring the target node into the
    /// visible viewport.
    pub fn route_action(&self, action: &AccessibilityAction) -> Vec<PlatformEvent> {
        match action {
            AccessibilityAction::Focus(id) => vec![PlatformEvent::FocusChanged {
                window: WindowId(self.window_id),
                gained: true,
                node_id: id.get(),
            }],
            AccessibilityAction::Click(id) | AccessibilityAction::Activate(id) => {
                let (cx, cy) = self.resolve_node_center(*id).unwrap_or((0.0, 0.0));
                vec![
                    // Emit FocusChanged first so the target widget receives
                    // keyboard focus before the click event.
                    PlatformEvent::FocusChanged {
                        window: WindowId(self.window_id),
                        gained: true,
                        node_id: id.get(),
                    },
                    PlatformEvent::PointerButton {
                        window: WindowId(self.window_id),
                        pressed: true,
                        x: cx,
                        y: cy,
                        button: 0,
                        pointer: 0,
                    },
                ]
            }
            AccessibilityAction::SetValue(_id, val) => {
                vec![PlatformEvent::ImeCommit {
                    window: WindowId(self.window_id),
                    text: val.clone(),
                }]
            }
            AccessibilityAction::ScrollIntoView(id) => {
                vec![PlatformEvent::AccessibilityScrollIntoView {
                    window: WindowId(self.window_id),
                    node_id: id.get(),
                }]
            }
        }
    }

    /// Track the given node as focused.
    ///
    /// The focused node is included when building the next [`TreeUpdate`]
    /// via [`build_tree_update`]. Call this method when keyboard focus
    /// moves to a new widget.
    pub fn focus_widget(&mut self, target_id: NodeId) {
        self.focused_node = Some(target_id);
    }

    /// Build an AccessKit [`TreeUpdate`] from the current tree with
    /// the tracked focus applied.
    pub fn build_tree_update(&self) -> Option<TreeUpdate> {
        let tree = self.tree.as_ref()?;
        let focus = self
            .focused_node
            .map(|id| AccessNodeId::from(id.get()))
            .unwrap_or(tree.root);
        Some(TreeUpdate {
            nodes: tree.nodes.clone(),
            tree: Some(Tree::new(tree.root)),
            focus,
        })
    }
}

/// Build a full [`TreeUpdate`] from a widget tree and its layout snapshot.
///
/// Walks the widget tree in DFS order, assigning sequential IDs that match
/// those produced by [`WidgetNode::to_layout`]. Each widget variant is mapped
/// to the AccessKit role most appropriate for assistive technology consumption:
///
/// | Widget variant          | AccessKit role    |
/// |-------------------------|-------------------|
/// | `Row` / `Column` / `Stack` / `Card` | `Group`       |
/// | `Label`                 | `Label`           |
/// | `Button`                | `Button`          |
/// | `Separator`             | `Splitter`        |
/// | `ScrollView`            | `ScrollView`      |
///
/// # Parameters
///
/// * `root` – The root of the widget tree.
/// * `snapshot` – A layout snapshot whose keys match the walk order of `root`.
/// * `focus_id` – The ID of the node that currently holds keyboard focus, if any.
/// * `label_override` – If `Some`, replaces the display text of every `Label` node
///   in the tree with this string.
pub fn build_tree_update<M>(
    root: &WidgetNode<M>,
    snapshot: &LayoutSnapshot,
    focus_id: Option<NodeId>,
    label_override: Option<&str>,
) -> TreeUpdate
where
    M: Clone + 'static,
{
    let mut next_id: u64 = 1;
    let mut nodes: Vec<(AccessNodeId, AccessNode)> = Vec::new();
    let root_id = walk_node(root, snapshot, &mut next_id, label_override, &mut nodes);

    let focus = match focus_id {
        Some(id) => AccessNodeId::from(id.get()),
        None => root_id,
    };

    TreeUpdate {
        nodes,
        tree: Some(Tree::new(root_id)),
        focus,
    }
}

/// Recursively walk the widget tree, producing AccessKit nodes.
pub fn walk_node<M: Clone + 'static>(
    node: &WidgetNode<M>,
    snapshot: &LayoutSnapshot,
    next_id: &mut u64,
    label_override: Option<&str>,
    nodes: &mut Vec<(AccessNodeId, AccessNode)>,
) -> AccessNodeId {
    let id = AccessNodeId::from(*next_id);
    let layout_id = NodeId::new(*next_id);
    *next_id += 1;

    let role = match node {
        WidgetNode::Row(_) | WidgetNode::Column(_) | WidgetNode::Stack(_) | WidgetNode::Card(_) => {
            Role::Group
        }
        WidgetNode::Label(_) => Role::Label,
        WidgetNode::Button(_) => Role::Button,
        WidgetNode::Separator(_) => Role::Splitter,
        WidgetNode::ScrollView(_) => Role::ScrollView,
        WidgetNode::VirtualList(_) => Role::List,
        WidgetNode::Tooltip(_) => Role::Tooltip,
        WidgetNode::Popover(_) => Role::Group,
        WidgetNode::Menu(_) => Role::Menu,
        WidgetNode::Dialog(_) => Role::Dialog,
        WidgetNode::Tree(_) => Role::Tree,
        WidgetNode::Table(_) => Role::Table,
        WidgetNode::DataGrid(_) => Role::Grid,
        WidgetNode::TextInput(_) => Role::TextInput,
        WidgetNode::NavRail(_) | WidgetNode::Sidebar(_) | WidgetNode::Breadcrumb(_) => {
            Role::Navigation
        }
        WidgetNode::TabBar(_) => Role::TabList,
    };

    let mut access_node = AccessNode::new(role);

    // Map the layout bounding box into an AccessKit rectangle.
    if let Some(rect) = snapshot.get(layout_id) {
        access_node.set_bounds(Rect::new(
            rect.x as f64,
            rect.y as f64,
            (rect.x + rect.width) as f64,
            (rect.y + rect.height) as f64,
        ));
    }

    // Determine the accessible name.
    let label = match node {
        WidgetNode::Label(l) => label_override.unwrap_or(&l.text).to_string(),
        WidgetNode::Button(b) => b.label.clone(),
        _ => String::new(),
    };
    if !label.is_empty() {
        access_node.set_label(label);
    }

    // Interactive state.
    if let WidgetNode::Button(b) = node {
        if b.disabled {
            access_node.set_disabled();
        }
        access_node.add_action(Action::Focus);
        access_node.add_action(Action::Click);
    }

    // Process children.
    let child_ids: Vec<AccessNodeId> = node
        .children()
        .iter()
        .map(|child| walk_node(child, snapshot, next_id, label_override, nodes))
        .collect();
    if !child_ids.is_empty() {
        access_node.set_children(child_ids);
    }

    nodes.push((id, access_node));
    id
}

/// Create a minimal initial [`TreeUpdate`] with a single root window node.
///
/// The root node is sized to the window default (1080×720). Applications
/// should call [`build_tree_update`] with the real widget tree as soon as
/// it is available to replace this placeholder.
pub fn initial_tree(snapshot: &LayoutSnapshot) -> TreeUpdate {
    let root_id = AccessNodeId::from(1u64);

    // Use a default window size; we cannot iterate the private rect map.
    let _ = snapshot; // snapshot size is logged or used by the caller.
    let bounds = Rect::new(0.0, 0.0, 1080.0, 720.0);

    let mut node = AccessNode::new(Role::Window);
    node.set_bounds(bounds);

    TreeUpdate {
        nodes: vec![(root_id, node)],
        tree: Some(Tree::new(root_id)),
        focus: root_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::{NodeId, RetainedTree, ViewNode};
    use acme_layout::LayoutEngine;
    use acme_widgets::{WidgetNode, button, column, label, row, scroll_view, separator, stack};

    #[derive(Clone, Debug, PartialEq)]
    struct TestMsg;

    /// Create a `NodeId` with a specific `u64` value by reconciling that many roots.
    fn make_node_id(n: u64) -> NodeId {
        let mut tree = RetainedTree::new();
        let keys: Vec<&str> = (0..n)
            .map(|i| match i {
                0 => "_a",
                1 => "_b",
                2 => "_c",
                3 => "_d",
                _ => "_x",
            })
            .collect();
        let views: Vec<ViewNode> = keys.iter().map(|k| ViewNode::new(*k, "_")).collect();
        tree.reconcile_roots(&views)
            .expect("reconciliation should succeed");
        let roots = tree.roots();
        roots[roots.len() - 1]
    }

    // ------------------------------------------------------------------
    // Role mapping tests — no layout needed.
    // ------------------------------------------------------------------

    #[test]
    fn label_uses_label_role() {
        let tree = column::<TestMsg>().child(label("Hello")).build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&tree, &snapshot, None, None);

        let count = update
            .nodes
            .iter()
            .filter(|(_, n)| n.role() == Role::Label)
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn label_node_carries_text() {
        let tree = column::<TestMsg>().child(label("Hello World")).build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&tree, &snapshot, None, None);

        let label_node = update
            .nodes
            .iter()
            .find(|(_, n)| n.role() == Role::Label)
            .expect("expected a StaticText node");
        assert_eq!(label_node.1.label(), Some("Hello World"));
    }

    #[test]
    fn button_uses_button_role() {
        let tree = column::<TestMsg>()
            .child(button("ok", "OK").on_click(TestMsg {}))
            .build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&tree, &snapshot, None, None);

        let count = update
            .nodes
            .iter()
            .filter(|(_, n)| n.role() == Role::Button)
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn button_node_carries_label() {
        let tree = column::<TestMsg>()
            .child(button("cancel", "Cancel").on_click(TestMsg {}))
            .build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&tree, &snapshot, None, None);

        let btn_node = update
            .nodes
            .iter()
            .find(|(_, n)| n.role() == Role::Button)
            .expect("expected a Button node");
        assert_eq!(btn_node.1.label(), Some("Cancel"));
    }

    #[test]
    fn button_supports_focus_and_click_actions() {
        let tree = column::<TestMsg>()
            .child(button("go", "Go").on_click(TestMsg {}))
            .build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&tree, &snapshot, None, None);

        let btn_node = update
            .nodes
            .iter()
            .find(|(_, n)| n.role() == Role::Button)
            .expect("expected a Button node");
        assert!(btn_node.1.supports_action(Action::Focus));
        assert!(btn_node.1.supports_action(Action::Click));
    }

    #[test]
    fn disabled_button_has_disabled_flag() {
        let tree = column::<TestMsg>()
            .child(button("save", "Save").disabled(true).on_click(TestMsg {}))
            .build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&tree, &snapshot, None, None);

        let btn_node = update
            .nodes
            .iter()
            .find(|(_, n)| n.role() == Role::Button)
            .expect("expected a Button node");
        assert!(btn_node.1.is_disabled());
    }

    #[test]
    fn separator_uses_splitter_role() {
        let tree = column::<TestMsg>().child(separator::<TestMsg>()).build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&tree, &snapshot, None, None);

        let count = update
            .nodes
            .iter()
            .filter(|(_, n)| n.role() == Role::Splitter)
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn scroll_view_uses_scroll_view_role() {
        let tree = column::<TestMsg>()
            .child(scroll_view::<TestMsg>("sv").child(label("inside")).build())
            .build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&tree, &snapshot, None, None);

        let count = update
            .nodes
            .iter()
            .filter(|(_, n)| n.role() == Role::ScrollView)
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn containers_use_group_role() {
        let tree = column::<TestMsg>()
            .child(row::<TestMsg>().child(label("A")).build())
            .child(stack::<TestMsg>().child(label("B")).build())
            .build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&tree, &snapshot, None, None);

        // Column + Row + Stack = 3 Group nodes
        let count = update
            .nodes
            .iter()
            .filter(|(_, n)| n.role() == Role::Group)
            .count();
        assert_eq!(count, 3);
    }

    // ------------------------------------------------------------------
    // label_override
    // ------------------------------------------------------------------

    #[test]
    fn label_override_replaces_label_text() {
        let tree = column::<TestMsg>().child(label("Original")).build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&tree, &snapshot, None, Some("Overridden"));

        let label_node = update
            .nodes
            .iter()
            .find(|(_, n)| n.role() == Role::Label)
            .expect("expected a StaticText node");
        assert_eq!(label_node.1.label(), Some("Overridden"));
    }

    // ------------------------------------------------------------------
    // Focus
    // ------------------------------------------------------------------

    #[test]
    fn focus_is_root_when_no_focus_id_provided() {
        let tree = column::<TestMsg>().child(label("Hi")).build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&tree, &snapshot, None, None);

        let root_id = AccessNodeId::from(1u64);
        assert_eq!(update.focus, root_id);
    }

    #[test]
    fn focus_is_mapped_from_node_id() {
        let tree = column::<TestMsg>().child(label("Hi")).build();
        let snapshot = LayoutSnapshot::default();
        // Node ID 2 corresponds to the label (second DFS walk).
        let focus_id = make_node_id(2);
        let update = build_tree_update(&tree, &snapshot, Some(focus_id), None);

        assert_eq!(update.focus, AccessNodeId::from(2u64));
    }

    // ------------------------------------------------------------------
    // Snapshot bounds mapping
    // ------------------------------------------------------------------

    fn compute_snapshot(
        tree: &WidgetNode<TestMsg>,
        start_id: u64,
        viewport: (f32, f32),
    ) -> LayoutSnapshot {
        let layout = tree.to_layout(NodeId::new(start_id));
        LayoutEngine::new()
            .compute(&layout, viewport)
            .expect("layout computation should succeed")
    }

    #[test]
    fn bounds_are_mapped_from_snapshot() {
        let tree = column::<TestMsg>()
            .child(label("Hello"))
            .child(separator::<TestMsg>())
            .child(button("btn", "Click").on_click(TestMsg {}))
            .build();
        let snapshot = compute_snapshot(&tree, 1, (200.0, 300.0));
        let update = build_tree_update(&tree, &snapshot, None, None);

        assert!(
            !update.nodes.is_empty(),
            "there should be at least one node"
        );

        // Every node should have a valid (non-negative width/height) bounding box.
        for (id, node) in &update.nodes {
            let bounds = node
                .bounds()
                .unwrap_or_else(|| panic!("node {id:?} is missing bounds"));
            assert!(bounds.width() >= 0.0, "node {id:?} has negative width");
            assert!(bounds.height() >= 0.0, "node {id:?} has negative height");
        }
    }

    #[test]
    fn all_bounds_are_present_after_layout() {
        // Verifies that every widget node gets a `Some(…)` bounds from the
        // layout snapshot.  Width/height may be zero for auto-sized nodes
        // because Taffy has no intrinsic content measurement.
        let tree = column::<TestMsg>()
            .child(label("Hello"))
            .child(button("a", "A").on_click(TestMsg {}))
            .build();
        let snapshot = compute_snapshot(&tree, 1, (400.0, 100.0));
        let update = build_tree_update(&tree, &snapshot, None, None);

        for (id, node) in &update.nodes {
            let bounds = node
                .bounds()
                .unwrap_or_else(|| panic!("node {id:?} is missing bounds after layout"));
            // Non-negative guarantee (NaN would also fail here).
            assert!(bounds.width() >= 0.0, "node {id:?} has negative width");
            assert!(bounds.height() >= 0.0, "node {id:?} has negative height");
        }
    }

    // ------------------------------------------------------------------
    // Tree structure: parent-child relationships
    // ------------------------------------------------------------------

    #[test]
    fn container_has_correct_child_ids() {
        let tree = column::<TestMsg>()
            .child(label("A"))
            .child(label("B"))
            .child(label("C"))
            .build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&tree, &snapshot, None, None);

        // Root (column, ID 1) should have children [2, 3, 4].
        let root_node = update
            .nodes
            .iter()
            .find(|(id, _)| *id == AccessNodeId::from(1u64))
            .expect("expected root node");

        let children = root_node.1.children();
        assert_eq!(
            children,
            &[
                AccessNodeId::from(2u64),
                AccessNodeId::from(3u64),
                AccessNodeId::from(4u64),
            ]
        );
    }

    // ------------------------------------------------------------------
    // Initial tree
    // ------------------------------------------------------------------

    #[test]
    fn initial_tree_has_window_role() {
        let snapshot = LayoutSnapshot::default();
        let update = initial_tree(&snapshot);

        assert_eq!(update.nodes.len(), 1);
        assert_eq!(update.nodes[0].1.role(), Role::Window);
        assert_eq!(update.focus, AccessNodeId::from(1u64));
        assert!(update.tree.is_some());
    }

    #[test]
    fn initial_tree_uses_default_bounds() {
        let snapshot = LayoutSnapshot::default();
        let update = initial_tree(&snapshot);

        let bounds = update.nodes[0]
            .1
            .bounds()
            .expect("window should have bounds");
        assert_eq!(bounds, Rect::new(0.0, 0.0, 1080.0, 720.0));
    }

    // ------------------------------------------------------------------
    // AccessibilityAdapter
    // ------------------------------------------------------------------

    #[test]
    fn adapter_creates_tree_from_widget_tree() {
        let tree = column::<TestMsg>()
            .child(label("Hello"))
            .child(button("btn", "OK").on_click(TestMsg {}))
            .build();
        let snapshot = LayoutSnapshot::default();
        let mut adapter = AccessibilityAdapter::new(1);
        adapter.update(&tree, &snapshot);

        let t = adapter.tree_ref().expect("tree should exist after update");
        assert_eq!(t.root, AccessNodeId::from(1u64), "root should be ID 1");
        assert_eq!(t.nodes.len(), 3, "expected 3 accessibility nodes");
    }

    #[test]
    fn adapter_node_roles_after_update() {
        let tree = column::<TestMsg>()
            .child(label("A"))
            .child(separator::<TestMsg>())
            .build();
        let snapshot = LayoutSnapshot::default();
        let mut adapter = AccessibilityAdapter::new(1);
        adapter.update(&tree, &snapshot);

        let t = adapter.tree_ref().expect("tree should exist");
        // walk_node uses post-order: children pushed before parent.
        // So order is: label (Label), separator (Splitter), column (Group).
        let roles: Vec<Role> = t.nodes.iter().map(|(_, n)| n.role()).collect();
        assert_eq!(roles, vec![Role::Label, Role::Splitter, Role::Group]);
    }

    #[test]
    fn adapter_routes_focus_action() {
        let adapter = AccessibilityAdapter::new(42);
        let action = AccessibilityAction::Focus(NodeId::new(7));
        let events = adapter.route_action(&action);
        assert_eq!(events.len(), 1, "Focus should produce one event");
        match &events[0] {
            PlatformEvent::FocusChanged {
                window,
                gained,
                node_id,
            } => {
                assert_eq!(*window, WindowId(42));
                assert!(*gained);
                assert_eq!(*node_id, 7, "Focus must use the actual NodeId, not 0");
            }
            other => panic!("expected FocusChanged, got {other:?}"),
        }
    }

    #[test]
    fn adapter_routes_click_action() {
        let adapter = AccessibilityAdapter::new(3);
        let action = AccessibilityAction::Click(NodeId::new(5));
        let events = adapter.route_action(&action);
        assert_eq!(
            events.len(),
            2,
            "Click should produce two events (FocusChanged + PointerButton)"
        );

        // First event must be FocusChanged with the real node_id.
        match &events[0] {
            PlatformEvent::FocusChanged {
                window,
                gained,
                node_id,
            } => {
                assert_eq!(*window, WindowId(3));
                assert!(*gained);
                assert_eq!(*node_id, 5, "Click must focus the target node first");
            }
            other => panic!("expected FocusChanged as first event, got {other:?}"),
        }

        // Second event is the click itself.
        match &events[1] {
            PlatformEvent::PointerButton {
                window, pressed, ..
            } => {
                assert_eq!(*window, WindowId(3));
                assert!(*pressed);
            }
            other => panic!("expected PointerButton as second event, got {other:?}"),
        }
    }

    #[test]
    fn adapter_routes_set_value_action() {
        let adapter = AccessibilityAdapter::new(1);
        let action = AccessibilityAction::SetValue(NodeId::new(2), "hello".into());
        let events = adapter.route_action(&action);
        assert_eq!(events.len(), 1, "SetValue should produce one event");
        match &events[0] {
            PlatformEvent::ImeCommit { window, text } => {
                assert_eq!(*window, WindowId(1));
                assert_eq!(text, "hello");
            }
            other => panic!("expected ImeCommit, got {other:?}"),
        }
    }

    #[test]
    fn adapter_scroll_into_view_returns_event() {
        let adapter = AccessibilityAdapter::new(1);
        let action = AccessibilityAction::ScrollIntoView(NodeId::new(3));
        let events = adapter.route_action(&action);
        assert_eq!(events.len(), 1, "ScrollIntoView should produce one event");
        match &events[0] {
            PlatformEvent::AccessibilityScrollIntoView { window, node_id } => {
                assert_eq!(*window, WindowId(1));
                assert_eq!(*node_id, 3);
            }
            other => panic!("expected AccessibilityScrollIntoView, got {other:?}"),
        }
    }

    #[test]
    fn adapter_activate_produces_focus_then_pointer() {
        let adapter = AccessibilityAdapter::new(1);
        let action = AccessibilityAction::Activate(NodeId::new(4));
        let events = adapter.route_action(&action);
        assert_eq!(
            events.len(),
            2,
            "Activate should produce FocusChanged + PointerButton"
        );
        // First event must be FocusChanged with the real node_id.
        match &events[0] {
            PlatformEvent::FocusChanged {
                window,
                gained,
                node_id,
            } => {
                assert_eq!(*window, WindowId(1));
                assert!(*gained);
                assert_eq!(*node_id, 4);
            }
            other => panic!("expected FocusChanged as first event, got {other:?}"),
        }
        // Second event is the click.
        match &events[1] {
            PlatformEvent::PointerButton {
                window, pressed, ..
            } => {
                assert_eq!(*window, WindowId(1));
                assert!(*pressed);
            }
            other => panic!("expected PointerButton as second event, got {other:?}"),
        }
    }

    #[test]
    fn adapter_build_tree_update_includes_focus() {
        let tree = column::<TestMsg>().child(label("Hi")).build();
        let snapshot = LayoutSnapshot::default();
        let mut adapter = AccessibilityAdapter::new(1);
        adapter.update(&tree, &snapshot);
        adapter.focus_widget(NodeId::new(2));

        let update = adapter
            .build_tree_update()
            .expect("build_tree_update should return Some");
        assert_eq!(update.focus, AccessNodeId::from(2u64));
    }

    #[test]
    fn adapter_build_tree_update_defaults_to_root_focus() {
        let tree = column::<TestMsg>().child(label("Hi")).build();
        let snapshot = LayoutSnapshot::default();
        let mut adapter = AccessibilityAdapter::new(1);
        adapter.update(&tree, &snapshot);
        // No focus_widget called → focus should be root (ID 1)

        let update = adapter
            .build_tree_update()
            .expect("build_tree_update should return Some");
        assert_eq!(update.focus, AccessNodeId::from(1u64));
    }

    #[test]
    fn adapter_build_tree_update_returns_none_without_update() {
        let adapter = AccessibilityAdapter::new(1);
        assert!(adapter.build_tree_update().is_none());
    }

    // ------------------------------------------------------------------
    // Coordinate resolution via LayoutSnapshot
    // ------------------------------------------------------------------

    fn snapshot_with_rect(id: NodeId, x: f32, y: f32, w: f32, h: f32) -> LayoutSnapshot {
        let mut snapshot = LayoutSnapshot::default();
        snapshot.insert(id, LayoutRect { x, y, width: w, height: h });
        snapshot
    }

    #[test]
    fn click_resolves_node_center_from_snapshot() {
        let mut adapter = AccessibilityAdapter::new(1);
        let id = NodeId::new(5);
        let snap = snapshot_with_rect(id, 100.0, 200.0, 50.0, 30.0);
        // Manually set snapshot (bypass update which needs a widget tree).
        adapter.snapshot = Some(snap);

        let action = AccessibilityAction::Click(id);
        let events = adapter.route_action(&action);
        assert_eq!(events.len(), 2);

        match &events[1] {
            PlatformEvent::PointerButton { x, y, .. } => {
                assert!(
                    (*x - 125.0).abs() < 0.001,
                    "expected center x=125, got {x}"
                );
                assert!(
                    (*y - 215.0).abs() < 0.001,
                    "expected center y=215, got {y}"
                );
            }
            other => panic!("expected PointerButton, got {other:?}"),
        }
    }

    #[test]
    fn activate_resolves_node_center_from_snapshot() {
        let mut adapter = AccessibilityAdapter::new(1);
        let id = NodeId::new(7);
        let snap = snapshot_with_rect(id, 10.0, 20.0, 80.0, 40.0);
        adapter.snapshot = Some(snap);

        let action = AccessibilityAction::Activate(id);
        let events = adapter.route_action(&action);
        assert_eq!(events.len(), 2);

        match &events[1] {
            PlatformEvent::PointerButton { x, y, .. } => {
                assert!((*x - 50.0).abs() < 0.001, "expected center x=50, got {x}");
                assert!((*y - 40.0).abs() < 0.001, "expected center y=40, got {y}");
            }
            other => panic!("expected PointerButton, got {other:?}"),
        }
    }

    #[test]
    fn click_falls_back_to_zero_without_snapshot() {
        let adapter = AccessibilityAdapter::new(1);
        let action = AccessibilityAction::Click(NodeId::new(5));
        let events = adapter.route_action(&action);
        assert_eq!(events.len(), 2);

        match &events[1] {
            PlatformEvent::PointerButton { x, y, .. } => {
                assert!((*x - 0.0).abs() < 0.001, "fallback x should be 0, got {x}");
                assert!((*y - 0.0).abs() < 0.001, "fallback y should be 0, got {y}");
            }
            other => panic!("expected PointerButton, got {other:?}"),
        }
    }

    #[test]
    fn click_falls_back_to_zero_for_missing_node_in_snapshot() {
        let mut adapter = AccessibilityAdapter::new(1);
        let snap = snapshot_with_rect(NodeId::new(1), 0.0, 0.0, 100.0, 100.0);
        adapter.snapshot = Some(snap);
        // Query a different node than what is in the snapshot.
        let action = AccessibilityAction::Click(NodeId::new(99));
        let events = adapter.route_action(&action);
        assert_eq!(events.len(), 2);

        match &events[1] {
            PlatformEvent::PointerButton { x, y, .. } => {
                assert!((*x - 0.0).abs() < 0.001, "missing node: x should be 0, got {x}");
                assert!((*y - 0.0).abs() < 0.001, "missing node: y should be 0, got {y}");
            }
            other => panic!("expected PointerButton, got {other:?}"),
        }
    }

    #[test]
    fn focus_is_unaffected_by_snapshot_absence() {
        let adapter = AccessibilityAdapter::new(1);
        // Focus does not depend on snapshot — it should always produce one event.
        let action = AccessibilityAction::Focus(NodeId::new(42));
        let events = adapter.route_action(&action);
        assert_eq!(events.len(), 1);
        match &events[0] {
            PlatformEvent::FocusChanged { node_id, .. } => {
                assert_eq!(*node_id, 42);
            }
            other => panic!("expected FocusChanged, got {other:?}"),
        }
    }

    // ------------------------------------------------------------------
    // Exactly-once delivery: every action produces a predictable number
    // of events, and no side-effect happens more than once per call.
    // ------------------------------------------------------------------

    #[test]
    fn focus_produces_exactly_one_event() {
        let adapter = AccessibilityAdapter::new(1);
        let events = adapter.route_action(&AccessibilityAction::Focus(NodeId::new(1)));
        assert_eq!(events.len(), 1, "Focus must produce exactly one event");
    }

    #[test]
    fn click_produces_exactly_two_events() {
        let adapter = AccessibilityAdapter::new(1);
        let events = adapter.route_action(&AccessibilityAction::Click(NodeId::new(1)));
        assert_eq!(events.len(), 2, "Click must produce exactly two events");
    }

    #[test]
    fn activate_produces_exactly_two_events() {
        let adapter = AccessibilityAdapter::new(1);
        let events = adapter.route_action(&AccessibilityAction::Activate(NodeId::new(1)));
        assert_eq!(events.len(), 2, "Activate must produce exactly two events");
    }

    #[test]
    fn set_value_produces_exactly_one_event() {
        let adapter = AccessibilityAdapter::new(1);
        let events =
            adapter.route_action(&AccessibilityAction::SetValue(NodeId::new(1), "test".into()));
        assert_eq!(
            events.len(),
            1,
            "SetValue must produce exactly one event"
        );
    }

    #[test]
    fn scroll_into_view_produces_exactly_one_event() {
        let adapter = AccessibilityAdapter::new(1);
        let events =
            adapter.route_action(&AccessibilityAction::ScrollIntoView(NodeId::new(1)));
        assert_eq!(
            events.len(),
            1,
            "ScrollIntoView must produce exactly one event"
        );
    }

    #[test]
    fn route_action_is_deterministic() {
        let mut adapter = AccessibilityAdapter::new(1);
        let snap = snapshot_with_rect(NodeId::new(1), 0.0, 0.0, 100.0, 100.0);
        adapter.snapshot = Some(snap);

        let action = AccessibilityAction::Click(NodeId::new(1));
        let first = adapter.route_action(&action);
        let second = adapter.route_action(&action);

        assert_eq!(
            first.len(),
            second.len(),
            "deterministic: same action must produce same event count"
        );
        // Verify both produce the same PointerButton position.
        if let (
            PlatformEvent::PointerButton { x: x1, y: y1, .. },
            PlatformEvent::PointerButton { x: x2, y: y2, .. },
        ) = (&first[1], &second[1])
        {
            assert!((x1 - x2).abs() < f32::EPSILON, "x must be identical");
            assert!((y1 - y2).abs() < f32::EPSILON, "y must be identical");
        } else {
            panic!("both calls must produce PointerButton");
        }
    }
}
