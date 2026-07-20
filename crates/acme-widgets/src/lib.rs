//! Declarative MVP widget descriptions.
#![forbid(unsafe_op_in_unsafe_fn)]

pub use acme_core::WidgetKey;
use acme_layout::{Edges, LayoutKind, LayoutNode, LayoutStyle, Length, Overflow};
use acme_theme::{Theme, ThemeColor};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
    Danger,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ButtonState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedButtonStyle {
    pub background: ThemeColor,
    pub foreground: ThemeColor,
    pub border: ThemeColor,
    pub focus: ThemeColor,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tooltip<M> {
    pub key: WidgetKey,
    pub child: Box<WidgetNode<M>>,
    pub text: String,
    pub delay_ms: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Popover<M> {
    pub key: WidgetKey,
    /// First element is anchor, second is content.
    pub children: Vec<WidgetNode<M>>,
    pub open: bool,
    pub placement: PopoverPlacement,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PopoverPlacement {
    Bottom,
    Top,
    Left,
    Right,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Menu<M> {
    pub key: WidgetKey,
    pub items: Vec<MenuItem<M>>,
    pub open: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MenuItem<M> {
    pub key: WidgetKey,
    pub label: String,
    pub disabled: bool,
    pub message: Option<M>,
    pub separator: bool,
    pub children: Vec<MenuItem<M>>,
}
impl<M> MenuItem<M> {
    pub fn activate(&self) -> Option<&M> {
        if self.disabled || self.separator {
            None
        } else {
            self.message.as_ref()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dialog<M> {
    pub key: WidgetKey,
    pub title: String,
    pub content: Box<WidgetNode<M>>,
    pub open: bool,
    pub modal: bool,
    pub width: Length,
    pub height: Length,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WidgetNode<M> {
    Row(Container<M>),
    Column(Container<M>),
    Stack(Container<M>),
    Label(Label),
    Button(Button<M>),
    Card(Container<M>),
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
}
impl<M> WidgetNode<M> {
    pub fn key(&self) -> Option<&WidgetKey> {
        match self {
            Self::Button(v) => Some(&v.key),
            Self::ScrollView(v) => Some(&v.key),
            Self::Tooltip(v) => Some(&v.key),
            Self::VirtualList(v) => Some(&v.key),
            Self::Row(v) | Self::Column(v) | Self::Stack(v) | Self::Card(v) => v.key.as_ref(),
            Self::Label(_) | Self::Separator(_) => None,
            Self::Popover(v) => Some(&v.key),
            Self::Menu(v) => Some(&v.key),
            Self::Dialog(v) => Some(&v.key),
            Self::Tree(v) => Some(&v.key),
            Self::Table(v) => Some(&v.key),
            Self::DataGrid(v) => Some(&v.key),
        }
    }
    pub fn children(&self) -> &[WidgetNode<M>] {
        match self {
            Self::Row(v) | Self::Column(v) | Self::Stack(v) | Self::Card(v) => &v.children,
            Self::ScrollView(v) => &v.children,
            Self::Tooltip(v) => std::slice::from_ref(&v.child),
            Self::VirtualList(_) => &[],
            Self::Popover(v) => &v.children,
            Self::Menu(_) => &[],
            Self::Dialog(v) => std::slice::from_ref(&v.content),
            Self::Tree(_) => &[],
            Self::Table(v) => &v.all_cells,
            Self::DataGrid(v) => &v.all_cells,
            Self::Label(_) | Self::Button(_) | Self::Separator(_) => &[],
        }
    }
    pub fn to_layout(&self, next: &mut u64) -> LayoutNode {
        let id = *next;
        *next += 1;
        match self {
            Self::Row(v) => LayoutNode::container(
                id,
                v.layout(LayoutKind::Row),
                v.children.iter().map(|c| c.to_layout(next)).collect(),
            ),
            Self::Column(v) => LayoutNode::container(
                id,
                v.layout(LayoutKind::Column),
                v.children.iter().map(|c| c.to_layout(next)).collect(),
            ),
            Self::Stack(v) => LayoutNode::container(
                id,
                v.layout(LayoutKind::Stack),
                v.children.iter().map(|c| c.to_layout(next)).collect(),
            ),
            Self::Card(v) => LayoutNode::container(
                id,
                v.layout(LayoutKind::Column),
                v.children.iter().map(|c| c.to_layout(next)).collect(),
            ),
            Self::ScrollView(v) => LayoutNode::container(
                id,
                v.layout(),
                v.children.iter().map(|c| c.to_layout(next)).collect(),
            ),
            Self::Label(_) => LayoutNode::leaf(id, LayoutStyle::default()),
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
            Self::Tooltip(v) => v.child.to_layout(next),
            Self::Popover(v) => v.children[0].to_layout(next),
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
                let mut child_nodes = Vec::with_capacity(v.items.len());
                for item in &v.items {
                    child_nodes.push(LayoutNode::leaf(
                        *next,
                        LayoutStyle {
                            width: Length::Auto,
                            height: Length::px(24.0),
                            padding: Edges {
                                left: item.depth as f32 * 20.0,
                                ..Edges::default()
                            },
                            ..Default::default()
                        },
                    ));
                    *next += 1;
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
                if v.header_visible {
                    let header_children: Vec<LayoutNode> = v
                        .columns
                        .iter()
                        .map(|col| {
                            let nid = *next;
                            *next += 1;
                            LayoutNode::leaf(
                                nid,
                                LayoutStyle {
                                    width: col.width,
                                    min_height: Length::px(24.0),
                                    ..Default::default()
                                },
                            )
                        })
                        .collect();
                    child_nodes.push(LayoutNode::container(
                        *next,
                        LayoutStyle::row(),
                        header_children,
                    ));
                    *next += 1;
                }
                for row in &v.rows {
                    let row_children: Vec<LayoutNode> = row
                        .iter()
                        .map(|_| {
                            let nid = *next;
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
                    child_nodes.push(LayoutNode::container(
                        *next,
                        LayoutStyle::row(),
                        row_children,
                    ));
                    *next += 1;
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
                // Header row
                let header_children: Vec<LayoutNode> = v
                    .columns
                    .iter()
                    .enumerate()
                    .map(|(i, col)| {
                        let nid = *next;
                        *next += 1;
                        let mut style = LayoutStyle {
                            width: col.width,
                            min_height: Length::px(24.0),
                            ..Default::default()
                        };
                        if v.sort_column == Some(i) {
                            style.flex_grow = 1.0;
                        }
                        LayoutNode::leaf(nid, style)
                    })
                    .collect();
                child_nodes.push(LayoutNode::container(
                    *next,
                    LayoutStyle::row(),
                    header_children,
                ));
                *next += 1;
                // Data rows
                for row in &v.rows {
                    let row_children: Vec<LayoutNode> = row
                        .cells
                        .iter()
                        .map(|_| {
                            let nid = *next;
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
                    child_nodes.push(LayoutNode::container(
                        *next,
                        LayoutStyle::row(),
                        row_children,
                    ));
                    *next += 1;
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
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Label {
    pub text: String,
}
pub fn label<M>(text: impl Into<String>) -> WidgetNode<M> {
    WidgetNode::Label(Label { text: text.into() })
}

#[derive(Clone, Debug, PartialEq)]
pub struct Button<M> {
    pub key: WidgetKey,
    pub label: String,
    pub variant: ButtonVariant,
    pub disabled: bool,
    message: Option<M>,
}
pub fn button<M>(key: impl Into<WidgetKey>, label: impl Into<String>) -> Button<M> {
    Button {
        key: key.into(),
        label: label.into(),
        variant: ButtonVariant::Secondary,
        disabled: false,
        message: None,
    }
}
impl<M> Button<M> {
    pub fn primary(mut self) -> Self {
        self.variant = ButtonVariant::Primary;
        self
    }
    pub fn variant(mut self, value: ButtonVariant) -> Self {
        self.variant = value;
        self
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    pub fn on_click(mut self, message: M) -> WidgetNode<M> {
        self.message = Some(message);
        WidgetNode::Button(self)
    }
    pub fn activate(&self) -> Option<&M> {
        if self.disabled {
            None
        } else {
            self.message.as_ref()
        }
    }
    pub fn resolve_style(&self, theme: &Theme, state: ButtonState) -> ResolvedButtonStyle {
        let c = theme.colors;
        let background = if self.disabled {
            c.surface
        } else {
            match self.variant {
                ButtonVariant::Primary => {
                    if state.hovered {
                        c.accent_hover
                    } else {
                        c.accent
                    }
                }
                ButtonVariant::Danger => c.danger,
                ButtonVariant::Secondary => {
                    if state.hovered {
                        c.surface_hover
                    } else {
                        c.surface
                    }
                }
                ButtonVariant::Ghost => c.background,
            }
        };
        ResolvedButtonStyle {
            background,
            foreground: if self.disabled {
                c.disabled_text
            } else if self.variant == ButtonVariant::Primary {
                c.on_accent
            } else if self.variant == ButtonVariant::Danger {
                c.on_danger
            } else {
                c.text
            },
            border: c.border,
            focus: c.focus,
        }
    }
}
impl<M> From<Button<M>> for WidgetNode<M> {
    fn from(value: Button<M>) -> Self {
        WidgetNode::Button(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Container<M> {
    pub key: Option<WidgetKey>,
    pub children: Vec<WidgetNode<M>>,
    pub gap: f32,
    pub padding: Edges,
}
impl<M> Container<M> {
    fn new() -> Self {
        Self {
            key: None,
            children: vec![],
            gap: 0.0,
            padding: Edges::default(),
        }
    }
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.children.push(child.into());
        self
    }
    pub fn gap(mut self, value: f32) -> Self {
        self.gap = finite(value);
        self
    }
    pub fn padding(mut self, value: f32) -> Self {
        self.padding = Edges::all(value);
        self
    }
    fn layout(&self, kind: LayoutKind) -> LayoutStyle {
        LayoutStyle {
            kind,
            gap: self.gap,
            padding: self.padding,
            ..Default::default()
        }
    }
}

pub struct ContainerBuilder<M> {
    container: Container<M>,
    kind: LayoutKind,
}
impl<M> ContainerBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.container = self.container.key(key);
        self
    }
    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.container = self.container.child(child);
        self
    }
    pub fn gap(mut self, value: f32) -> Self {
        self.container = self.container.gap(value);
        self
    }
    pub fn padding(mut self, value: f32) -> Self {
        self.container = self.container.padding(value);
        self
    }
    pub fn build(self) -> WidgetNode<M> {
        match self.kind {
            LayoutKind::Row => WidgetNode::Row(self.container),
            LayoutKind::Column => WidgetNode::Column(self.container),
            LayoutKind::Stack => WidgetNode::Stack(self.container),
            LayoutKind::Leaf => WidgetNode::Card(self.container),
        }
    }
}
impl<M> From<ContainerBuilder<M>> for WidgetNode<M> {
    fn from(value: ContainerBuilder<M>) -> Self {
        value.build()
    }
}
pub fn row<M>() -> ContainerBuilder<M> {
    ContainerBuilder {
        container: Container::new(),
        kind: LayoutKind::Row,
    }
}
pub fn column<M>() -> ContainerBuilder<M> {
    ContainerBuilder {
        container: Container::new(),
        kind: LayoutKind::Column,
    }
}
pub fn stack<M>() -> ContainerBuilder<M> {
    ContainerBuilder {
        container: Container::new(),
        kind: LayoutKind::Stack,
    }
}
pub fn card<M>() -> ContainerBuilder<M> {
    ContainerBuilder {
        container: Container::new(),
        kind: LayoutKind::Leaf,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Separator {
    pub thickness: f32,
}
pub fn separator<M>() -> WidgetNode<M> {
    WidgetNode::Separator(Separator { thickness: 1.0 })
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollView<M> {
    pub key: WidgetKey,
    pub children: Vec<WidgetNode<M>>,
    pub viewport_height: Length,
}
impl<M> ScrollView<M> {
    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.children.push(child.into());
        self
    }
    pub fn viewport_height(mut self, value: f32) -> Self {
        self.viewport_height = Length::px(value);
        self
    }
    pub fn build(self) -> WidgetNode<M> {
        WidgetNode::ScrollView(self)
    }
    fn layout(&self) -> LayoutStyle {
        LayoutStyle {
            kind: LayoutKind::Column,
            height: self.viewport_height,
            overflow: Overflow::Scroll,
            ..Default::default()
        }
    }
}
pub fn scroll_view<M>(key: impl Into<WidgetKey>) -> ScrollView<M> {
    ScrollView {
        key: key.into(),
        children: vec![],
        viewport_height: Length::Auto,
    }
}

/// A virtual scrolling list that only renders visible items.
///
/// The list computes which items are visible based on the current scroll offset
/// and viewport height, and only calls the builder for those items.
pub struct VirtualList<M> {
    pub key: WidgetKey,
    /// Total number of items.
    pub item_count: usize,
    /// Height of each item in logical pixels.
    pub item_height: f32,
    /// Current scroll offset in logical pixels.
    pub scroll_offset: f32,
    /// Viewport height in logical pixels.
    pub viewport_height: f32,
    /// Builder function for creating item widgets.
    /// Called with `(index)` — only visible items are rendered.
    pub item_builder: Option<Arc<dyn Fn(usize) -> WidgetNode<M>>>,
}

impl<M> std::clone::Clone for VirtualList<M> {
    fn clone(&self) -> Self {
        Self {
            key: self.key.clone(),
            item_count: self.item_count,
            item_height: self.item_height,
            scroll_offset: self.scroll_offset,
            viewport_height: self.viewport_height,
            item_builder: self.item_builder.clone(),
        }
    }
}

impl<M> std::fmt::Debug for VirtualList<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VirtualList")
            .field("key", &self.key)
            .field("item_count", &self.item_count)
            .field("item_height", &self.item_height)
            .field("scroll_offset", &self.scroll_offset)
            .field("viewport_height", &self.viewport_height)
            .field("item_builder", &"<closure>")
            .finish()
    }
}

impl<M> PartialEq for VirtualList<M> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.item_count == other.item_count
            && self.item_height == other.item_height
            && self.scroll_offset == other.scroll_offset
            && self.viewport_height == other.viewport_height
    }
}

impl<M> VirtualList<M> {
    /// Set the viewport height in logical pixels.
    pub fn viewport_height(mut self, value: f32) -> Self {
        self.viewport_height = finite(value);
        self
    }

    /// Set the scroll offset in logical pixels.
    pub fn scroll_offset(mut self, value: f32) -> Self {
        self.scroll_offset = value.max(0.0);
        self
    }

    /// Set the item builder function.
    pub fn builder(mut self, builder: impl Fn(usize) -> WidgetNode<M> + 'static) -> Self {
        self.item_builder = Some(Arc::new(builder));
        self
    }

    /// Build into a `WidgetNode`.
    pub fn build(self) -> WidgetNode<M> {
        WidgetNode::VirtualList(self)
    }

    /// Get the range of visible item indices, with one extra item of overscan
    /// at each end for smooth scrolling.
    pub fn visible_range(&self) -> std::ops::Range<usize> {
        if self.item_count == 0 || self.item_height <= 0.0 {
            return 0..0;
        }
        let first = (self.scroll_offset / self.item_height).floor() as usize;
        // +2 for overscan (one on each side)
        let count = (self.viewport_height / self.item_height).ceil() as usize + 2;
        first..(first + count).min(self.item_count)
    }

    /// Total content height in logical pixels.
    pub fn content_height(&self) -> f32 {
        self.item_count as f32 * self.item_height
    }
}

/// Create a new `VirtualList` builder.
///
/// # Example
/// ```ignore
/// virtual_list::<Msg>("list", 1000, 40.0)
///     .viewport_height(300.0)
///     .builder(|i| label(format!("Item {i}")))
///     .build()
/// ```
pub fn virtual_list<M>(
    key: impl Into<WidgetKey>,
    item_count: usize,
    item_height: f32,
) -> VirtualList<M> {
    VirtualList {
        key: key.into(),
        item_count,
        item_height: finite(item_height),
        scroll_offset: 0.0,
        viewport_height: 0.0,
        item_builder: None,
    }
}

pub fn tooltip<M>(
    key: impl Into<WidgetKey>,
    child: impl Into<WidgetNode<M>>,
    text: impl Into<String>,
) -> Tooltip<M> {
    Tooltip {
        key: key.into(),
        child: Box::new(child.into()),
        text: text.into(),
        delay_ms: 500,
    }
}
impl<M> From<Tooltip<M>> for WidgetNode<M> {
    fn from(value: Tooltip<M>) -> Self {
        WidgetNode::Tooltip(value)
    }
}

// ---------------------------------------------------------------------------
// Popover builder
// ---------------------------------------------------------------------------

pub fn popover<M>(key: impl Into<WidgetKey>, anchor: impl Into<WidgetNode<M>>) -> Popover<M> {
    Popover {
        key: key.into(),
        children: vec![anchor.into()],
        open: false,
        placement: PopoverPlacement::Bottom,
    }
}
impl<M> Popover<M> {
    pub fn content(mut self, content: impl Into<WidgetNode<M>>) -> Self {
        self.children.push(content.into());
        self
    }
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }
    pub fn placement(mut self, value: PopoverPlacement) -> Self {
        self.placement = value;
        self
    }
    pub fn build(self) -> WidgetNode<M> {
        WidgetNode::Popover(self)
    }
}
impl<M> From<Popover<M>> for WidgetNode<M> {
    fn from(value: Popover<M>) -> Self {
        WidgetNode::Popover(value)
    }
}

// ---------------------------------------------------------------------------
// Menu builder
// ---------------------------------------------------------------------------

pub fn menu<M>(key: impl Into<WidgetKey>) -> Menu<M> {
    Menu {
        key: key.into(),
        items: vec![],
        open: false,
    }
}
impl<M> Menu<M> {
    pub fn item(mut self, item: MenuItem<M>) -> Self {
        self.items.push(item);
        self
    }
    pub fn separator(mut self) -> Self {
        self.items.push(MenuItem {
            key: WidgetKey::from(""),
            label: String::new(),
            disabled: true,
            message: None,
            separator: true,
            children: vec![],
        });
        self
    }
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }
    pub fn build(self) -> WidgetNode<M> {
        WidgetNode::Menu(self)
    }
}
impl<M> From<Menu<M>> for WidgetNode<M> {
    fn from(value: Menu<M>) -> Self {
        WidgetNode::Menu(value)
    }
}

pub fn menu_item<M>(key: impl Into<WidgetKey>, label: impl Into<String>) -> MenuItem<M> {
    MenuItem {
        key: key.into(),
        label: label.into(),
        disabled: false,
        message: None,
        separator: false,
        children: vec![],
    }
}
impl<M> MenuItem<M> {
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    pub fn on_click(mut self, message: M) -> Self {
        self.message = Some(message);
        self
    }
    pub fn submenu(mut self, child: MenuItem<M>) -> Self {
        self.children.push(child);
        self
    }
}

// ---------------------------------------------------------------------------
// Dialog builder
// ---------------------------------------------------------------------------

pub fn dialog<M>(key: impl Into<WidgetKey>, content: impl Into<WidgetNode<M>>) -> Dialog<M> {
    Dialog {
        key: key.into(),
        title: String::new(),
        content: Box::new(content.into()),
        open: false,
        modal: true,
        width: Length::px(480.0),
        height: Length::Auto,
    }
}
impl<M> Dialog<M> {
    pub fn title(mut self, value: impl Into<String>) -> Self {
        self.title = value.into();
        self
    }
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }
    pub fn modal(mut self, value: bool) -> Self {
        self.modal = value;
        self
    }
    pub fn width(mut self, value: f32) -> Self {
        self.width = Length::px(finite(value));
        self
    }
    pub fn height(mut self, value: f32) -> Self {
        self.height = Length::px(finite(value));
        self
    }
    pub fn build(self) -> WidgetNode<M> {
        WidgetNode::Dialog(self)
    }
}
impl<M> From<Dialog<M>> for WidgetNode<M> {
    fn from(value: Dialog<M>) -> Self {
        WidgetNode::Dialog(value)
    }
}

// ---------------------------------------------------------------------------
// Tree widget
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct TreeNode<M> {
    pub key: WidgetKey,
    pub label: String,
    pub children: Vec<TreeNode<M>>,
    pub expanded: bool,
    pub depth: usize,
    pub disabled: bool,
    pub message: Option<M>,
}

impl<M> TreeNode<M> {
    pub fn activate(&self) -> Option<&M> {
        if self.disabled {
            None
        } else {
            self.message.as_ref()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Tree<M> {
    pub key: WidgetKey,
    pub items: Vec<TreeNode<M>>,
}

pub fn tree<M>(key: impl Into<WidgetKey>) -> Tree<M> {
    Tree {
        key: key.into(),
        items: vec![],
    }
}

pub fn tree_node<M>(key: impl Into<WidgetKey>, label: impl Into<String>) -> TreeNode<M> {
    TreeNode {
        key: key.into(),
        label: label.into(),
        children: vec![],
        expanded: false,
        depth: 0,
        disabled: false,
        message: None,
    }
}

impl<M> TreeNode<M> {
    pub fn child(mut self, node: TreeNode<M>) -> Self {
        self.children.push(node);
        self
    }
    pub fn expanded(mut self, value: bool) -> Self {
        self.expanded = value;
        self
    }
    pub fn depth(mut self, value: usize) -> Self {
        self.depth = value;
        self
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    pub fn on_activate(mut self, message: M) -> Self {
        self.message = Some(message);
        self
    }
}

impl<M> Tree<M> {
    pub fn item(mut self, node: TreeNode<M>) -> Self {
        self.items.push(node);
        self
    }
    pub fn build(self) -> WidgetNode<M> {
        WidgetNode::Tree(self)
    }
}

impl<M> From<Tree<M>> for WidgetNode<M> {
    fn from(value: Tree<M>) -> Self {
        WidgetNode::Tree(value)
    }
}

// ---------------------------------------------------------------------------
// Table widget
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct TableColumn {
    pub key: WidgetKey,
    pub title: String,
    pub width: Length,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Table<M> {
    pub key: WidgetKey,
    pub columns: Vec<TableColumn>,
    pub rows: Vec<Vec<WidgetNode<M>>>,
    pub header_visible: bool,
    pub(crate) all_cells: Vec<WidgetNode<M>>,
}

impl TableColumn {
    pub fn width(mut self, value: Length) -> Self {
        self.width = value;
        self
    }
}

pub fn table<M>(key: impl Into<WidgetKey>) -> Table<M> {
    Table {
        key: key.into(),
        columns: vec![],
        rows: vec![],
        header_visible: true,
        all_cells: vec![],
    }
}

pub fn table_column(key: impl Into<WidgetKey>, title: impl Into<String>) -> TableColumn {
    TableColumn {
        key: key.into(),
        title: title.into(),
        width: Length::Auto,
    }
}

impl<M: Clone> Table<M> {
    pub fn column(mut self, col: TableColumn) -> Self {
        self.columns.push(col);
        self
    }
    pub fn add_row(mut self, cells: Vec<WidgetNode<M>>) -> Self {
        self.rows.push(cells);
        self
    }
    pub fn header_visible(mut self, value: bool) -> Self {
        self.header_visible = value;
        self
    }
    pub fn build(mut self) -> WidgetNode<M> {
        self.all_cells = self.rows.iter().flat_map(|r| r.iter().cloned()).collect();
        WidgetNode::Table(self)
    }
}

impl<M> From<Table<M>> for WidgetNode<M> {
    fn from(value: Table<M>) -> Self {
        WidgetNode::Table(value)
    }
}

// ---------------------------------------------------------------------------
// DataGrid widget
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct DataGridColumn {
    pub key: WidgetKey,
    pub title: String,
    pub width: Length,
    pub sortable: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DataGridRow<M> {
    pub cells: Vec<WidgetNode<M>>,
    pub selected: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DataGrid<M> {
    pub key: WidgetKey,
    pub columns: Vec<DataGridColumn>,
    pub rows: Vec<DataGridRow<M>>,
    pub selected_row: Option<usize>,
    pub sort_column: Option<usize>,
    pub sort_ascending: bool,
    pub(crate) all_cells: Vec<WidgetNode<M>>,
}

pub fn datagrid<M>(key: impl Into<WidgetKey>) -> DataGrid<M> {
    DataGrid {
        key: key.into(),
        columns: vec![],
        rows: vec![],
        selected_row: None,
        sort_column: None,
        sort_ascending: true,
        all_cells: vec![],
    }
}

pub fn datagrid_column(key: impl Into<WidgetKey>, title: impl Into<String>) -> DataGridColumn {
    DataGridColumn {
        key: key.into(),
        title: title.into(),
        width: Length::Auto,
        sortable: false,
    }
}

impl DataGridColumn {
    pub fn sortable(mut self, value: bool) -> Self {
        self.sortable = value;
        self
    }
    pub fn width(mut self, value: Length) -> Self {
        self.width = value;
        self
    }
}

impl<M> DataGridRow<M> {
    pub fn new(cells: Vec<WidgetNode<M>>) -> Self {
        Self {
            cells,
            selected: false,
        }
    }
    pub fn selected(mut self, value: bool) -> Self {
        self.selected = value;
        self
    }
}

impl<M: Clone> DataGrid<M> {
    pub fn column(mut self, col: DataGridColumn) -> Self {
        self.columns.push(col);
        self
    }
    pub fn add_row(mut self, row: DataGridRow<M>) -> Self {
        self.rows.push(row);
        self
    }
    pub fn sort_column(mut self, col_index: usize) -> Self {
        self.sort_column = Some(col_index);
        self
    }
    pub fn build(mut self) -> WidgetNode<M> {
        self.all_cells = self
            .rows
            .iter()
            .flat_map(|r| r.cells.iter().cloned())
            .collect();
        WidgetNode::DataGrid(self)
    }
}

impl<M> From<DataGrid<M>> for WidgetNode<M> {
    fn from(value: DataGrid<M>) -> Self {
        WidgetNode::DataGrid(value)
    }
}

fn finite(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
            theme.colors.accent
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
            theme.colors.accent_hover
        );
    }
    #[test]
    fn scroll_layout_is_clipped() {
        let tree = scroll_view::<Msg>("scroll")
            .viewport_height(100.0)
            .child(label("內容"))
            .build();
        let mut id = 1;
        let layout = tree.to_layout(&mut id);
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
        let mut next = 1;
        let layout = node.to_layout(&mut next);
        // Tooltip delegates to child; LayoutNode id is auto-incremented.
        // Verify structural delegation: leaf with no children.
        assert_eq!(layout.children.len(), 0);
        assert_eq!(layout.style.kind, acme_layout::LayoutKind::Leaf);
    }

    // ------------------------------------------------------------------
    // VirtualList tests
    // ------------------------------------------------------------------

    #[test]
    fn virtual_list_visible_range_at_start() {
        let list = virtual_list::<Msg>("list", 100, 40.0).viewport_height(200.0);
        // scroll_offset=0, viewport_height=200, item_height=40
        // first = 0, count = ceil(200/40) + 2 = 5 + 2 = 7
        let range = list.visible_range();
        assert_eq!(range, 0..7);
    }

    #[test]
    fn virtual_list_visible_range_scrolled() {
        let list = virtual_list::<Msg>("list", 100, 40.0)
            .viewport_height(200.0)
            .scroll_offset(100.0);
        // first = floor(100/40) = 2, count = 7
        let range = list.visible_range();
        assert_eq!(range, 2..9);
    }

    #[test]
    fn virtual_list_visible_range_at_end() {
        let list = virtual_list::<Msg>("list", 10, 40.0)
            .viewport_height(200.0)
            .scroll_offset(320.0);
        // first = floor(320/40) = 8, count = 7, max = 8+7=15, clamped to 10
        let range = list.visible_range();
        assert_eq!(range, 8..10);
    }

    #[test]
    fn virtual_list_visible_range_empty_list() {
        let list = virtual_list::<Msg>("list", 0, 40.0).viewport_height(200.0);
        let range = list.visible_range();
        assert_eq!(range, 0..0);
    }

    #[test]
    fn virtual_list_content_height() {
        let list = virtual_list::<Msg>("list", 50, 30.0);
        assert!((list.content_height() - 1500.0).abs() < f32::EPSILON);
    }

    #[test]
    fn virtual_list_content_height_empty() {
        let list = virtual_list::<Msg>("list", 0, 30.0);
        assert!((list.content_height() - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn virtual_list_key_is_accessible() {
        let node: WidgetNode<Msg> = virtual_list::<Msg>("mylist", 10, 40.0)
            .viewport_height(200.0)
            .build();
        assert_eq!(node.key().unwrap().as_str(), "mylist");
    }

    #[test]
    fn virtual_list_to_layout_has_viewport_height() {
        let node: WidgetNode<Msg> = virtual_list::<Msg>("list", 100, 40.0)
            .viewport_height(300.0)
            .build();
        let mut next = 1;
        let layout = node.to_layout(&mut next);
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
        let mut next = 1;
        let layout = node.to_layout(&mut next);
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
        let mut next = 1;
        let layout = node.to_layout(&mut next);
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
        let mut next = 1;
        let layout = d.to_layout(&mut next);
        assert_eq!(layout.style.width, Length::px(400.0));
        assert_eq!(layout.style.height, Length::px(300.0));
        assert_eq!(layout.style.kind, LayoutKind::Leaf);
    }

    #[test]
    fn dialog_to_layout_defaults() {
        let d: WidgetNode<Msg> = dialog::<Msg>("d", label("x")).build();
        let mut next = 1;
        let layout = d.to_layout(&mut next);
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
    fn tree_creates_items_with_labels_and_depth() {
        let node: WidgetNode<Msg> = tree::<Msg>("tree")
            .item(tree_node("n1", "Node 1").depth(0))
            .item(tree_node("n2", "Node 2").depth(1).expanded(true))
            .build();
        let WidgetNode::Tree(t) = &node else {
            panic!("expected Tree variant");
        };
        assert_eq!(t.items.len(), 2);
        assert_eq!(t.items[0].label, "Node 1");
        assert_eq!(t.items[0].depth, 0);
        assert_eq!(t.items[1].label, "Node 2");
        assert_eq!(t.items[1].depth, 1);
        assert!(t.items[1].expanded);
    }

    #[test]
    fn tree_item_activation_returns_message() {
        let item = tree_node::<Msg>("n1", "Item").on_activate(Msg::Save);
        assert_eq!(item.activate(), Some(&Msg::Save));
    }

    #[test]
    fn tree_item_disabled_does_not_activate() {
        let item = tree_node::<Msg>("n1", "Item")
            .disabled(true)
            .on_activate(Msg::Save);
        assert_eq!(item.activate(), None);
    }

    #[test]
    fn tree_to_layout_creates_column_with_children() {
        let node: WidgetNode<Msg> = tree::<Msg>("t")
            .item(tree_node("n1", "Root").depth(0))
            .item(tree_node("n2", "Child").depth(1))
            .build();
        let mut next = 1;
        let layout = node.to_layout(&mut next);
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert_eq!(layout.children.len(), 2);
        // First child (depth 0): padding left should be 0
        assert_eq!(layout.children[0].style.padding.left, 0.0);
        // Second child (depth 1): padding left should be 20.0
        assert_eq!(layout.children[1].style.padding.left, 20.0);
    }

    #[test]
    fn tree_key_is_accessible() {
        let node: WidgetNode<Msg> = tree::<Msg>("mytree").build();
        assert_eq!(node.key().unwrap().as_str(), "mytree");
    }

    #[test]
    fn tree_children_is_empty() {
        let node: WidgetNode<Msg> = tree::<Msg>("t").item(tree_node("n1", "Item")).build();
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
            .column(table_column("name", "Name").width(Length::px(100.0)))
            .column(table_column("age", "Age"))
            .add_row(vec![label("Alice"), label("30")])
            .add_row(vec![label("Bob"), label("25")])
            .build();
        let WidgetNode::Table(t) = &node else {
            panic!("expected Table variant");
        };
        assert_eq!(t.columns.len(), 2);
        assert_eq!(t.columns[0].title, "Name");
        assert_eq!(t.columns[0].width, Length::px(100.0));
        assert_eq!(t.columns[1].title, "Age");
        assert_eq!(t.rows.len(), 2);
        assert_eq!(t.rows[0].len(), 2);
        assert!(t.header_visible);
    }

    #[test]
    fn table_header_can_be_hidden() {
        let node: WidgetNode<Msg> = table::<Msg>("t")
            .column(table_column("a", "A"))
            .add_row(vec![label("v1")])
            .header_visible(false)
            .build();
        let WidgetNode::Table(t) = &node else {
            panic!("expected Table variant");
        };
        assert!(!t.header_visible);
    }

    #[test]
    fn table_to_layout_produces_correct_structure() {
        let node: WidgetNode<Msg> = table::<Msg>("t")
            .column(table_column("a", "A"))
            .column(table_column("b", "B"))
            .add_row(vec![label("a1"), label("b1")])
            .add_row(vec![label("a2"), label("b2")])
            .build();
        let mut next = 1;
        let layout = node.to_layout(&mut next);
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
            .column(table_column("a", "A"))
            .add_row(vec![label("c1")])
            .add_row(vec![label("c2")])
            .build();
        assert_eq!(node.children().len(), 2);
    }

    #[test]
    fn table_key_is_accessible() {
        let node: WidgetNode<Msg> = table::<Msg>("mytable")
            .column(table_column("a", "A"))
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
            .column(datagrid_column("name", "Name"))
            .column(datagrid_column("score", "Score"))
            .add_row(DataGridRow::new(vec![label("Alice"), label("95")]))
            .add_row(DataGridRow::new(vec![label("Bob"), label("87")]))
            .build();
        let WidgetNode::DataGrid(d) = &node else {
            panic!("expected DataGrid variant");
        };
        assert_eq!(d.columns.len(), 2);
        assert_eq!(d.columns[0].title, "Name");
        assert_eq!(d.columns[1].title, "Score");
        assert_eq!(d.rows.len(), 2);
        assert_eq!(d.rows[0].cells.len(), 2);
        assert!(!d.rows[0].selected);
    }

    #[test]
    fn datagrid_sort_column_tracking() {
        let node: WidgetNode<Msg> = datagrid::<Msg>("grid")
            .column(datagrid_column("a", "A").sortable(true))
            .column(datagrid_column("b", "B"))
            .add_row(DataGridRow::new(vec![label("x"), label("y")]))
            .sort_column(0)
            .build();
        let WidgetNode::DataGrid(d) = &node else {
            panic!("expected DataGrid variant");
        };
        assert_eq!(d.sort_column, Some(0));
        assert!(d.sort_ascending);
    }

    #[test]
    fn datagrid_selected_row_tracking() {
        let mut grid = datagrid::<Msg>("grid")
            .column(datagrid_column("a", "A"))
            .add_row(DataGridRow::new(vec![label("r1")]).selected(true))
            .add_row(DataGridRow::new(vec![label("r2")]));
        grid.selected_row = Some(0);
        let node: WidgetNode<Msg> = grid.build();
        let WidgetNode::DataGrid(d) = &node else {
            panic!("expected DataGrid variant");
        };
        assert_eq!(d.selected_row, Some(0));
        assert!(d.rows[0].selected);
        assert!(!d.rows[1].selected);
    }

    #[test]
    fn datagrid_children_returns_cells() {
        let node: WidgetNode<Msg> = datagrid::<Msg>("grid")
            .column(datagrid_column("a", "A"))
            .add_row(DataGridRow::new(vec![label("x")]))
            .add_row(DataGridRow::new(vec![label("y")]))
            .build();
        assert_eq!(node.children().len(), 2);
    }

    #[test]
    fn datagrid_to_layout_produces_correct_structure() {
        let node: WidgetNode<Msg> = datagrid::<Msg>("grid")
            .column(datagrid_column("a", "A"))
            .column(datagrid_column("b", "B"))
            .add_row(DataGridRow::new(vec![label("a1"), label("b1")]))
            .build();
        let mut next = 1;
        let layout = node.to_layout(&mut next);
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
            .column(datagrid_column("a", "A"))
            .build();
        assert_eq!(node.key().unwrap().as_str(), "mygrid");
    }

    #[test]
    fn datagrid_from_conversion() {
        let d = datagrid::<Msg>("d");
        let node: WidgetNode<Msg> = d.into();
        assert!(matches!(node, WidgetNode::DataGrid(_)));
    }
}
