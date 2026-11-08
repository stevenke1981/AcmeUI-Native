//! AccessKit integration for AcmeUI Native.
//!
//! Provides functions to build [`TreeUpdate`] values from the widget tree
//! and layout snapshot used by the AcmeUI framework. Also provides a
//! per-window [`AccessibilityAdapter`] that bridges the accessibility tree
//! with the platform event system.

#![forbid(unsafe_op_in_unsafe_fn)]

use accesskit::{
    Action, ActionData, ActionRequest, Invalid, Node as AccessNode, NodeId as AccessNodeId, Rect,
    Role, Tree, TreeUpdate,
};
use acme_core::NodeId;
use acme_layout::LayoutSnapshot;
use acme_platform::{PlatformEvent, WindowId};
use acme_widgets::retained_binding::{RetainedBinding, SyntheticIds};
use acme_widgets::{TextInput, WidgetNode};

mod structure;
#[cfg(test)]
mod tests_requests;

/// Re-export AccessKit types used in the public API.
pub use accesskit::{Node as AccNode, NodeId as AccNodeId};

/// Converts an AcmeUI [`NodeId`] to an AccessKit [`AccessNodeId`].
#[inline]
pub fn to_access_id(id: NodeId) -> AccessNodeId {
    AccessNodeId::from(id.get())
}

/// How far inside a tree row's trailing edge a press lands, in logical pixels.
const TREE_ITEM_PRESS_INSET: f64 = 2.0;

/// Converts an AccessKit [`AccessNodeId`] back to an AcmeUI [`NodeId`]: the
/// exact inverse of [`to_access_id`], for layout IDs and virtual IDs alike.
#[inline]
pub fn from_access_id(id: AccessNodeId) -> NodeId {
    NodeId::new(id.0)
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
    /// Expand a collapsed tree item. Additive (ROADMAP 013): routed as a
    /// row selection followed by the `ArrowRight` key, as the keyboard does.
    Expand(NodeId),
    /// Collapse an expanded tree item. Additive (ROADMAP 013): routed as a
    /// row selection followed by the `ArrowLeft` key, as the keyboard does.
    Collapse(NodeId),
}

impl AccessibilityAction {
    /// Converts an AccessKit [`ActionRequest`] (what a platform adapter or a
    /// screen reader delivers) into an action. Pure, and never panics.
    ///
    /// | AccessKit action   | Result                                                   |
    /// | ------------------ | -------------------------------------------------------- |
    /// | `Focus`            | [`AccessibilityAction::Focus`]                           |
    /// | `Click`            | [`AccessibilityAction::Click`] (AccessKit 0.21 has no separate default action) |
    /// | `Expand`/`Collapse`| [`AccessibilityAction::Expand`] / [`AccessibilityAction::Collapse`] |
    /// | `ScrollIntoView`   | [`AccessibilityAction::ScrollIntoView`]                  |
    /// | `SetValue`         | [`AccessibilityAction::SetValue`], only with [`ActionData::Value`] |
    ///
    /// Any other action, or `SetValue` without a string value, is `None`. The
    /// target is converted with [`from_access_id`].
    pub fn from_request(request: &ActionRequest) -> Option<Self> {
        let target = from_access_id(request.target);
        match request.action {
            Action::Focus => Some(Self::Focus(target)),
            Action::Click => Some(Self::Click(target)),
            Action::Expand => Some(Self::Expand(target)),
            Action::Collapse => Some(Self::Collapse(target)),
            Action::ScrollIntoView => Some(Self::ScrollIntoView(target)),
            Action::SetValue => match &request.data {
                Some(ActionData::Value(value)) => Some(Self::SetValue(target, value.to_string())),
                _ => None,
            },
            _ => None,
        }
    }
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
    /// Walks `root` in DFS order, taking the IDs that [`WidgetNode::to_layout`]
    /// takes, so each widget that layout lowers keeps its layout ID and reads
    /// its layout rectangle. Tooltip and Popover wrappers, Dialog content,
    /// VirtualList children, the non-first children of a Popover, and grid cell
    /// content have no layout node of their own. They take IDs at or above
    /// 2^56, which never match a layout ID, and have no rectangle. Tree items,
    /// grid rows, and column headers are numbered by layout and read its
    /// rectangles, so their IDs match layout too.
    ///
    /// The result is stored as the current tree. The IDs restart at 1 on every
    /// call, so they are not stable across rebuilds. Use
    /// [`AccessibilityAdapter::update_with_binding`] with a [`RetainedBinding`]
    /// for IDs that stay stable across frames.
    pub fn update<M>(&mut self, root: &WidgetNode<M>, snapshot: &LayoutSnapshot)
    where
        M: Clone + 'static,
    {
        let mut next = 1u64;
        let mut nodes = Vec::new();
        let root_id = walk_node(root, snapshot, &mut next, None, &mut nodes);
        self.store(root_id, nodes, snapshot);
    }

    /// Rebuild the tree like [`AccessibilityAdapter::update`], but take every
    /// node's ID from `binding` instead of a per-call counter.
    ///
    /// `binding` must be bound to `root` with
    /// [`acme_widgets::retained_binding::bind_retained`] for the same frame.
    /// Each widget keeps its retained ID across frames, so the AccessKit ID and
    /// the `snapshot` lookup both stay stable, and match a layout computed with
    /// the same binding. Widgets the binding does not know take IDs from
    /// [`SyntheticIds`]; those are unique within the frame but are only
    /// guaranteed to match layout when layout allocates them in the same order.
    ///
    /// Tooltip and Popover nodes read the rectangle of their first child, as
    /// their layout nodes are that child's.
    pub fn update_with_binding<M>(
        &mut self,
        root: &WidgetNode<M>,
        snapshot: &LayoutSnapshot,
        binding: &RetainedBinding<'_>,
    ) where
        M: Clone + 'static,
    {
        let mut ids = IdSource::bound(binding);
        let mut nodes = Vec::new();
        let ctx = Ctx {
            snapshot,
            label_override: None,
        };
        let root_id = walk(
            root,
            ctx,
            &mut ids,
            &mut nodes,
            Slot::Plain,
            Lowering::Lowered,
        )
        .id;
        self.store(root_id, nodes, snapshot);
    }

    /// Replace the current tree and the stored layout snapshot.
    fn store(
        &mut self,
        root_id: AccessNodeId,
        nodes: Vec<(AccessNodeId, AccessNode)>,
        snapshot: &LayoutSnapshot,
    ) {
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
        if let Some(rect) = self.snapshot.as_ref().and_then(|s| s.get(id)) {
            return Some((rect.x + rect.width * 0.5, rect.y + rect.height * 0.5));
        }
        // A node with a virtual ID (retained walk) has no snapshot entry but
        // carries the bounds of its layout node.
        let tree = self.tree.as_ref()?;
        let target = to_access_id(id);
        let (_, node) = tree.nodes.iter().find(|(node_id, _)| *node_id == target)?;
        let b = node.bounds()?;
        Some((((b.x0 + b.x1) * 0.5) as f32, ((b.y0 + b.y1) * 0.5) as f32))
    }

    /// Route an AccessKit [`ActionRequest`]: [`AccessibilityAction::from_request`]
    /// followed by [`AccessibilityAdapter::route_action`]. An unsupported
    /// request yields no events.
    ///
    /// # Wiring a platform adapter
    ///
    /// Implement `accesskit::ActionHandler::do_action` (for example behind
    /// `accesskit_winit`) so that it forwards each request to the UI thread,
    /// call `handle_request` there, and dispatch the returned events in order
    /// through the normal event pipeline.
    pub fn handle_request(&self, request: &ActionRequest) -> Vec<PlatformEvent> {
        AccessibilityAction::from_request(request)
            .map(|action| self.route_action(&action))
            .unwrap_or_default()
    }

    /// The point a pointer press targets for `id`. A `TreeItem` is pressed at
    /// its trailing edge, inside the row, so the press selects the row instead
    /// of landing on its content. Row content (a checkbox, a disclosure arrow,
    /// an icon, a button) sits at the leading edge or the centre, and the end
    /// of a row is usually empty. Every other node is pressed at its centre.
    fn press_point(&self, id: NodeId) -> (f32, f32) {
        let tree_item_bounds = self.tree.as_ref().and_then(|tree| {
            let target = to_access_id(id);
            let (_, node) = tree.nodes.iter().find(|(node_id, _)| *node_id == target)?;
            (node.role() == Role::TreeItem)
                .then(|| node.bounds())
                .flatten()
        });
        match tree_item_bounds {
            Some(b) => {
                let inset = TREE_ITEM_PRESS_INSET.min((b.x1 - b.x0) / 2.0);
                ((b.x1 - inset) as f32, ((b.y0 + b.y1) * 0.5) as f32)
            }
            None => self.resolve_node_center(id).unwrap_or((0.0, 0.0)),
        }
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
    /// from the stored [`LayoutSnapshot`] (populated during [`update`](Self::update)).  If
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
                let (cx, cy) = self.press_point(*id);
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
                        shift: false,
                        ctrl: false,
                        alt: false,
                    },
                ]
            }
            AccessibilityAction::SetValue(_id, val) => {
                vec![PlatformEvent::ImeCommit {
                    window: WindowId(self.window_id),
                    text: val.clone(),
                }]
            }
            AccessibilityAction::Expand(id) | AccessibilityAction::Collapse(id) => {
                let key = if matches!(action, AccessibilityAction::Expand(_)) {
                    acme_platform::PlatformKey::ArrowRight
                } else {
                    acme_platform::PlatformKey::ArrowLeft
                };
                let window = WindowId(self.window_id);
                let (cx, cy) = self.press_point(*id);
                let key_event = |pressed| PlatformEvent::Key {
                    window,
                    key,
                    pressed,
                    shift: false,
                    ctrl: false,
                    alt: false,
                    meta: false,
                    repeat: false,
                    text: None,
                    character: None,
                };
                // Select the row as a click does, then expand or collapse it
                // with the key the keyboard uses for the selected row.
                vec![
                    PlatformEvent::FocusChanged {
                        window,
                        gained: true,
                        node_id: id.get(),
                    },
                    PlatformEvent::PointerButton {
                        window,
                        pressed: true,
                        x: cx,
                        y: cy,
                        button: 0,
                        pointer: 0,
                        shift: false,
                        ctrl: false,
                        alt: false,
                    },
                    key_event(true),
                    key_event(false),
                ]
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

    /// Forget the tracked widget focus, so the next [`TreeUpdate`] reports the
    /// root as focused. Call this when keyboard focus moves to something that
    /// has no accessibility node (for example a synthetic table row).
    pub fn clear_widget_focus(&mut self) {
        self.focused_node = None;
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
/// | Widget variant                      | AccessKit role                                                                                                                                              |
/// | ----------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
/// | `Row` / `Column` / `Stack` / `Card` | `Group`                                                                                                                                                     |
/// | `Label`                             | `Label`                                                                                                                                                     |
/// | `Button`                            | `Button`                                                                                                                                                    |
/// | `Separator`                         | `Splitter`                                                                                                                                                  |
/// | `ScrollView`                        | `ScrollView`                                                                                                                                                |
/// | `TextInput`                         | `TextInput`, or `PasswordInput` when `password`                                                                                                             |
/// | `TabBar`                            | `TabList`; each tab child is `Tab`, with `selected`                                                                                                         |
/// | `NavRail`                           | `Navigation`; the selected item is `selected`                                                                                                               |
/// | `Dialog`                            | `Dialog`, `modal` when `modal`                                                                                                                              |
/// | `Tree`                              | `Tree`; each visible row is a `TreeItem` child, in visible order                                                                                            |
/// | `Table` / `DataGrid`                | `Table` / `Grid`; a header `Row` of `ColumnHeader` cells, then one `Row` per data row; each cell child is `Cell` when its content is a `Label` or container |
///
/// A disabled `Button` has no `Click` action. A `TextInput` never exposes a
/// password's value. A `TextInput` with a `validation_message` is `invalid`,
/// and the message is its description.
///
/// The IDs and rectangles follow layout. A walk takes the IDs that layout
/// lowering takes, in layout's order: a tree's visible rows, a grid's header
/// leaves and header row, then each row's cell leaves and row container. A
/// sequential walk uses those IDs as AccessKit IDs, so a node reads its layout
/// node's rectangle. A retained walk keeps each widget's retained ID and gives
/// tree items, grid rows, column headers and grid cell content a virtual ID,
/// which never collides with a layout ID, while each still reads the rectangle
/// of its layout node. Widgets that layout does not number (dialog content,
/// the cell content of a grid, the non-first children of a popover) take a
/// virtual ID and have no rectangle. A grid whose cells match its rows is
/// grouped by each row's own cells, even when the rows differ in length. A
/// grid whose cells do not match its rows, or that has no columns, keeps its
/// cells flat.
///
/// A `TreeItem` has its `level`, and its `position_in_set` and `size_of_set`
/// among the visible items that share its parent (zero-based position, as
/// AccessKit expects). `expanded` is set only when the item has children, and
/// `selected` is `false` on unselected items. A `Row` has `row_index` (header
/// `0`, data rows from `1`); a cell has `row_index` and `column_index`, and the
/// table or grid has `row_count` (including the header) and `column_count`.
/// The selected row or cell is `selected`. A table whose mode selects rows
/// states `false` on its other rows; one whose mode selects cells does the same
/// for its other cells. A widget that layout does not lower, or whose rectangle
/// the snapshot lacks, has no bounds from the layout. Without a snapshot entry
/// for a row, the fallback is the table's rectangle: its width, 24 px per row,
/// clipped to it.
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
    build_with_ids(
        root,
        snapshot,
        &mut IdSource::sequential(&mut next_id),
        focus_id,
        label_override,
    )
}

/// Build a full [`TreeUpdate`] like [`build_tree_update`], but take the node
/// IDs from `binding` (see [`AccessibilityAdapter::update_with_binding`]).
///
/// `binding` must be bound to `root`. Unbound widgets take IDs from
/// [`SyntheticIds`].
pub fn build_tree_update_with_binding<M>(
    root: &WidgetNode<M>,
    snapshot: &LayoutSnapshot,
    binding: &RetainedBinding<'_>,
    focus_id: Option<NodeId>,
    label_override: Option<&str>,
) -> TreeUpdate
where
    M: Clone + 'static,
{
    let mut ids = IdSource::bound(binding);
    build_with_ids(root, snapshot, &mut ids, focus_id, label_override)
}

fn build_with_ids<M: Clone + 'static>(
    root: &WidgetNode<M>,
    snapshot: &LayoutSnapshot,
    ids: &mut IdSource<'_, '_>,
    focus_id: Option<NodeId>,
    label_override: Option<&str>,
) -> TreeUpdate {
    let mut nodes: Vec<(AccessNodeId, AccessNode)> = Vec::new();
    let ctx = Ctx {
        snapshot,
        label_override,
    };
    let root_id = walk(root, ctx, ids, &mut nodes, Slot::Plain, Lowering::Lowered).id;

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

/// Where the IDs of one walk come from.
///
/// A walk takes the IDs that layout lowering takes, in the same order as
/// `WidgetNode::to_layout` (sequential) and `WidgetNode::to_layout_with_binding`
/// (bound). Every node that has a layout node therefore reads that node's
/// rectangle. Nodes that layout does not number take [`structure::VirtualIds`].
enum IdSource<'s, 'b> {
    /// Layout's sequential counter.
    Sequential {
        next: &'s mut u64,
        virtual_ids: structure::VirtualIds,
    },
    /// Retained IDs from `binding`, and [`SyntheticIds`] for what layout
    /// numbers without a retained ID.
    Bound {
        binding: &'b RetainedBinding<'b>,
        synthetic: SyntheticIds,
        virtual_ids: structure::VirtualIds,
    },
}

impl<'s, 'b> IdSource<'s, 'b> {
    fn sequential(next: &'s mut u64) -> Self {
        Self::Sequential {
            next,
            virtual_ids: structure::VirtualIds::new(),
        }
    }

    fn bound(binding: &'b RetainedBinding<'b>) -> Self {
        Self::Bound {
            binding,
            synthetic: SyntheticIds::new(),
            virtual_ids: structure::VirtualIds::new(),
        }
    }
}

/// Takes the next value of a sequential counter.
fn take_next(next: &mut u64) -> NodeId {
    let id = NodeId::new(*next);
    *next += 1;
    id
}

impl IdSource<'_, '_> {
    /// The layout ID of a lowered widget, taken before its children: the
    /// counter, or the widget's retained ID (a synthetic ID when unbound).
    fn lowered_widget<M>(&mut self, widget: &WidgetNode<M>) -> NodeId {
        match self {
            Self::Sequential { next, .. } => take_next(next),
            Self::Bound {
                binding, synthetic, ..
            } => binding
                .id_for(widget)
                .unwrap_or_else(|| synthetic.allocate()),
        }
    }

    /// The next layout ID of a node that has no widget of its own: a tree row,
    /// a header leaf or row, a cell leaf or a row container.
    fn structural(&mut self) -> NodeId {
        match self {
            Self::Sequential { next, .. } => take_next(next),
            Self::Bound { synthetic, .. } => synthetic.allocate(),
        }
    }

    /// The AccessKit ID of a node layout numbers as `layout` (`None` when it has
    /// no layout node). A sequential walk uses the layout ID, so AccessKit IDs
    /// are layout IDs; a retained walk uses a virtual ID.
    fn structural_identity(&mut self, layout: Option<NodeId>) -> AccessNodeId {
        match self {
            Self::Sequential { virtual_ids, .. } => {
                layout.map_or_else(|| to_access_id(virtual_ids.allocate()), to_access_id)
            }
            Self::Bound { virtual_ids, .. } => to_access_id(virtual_ids.allocate()),
        }
    }

    /// The AccessKit ID of a widget that is not numbered as a widget of its own:
    /// a grid cell (`layout` is its leaf), a layout-transparent wrapper, or a
    /// widget under a leaf or outside layout. A retained walk keeps the retained
    /// ID of a bound widget.
    fn detached_identity<M>(
        &mut self,
        widget: &WidgetNode<M>,
        layout: Option<NodeId>,
    ) -> AccessNodeId {
        let retained = match self {
            Self::Bound { binding, .. } => binding.id_for(widget),
            Self::Sequential { .. } => None,
        };
        retained.map_or_else(|| self.structural_identity(layout), to_access_id)
    }
}

/// How layout treats a widget during a walk.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Lowering {
    /// Layout lowers the widget and numbers it, along with the structure it owns.
    Lowered,
    /// A grid cell: layout has one leaf for the cell, with this ID, and nothing
    /// for the cell's content.
    Cell(NodeId),
    /// Layout has no node for the widget or anything under it.
    Unlowered,
}

/// The AccessKit ID of a walked node, and the layout node whose rectangle it
/// reads.
#[derive(Clone, Copy, Debug)]
struct Walked {
    id: AccessNodeId,
    layout: Option<NodeId>,
}

/// What a walk reads from the layout snapshot, and the label override.
#[derive(Clone, Copy)]
struct Ctx<'a> {
    snapshot: &'a LayoutSnapshot,
    label_override: Option<&'a str>,
}

/// Where a node sits inside its parent, when the parent gives it extra meaning.
#[derive(Clone, Copy, Debug)]
enum Slot<'a> {
    /// No parent-specific meaning: role and state come from the widget alone.
    Plain,
    /// A tab of a TabBar: `Role::Tab`, named by the tab's own label.
    Tab { label: &'a str, selected: bool },
    /// An item of a NavRail: named by the item's own label.
    NavItem { label: &'a str, selected: bool },
    /// A cell child of a Table or DataGrid. Text and plain containers take
    /// `Role::Cell`; controls keep their own role (see [`walk`]). `place` is set
    /// when the cells are grouped into rows; `selected` is `Some(true)` for the
    /// selected cell, `Some(false)` for the other cells when the grid selects
    /// cells, and `None` otherwise.
    Cell {
        place: Option<structure::CellPlace>,
        selected: Option<bool>,
    },
}

/// The slot of child `index` within `parent`.
///
/// TabBar and NavRail children are materialized one per tab or item, in the
/// same order, so the index maps onto the parent's `tabs` or `items`.
fn child_slot<M>(parent: &WidgetNode<M>, index: usize) -> Slot<'_> {
    match parent {
        // Tabs and items map to children by index; if a caller mutated one list
        // after `build()`, the mapping is unreliable, so expose plain children.
        WidgetNode::TabBar(bar) if bar.tabs.len() != bar.children.len() => Slot::Plain,
        WidgetNode::NavRail(rail) if rail.items.len() != rail.children.len() => Slot::Plain,
        WidgetNode::TabBar(bar) => bar.tabs.get(index).map_or(Slot::Plain, |tab| Slot::Tab {
            label: &tab.label,
            selected: index == bar.selected,
        }),
        WidgetNode::NavRail(rail) => {
            rail.items
                .get(index)
                .map_or(Slot::Plain, |item| Slot::NavItem {
                    label: &item.label,
                    selected: rail.selected == Some(index),
                })
        }
        _ => Slot::Plain,
    }
}

/// How layout treats child `index` of `parent`, when `parent` is `lowering`.
fn child_lowering<M>(parent: &WidgetNode<M>, index: usize, lowering: Lowering) -> Lowering {
    match (lowering, parent) {
        // A Dialog's content and a VirtualList's children are not laid out.
        (Lowering::Lowered, WidgetNode::Dialog(_) | WidgetNode::VirtualList(_)) => {
            Lowering::Unlowered
        }
        // A Popover lays out only its first child, in its own place.
        (Lowering::Lowered, WidgetNode::Popover(_)) if index > 0 => Lowering::Unlowered,
        (Lowering::Lowered, _) => Lowering::Lowered,
        // Under a grid cell, or under anything that is not laid out, nothing is.
        _ => Lowering::Unlowered,
    }
}

/// The AccessKit role of a widget, before any parent-specific override.
fn widget_role<M>(node: &WidgetNode<M>) -> Role {
    match node {
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
        WidgetNode::TextInput(input) if input.password => Role::PasswordInput,
        WidgetNode::TextInput(_) => Role::TextInput,
        WidgetNode::NavRail(_) | WidgetNode::Sidebar(_) | WidgetNode::Breadcrumb(_) => {
            Role::Navigation
        }
        WidgetNode::TabBar(_) => Role::TabList,
    }
}

/// Recursively walk the widget tree, producing AccessKit nodes.
///
/// IDs are sequential, starting at `*next_id`, as `to_layout` numbers them;
/// `*next_id` is advanced past the last layout ID taken. Returns the root's
/// AccessKit ID.
pub fn walk_node<M: Clone + 'static>(
    node: &WidgetNode<M>,
    snapshot: &LayoutSnapshot,
    next_id: &mut u64,
    label_override: Option<&str>,
    nodes: &mut Vec<(AccessNodeId, AccessNode)>,
) -> AccessNodeId {
    let mut ids = IdSource::sequential(next_id);
    let ctx = Ctx {
        snapshot,
        label_override,
    };
    walk(node, ctx, &mut ids, nodes, Slot::Plain, Lowering::Lowered).id
}

/// Walk `node` and its descendants, taking IDs from `ids` in layout's order.
///
/// `lowering` says how layout treats `node`. The node's own ID comes first,
/// then its structure (a tree's rows, a grid's header and rows), then its
/// children: the order in which layout lowering takes them.
fn walk<M: Clone + 'static>(
    node: &WidgetNode<M>,
    ctx: Ctx<'_>,
    ids: &mut IdSource<'_, '_>,
    nodes: &mut Vec<(AccessNodeId, AccessNode)>,
    slot: Slot<'_>,
    lowering: Lowering,
) -> Walked {
    let (id, own_layout) = own_identity(node, lowering, ids);
    let is_lowered = lowering == Lowering::Lowered;
    let plan = take_plan(node, is_lowered, ids);
    let children = walk_children(node, ctx, ids, nodes, lowering, &plan);
    // A layout-transparent wrapper is laid out as its first child.
    let layout = if is_transparent(node) && is_lowered {
        children.first().and_then(|child| child.layout)
    } else {
        own_layout
    };

    let mut access_node = AccessNode::new(role_of(node, slot));
    if let Some(rect) = layout.and_then(|layout_id| ctx.snapshot.get(layout_id)) {
        access_node.set_bounds(structure::rect_of(*rect));
    }
    let label = node_label(node, slot, ctx.label_override);
    if !label.is_empty() {
        access_node.set_label(label);
    }
    apply_widget_state(&mut access_node, node, slot);

    let child_ids = structure_child_ids(node, layout, plan, &children, ctx, ids, nodes);
    if !child_ids.is_empty() {
        access_node.set_children(child_ids);
    }

    nodes.push((id, access_node));
    Walked { id, layout }
}

/// The AccessKit ID of `node` and the layout node whose rectangle it reads.
/// The ID is taken before the node's structure and children, as layout does.
fn own_identity<M>(
    node: &WidgetNode<M>,
    lowering: Lowering,
    ids: &mut IdSource<'_, '_>,
) -> (AccessNodeId, Option<NodeId>) {
    match lowering {
        // A layout-transparent wrapper takes no ID: its first child's layout
        // node is its own, and is read once that child has been walked.
        Lowering::Lowered if is_transparent(node) => (ids.detached_identity(node, None), None),
        Lowering::Lowered => {
            let own = ids.lowered_widget(node);
            (to_access_id(own), Some(own))
        }
        Lowering::Cell(leaf) => (ids.detached_identity(node, Some(leaf)), Some(leaf)),
        Lowering::Unlowered => (ids.detached_identity(node, None), None),
    }
}

/// What layout numbers below a node, taken before its children.
enum Plan {
    /// A table or grid: its header leaves and header row, then each row's cells and row container.
    Grid(structure::GridLayout),
    /// A tree: one layout ID per visible row.
    Tree(Vec<Option<NodeId>>),
    /// Anything else: nothing beyond the children.
    Flat,
}

fn take_plan<M: Clone + 'static>(
    node: &WidgetNode<M>,
    lowered: bool,
    ids: &mut IdSource<'_, '_>,
) -> Plan {
    match node {
        WidgetNode::Table(_) | WidgetNode::DataGrid(_) => {
            Plan::Grid(structure::plan_grid(node, lowered, ids))
        }
        WidgetNode::Tree(tree) => Plan::Tree(structure::tree_row_layouts(tree, lowered, ids)),
        _ => Plan::Flat,
    }
}

/// Walks the children of `node`, in order. A grid's children are its cells,
/// each lowered as its leaf in `plan`.
fn walk_children<M: Clone + 'static>(
    node: &WidgetNode<M>,
    ctx: Ctx<'_>,
    ids: &mut IdSource<'_, '_>,
    nodes: &mut Vec<(AccessNodeId, AccessNode)>,
    lowering: Lowering,
    plan: &Plan,
) -> Vec<Walked> {
    match plan {
        Plan::Grid(grid) => structure::cell_slots(node, grid)
            .into_iter()
            .zip(node.children())
            .map(|(cell, child)| {
                let slot = Slot::Cell {
                    place: cell.place,
                    selected: cell.selected,
                };
                walk(child, ctx, ids, nodes, slot, cell.lowering)
            })
            .collect(),
        Plan::Tree(_) | Plan::Flat => node
            .children()
            .iter()
            .enumerate()
            .map(|(index, child)| {
                let child_lowering = child_lowering(node, index, lowering);
                walk(
                    child,
                    ctx,
                    ids,
                    nodes,
                    child_slot(node, index),
                    child_lowering,
                )
            })
            .collect(),
    }
}

/// The AccessKit IDs of the children of `node`: a tree's rows, a grid's header
/// and row nodes, or the walked children themselves.
fn structure_child_ids<M: Clone + 'static>(
    node: &WidgetNode<M>,
    layout: Option<NodeId>,
    plan: Plan,
    children: &[Walked],
    ctx: Ctx<'_>,
    ids: &mut IdSource<'_, '_>,
    nodes: &mut Vec<(AccessNodeId, AccessNode)>,
) -> Vec<AccessNodeId> {
    let walked: Vec<AccessNodeId> = children.iter().map(|child| child.id).collect();
    match (node, plan) {
        (WidgetNode::Tree(tree), Plan::Tree(rows)) => {
            structure::tree_item_ids(tree, layout, &rows, ctx, ids, nodes)
        }
        (WidgetNode::Table(_) | WidgetNode::DataGrid(_), Plan::Grid(grid)) => {
            structure::grid_row_ids(node, &grid, layout, children, ctx, ids, nodes)
                .unwrap_or(walked)
        }
        _ => walked,
    }
}

/// Whether `node` is a layout-transparent wrapper: a Tooltip or a Popover.
fn is_transparent<M>(node: &WidgetNode<M>) -> bool {
    matches!(node, WidgetNode::Tooltip(_) | WidgetNode::Popover(_))
}

/// The AccessKit role of `node` in `slot`.
fn role_of<M>(node: &WidgetNode<M>, slot: Slot<'_>) -> Role {
    match (slot, widget_role(node)) {
        (Slot::Tab { .. }, _) => Role::Tab,
        // A cell's text or plain container is the cell itself. A control or a
        // nested composite keeps its own role: AccessKit has no wrapper node
        // to hold it, and `Cell` would hide its activation semantics.
        (Slot::Cell { .. }, Role::Label | Role::Group) => Role::Cell,
        (_, own) => own,
    }
}

/// The accessible name of `node` in `slot`, or an empty string.
///
/// A tab or nav item is named by its own label, because the widget text carries
/// visual decoration ("[Tab]", "• Item"). An explicit `label_override` still
/// replaces the text of every Label.
fn node_label<M>(node: &WidgetNode<M>, slot: Slot<'_>, label_override: Option<&str>) -> String {
    match (slot, node) {
        (_, WidgetNode::Label(_)) if label_override.is_some() => {
            label_override.unwrap_or_default().to_string()
        }
        (Slot::Tab { label, .. } | Slot::NavItem { label, .. }, _) => label.to_string(),
        (_, WidgetNode::Label(l)) => l.text.clone(),
        (_, WidgetNode::Button(b)) => b.label.clone(),
        (_, WidgetNode::TextInput(input)) => text_input_name(input).to_string(),
        _ => String::new(),
    }
}

/// Accessible name of a text input: its label, else its placeholder.
fn text_input_name<M>(input: &TextInput<M>) -> &str {
    non_empty(input.label.as_deref())
        .or_else(|| non_empty(input.placeholder.as_deref()))
        .unwrap_or("")
}

fn non_empty(text: Option<&str>) -> Option<&str> {
    text.filter(|text| !text.is_empty())
}

/// Adds the state and actions that `node` exposes beyond its name and role.
fn apply_widget_state<M>(access_node: &mut AccessNode, node: &WidgetNode<M>, slot: Slot<'_>) {
    match node {
        WidgetNode::Button(button) => {
            if button.disabled {
                access_node.set_disabled();
            }
            access_node.add_action(Action::Focus);
            // A disabled button must not advertise activation.
            if !button.disabled {
                access_node.add_action(Action::Click);
            }
        }
        WidgetNode::TextInput(input) => apply_text_input_state(access_node, input),
        WidgetNode::Dialog(dialog) => {
            if dialog.modal {
                access_node.set_modal();
            }
        }
        WidgetNode::Table(_) | WidgetNode::DataGrid(_) => {
            structure::set_grid_counts(node, access_node);
        }
        _ => {}
    }
    match slot {
        // A tab is selectable, so an unselected tab states `false` explicitly.
        Slot::Tab { selected, .. } => access_node.set_selected(selected),
        // A nav item is button-like: omit the flag rather than announce "not selected".
        Slot::NavItem { selected: true, .. } => access_node.set_selected(true),
        Slot::Cell { place, selected } => {
            if let Some(place) = place {
                access_node.set_row_index(place.row_index());
                access_node.set_column_index(place.column);
            }
            if let Some(selected) = selected {
                access_node.set_selected(selected);
            }
        }
        Slot::Plain
        | Slot::NavItem {
            selected: false, ..
        } => {}
    }
}

/// Text input state and actions.
///
/// A password's value is never written to the node: it carries no value
/// property at all, so the secret cannot reach any assistive technology.
fn apply_text_input_state<M>(access_node: &mut AccessNode, input: &TextInput<M>) {
    if !input.password {
        access_node.set_value(input.value.clone());
    }
    if input.disabled {
        access_node.set_disabled();
    }
    if input.readonly {
        access_node.set_read_only();
    }
    // A validation message implies an invalid value. AccessKit 0.21 has an
    // ErrorMessage relation, but it needs a separate node holding the text;
    // the message is exposed as the description instead, which adds no node.
    if input.invalid || input.validation_message.is_some() {
        access_node.set_invalid(Invalid::True);
    }
    if let Some(message) = non_empty(input.validation_message.as_deref()) {
        access_node.set_description(message);
    }
    access_node.add_action(Action::Focus);
    // Only an editable field accepts a value from assistive technology.
    if !input.disabled && !input.readonly {
        access_node.add_action(Action::SetValue);
    }
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
    use acme_layout::LayoutRect;
    use acme_widgets::retained_binding::bind_retained;
    use acme_widgets::view_bridge::widget_to_views;
    use acme_widgets::{
        DataGridColumn, DataGridRow, TabItem, TableColumn, TableRow, WidgetNode, button, column,
        datagrid, dialog, label, nav_item, nav_rail, row, scroll_view, separator, stack, tab_bar,
        table, text_input, tooltip,
    };

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
    fn label_override_still_renames_label_only_tabs() {
        let bar: WidgetNode<TestMsg> = tab_bar("t").tab("A").tab("B").build();
        let snapshot = LayoutSnapshot::default();
        let update = build_tree_update(&bar, &snapshot, None, Some("X"));
        let tabs: Vec<_> = update
            .nodes
            .iter()
            .filter(|(_, n)| n.role() == Role::Tab)
            .map(|(_, n)| n.label().map(str::to_string))
            .collect();
        assert_eq!(tabs, vec![Some("X".to_string()), Some("X".to_string())]);
    }

    #[test]
    fn tab_semantics_are_dropped_when_tabs_and_children_diverge() {
        let mut bar = match tab_bar::<TestMsg>("t").tab("A").tab("B").build() {
            WidgetNode::TabBar(bar) => bar,
            other => panic!("expected TabBar, got {other:?}"),
        };
        bar.tabs.pop();
        let node = WidgetNode::TabBar(bar);
        let update = build_tree_update(&node, &LayoutSnapshot::default(), None, None);
        assert!(
            update.nodes.iter().all(|(_, n)| n.role() != Role::Tab),
            "diverged tab lists must not produce mislabelled tabs"
        );
    }

    #[test]
    fn clear_widget_focus_returns_focus_to_the_root() {
        let tree = column::<TestMsg>().child(label("Hi")).build();
        let snapshot = LayoutSnapshot::default();
        let mut adapter = AccessibilityAdapter::new(1);
        adapter.update(&tree, &snapshot);
        adapter.focus_widget(NodeId::new(2));
        adapter.clear_widget_focus();

        let update = adapter
            .build_tree_update()
            .expect("build_tree_update should return Some");
        assert_eq!(update.focus, AccessNodeId::from(1u64));
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
        snapshot.insert(
            id,
            LayoutRect {
                x,
                y,
                width: w,
                height: h,
            },
        );
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
                assert!((*x - 125.0).abs() < 0.001, "expected center x=125, got {x}");
                assert!((*y - 215.0).abs() < 0.001, "expected center y=215, got {y}");
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
                assert!(
                    (*x - 0.0).abs() < 0.001,
                    "missing node: x should be 0, got {x}"
                );
                assert!(
                    (*y - 0.0).abs() < 0.001,
                    "missing node: y should be 0, got {y}"
                );
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
        let events = adapter.route_action(&AccessibilityAction::SetValue(
            NodeId::new(1),
            "test".into(),
        ));
        assert_eq!(events.len(), 1, "SetValue must produce exactly one event");
    }

    #[test]
    fn scroll_into_view_produces_exactly_one_event() {
        let adapter = AccessibilityAdapter::new(1);
        let events = adapter.route_action(&AccessibilityAction::ScrollIntoView(NodeId::new(1)));
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

    // ------------------------------------------------------------------
    // Retained binding (update_with_binding / build_tree_update_with_binding)
    // ------------------------------------------------------------------

    fn reconcile_frame(tree: &mut RetainedTree, roots: &[WidgetNode<TestMsg>]) {
        tree.reconcile_roots(&widget_to_views(roots))
            .expect("reconciliation should succeed");
    }

    /// A row of buttons, each keyed and labelled with its key.
    fn keyed_buttons(keys: &[&str]) -> WidgetNode<TestMsg> {
        let mut r = row::<TestMsg>();
        for key in keys {
            r = r.child(button::<TestMsg>(*key, *key));
        }
        r.build()
    }

    fn find_button(tree: &AccessibilityTree, label: &str) -> (AccessNodeId, AccessNode) {
        tree.nodes
            .iter()
            .find(|(_, n)| n.role() == Role::Button && n.label() == Some(label))
            .cloned()
            .unwrap_or_else(|| panic!("expected a button labelled {label}"))
    }

    #[test]
    fn bound_update_uses_binding_ids_for_every_node() {
        let mut tree = RetainedTree::new();
        let roots = vec![keyed_buttons(&["a", "b"])];
        reconcile_frame(&mut tree, &roots);
        let binding = bind_retained(&roots, &tree);
        assert_eq!(binding.mismatches(), 0);

        let root_id = binding.id_for(&roots[0]).expect("root should be bound");
        let [button_a, button_b] = roots[0].children() else {
            panic!("expected two buttons");
        };
        let expected = [
            root_id,
            binding.id_for(button_a).expect("a should be bound"),
            binding.id_for(button_b).expect("b should be bound"),
        ];

        let mut adapter = AccessibilityAdapter::new(1);
        adapter.update_with_binding(&roots[0], &LayoutSnapshot::default(), &binding);
        let t = adapter.tree_ref().expect("tree should exist after update");

        assert_eq!(t.root, to_access_id(root_id));
        assert_eq!(t.nodes.len(), expected.len());
        for id in expected {
            assert!(
                t.nodes
                    .iter()
                    .any(|(node_id, _)| *node_id == to_access_id(id)),
                "retained id {id:?} missing from accessibility tree"
            );
        }
        assert_eq!(find_button(t, "a").0, to_access_id(expected[1]));
        assert_eq!(find_button(t, "b").0, to_access_id(expected[2]));
    }

    #[test]
    fn widget_keeps_accessibility_id_when_keyed_sibling_is_inserted_before_it() {
        let mut tree = RetainedTree::new();
        let mut ids_by_frame = Vec::new();
        for keys in [&["a", "b"][..], &["new", "a", "b"][..]] {
            let roots = vec![keyed_buttons(keys)];
            reconcile_frame(&mut tree, &roots);
            let binding = bind_retained(&roots, &tree);
            let mut adapter = AccessibilityAdapter::new(1);
            adapter.update_with_binding(&roots[0], &LayoutSnapshot::default(), &binding);
            let t = adapter.tree_ref().expect("tree should exist");
            ids_by_frame.push((find_button(t, "a").0, find_button(t, "b").0));
        }

        assert_eq!(ids_by_frame[0], ids_by_frame[1]);
    }

    #[test]
    fn retained_ids_focus_and_bounds_survive_reorder_insert_and_remove() {
        let mut tree = RetainedTree::new();
        let mut adapter = AccessibilityAdapter::new(1);
        let mut stable = None;
        for keys in [
            &["a", "b"][..],
            &["new", "b", "a"][..],
            &["b"][..],
            &["b", "last"][..],
        ] {
            let roots = vec![keyed_buttons(keys)];
            reconcile_frame(&mut tree, &roots);
            let binding = bind_retained(&roots, &tree);
            assert_eq!(binding.mismatches(), 0);
            let layout = roots[0].to_layout_with_binding(
                &binding,
                &acme_layout::WidgetLayoutContext {
                    body_font_size: 14.0,
                    body_line_height: 21.0,
                    label_font_size: 13.0,
                    control_height: 36.0,
                    scale_factor: 1.0,
                },
            );
            let snapshot = LayoutEngine::new()
                .compute(&layout, (400.0, 100.0))
                .expect("bound layout should compute");
            adapter.update_with_binding(&roots[0], &snapshot, &binding);
            let (id, node) = find_button(adapter.tree_ref().expect("tree exists"), "b");
            let retained = binding
                .id_for(
                    &roots[0].children()[keys
                        .iter()
                        .position(|key| *key == "b")
                        .expect("b is present in every test frame")],
                )
                .expect("b must be bound");
            assert_eq!(id, to_access_id(retained));
            if let Some(previous) = stable {
                assert_eq!(id, previous, "b changed ID in frame {keys:?}");
            }
            stable = Some(id);
            let rect = snapshot.get(retained).expect("b has layout bounds");
            assert_eq!(
                node.bounds(),
                Some(Rect::new(
                    f64::from(rect.x),
                    f64::from(rect.y),
                    f64::from(rect.x + rect.width),
                    f64::from(rect.y + rect.height),
                )),
                "b must read its own bounds after {keys:?}"
            );
            adapter.focus_widget(retained);
            let update = adapter.build_tree_update().expect("tree update exists");
            assert_eq!(update.focus, id);
            let free_update = build_tree_update_with_binding(
                &roots[0],
                &snapshot,
                &binding,
                Some(retained),
                None,
            );
            assert_eq!(free_update.focus, id);
            assert_eq!(free_update.nodes, update.nodes);
        }
    }

    #[test]
    fn click_resolves_center_from_snapshot_keyed_by_retained_id() {
        let mut tree = RetainedTree::new();
        let roots = vec![keyed_buttons(&["a", "b"])];
        reconcile_frame(&mut tree, &roots);
        let binding = bind_retained(&roots, &tree);
        let [button_a, button_b] = roots[0].children() else {
            panic!("expected two buttons");
        };
        let id_a = binding.id_for(button_a).expect("a should be bound");
        let id_b = binding.id_for(button_b).expect("b should be bound");

        // The neighbour's ID carries a different rectangle; Click(b) must not
        // read it.
        let mut snapshot = LayoutSnapshot::default();
        snapshot.insert(
            id_a,
            LayoutRect {
                x: 0.0,
                y: 0.0,
                width: 10.0,
                height: 10.0,
            },
        );
        snapshot.insert(
            id_b,
            LayoutRect {
                x: 100.0,
                y: 200.0,
                width: 50.0,
                height: 30.0,
            },
        );

        let mut adapter = AccessibilityAdapter::new(7);
        adapter.update_with_binding(&roots[0], &snapshot, &binding);

        let t = adapter.tree_ref().expect("tree should exist");
        let (_, node_b) = find_button(t, "b");
        assert_eq!(node_b.bounds(), Some(Rect::new(100.0, 200.0, 150.0, 230.0)));

        let events = adapter.route_action(&AccessibilityAction::Click(id_b));
        assert_eq!(events.len(), 2, "Click should produce two events");
        match &events[0] {
            PlatformEvent::FocusChanged { node_id, .. } => {
                assert_eq!(*node_id, id_b.get(), "focus must use the retained ID");
            }
            other => panic!("expected FocusChanged, got {other:?}"),
        }
        match &events[1] {
            PlatformEvent::PointerButton { x, y, .. } => {
                assert!((*x - 125.0).abs() < 0.001, "expected center x=125, got {x}");
                assert!((*y - 215.0).abs() < 0.001, "expected center y=215, got {y}");
            }
            other => panic!("expected PointerButton, got {other:?}"),
        }
    }

    #[test]
    fn widgets_missing_from_binding_take_synthetic_ids() {
        let roots = vec![keyed_buttons(&["a", "b"])];
        let other: Vec<WidgetNode<TestMsg>> = Vec::new();
        let binding = bind_retained(&other, &RetainedTree::new());
        assert!(binding.is_empty());

        let mut adapter = AccessibilityAdapter::new(1);
        adapter.update_with_binding(&roots[0], &LayoutSnapshot::default(), &binding);
        let t = adapter.tree_ref().expect("tree should exist");

        // Pre-order allocation: root, then a, then b.
        let mut synthetic = SyntheticIds::new();
        let expected: Vec<AccessNodeId> =
            (0..3).map(|_| to_access_id(synthetic.allocate())).collect();
        assert_eq!(t.root, expected[0]);
        assert_eq!(t.nodes.len(), expected.len());
        for (id, _) in &t.nodes {
            assert!(expected.contains(id), "unexpected id {id:?}");
        }
    }

    #[test]
    fn tooltip_node_reads_its_child_rectangle_as_layout_does() {
        let mut tree = RetainedTree::new();
        let roots: Vec<WidgetNode<TestMsg>> = vec![
            column::<TestMsg>()
                .child(tooltip("tip", button::<TestMsg>("ok", "OK"), "help"))
                .build(),
        ];
        reconcile_frame(&mut tree, &roots);
        let binding = bind_retained(&roots, &tree);

        let tip = &roots[0].children()[0];
        let child = &tip.children()[0];
        let id_tip = binding.id_for(tip).expect("tooltip should be bound");
        let id_child = binding.id_for(child).expect("child should be bound");

        let mut snapshot = LayoutSnapshot::default();
        snapshot.insert(
            id_child,
            LayoutRect {
                x: 10.0,
                y: 20.0,
                width: 30.0,
                height: 40.0,
            },
        );

        let mut adapter = AccessibilityAdapter::new(1);
        adapter.update_with_binding(&roots[0], &snapshot, &binding);
        let t = adapter.tree_ref().expect("tree should exist");

        let (tooltip_id, tooltip_node) = t
            .nodes
            .iter()
            .find(|(_, n)| n.role() == Role::Tooltip)
            .expect("expected a tooltip node");
        assert_eq!(*tooltip_id, to_access_id(id_tip));
        assert_eq!(
            tooltip_node.bounds(),
            Some(Rect::new(10.0, 20.0, 40.0, 60.0)),
            "tooltip reads the child's layout rectangle"
        );
    }

    #[test]
    fn free_bound_build_matches_focus_and_node_count() {
        let mut tree = RetainedTree::new();
        let roots = vec![keyed_buttons(&["a", "b"])];
        reconcile_frame(&mut tree, &roots);
        let binding = bind_retained(&roots, &tree);
        let [_, button_b] = roots[0].children() else {
            panic!("expected two buttons");
        };
        let id_b = binding.id_for(button_b).expect("b should be bound");

        let update = build_tree_update_with_binding(
            &roots[0],
            &LayoutSnapshot::default(),
            &binding,
            Some(id_b),
            None,
        );

        assert_eq!(update.nodes.len(), 3);
        assert_eq!(update.focus, to_access_id(id_b));
    }

    // ------------------------------------------------------------------
    // Control semantics: roles, names, state and actions of core controls.
    // ------------------------------------------------------------------

    fn nodes_of(root: &WidgetNode<TestMsg>) -> Vec<(AccessNodeId, AccessNode)> {
        build_tree_update(root, &LayoutSnapshot::default(), None, None).nodes
    }

    fn node_with_id(nodes: &[(AccessNodeId, AccessNode)], id: AccessNodeId) -> &AccessNode {
        nodes
            .iter()
            .find(|(node_id, _)| *node_id == id)
            .map(|(_, node)| node)
            .expect("child id should be in the tree")
    }

    fn nodes_with_role(nodes: &[(AccessNodeId, AccessNode)], role: Role) -> Vec<&AccessNode> {
        nodes
            .iter()
            .filter(|(_, node)| node.role() == role)
            .map(|(_, node)| node)
            .collect()
    }

    fn only_node_with_role(nodes: &[(AccessNodeId, AccessNode)], role: Role) -> &AccessNode {
        let found = nodes_with_role(nodes, role);
        assert_eq!(found.len(), 1, "expected exactly one {role:?} node");
        found[0]
    }

    /// The accessibility node of a single text input built from `input`.
    fn input_node(input: impl Into<WidgetNode<TestMsg>>) -> AccessNode {
        let tree = column::<TestMsg>().child(input).build();
        nodes_of(&tree)
            .into_iter()
            .map(|(_, node)| node)
            .find(|node| matches!(node.role(), Role::TextInput | Role::PasswordInput))
            .expect("expected a text input node")
    }

    fn tab_strip(selected: usize) -> WidgetNode<TestMsg> {
        tab_bar::<TestMsg>("tabs")
            .item(TabItem::new("Home").on_click(TestMsg))
            .item(TabItem::new("Files").on_click(TestMsg))
            .tab("Logs")
            .selected(selected)
            .build()
    }

    fn nav(selected: usize) -> WidgetNode<TestMsg> {
        nav_rail::<TestMsg>("nav")
            .item(nav_item("Home").on_click(TestMsg))
            .item(nav_item("Settings").on_click(TestMsg))
            .selected(selected)
            .build()
    }

    // --- TextInput -----------------------------------------------------

    #[test]
    fn text_input_is_named_by_its_label_before_its_placeholder() {
        let node = input_node(
            text_input::<TestMsg>("name")
                .label("Full name")
                .placeholder("e.g. Ada"),
        );
        assert_eq!(node.label(), Some("Full name"));
    }

    #[test]
    fn text_input_without_label_is_named_by_its_placeholder() {
        let node = input_node(text_input::<TestMsg>("query").placeholder("Search"));
        assert_eq!(node.label(), Some("Search"));
    }

    #[test]
    fn text_input_with_neither_label_nor_placeholder_has_no_name() {
        let node = input_node(text_input::<TestMsg>("query"));
        assert_eq!(node.label(), None);
    }

    #[test]
    fn text_input_exposes_its_value_when_not_a_password() {
        let node = input_node(text_input::<TestMsg>("name").label("Name").value("Ada"));
        assert_eq!(node.role(), Role::TextInput);
        assert_eq!(node.value(), Some("Ada"));
    }

    #[test]
    fn password_input_uses_password_role() {
        let node = input_node(text_input::<TestMsg>("pw").label("Password").password(true));
        assert_eq!(node.role(), Role::PasswordInput);
    }

    #[test]
    fn disabled_text_input_is_disabled_focusable_and_not_settable() {
        let node = input_node(
            text_input::<TestMsg>("id")
                .label("Id")
                .disabled(true)
                .value("42"),
        );
        assert!(node.is_disabled());
        assert!(node.supports_action(Action::Focus));
        assert!(!node.supports_action(Action::SetValue));
    }

    #[test]
    fn read_only_text_input_is_read_only_and_not_settable() {
        let node = input_node(
            text_input::<TestMsg>("code")
                .label("Code")
                .readonly(true)
                .value("X1"),
        );
        assert!(node.is_read_only());
        assert!(!node.is_disabled());
        assert!(node.supports_action(Action::Focus));
        assert!(!node.supports_action(Action::SetValue));
    }

    #[test]
    fn editable_text_input_supports_focus_and_set_value() {
        let node = input_node(text_input::<TestMsg>("note").label("Note"));
        assert!(!node.is_read_only());
        assert!(!node.is_disabled());
        assert!(node.supports_action(Action::Focus));
        assert!(node.supports_action(Action::SetValue));
    }

    #[test]
    fn invalid_text_input_is_marked_invalid_and_valid_one_is_not() {
        let invalid = input_node(text_input::<TestMsg>("email").label("Email").invalid(true));
        assert_eq!(invalid.invalid(), Some(Invalid::True));

        let valid = input_node(text_input::<TestMsg>("email").label("Email"));
        assert_eq!(valid.invalid(), None);
    }

    #[test]
    fn password_value_never_appears_in_any_node_property() {
        const SECRET: &str = "hunter2-s3cret";
        let tree = column::<TestMsg>()
            .child(
                text_input::<TestMsg>("pw")
                    .label("Password")
                    .password(true)
                    .value(SECRET),
            )
            .child(
                text_input::<TestMsg>("pin")
                    .placeholder("PIN")
                    .password(true)
                    .value(SECRET)
                    .disabled(true)
                    .invalid(true),
            )
            .child(
                text_input::<TestMsg>("ro")
                    .label("Read only secret")
                    .password(true)
                    .value(SECRET)
                    .readonly(true),
            )
            .build();
        let nodes = nodes_of(&tree);

        let passwords = nodes_with_role(&nodes, Role::PasswordInput);
        assert_eq!(passwords.len(), 3, "each password field is a PasswordInput");
        assert!(passwords.iter().all(|node| node.value().is_none()));

        // The Debug dump lists every property that is set, so a secret in any
        // property shows up in it. The sentinel label proves the dump is not empty.
        let label_dump = format!("{:?}", passwords[0]);
        assert!(
            label_dump.contains("Password"),
            "dump should list the label"
        );

        for (id, node) in &nodes {
            let dump = format!("{node:?}");
            assert!(
                !dump.contains(SECRET),
                "node {id:?} exposes the password: {dump}"
            );
            let texts = [
                node.label(),
                node.value(),
                node.placeholder(),
                node.description(),
                node.tooltip(),
                node.role_description(),
                node.state_description(),
            ];
            for text in texts.into_iter().flatten() {
                assert!(
                    !text.contains(SECRET),
                    "node {id:?} has the password as text"
                );
            }
        }
    }

    // --- Button --------------------------------------------------------

    #[test]
    fn disabled_button_keeps_focus_but_has_no_click() {
        let tree = column::<TestMsg>()
            .child(button("save", "Save").disabled(true).on_click(TestMsg))
            .build();
        let nodes = nodes_of(&tree);
        let save = only_node_with_role(&nodes, Role::Button);
        assert!(save.supports_action(Action::Focus));
        assert!(!save.supports_action(Action::Click));
    }

    // --- TabBar --------------------------------------------------------

    #[test]
    fn tab_children_have_tab_role_inside_the_tab_list() {
        let nodes = nodes_of(&tab_strip(0));
        let tab_list = only_node_with_role(&nodes, Role::TabList);
        let tabs: Vec<&AccessNode> = tab_list
            .children()
            .iter()
            .map(|id| node_with_id(&nodes, *id))
            .collect();
        assert_eq!(tabs.len(), 3);
        assert!(tabs.iter().all(|tab| tab.role() == Role::Tab));
    }

    #[test]
    fn only_the_selected_tab_is_marked_selected() {
        for selected in 0..3 {
            let nodes = nodes_of(&tab_strip(selected));
            let states: Vec<Option<bool>> = nodes_with_role(&nodes, Role::Tab)
                .iter()
                .map(|tab| tab.is_selected())
                .collect();
            let expected: Vec<Option<bool>> = (0..3).map(|i| Some(i == selected)).collect();
            assert_eq!(states, expected, "selected index {selected}");
        }
    }

    #[test]
    fn tab_is_named_by_its_label_without_visual_decoration() {
        // The selected tab's widget text is "[Files]"; its name must be "Files".
        let nodes = nodes_of(&tab_strip(1));
        let names: Vec<&str> = nodes_with_role(&nodes, Role::Tab)
            .iter()
            .map(|tab| tab.label().unwrap_or(""))
            .collect();
        assert_eq!(names, ["Home", "Files", "Logs"]);
    }

    #[test]
    fn disabled_tab_is_disabled_and_not_clickable() {
        let tree = tab_bar::<TestMsg>("locked")
            .item(TabItem::new("Locked").disabled(true).on_click(TestMsg))
            .build();
        let nodes = nodes_of(&tree);
        let tab = only_node_with_role(&nodes, Role::Tab);
        assert!(tab.is_disabled());
        assert!(!tab.supports_action(Action::Click));
    }

    // --- NavRail -------------------------------------------------------

    #[test]
    fn selected_nav_item_is_selected_and_named_without_decoration() {
        let nodes = nodes_of(&nav(1));
        let rail = only_node_with_role(&nodes, Role::Navigation);
        let items: Vec<&AccessNode> = rail
            .children()
            .iter()
            .map(|id| node_with_id(&nodes, *id))
            .collect();

        let names: Vec<&str> = items
            .iter()
            .map(|item| item.label().unwrap_or(""))
            .collect();
        assert_eq!(names, ["Home", "Settings"]);
        // Absent means "not selected" for a button-like item; only the selected one is flagged.
        let states: Vec<Option<bool>> = items.iter().map(|item| item.is_selected()).collect();
        assert_eq!(states, [None, Some(true)]);
    }

    // --- Dialog --------------------------------------------------------

    #[test]
    fn modal_dialog_is_marked_modal_and_modeless_is_not() {
        let modal = column::<TestMsg>()
            .child(dialog::<TestMsg>("confirm", label("Sure?")).modal(true))
            .build();
        let modeless = column::<TestMsg>()
            .child(dialog::<TestMsg>("hint", label("Tip")).modal(false))
            .build();

        assert!(only_node_with_role(&nodes_of(&modal), Role::Dialog).is_modal());
        assert!(!only_node_with_role(&nodes_of(&modeless), Role::Dialog).is_modal());
    }

    // --- Retained binding path -----------------------------------------

    #[test]
    fn retained_binding_path_applies_the_same_control_semantics() {
        const SECRET: &str = "hunter2-s3cret";
        let mut tree = RetainedTree::new();
        let roots: Vec<WidgetNode<TestMsg>> = vec![
            column::<TestMsg>()
                .child(
                    text_input::<TestMsg>("pw")
                        .label("Password")
                        .password(true)
                        .value(SECRET),
                )
                .child(tab_strip(1))
                .child(nav(0))
                .child(
                    button::<TestMsg>("off", "Off")
                        .disabled(true)
                        .on_click(TestMsg),
                )
                .build(),
        ];
        reconcile_frame(&mut tree, &roots);
        let binding = bind_retained(&roots, &tree);

        let mut adapter = AccessibilityAdapter::new(1);
        adapter.update_with_binding(&roots[0], &LayoutSnapshot::default(), &binding);
        let nodes = &adapter.tree_ref().expect("tree should exist").nodes;

        let password = only_node_with_role(nodes, Role::PasswordInput);
        assert_eq!(password.label(), Some("Password"));
        assert_eq!(password.value(), None);
        for (_, node) in nodes {
            assert!(!format!("{node:?}").contains(SECRET), "password leaked");
        }

        let tab_names: Vec<&str> = nodes_with_role(nodes, Role::Tab)
            .iter()
            .map(|tab| tab.label().unwrap_or(""))
            .collect();
        assert_eq!(tab_names, ["Home", "Files", "Logs"]);
        let tab_states: Vec<Option<bool>> = nodes_with_role(nodes, Role::Tab)
            .iter()
            .map(|tab| tab.is_selected())
            .collect();
        assert_eq!(tab_states, [Some(false), Some(true), Some(false)]);

        let selected_buttons: Vec<&str> = nodes_with_role(nodes, Role::Button)
            .iter()
            .filter(|button| button.is_selected() == Some(true))
            .map(|button| button.label().unwrap_or(""))
            .collect();
        assert_eq!(selected_buttons, ["Home"]);

        let off = nodes_with_role(nodes, Role::Button)
            .into_iter()
            .find(|button| button.label() == Some("Off"))
            .expect("expected the disabled button");
        assert!(off.is_disabled());
        assert!(!off.supports_action(Action::Click));
    }

    // --- TextInput validation message ----------------------------------

    #[test]
    fn validation_message_marks_input_invalid_and_is_its_description() {
        let node = input_node(
            text_input::<TestMsg>("email")
                .label("Email")
                .validation("Enter a valid email address"),
        );
        assert_eq!(node.invalid(), Some(Invalid::True));
        assert_eq!(node.description(), Some("Enter a valid email address"));
    }

    #[test]
    fn input_without_validation_message_has_no_description_or_invalid_state() {
        let node = input_node(text_input::<TestMsg>("email").label("Email"));
        assert_eq!(node.invalid(), None);
        assert_eq!(node.description(), None);
    }

    #[test]
    fn invalid_flag_alone_marks_input_invalid_without_a_description() {
        let node = input_node(text_input::<TestMsg>("email").label("Email").invalid(true));
        assert_eq!(node.invalid(), Some(Invalid::True));
        assert_eq!(node.description(), None);
    }

    #[test]
    fn validation_message_reaches_the_retained_binding_path() {
        let mut tree = RetainedTree::new();
        let roots = vec![
            column::<TestMsg>()
                .child(
                    text_input::<TestMsg>("email")
                        .label("Email")
                        .validation("Required"),
                )
                .build(),
        ];
        reconcile_frame(&mut tree, &roots);
        let binding = bind_retained(&roots, &tree);

        let mut adapter = AccessibilityAdapter::new(1);
        adapter.update_with_binding(&roots[0], &LayoutSnapshot::default(), &binding);
        let nodes = &adapter.tree_ref().expect("tree should exist").nodes;

        let input = only_node_with_role(nodes, Role::TextInput);
        assert_eq!(input.invalid(), Some(Invalid::True));
        assert_eq!(input.description(), Some("Required"));
    }

    // --- Table and DataGrid cells --------------------------------------

    fn two_by_one_table() -> WidgetNode<TestMsg> {
        table::<TestMsg>("people")
            .column(TableColumn::new("name", label::<TestMsg>("Name"), 120.0))
            .column(TableColumn::new("age", label::<TestMsg>("Age"), 80.0))
            .add_row(TableRow::new(vec![
                label::<TestMsg>("Ada"),
                label::<TestMsg>("36"),
            ]))
            .build()
    }

    fn one_by_one_grid() -> WidgetNode<TestMsg> {
        datagrid::<TestMsg>("grid")
            .column(DataGridColumn::new("name", label::<TestMsg>("Name"), 120.0))
            .add_row(DataGridRow::new(vec![label::<TestMsg>("Ada")]))
            .build()
    }

    #[test]
    fn table_cells_have_cell_role_and_are_the_row_children() {
        let nodes = nodes_of(&two_by_one_table());
        let table_node = only_node_with_role(&nodes, Role::Table);
        // The table's children are its header row and its one data row; the
        // cells are the data row's children.
        assert_eq!(table_node.children().len(), 2);
        let data_row = node_with_id(&nodes, table_node.children()[1]);
        assert_eq!(data_row.role(), Role::Row);
        assert_eq!(data_row.children().len(), 2);
        for child in data_row.children() {
            assert_eq!(node_with_id(&nodes, *child).role(), Role::Cell);
        }
        let names: Vec<&str> = nodes_with_role(&nodes, Role::Cell)
            .iter()
            .map(|cell| cell.label().unwrap_or(""))
            .collect();
        assert_eq!(names, ["Ada", "36"]);
    }

    #[test]
    fn data_grid_cells_have_cell_role() {
        let nodes = nodes_of(&one_by_one_grid());
        let cell = only_node_with_role(&nodes, Role::Cell);
        assert_eq!(cell.label(), Some("Ada"));
        only_node_with_role(&nodes, Role::Grid);
    }

    #[test]
    fn table_cells_have_cell_role_on_the_retained_path() {
        let mut tree = RetainedTree::new();
        let roots = vec![two_by_one_table()];
        reconcile_frame(&mut tree, &roots);
        let binding = bind_retained(&roots, &tree);

        let mut adapter = AccessibilityAdapter::new(1);
        adapter.update_with_binding(&roots[0], &LayoutSnapshot::default(), &binding);
        let nodes = &adapter.tree_ref().expect("tree should exist").nodes;

        assert_eq!(nodes_with_role(nodes, Role::Cell).len(), 2);
    }

    #[test]
    fn button_in_a_table_cell_keeps_its_button_role_and_click() {
        let tree = table::<TestMsg>("actions")
            .column(TableColumn::new("act", label::<TestMsg>("Act"), 80.0))
            .add_row(TableRow::new(vec![
                button::<TestMsg>("delete", "Delete").on_click(TestMsg),
            ]))
            .build();
        let nodes = nodes_of(&tree);

        assert!(nodes_with_role(&nodes, Role::Cell).is_empty());
        let delete = only_node_with_role(&nodes, Role::Button);
        assert_eq!(delete.label(), Some("Delete"));
        assert!(delete.supports_action(Action::Click));
    }
}
