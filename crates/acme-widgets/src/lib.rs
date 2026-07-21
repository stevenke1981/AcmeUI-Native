//! Declarative MVP widget descriptions.
#![forbid(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

// ============================================================================
// Module declarations
// ============================================================================

pub mod data;
pub mod foundations;
pub mod inputs;
pub mod navigation;
pub mod overlay;
pub mod prelude;

mod overlay_manager;
mod visual_state;

// ============================================================================
// Re-exports — all public types are available at crate root
// ============================================================================

pub use data::*;
pub use foundations::*;
pub use inputs::*;
pub use navigation::*;
pub use overlay::*;
pub use overlay_manager::*;
#[allow(unused_imports)]
pub use prelude::*;
pub use visual_state::*;

/// Re-export core identifiers for convenience.
pub use acme_core::{NodeId, WidgetKey};

/// Re-export the acme-style styling system.
pub use acme_style as style;
pub use acme_style::Style;

// ============================================================================
// Imports used by WidgetNode and RuntimeNode
// ============================================================================

use acme_layout::{
    Edges, LayoutKind, LayoutNode, LayoutStyle, Length, Overflow, TextMeasureSpec,
    WidgetLayoutContext,
};

// ============================================================================
// RuntimeNode — compiled node with stable identity
// ============================================================================

/// A compiled node that ties together a widget, its stable NodeId, and its children.
#[derive(Clone, Debug)]
pub struct RuntimeNode<M> {
    pub id: NodeId,
    pub widget: WidgetNode<M>,
    pub children: Vec<NodeId>,
}

// ============================================================================
// WidgetNode — the core widget enum
// ============================================================================

/// The primary widget enum. Each variant holds a widget-specific data struct.
/// To add a new widget, add a variant here and a corresponding struct in the
/// appropriate submodule.
#[derive(Clone, Debug, PartialEq)]
pub enum WidgetNode<M> {
    Row(Container<M>),
    Column(Container<M>),
    Stack(Container<M>),
    Label(Label),
    Button(Button<M>),
    Card(Card<M>),
    Separator(Separator),
    ScrollView(ScrollView<M>),
    Tooltip(Tooltip<M>),
    VirtualList(VirtualList<M>),
    Popover(Popover<M>),
    Menu(Menu<M>),
    Dialog(Dialog<M>),
    Tree(Tree<M>),
    Table(Table<M>),
    DataGrid(DataGrid<M>),
    TextInput(TextInput<M>),
    NavRail(NavRail<M>),
    Sidebar(Sidebar<M>),
    TabBar(TabBar<M>),
    Breadcrumb(Breadcrumb<M>),
}

// ============================================================================
// WidgetNode methods
// ============================================================================

impl<M> WidgetNode<M> {
    /// Return the optional stable key.
    pub fn key(&self) -> Option<&WidgetKey> {
        match self {
            Self::Button(v) => Some(&v.key),
            Self::ScrollView(v) => Some(&v.key),
            Self::Tooltip(v) => Some(&v.key),
            Self::VirtualList(v) => Some(&v.key),
            Self::Row(v) | Self::Column(v) | Self::Stack(v) => v.key.as_ref(),
            Self::Card(v) => v.key.as_ref(),
            Self::Label(_) | Self::Separator(_) => None,
            Self::Popover(v) => Some(&v.key),
            Self::Menu(v) => Some(&v.key),
            Self::Dialog(v) => Some(&v.key),
            Self::Tree(v) => Some(&v.key),
            Self::Table(v) => Some(&v.key),
            Self::DataGrid(v) => Some(&v.key),
            Self::TextInput(v) => Some(&v.key),
            Self::NavRail(v) => Some(&v.key),
            Self::Sidebar(v) => Some(&v.key),
            Self::TabBar(v) => Some(&v.key),
            Self::Breadcrumb(v) => Some(&v.key),
        }
    }

    /// Return the immediate child widgets.
    pub fn children(&self) -> &[WidgetNode<M>] {
        match self {
            Self::Row(v) | Self::Column(v) | Self::Stack(v) => &v.children,
            Self::Card(v) => &v.children,
            Self::ScrollView(v) => &v.children,
            Self::Tooltip(v) => std::slice::from_ref(&v.child),
            Self::VirtualList(v) => &v.children,
            Self::Popover(v) => &v.children,
            Self::Menu(_) => &[],
            Self::Dialog(v) => std::slice::from_ref(&v.content),
            Self::Tree(_) => &[],
            Self::Table(v) => &v.all_cells,
            Self::DataGrid(v) => &v.all_cells,
            Self::NavRail(v) => &v.children,
            Self::Sidebar(v) => &v.children,
            Self::TabBar(v) => &v.children,
            Self::Breadcrumb(v) => &v.children,
            Self::Label(_) | Self::Button(_) | Self::Separator(_) | Self::TextInput(_) => &[],
        }
    }

    /// Convert to a layout tree using the given NodeId.
    /// The caller is responsible for providing the correct NodeId.
    /// Convert to a layout tree with unique [`NodeId`]s.
    ///
    /// `id` is the root identity; descendants receive monotonically increasing
    /// IDs via a shared counter (DFS). Sibling containers never reuse each
    /// other's child ranges — the previous `parent+1` offset scheme collided.
    pub fn to_layout(&self, id: NodeId) -> LayoutNode
    where
        M: Clone,
    {
        let mut next = id.get();
        self.to_layout_alloc(&mut next)
    }

    fn to_layout_alloc(&self, next: &mut u64) -> LayoutNode
    where
        M: Clone,
    {
        let id = NodeId::new(*next);
        *next += 1;
        match self {
            Self::Row(v) => LayoutNode::container(
                id,
                v.layout(LayoutKind::Row),
                v.children.iter().map(|c| c.to_layout_alloc(next)).collect(),
            ),
            Self::Column(v) => LayoutNode::container(
                id,
                v.layout(LayoutKind::Column),
                v.children.iter().map(|c| c.to_layout_alloc(next)).collect(),
            ),
            Self::Stack(v) => LayoutNode::container(
                id,
                v.layout(LayoutKind::Stack),
                v.children.iter().map(|c| c.to_layout_alloc(next)).collect(),
            ),
            Self::Card(v) => LayoutNode::container(
                id,
                LayoutStyle {
                    kind: LayoutKind::Column,
                    gap: v.gap,
                    padding: v.padding,
                    ..Default::default()
                },
                v.children.iter().map(|c| c.to_layout_alloc(next)).collect(),
            ),
            Self::ScrollView(v) => LayoutNode::container(
                id,
                v.layout(),
                v.children.iter().map(|c| c.to_layout_alloc(next)).collect(),
            ),
            Self::Label(l) => {
                let font_size = l.font_size.unwrap_or(16.0);
                let line_height = (font_size * 1.5).ceil();
                LayoutNode::leaf(
                    id,
                    LayoutStyle {
                        min_height: Length::px(line_height),
                        flex_shrink: 0.0,
                        ..Default::default()
                    },
                )
            }
            Self::TextInput(_) => LayoutNode::leaf(
                id,
                LayoutStyle {
                    min_height: Length::px(40.0),
                    flex_shrink: 0.0,
                    ..Default::default()
                },
            ),
            Self::Button(_) => LayoutNode::leaf(
                id,
                LayoutStyle {
                    height: Length::px(36.0),
                    ..Default::default()
                },
            ),
            Self::VirtualList(v) => LayoutNode::container(
                id,
                LayoutStyle {
                    kind: LayoutKind::Column,
                    height: Length::px(v.viewport_height),
                    overflow: Overflow::Scroll,
                    ..Default::default()
                },
                vec![],
            ),
            // Tooltip/Popover: keep the wrapper's id for the outer node by
            // allocating a fresh id for the inner content so both are unique.
            Self::Tooltip(v) => {
                let _ = id;
                // Re-use the id we already took for the tooltip as the child's
                // identity (tooltip is layout-transparent).
                *next -= 1;
                v.child.to_layout_alloc(next)
            }
            Self::Popover(v) => {
                let _ = id;
                *next -= 1;
                v.children[0].to_layout_alloc(next)
            }
            Self::Menu(_) => LayoutNode::leaf(
                id,
                LayoutStyle {
                    width: Length::px(200.0),
                    height: Length::Auto,
                    ..Default::default()
                },
            ),
            Self::Dialog(v) => LayoutNode::leaf(
                id,
                LayoutStyle {
                    width: v.width,
                    height: v.height,
                    ..Default::default()
                },
            ),
            Self::Separator(v) => LayoutNode::leaf(
                id,
                LayoutStyle {
                    height: Length::px(v.thickness),
                    ..Default::default()
                },
            ),
            Self::Tree(v) => {
                let visible = v.visible_nodes();
                let mut child_nodes = Vec::with_capacity(visible.len());
                for node in &visible {
                    let leaf_id = NodeId::new(*next);
                    *next += 1;
                    child_nodes.push(LayoutNode::leaf(
                        leaf_id,
                        LayoutStyle {
                            width: Length::Auto,
                            height: Length::px(24.0),
                            padding: Edges {
                                left: node.depth as f32 * v.indent,
                                ..Edges::default()
                            },
                            ..Default::default()
                        },
                    ));
                }
                LayoutNode::container(
                    id,
                    LayoutStyle {
                        kind: LayoutKind::Column,
                        ..Default::default()
                    },
                    child_nodes,
                )
            }
            Self::Table(v) => {
                let mut child_nodes: Vec<LayoutNode> = Vec::new();
                if !v.columns.is_empty() {
                    let header_children: Vec<LayoutNode> = v
                        .columns
                        .iter()
                        .map(|col| {
                            let nid = NodeId::new(*next);
                            *next += 1;
                            LayoutNode::leaf(
                                nid,
                                LayoutStyle {
                                    width: Length::px(col.width),
                                    min_height: Length::px(24.0),
                                    ..Default::default()
                                },
                            )
                        })
                        .collect();
                    let header_id = NodeId::new(*next);
                    *next += 1;
                    child_nodes.push(LayoutNode::container(
                        header_id,
                        LayoutStyle::row(),
                        header_children,
                    ));
                }
                for row in &v.rows {
                    let row_children: Vec<LayoutNode> = row
                        .cells
                        .iter()
                        .map(|_| {
                            let nid = NodeId::new(*next);
                            *next += 1;
                            LayoutNode::leaf(
                                nid,
                                LayoutStyle {
                                    min_height: Length::px(24.0),
                                    ..Default::default()
                                },
                            )
                        })
                        .collect();
                    let row_id = NodeId::new(*next);
                    *next += 1;
                    child_nodes.push(LayoutNode::container(
                        row_id,
                        LayoutStyle::row(),
                        row_children,
                    ));
                }
                LayoutNode::container(
                    id,
                    LayoutStyle {
                        kind: LayoutKind::Column,
                        ..Default::default()
                    },
                    child_nodes,
                )
            }
            Self::DataGrid(v) => {
                let mut child_nodes: Vec<LayoutNode> = Vec::new();
                let header_children: Vec<LayoutNode> = v
                    .columns
                    .iter()
                    .map(|col| {
                        let nid = NodeId::new(*next);
                        *next += 1;
                        LayoutNode::leaf(
                            nid,
                            LayoutStyle {
                                width: Length::px(col.width),
                                min_height: Length::px(24.0),
                                ..Default::default()
                            },
                        )
                    })
                    .collect();
                let header_id = NodeId::new(*next);
                *next += 1;
                child_nodes.push(LayoutNode::container(
                    header_id,
                    LayoutStyle::row(),
                    header_children,
                ));
                for row in &v.rows {
                    let row_children: Vec<LayoutNode> = row
                        .cells
                        .iter()
                        .map(|_| {
                            let nid = NodeId::new(*next);
                            *next += 1;
                            LayoutNode::leaf(
                                nid,
                                LayoutStyle {
                                    min_height: Length::px(24.0),
                                    ..Default::default()
                                },
                            )
                        })
                        .collect();
                    let row_id = NodeId::new(*next);
                    *next += 1;
                    child_nodes.push(LayoutNode::container(
                        row_id,
                        LayoutStyle::row(),
                        row_children,
                    ));
                }
                LayoutNode::container(
                    id,
                    LayoutStyle {
                        kind: LayoutKind::Column,
                        ..Default::default()
                    },
                    child_nodes,
                )
            }
            Self::NavRail(v) => LayoutNode::container(
                id,
                v.layout_style(),
                v.children.iter().map(|c| c.to_layout_alloc(next)).collect(),
            ),
            Self::Sidebar(v) => LayoutNode::container(
                id,
                v.layout_style(),
                v.children.iter().map(|c| c.to_layout_alloc(next)).collect(),
            ),
            Self::TabBar(v) => LayoutNode::container(
                id,
                v.layout_style(),
                v.children.iter().map(|c| c.to_layout_alloc(next)).collect(),
            ),
            Self::Breadcrumb(v) => LayoutNode::container(
                id,
                v.layout_style(),
                v.children.iter().map(|c| c.to_layout_alloc(next)).collect(),
            ),
        }
    }

    /// Convert to a layout tree using the given [`NodeId`] and typography
    /// context from the theme / application.
    ///
    /// `id` is the root identity; descendants receive monotonically increasing
    /// IDs via a shared counter (DFS).
    ///
    /// For [`Label`](crate::Label) nodes this produces a [`LayoutNode::text_leaf`]
    /// with a [`TextMeasureSpec`] that consults the label's own fields and falls
    /// back to `context.body_font_size` / `context.body_line_height`.
    ///
    /// For [`TextInput`](crate::TextInput) nodes the minimum height is derived
    /// from `context.control_height`.
    pub fn to_layout_with_context(&self, id: NodeId, context: &WidgetLayoutContext) -> LayoutNode
    where
        M: Clone,
    {
        let mut next = id.get();
        self.to_layout_alloc_with_context(&mut next, context)
    }

    fn to_layout_alloc_with_context(
        &self,
        next: &mut u64,
        context: &WidgetLayoutContext,
    ) -> LayoutNode
    where
        M: Clone,
    {
        let id = NodeId::new(*next);
        *next += 1;
        match self {
            Self::Row(v) => LayoutNode::container(
                id,
                v.layout(LayoutKind::Row),
                v.children
                    .iter()
                    .map(|c| c.to_layout_alloc_with_context(next, context))
                    .collect(),
            ),
            Self::Column(v) => LayoutNode::container(
                id,
                v.layout(LayoutKind::Column),
                v.children
                    .iter()
                    .map(|c| c.to_layout_alloc_with_context(next, context))
                    .collect(),
            ),
            Self::Stack(v) => LayoutNode::container(
                id,
                v.layout(LayoutKind::Stack),
                v.children
                    .iter()
                    .map(|c| c.to_layout_alloc_with_context(next, context))
                    .collect(),
            ),
            Self::Card(v) => LayoutNode::container(
                id,
                LayoutStyle {
                    kind: LayoutKind::Column,
                    gap: v.gap,
                    padding: v.padding,
                    ..Default::default()
                },
                v.children
                    .iter()
                    .map(|c| c.to_layout_alloc_with_context(next, context))
                    .collect(),
            ),
            Self::ScrollView(v) => LayoutNode::container(
                id,
                v.layout(),
                v.children
                    .iter()
                    .map(|c| c.to_layout_alloc_with_context(next, context))
                    .collect(),
            ),
            Self::Label(l) => {
                let font_size = l.font_size.unwrap_or(context.body_font_size);
                let line_height = l.line_height.unwrap_or(context.body_line_height);
                let min_h = (font_size * 1.5).ceil();
                let spec = TextMeasureSpec {
                    text: l.text.clone(),
                    font_size,
                    line_height,
                    wrap: crate::foundations::label::map_label_wrap(l.wrap),
                    max_lines: l.max_lines,
                };
                LayoutNode::text_leaf(
                    id,
                    LayoutStyle {
                        min_height: Length::px(min_h),
                        flex_shrink: 0.0,
                        ..Default::default()
                    },
                    spec,
                )
            }
            Self::TextInput(_) => LayoutNode::leaf(
                id,
                LayoutStyle {
                    min_height: Length::px(context.control_height),
                    flex_shrink: 0.0,
                    ..Default::default()
                },
            ),
            Self::Button(_) => LayoutNode::leaf(
                id,
                LayoutStyle {
                    height: Length::px(36.0),
                    ..Default::default()
                },
            ),
            Self::VirtualList(v) => LayoutNode::container(
                id,
                LayoutStyle {
                    kind: LayoutKind::Column,
                    height: Length::px(v.viewport_height),
                    overflow: Overflow::Scroll,
                    ..Default::default()
                },
                vec![],
            ),
            // Tooltip/Popover: keep the wrapper's id for the outer node by
            // allocating a fresh id for the inner content so both are unique.
            Self::Tooltip(v) => {
                let _ = id;
                *next -= 1;
                v.child.to_layout_alloc_with_context(next, context)
            }
            Self::Popover(v) => {
                let _ = id;
                *next -= 1;
                v.children[0].to_layout_alloc_with_context(next, context)
            }
            Self::Menu(_) => LayoutNode::leaf(
                id,
                LayoutStyle {
                    width: Length::px(200.0),
                    height: Length::Auto,
                    ..Default::default()
                },
            ),
            Self::Dialog(v) => LayoutNode::leaf(
                id,
                LayoutStyle {
                    width: v.width,
                    height: v.height,
                    ..Default::default()
                },
            ),
            Self::Separator(v) => LayoutNode::leaf(
                id,
                LayoutStyle {
                    height: Length::px(v.thickness),
                    ..Default::default()
                },
            ),
            Self::Tree(v) => {
                let visible = v.visible_nodes();
                let mut child_nodes = Vec::with_capacity(visible.len());
                for node in &visible {
                    let leaf_id = NodeId::new(*next);
                    *next += 1;
                    child_nodes.push(LayoutNode::leaf(
                        leaf_id,
                        LayoutStyle {
                            width: Length::Auto,
                            height: Length::px(24.0),
                            padding: Edges {
                                left: node.depth as f32 * v.indent,
                                ..Edges::default()
                            },
                            ..Default::default()
                        },
                    ));
                }
                LayoutNode::container(
                    id,
                    LayoutStyle {
                        kind: LayoutKind::Column,
                        ..Default::default()
                    },
                    child_nodes,
                )
            }
            Self::Table(v) => {
                let mut child_nodes: Vec<LayoutNode> = Vec::new();
                if !v.columns.is_empty() {
                    let header_children: Vec<LayoutNode> = v
                        .columns
                        .iter()
                        .map(|col| {
                            let nid = NodeId::new(*next);
                            *next += 1;
                            LayoutNode::leaf(
                                nid,
                                LayoutStyle {
                                    width: Length::px(col.width),
                                    min_height: Length::px(24.0),
                                    ..Default::default()
                                },
                            )
                        })
                        .collect();
                    let header_id = NodeId::new(*next);
                    *next += 1;
                    child_nodes.push(LayoutNode::container(
                        header_id,
                        LayoutStyle::row(),
                        header_children,
                    ));
                }
                for row in &v.rows {
                    let row_children: Vec<LayoutNode> = row
                        .cells
                        .iter()
                        .map(|_| {
                            let nid = NodeId::new(*next);
                            *next += 1;
                            LayoutNode::leaf(
                                nid,
                                LayoutStyle {
                                    min_height: Length::px(24.0),
                                    ..Default::default()
                                },
                            )
                        })
                        .collect();
                    let row_id = NodeId::new(*next);
                    *next += 1;
                    child_nodes.push(LayoutNode::container(
                        row_id,
                        LayoutStyle::row(),
                        row_children,
                    ));
                }
                LayoutNode::container(
                    id,
                    LayoutStyle {
                        kind: LayoutKind::Column,
                        ..Default::default()
                    },
                    child_nodes,
                )
            }
            Self::DataGrid(v) => {
                let mut child_nodes: Vec<LayoutNode> = Vec::new();
                let header_children: Vec<LayoutNode> = v
                    .columns
                    .iter()
                    .map(|col| {
                        let nid = NodeId::new(*next);
                        *next += 1;
                        LayoutNode::leaf(
                            nid,
                            LayoutStyle {
                                width: Length::px(col.width),
                                min_height: Length::px(24.0),
                                ..Default::default()
                            },
                        )
                    })
                    .collect();
                let header_id = NodeId::new(*next);
                *next += 1;
                child_nodes.push(LayoutNode::container(
                    header_id,
                    LayoutStyle::row(),
                    header_children,
                ));
                for row in &v.rows {
                    let row_children: Vec<LayoutNode> = row
                        .cells
                        .iter()
                        .map(|_| {
                            let nid = NodeId::new(*next);
                            *next += 1;
                            LayoutNode::leaf(
                                nid,
                                LayoutStyle {
                                    min_height: Length::px(24.0),
                                    ..Default::default()
                                },
                            )
                        })
                        .collect();
                    let row_id = NodeId::new(*next);
                    *next += 1;
                    child_nodes.push(LayoutNode::container(
                        row_id,
                        LayoutStyle::row(),
                        row_children,
                    ));
                }
                LayoutNode::container(
                    id,
                    LayoutStyle {
                        kind: LayoutKind::Column,
                        ..Default::default()
                    },
                    child_nodes,
                )
            }
            Self::NavRail(v) => LayoutNode::container(
                id,
                v.layout_style(),
                v.children
                    .iter()
                    .map(|c| c.to_layout_alloc_with_context(next, context))
                    .collect(),
            ),
            Self::Sidebar(v) => LayoutNode::container(
                id,
                v.layout_style(),
                v.children
                    .iter()
                    .map(|c| c.to_layout_alloc_with_context(next, context))
                    .collect(),
            ),
            Self::TabBar(v) => LayoutNode::container(
                id,
                v.layout_style(),
                v.children
                    .iter()
                    .map(|c| c.to_layout_alloc_with_context(next, context))
                    .collect(),
            ),
            Self::Breadcrumb(v) => LayoutNode::container(
                id,
                v.layout_style(),
                v.children
                    .iter()
                    .map(|c| c.to_layout_alloc_with_context(next, context))
                    .collect(),
            ),
        }
    }

    /// Compile the widget tree into a `RuntimeNode` with stable identities.
    pub fn compile(&self, next: &mut u64) -> RuntimeNode<M>
    where
        M: Clone,
    {
        let id = NodeId::new(*next);
        *next += 1;
        let widget = self.clone();
        let children: Vec<NodeId> = self
            .children()
            .iter()
            .map(|c| {
                let compiled = c.compile(next);
                compiled.id
            })
            .collect();
        RuntimeNode {
            id,
            widget,
            children,
        }
    }
}

// ============================================================================
// Helper
// ============================================================================

fn finite(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::Rect;
    use acme_theme::Theme;
    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Save,
    }
    #[test]
    fn builders_make_expected_tree() {
        let tree = column::<Msg>()
            .key("root")
            .gap(8.0)
            .child(label("標題"))
            .child(button("save", "儲存").primary().on_click(Msg::Save))
            .build();
        assert_eq!(tree.children().len(), 2);
        assert_eq!(tree.key().unwrap().as_str(), "root");
    }
    #[test]
    fn disabled_button_does_not_activate() {
        let node = button("save", "save").disabled(true).on_click(Msg::Save);
        let WidgetNode::Button(b) = node else {
            panic!()
        };
        assert_eq!(b.activate(), None);
    }
    #[test]
    fn button_uses_theme_tokens() {
        let b = button::<Msg>("save", "save").primary();
        let theme = Theme::dark();
        assert_eq!(
            b.resolve_style(&theme, ButtonState::default()).background,
            theme.colors.primary
        );
        assert_eq!(
            b.resolve_style(
                &theme,
                ButtonState {
                    hovered: true,
                    ..Default::default()
                }
            )
            .background,
            theme.colors.primary_hover
        );
        // Pressed uses a separate token from hover
        assert_eq!(
            b.resolve_style(
                &theme,
                ButtonState {
                    pressed: true,
                    ..Default::default()
                }
            )
            .background,
            theme.colors.primary_pressed
        );
    }
    #[test]
    fn scroll_layout_is_clipped() {
        let tree = scroll_view::<Msg>("scroll")
            .viewport_height(100.0)
            .child(label("內容"))
            .build();
        let layout = tree.to_layout(NodeId::new(1));
        assert_eq!(layout.style.overflow, Overflow::Scroll);
        assert_eq!(layout.style.height, Length::px(100.0));
    }
    #[test]
    fn tooltip_wraps_child_correctly() {
        let child = label("hover me");
        let tt = tooltip::<Msg>("tt", child, "helper text");
        let WidgetNode::Tooltip(t) = WidgetNode::from(tt) else {
            panic!("expected Tooltip variant");
        };
        assert_eq!(t.text, "helper text");
        assert_eq!(t.delay_ms, 500);
        let WidgetNode::Label(l) = &*t.child else {
            panic!("expected Label child");
        };
        assert_eq!(l.text, "hover me");
    }
    #[test]
    fn tooltip_key_is_accessible() {
        let node: WidgetNode<Msg> = tooltip::<Msg>("tt", label("hi"), "help").into();
        assert_eq!(node.key().unwrap().as_str(), "tt");
    }
    #[test]
    fn tooltip_to_layout_delegates_to_child() {
        let node: WidgetNode<Msg> = tooltip::<Msg>("tt", label("hi"), "help").into();
        let layout = node.to_layout(NodeId::new(1));
        // Tooltip delegates to child; LayoutNode id is auto-incremented.
        // Verify structural delegation: leaf with no children.
        assert_eq!(layout.children.len(), 0);
        assert_eq!(layout.style.kind, LayoutKind::Leaf);
    }

    // ------------------------------------------------------------------
    // VirtualList tests
    // ------------------------------------------------------------------

    fn filled_list(n: usize, item_h: f32) -> VirtualList<Msg> {
        let mut list = virtual_list::<Msg>("list")
            .item_height(Some(item_h))
            .overscan(1);
        for i in 0..n {
            list = list.child(label(format!("Item {i}")));
        }
        list
    }

    #[test]
    fn virtual_list_visible_range_at_start() {
        let list = filled_list(100, 40.0).viewport_height(200.0);
        let (first, last) = list.visible_range();
        assert_eq!(first, 0);
        assert!(last > 0);
    }

    #[test]
    fn virtual_list_visible_range_scrolled() {
        let list = filled_list(100, 40.0)
            .viewport_height(200.0)
            .scroll_offset(100.0);
        let (first, last) = list.visible_range();
        assert!(first <= 2);
        assert!(last > first);
    }

    #[test]
    fn virtual_list_visible_range_at_end() {
        let list = filled_list(10, 40.0)
            .viewport_height(200.0)
            .scroll_offset(320.0);
        let (first, last) = list.visible_range();
        assert!(first <= 8);
        assert_eq!(last, 10);
    }

    #[test]
    fn virtual_list_visible_range_empty_list() {
        let list = virtual_list::<Msg>("empty").viewport_height(200.0);
        let range = list.visible_range();
        assert_eq!(range, (0, 0));
    }

    #[test]
    fn virtual_list_content_height() {
        let list = filled_list(50, 30.0);
        assert!((list.content_height() - 1500.0).abs() < f32::EPSILON);
    }

    #[test]
    fn virtual_list_content_height_empty() {
        let list = virtual_list::<Msg>("empty");
        assert!((list.content_height() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn virtual_list_key_is_accessible() {
        let node: WidgetNode<Msg> = virtual_list::<Msg>("mylist")
            .item_height(Some(40.0))
            .viewport_height(200.0)
            .build();
        assert_eq!(node.key().unwrap().as_str(), "mylist");
    }

    #[test]
    fn virtual_list_to_layout_has_viewport_height() {
        let node: WidgetNode<Msg> = virtual_list::<Msg>("list")
            .item_height(Some(40.0))
            .viewport_height(300.0)
            .build();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.height, Length::px(300.0));
        assert_eq!(layout.style.overflow, Overflow::Scroll);
        assert!(layout.children.is_empty()); // virtual children
    }

    // ------------------------------------------------------------------
    // Popover tests
    // ------------------------------------------------------------------

    #[test]
    fn popover_wraps_anchor_and_content() {
        let anchor = label("click me");
        let content = label("popup content");
        let node: WidgetNode<Msg> = popover::<Msg>("p1", anchor).content(content).build();
        let WidgetNode::Popover(v) = &node else {
            panic!("expected Popover variant");
        };
        assert_eq!(v.children.len(), 2);
        // Anchor is children[0]
        let WidgetNode::Label(a) = &v.children[0] else {
            panic!("expected Label as anchor");
        };
        assert_eq!(a.text, "click me");
        // Content is children[1]
        let WidgetNode::Label(c) = &v.children[1] else {
            panic!("expected Label as content");
        };
        assert_eq!(c.text, "popup content");
    }

    #[test]
    fn popover_key_and_placement_accessible() {
        let node: WidgetNode<Msg> = popover::<Msg>("pop", label("a"))
            .content(label("b"))
            .placement(PopoverPlacement::Top)
            .build();
        assert_eq!(node.key().unwrap().as_str(), "pop");
        let WidgetNode::Popover(v) = &node else {
            panic!("expected Popover");
        };
        assert_eq!(v.placement, PopoverPlacement::Top);
        assert!(!v.open);
    }

    #[test]
    fn popover_to_layout_delegates_to_anchor() {
        let node: WidgetNode<Msg> = popover::<Msg>("p", label("anchor"))
            .content(label("content"))
            .build();
        let layout = node.to_layout(NodeId::new(1));
        // Popover delegates to anchor (label -> leaf)
        assert_eq!(layout.children.len(), 0);
        assert_eq!(layout.style.kind, LayoutKind::Leaf);
    }

    #[test]
    fn popover_children_returns_both() {
        let node: WidgetNode<Msg> = popover::<Msg>("p", label("a")).content(label("b")).build();
        assert_eq!(node.children().len(), 2);
    }

    // ------------------------------------------------------------------
    // Menu tests
    // ------------------------------------------------------------------

    #[test]
    fn menu_creates_items_with_labels_and_separators() {
        let node: WidgetNode<Msg> = menu::<Msg>("file")
            .item(menu_item("new", "New File"))
            .item(menu_item("open", "Open..."))
            .separator()
            .item(menu_item("exit", "Exit"))
            .build();
        let WidgetNode::Menu(m) = &node else {
            panic!("expected Menu variant");
        };
        assert_eq!(m.items.len(), 4);
        assert_eq!(m.items[0].label, "New File");
        assert_eq!(m.items[1].label, "Open...");
        assert!(m.items[2].separator);
        assert!(!m.items[3].separator);
        assert_eq!(m.items[3].label, "Exit");
    }

    #[test]
    fn menu_children_is_empty() {
        let node: WidgetNode<Msg> = menu::<Msg>("m").item(menu_item("a", "A")).build();
        assert!(node.children().is_empty());
    }

    #[test]
    fn menu_item_activation_returns_correct_message() {
        let item = menu_item::<Msg>("save", "Save").on_click(Msg::Save);
        assert_eq!(item.activate(), Some(&Msg::Save));
    }

    #[test]
    fn menu_item_disabled_does_not_activate() {
        let item = menu_item::<Msg>("save", "Save")
            .disabled(true)
            .on_click(Msg::Save);
        assert_eq!(item.activate(), None);
    }

    #[test]
    fn menu_separator_does_not_activate() {
        let item = MenuItem::<Msg> {
            key: WidgetKey::from("sep"),
            label: String::new(),
            disabled: true,
            message: None,
            separator: true,
            children: vec![],
        };
        assert_eq!(item.activate(), None);
    }

    #[test]
    fn menu_to_layout_produces_leaf() {
        let node: WidgetNode<Msg> = menu::<Msg>("m").item(menu_item("a", "A")).build();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Leaf);
        assert_eq!(layout.style.width, Length::px(200.0));
    }

    // ------------------------------------------------------------------
    // Dialog tests
    // ------------------------------------------------------------------

    #[test]
    fn dialog_has_correct_title_and_size() {
        let content = label("dialog body");
        let d = dialog::<Msg>("dlg", content)
            .title("Confirm")
            .width(640.0)
            .height(480.0)
            .build();
        let WidgetNode::Dialog(v) = &d else {
            panic!("expected Dialog variant");
        };
        assert_eq!(v.key.as_str(), "dlg");
        assert_eq!(v.title, "Confirm");
        assert_eq!(v.width, Length::px(640.0));
        assert_eq!(v.height, Length::px(480.0));
        assert!(v.modal);
    }

    #[test]
    fn dialog_wraps_content() {
        let content = label("hello from dialog");
        let d: WidgetNode<Msg> = dialog::<Msg>("d", content).build();
        let WidgetNode::Dialog(v) = &d else {
            panic!("expected Dialog variant");
        };
        let WidgetNode::Label(l) = &*v.content else {
            panic!("expected Label content");
        };
        assert_eq!(l.text, "hello from dialog");
    }

    #[test]
    fn dialog_default_is_modal() {
        let d: WidgetNode<Msg> = dialog::<Msg>("d", label("x")).build();
        let WidgetNode::Dialog(v) = &d else {
            panic!("expected Dialog");
        };
        assert!(v.modal);
        assert!(!v.open);
    }

    #[test]
    fn dialog_children_returns_content() {
        let d: WidgetNode<Msg> = dialog::<Msg>("d", label("x")).build();
        assert_eq!(d.children().len(), 1);
    }

    #[test]
    fn dialog_to_layout_uses_width_and_height() {
        let d: WidgetNode<Msg> = dialog::<Msg>("d", label("body"))
            .width(400.0)
            .height(300.0)
            .build();
        let layout = d.to_layout(NodeId::new(1));
        assert_eq!(layout.style.width, Length::px(400.0));
        assert_eq!(layout.style.height, Length::px(300.0));
        assert_eq!(layout.style.kind, LayoutKind::Leaf);
    }

    #[test]
    fn dialog_to_layout_defaults() {
        let d: WidgetNode<Msg> = dialog::<Msg>("d", label("x")).build();
        let layout = d.to_layout(NodeId::new(1));
        assert_eq!(layout.style.width, Length::px(480.0));
        assert_eq!(layout.style.height, Length::Auto);
    }

    #[test]
    fn popover_from_conversion() {
        let pop = popover::<Msg>("p", label("a")).content(label("b"));
        let node: WidgetNode<Msg> = pop.into();
        assert!(matches!(node, WidgetNode::Popover(_)));
    }

    #[test]
    fn menu_from_conversion() {
        let m = menu::<Msg>("m");
        let node: WidgetNode<Msg> = m.into();
        assert!(matches!(node, WidgetNode::Menu(_)));
    }

    #[test]
    fn dialog_from_conversion() {
        let d = dialog::<Msg>("d", label("x"));
        let node: WidgetNode<Msg> = d.into();
        assert!(matches!(node, WidgetNode::Dialog(_)));
    }

    #[test]
    fn menu_key_is_accessible() {
        let node: WidgetNode<Msg> = menu::<Msg>("editmenu").build();
        assert_eq!(node.key().unwrap().as_str(), "editmenu");
    }

    #[test]
    fn dialog_key_is_accessible() {
        let node: WidgetNode<Msg> = dialog::<Msg>("confirm", label("y")).build();
        assert_eq!(node.key().unwrap().as_str(), "confirm");
    }

    // ------------------------------------------------------------------
    // Tree tests
    // ------------------------------------------------------------------

    #[test]
    fn tree_creates_nodes() {
        let node: WidgetNode<Msg> = tree::<Msg>("tree")
            .child(TreeNode::new("n1", label("Root")))
            .child(TreeNode::new("n2", label("Child")).expanded(true))
            .build();
        let WidgetNode::Tree(t) = &node else {
            panic!("expected Tree variant");
        };
        assert_eq!(t.children.len(), 2);
        assert_eq!(t.key.as_str(), "tree");
    }

    #[test]
    fn tree_to_layout_creates_column_with_children() {
        let node: WidgetNode<Msg> = tree::<Msg>("t")
            .child(TreeNode::new("n1", label("Root")))
            .child(TreeNode::new("n2", label("Child")))
            .build();
        let layout = node.to_layout(NodeId::new(1));
        // visible_nodes flattens to 2 items since none are expanded
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert_eq!(layout.children.len(), 2);
        // All children at depth 0 -> left padding = 0
        assert_eq!(layout.children[0].style.padding.left, 0.0);
        assert_eq!(layout.children[1].style.padding.left, 0.0);
    }

    #[test]
    fn tree_key_is_accessible() {
        let node: WidgetNode<Msg> = tree::<Msg>("mytree").build();
        assert_eq!(node.key().unwrap().as_str(), "mytree");
    }

    #[test]
    fn tree_children_is_empty() {
        let node: WidgetNode<Msg> = tree::<Msg>("t")
            .child(TreeNode::new("n1", label("Item")))
            .build();
        assert!(node.children().is_empty());
    }

    #[test]
    fn tree_from_conversion() {
        let t = tree::<Msg>("t");
        let node: WidgetNode<Msg> = t.into();
        assert!(matches!(node, WidgetNode::Tree(_)));
    }

    // ------------------------------------------------------------------
    // Table tests
    // ------------------------------------------------------------------

    #[test]
    fn table_creates_columns_and_rows() {
        let node: WidgetNode<Msg> = table::<Msg>("t")
            .column(TableColumn::new("name", label("Name"), 100.0))
            .column(TableColumn::new("age", label("Age"), 80.0))
            .add_row(TableRow::new(vec![label("Alice"), label("30")]))
            .add_row(TableRow::new(vec![label("Bob"), label("25")]))
            .build();
        let WidgetNode::Table(t) = &node else {
            panic!("expected Table variant");
        };
        assert_eq!(t.columns.len(), 2);
        assert_eq!(t.columns[0].key.as_str(), "name");
        assert_eq!(t.columns[0].width, 100.0);
        assert_eq!(t.columns[1].key.as_str(), "age");
        assert_eq!(t.rows.len(), 2);
        assert_eq!(t.rows[0].cells.len(), 2);
    }

    #[test]
    fn table_to_layout_produces_correct_structure() {
        let node: WidgetNode<Msg> = table::<Msg>("t")
            .column(TableColumn::new("a", label("A"), 100.0))
            .column(TableColumn::new("b", label("B"), 100.0))
            .add_row(TableRow::new(vec![label("a1"), label("b1")]))
            .add_row(TableRow::new(vec![label("a2"), label("b2")]))
            .build();
        let layout = node.to_layout(NodeId::new(1));
        // Outer column
        assert_eq!(layout.style.kind, LayoutKind::Column);
        // Header row + 2 data rows = 3 children
        assert_eq!(layout.children.len(), 3);
        // First child is header row
        assert_eq!(layout.children[0].style.kind, LayoutKind::Row);
        assert_eq!(layout.children[0].children.len(), 2);
        // Second child is first data row
        assert_eq!(layout.children[1].style.kind, LayoutKind::Row);
        assert_eq!(layout.children[1].children.len(), 2);
    }

    #[test]
    fn table_children_returns_flat_cells() {
        let node: WidgetNode<Msg> = table::<Msg>("t")
            .column(TableColumn::new("a", label("A"), 100.0))
            .add_row(TableRow::new(vec![label("c1")]))
            .add_row(TableRow::new(vec![label("c2")]))
            .build();
        assert_eq!(node.children().len(), 2);
    }

    #[test]
    fn table_key_is_accessible() {
        let node: WidgetNode<Msg> = table::<Msg>("mytable")
            .column(TableColumn::new("a", label("A"), 100.0))
            .build();
        assert_eq!(node.key().unwrap().as_str(), "mytable");
    }

    #[test]
    fn table_from_conversion() {
        let t = table::<Msg>("t");
        let node: WidgetNode<Msg> = t.into();
        assert!(matches!(node, WidgetNode::Table(_)));
    }

    // ------------------------------------------------------------------
    // DataGrid tests
    // ------------------------------------------------------------------

    #[test]
    fn datagrid_creates_columns_and_rows() {
        let node: WidgetNode<Msg> = datagrid::<Msg>("grid")
            .column(DataGridColumn::new("name", label("Name"), 150.0))
            .column(DataGridColumn::new("score", label("Score"), 100.0))
            .add_row(DataGridRow::new(vec![label("Alice"), label("95")]))
            .add_row(DataGridRow::new(vec![label("Bob"), label("87")]))
            .build();
        let WidgetNode::DataGrid(d) = &node else {
            panic!("expected DataGrid variant");
        };
        assert_eq!(d.columns.len(), 2);
        assert_eq!(d.columns[0].key.as_str(), "name");
        assert_eq!(d.columns[1].key.as_str(), "score");
        assert_eq!(d.rows.len(), 2);
        assert_eq!(d.rows[0].cells.len(), 2);
        assert_eq!(d.state, TableState::Normal);
    }

    #[test]
    fn datagrid_frozen_rows_and_cols() {
        let node: WidgetNode<Msg> = datagrid::<Msg>("grid")
            .column(DataGridColumn::new("a", label("A"), 100.0))
            .add_row(DataGridRow::new(vec![label("x")]))
            .frozen_rows(1)
            .frozen_cols(0)
            .build();
        let WidgetNode::DataGrid(d) = &node else {
            panic!("expected DataGrid variant");
        };
        assert_eq!(d.frozen_rows, 1);
        assert_eq!(d.frozen_cols, 0);
    }

    #[test]
    fn datagrid_children_returns_cells() {
        let node: WidgetNode<Msg> = datagrid::<Msg>("grid")
            .column(DataGridColumn::new("a", label("A"), 100.0))
            .add_row(DataGridRow::new(vec![label("x")]))
            .add_row(DataGridRow::new(vec![label("y")]))
            .build();
        assert_eq!(node.children().len(), 2);
    }

    #[test]
    fn datagrid_to_layout_produces_correct_structure() {
        let node: WidgetNode<Msg> = datagrid::<Msg>("grid")
            .column(DataGridColumn::new("a", label("A"), 100.0))
            .column(DataGridColumn::new("b", label("B"), 100.0))
            .add_row(DataGridRow::new(vec![label("a1"), label("b1")]))
            .build();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        // Header row + 1 data row = 2 children
        assert_eq!(layout.children.len(), 2);
        assert_eq!(layout.children[0].style.kind, LayoutKind::Row);
        assert_eq!(layout.children[0].children.len(), 2);
        assert_eq!(layout.children[1].style.kind, LayoutKind::Row);
        assert_eq!(layout.children[1].children.len(), 2);
    }

    #[test]
    fn datagrid_key_is_accessible() {
        let node: WidgetNode<Msg> = datagrid::<Msg>("mygrid")
            .column(DataGridColumn::new("a", label("A"), 100.0))
            .build();
        assert_eq!(node.key().unwrap().as_str(), "mygrid");
    }

    #[test]
    fn datagrid_from_conversion() {
        let d = datagrid::<Msg>("d");
        let node: WidgetNode<Msg> = d.into();
        assert!(matches!(node, WidgetNode::DataGrid(_)));
    }

    #[test]
    fn card_builder_creates_card() {
        let node: WidgetNode<Msg> = card()
            .key("my-card")
            .child(label("Card Title"))
            .child(label("Body"))
            .gap(8.0)
            .padding(12.0)
            .variant(CardVariant::Elevated)
            .build();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        assert_eq!(c.key.as_ref().unwrap().as_str(), "my-card");
        assert_eq!(c.children.len(), 2);
        assert_eq!(c.variant, CardVariant::Elevated);
        assert_eq!(c.gap, 8.0);
        assert_eq!(c.padding, Edges::all(12.0));
    }

    #[test]
    fn visual_state_enum_has_expected_variants() {
        let states = [
            VisualState::Default,
            VisualState::Hover,
            VisualState::Pressed,
            VisualState::FocusVisible,
            VisualState::Selected,
            VisualState::Disabled,
            VisualState::Loading,
            VisualState::Invalid,
        ];
        assert_eq!(states.len(), 8);
        assert_eq!(states[0] as u8, 0);
    }

    #[test]
    fn text_input_builder_sets_all_fields() {
        let input = text_input::<Msg>("email")
            .label("Email")
            .description("Enter your email address")
            .placeholder("user@example.com")
            .value("test@test.com")
            .clearable(true)
            .readonly(false)
            .password(false)
            .invalid(true)
            .validation("Invalid email format")
            .disabled(false);
        assert_eq!(input.key.as_str(), "email");
        assert_eq!(input.label.as_deref(), Some("Email"));
        assert_eq!(
            input.description.as_deref(),
            Some("Enter your email address")
        );
        assert_eq!(input.placeholder.as_deref(), Some("user@example.com"));
        assert_eq!(input.value, "test@test.com");
        assert!(input.clearable);
        assert!(!input.readonly);
        assert!(!input.password);
        assert!(input.invalid);
        assert_eq!(
            input.validation_message.as_deref(),
            Some("Invalid email format")
        );
        assert!(!input.disabled);
    }

    #[test]
    fn text_input_build_creates_widget_node() {
        let node = text_input::<Msg>("name").label("Name").build();
        let WidgetNode::TextInput(t) = &node else {
            panic!("expected TextInput variant");
        };
        assert_eq!(t.label.as_deref(), Some("Name"));
    }

    #[test]
    fn overlay_manager_push_and_top_of() {
        let mut mgr = OverlayManager::new();
        mgr.push(
            OverlayLayer::Modal,
            NodeId::new(1),
            Rect::new(0.0, 0.0, 100.0, 100.0),
        );
        mgr.push(
            OverlayLayer::Modal,
            NodeId::new(2),
            Rect::new(0.0, 0.0, 200.0, 200.0),
        );
        mgr.push(
            OverlayLayer::Tooltip,
            NodeId::new(3),
            Rect::new(0.0, 0.0, 50.0, 50.0),
        );
        assert_eq!(mgr.top_of(OverlayLayer::Modal), Some(NodeId::new(2)));
        assert_eq!(mgr.top_of(OverlayLayer::Tooltip), Some(NodeId::new(3)));
        assert_eq!(mgr.top_of(OverlayLayer::Debug), None);
    }

    #[test]
    fn overlay_manager_dismiss_and_raise() {
        let mut mgr = OverlayManager::new();
        mgr.push(
            OverlayLayer::Modal,
            NodeId::new(1),
            Rect::new(0.0, 0.0, 100.0, 100.0),
        );
        mgr.push(
            OverlayLayer::Modal,
            NodeId::new(2),
            Rect::new(0.0, 0.0, 200.0, 200.0),
        );
        // Dismiss the top one
        mgr.dismiss(NodeId::new(2));
        assert_eq!(mgr.top_of(OverlayLayer::Modal), Some(NodeId::new(1)));
        // Raise the bottom one (push it to top)
        mgr.raise(NodeId::new(1));
        assert_eq!(mgr.top_of(OverlayLayer::Modal), Some(NodeId::new(1)));
    }

    #[test]
    fn button_new_fields_default() {
        let b = button::<Msg>("test", "Test");
        assert_eq!(b.size, ButtonSize::Medium);
        assert!(!b.loading);
        assert!(!b.full_width);
        assert!(b.leading_icon.is_none());
        assert!(b.trailing_icon.is_none());
    }

    #[test]
    fn button_size_fluent_methods() {
        let b = button::<Msg>("test", "Test")
            .size(ButtonSize::Small)
            .loading(true)
            .full_width(true)
            .leading_icon("search")
            .trailing_icon("arrow");
        assert_eq!(b.size, ButtonSize::Small);
        assert!(b.loading);
        assert!(b.full_width);
        assert_eq!(b.leading_icon.as_deref(), Some("search"));
        assert_eq!(b.trailing_icon.as_deref(), Some("arrow"));
    }

    #[test]
    fn button_disabled_uses_disabled_bg() {
        let b = button::<Msg>("test", "Test").disabled(true);
        let theme = Theme::dark();
        let resolved = b.resolve_style(&theme, ButtonState::default());
        assert_eq!(resolved.background, theme.colors.disabled_bg);
        assert_eq!(resolved.foreground, theme.colors.disabled_text);
    }

    // ------------------------------------------------------------------
    // Navigation widgets
    // ------------------------------------------------------------------

    #[test]
    fn nav_rail_builder_and_layout() {
        let node = nav_rail::<Msg>("rail")
            .item(nav_item("Home").icon("⌂"))
            .item(nav_item("Search").on_click(Msg::Save))
            .item(nav_item("Settings").disabled(true))
            .selected(1)
            .collapsed(false)
            .build();
        assert!(matches!(node, WidgetNode::NavRail(_)));
        assert_eq!(node.key().unwrap().as_str(), "rail");
        let WidgetNode::NavRail(r) = &node else {
            panic!("expected NavRail");
        };
        assert_eq!(r.items.len(), 3);
        assert_eq!(r.selected, Some(1));
        assert!(!r.collapsed);
        assert_eq!(r.children.len(), 3);
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert_eq!(layout.style.width, Length::px(200.0));
        assert_eq!(layout.children.len(), 3);
        assert!(!layout.children.is_empty());
    }

    #[test]
    fn nav_rail_collapsed_width_and_short_labels() {
        let node = nav_rail::<Msg>("rail")
            .item(nav_item("Home"))
            .item(nav_item("Docs").icon("D"))
            .selected(0)
            .collapsed(true)
            .build();
        let WidgetNode::NavRail(r) = &node else {
            panic!("expected NavRail");
        };
        assert!(r.collapsed);
        let layout = node.to_layout(NodeId::new(10));
        assert_eq!(layout.style.width, Length::px(56.0));
        let WidgetNode::Label(l) = &r.children[0] else {
            panic!("expected Label for item without message");
        };
        assert_eq!(l.text, "• H");
        let WidgetNode::Label(l2) = &r.children[1] else {
            panic!("expected Label");
        };
        assert_eq!(l2.text, "D");
    }

    #[test]
    fn sidebar_builder_width_header_children() {
        let node = sidebar::<Msg>("sb")
            .width(240.0)
            .header("Library")
            .child(label("Item A"))
            .child(label("Item B"))
            .build();
        assert!(matches!(node, WidgetNode::Sidebar(_)));
        let WidgetNode::Sidebar(s) = &node else {
            panic!("expected Sidebar");
        };
        assert_eq!(s.key.as_str(), "sb");
        assert_eq!(s.width, 240.0);
        assert_eq!(s.header.as_deref(), Some("Library"));
        assert_eq!(s.children.len(), 3);
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert_eq!(layout.style.width, Length::px(240.0));
        assert_eq!(layout.children.len(), 3);
    }

    #[test]
    fn sidebar_default_width() {
        let node = sidebar::<Msg>("sb").child(label("only")).build();
        let WidgetNode::Sidebar(s) = &node else {
            panic!("expected Sidebar");
        };
        assert_eq!(s.width, 224.0);
        assert_eq!(s.children.len(), 1);
    }

    #[test]
    fn tab_bar_builder_selected_and_layout() {
        let node = tab_bar::<Msg>("tabs")
            .tab("Overview")
            .tab("Details")
            .tab("History")
            .selected(1)
            .build();
        assert!(matches!(node, WidgetNode::TabBar(_)));
        let WidgetNode::TabBar(t) = &node else {
            panic!("expected TabBar");
        };
        assert_eq!(t.tabs.len(), 3);
        assert_eq!(t.selected, 1);
        assert_eq!(t.children.len(), 3);
        let WidgetNode::Label(sel) = &t.children[1] else {
            panic!("expected Label");
        };
        assert_eq!(sel.text, "[Details]");
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Row);
        assert_eq!(layout.children.len(), 3);
        assert!(!layout.children.is_empty());
    }

    #[test]
    fn tab_bar_item_with_message() {
        let node = tab_bar::<Msg>("tabs")
            .item(TabItem::new("A").on_click(Msg::Save))
            .tab("B")
            .selected(0)
            .build();
        let WidgetNode::TabBar(t) = &node else {
            panic!("expected TabBar");
        };
        assert!(matches!(t.children[0], WidgetNode::Button(_)));
        assert!(t.tabs[0].activate().is_some());
    }

    #[test]
    fn breadcrumb_builder_and_separators() {
        let node = breadcrumb::<Msg>("bc")
            .segment("Home")
            .segment("Library")
            .segment("Data")
            .build();
        assert!(matches!(node, WidgetNode::Breadcrumb(_)));
        let WidgetNode::Breadcrumb(b) = &node else {
            panic!("expected Breadcrumb");
        };
        assert_eq!(b.segments.len(), 3);
        assert_eq!(b.children.len(), 5);
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Row);
        assert_eq!(layout.children.len(), 5);
        assert!(!layout.children.is_empty());
    }

    #[test]
    fn breadcrumb_custom_separator_and_clickable_segment() {
        let node = breadcrumb::<Msg>("bc")
            .separator(">")
            .item(BreadcrumbSegment::new("Root").on_click(Msg::Save))
            .segment("Leaf")
            .build();
        let WidgetNode::Breadcrumb(b) = &node else {
            panic!("expected Breadcrumb");
        };
        assert_eq!(b.separator, ">");
        assert!(matches!(b.children[0], WidgetNode::Button(_)));
        let WidgetNode::Label(sep) = &b.children[1] else {
            panic!("expected separator label");
        };
        assert_eq!(sep.text, " > ");
        assert_eq!(node.key().unwrap().as_str(), "bc");
    }

    #[test]
    fn navigation_widgets_convert_via_from() {
        let rail: WidgetNode<Msg> = nav_rail("r").item(nav_item("A")).into();
        let sb: WidgetNode<Msg> = sidebar("s").child(label("x")).into();
        let tabs: WidgetNode<Msg> = tab_bar("t").tab("One").into();
        let bc: WidgetNode<Msg> = breadcrumb("b").segment("Home").into();
        assert!(matches!(rail, WidgetNode::NavRail(_)));
        assert!(matches!(sb, WidgetNode::Sidebar(_)));
        assert!(matches!(tabs, WidgetNode::TabBar(_)));
        assert!(matches!(bc, WidgetNode::Breadcrumb(_)));
    }
}
