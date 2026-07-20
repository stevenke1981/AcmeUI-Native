//! Navigation widgets: NavRail, Sidebar, TabBar, Breadcrumb.

use crate::WidgetNode;
use crate::foundations::label;
use crate::inputs::button;
use acme_core::WidgetKey;
use acme_layout::{LayoutKind, LayoutStyle, Length};

// ============================================================================
// NavItem / NavRail
// ============================================================================

/// A single destination in a [`NavRail`].
#[derive(Clone, Debug, PartialEq)]
pub struct NavItem<M> {
    pub label: String,
    pub icon: Option<String>,
    pub disabled: bool,
    pub message: Option<M>,
}

impl<M> NavItem<M> {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            disabled: false,
            message: None,
        }
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    pub fn on_click(mut self, message: M) -> Self {
        self.message = Some(message);
        self
    }

    pub fn activate(&self) -> Option<&M> {
        if self.disabled {
            None
        } else {
            self.message.as_ref()
        }
    }
}

/// Create a [`NavItem`].
pub fn nav_item<M>(label: impl Into<String>) -> NavItem<M> {
    NavItem::new(label)
}

/// Vertical navigation rail (expanded or collapsed).
#[derive(Clone, Debug, PartialEq)]
pub struct NavRail<M> {
    pub key: WidgetKey,
    pub items: Vec<NavItem<M>>,
    pub selected: Option<usize>,
    pub collapsed: bool,
    /// Materialized item nodes for layout / gallery paint.
    pub children: Vec<WidgetNode<M>>,
}

/// Create a [`NavRail`] builder.
pub fn nav_rail<M>(key: impl Into<WidgetKey>) -> NavRail<M> {
    NavRail {
        key: key.into(),
        items: Vec::new(),
        selected: None,
        collapsed: false,
        children: Vec::new(),
    }
}

impl<M> NavRail<M> {
    pub fn item(mut self, item: NavItem<M>) -> Self {
        self.items.push(item);
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = Some(index);
        self
    }

    pub fn collapsed(mut self, value: bool) -> Self {
        self.collapsed = value;
        self
    }
}

impl<M: Clone> NavRail<M> {
    /// Materialize child nodes and produce a [`WidgetNode`].
    pub fn build(mut self) -> WidgetNode<M> {
        self.rematerialize();
        WidgetNode::NavRail(self)
    }

    fn rematerialize(&mut self) {
        let key = self.key.as_str();
        self.children = self
            .items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let selected = self.selected == Some(i);
                materialize_nav_item(key, i, item, self.collapsed, selected)
            })
            .collect();
    }

    pub(crate) fn layout_style(&self) -> LayoutStyle {
        let width = if self.collapsed {
            Length::px(56.0)
        } else {
            Length::px(200.0)
        };
        LayoutStyle {
            kind: LayoutKind::Column,
            width,
            gap: 4.0,
            ..Default::default()
        }
    }
}

impl<M: Clone> From<NavRail<M>> for WidgetNode<M> {
    fn from(value: NavRail<M>) -> Self {
        value.build()
    }
}

// ============================================================================
// Sidebar
// ============================================================================

/// Fixed-width side panel with optional header and arbitrary children.
#[derive(Clone, Debug, PartialEq)]
pub struct Sidebar<M> {
    pub key: WidgetKey,
    pub width: f32,
    pub header: Option<String>,
    pub children: Vec<WidgetNode<M>>,
}

/// Create a [`Sidebar`] builder.
pub fn sidebar<M>(key: impl Into<WidgetKey>) -> Sidebar<M> {
    Sidebar {
        key: key.into(),
        width: 224.0,
        header: None,
        children: Vec::new(),
    }
}

impl<M> Sidebar<M> {
    pub fn width(mut self, value: f32) -> Self {
        self.width = crate::finite(value);
        self
    }

    pub fn header(mut self, text: impl Into<String>) -> Self {
        self.header = Some(text.into());
        self
    }

    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.children.push(child.into());
        self
    }
}

impl<M: Clone> Sidebar<M> {
    /// Materialize header (if any) ahead of body children and produce a node.
    pub fn build(mut self) -> WidgetNode<M> {
        if let Some(title) = self.header.clone() {
            let mut nodes = Vec::with_capacity(self.children.len() + 1);
            nodes.push(label::<M>(title));
            nodes.append(&mut self.children);
            self.children = nodes;
        }
        WidgetNode::Sidebar(self)
    }

    pub(crate) fn layout_style(&self) -> LayoutStyle {
        LayoutStyle {
            kind: LayoutKind::Column,
            width: Length::px(self.width),
            gap: 8.0,
            ..Default::default()
        }
    }
}

impl<M: Clone> From<Sidebar<M>> for WidgetNode<M> {
    fn from(value: Sidebar<M>) -> Self {
        value.build()
    }
}

// ============================================================================
// TabBar / TabItem
// ============================================================================

/// A single tab in a [`TabBar`].
#[derive(Clone, Debug, PartialEq)]
pub struct TabItem<M> {
    pub label: String,
    pub disabled: bool,
    pub message: Option<M>,
}

impl<M> TabItem<M> {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            disabled: false,
            message: None,
        }
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    pub fn on_click(mut self, message: M) -> Self {
        self.message = Some(message);
        self
    }

    pub fn activate(&self) -> Option<&M> {
        if self.disabled {
            None
        } else {
            self.message.as_ref()
        }
    }
}

/// Horizontal tab strip.
#[derive(Clone, Debug, PartialEq)]
pub struct TabBar<M> {
    pub key: WidgetKey,
    pub tabs: Vec<TabItem<M>>,
    pub selected: usize,
    /// Materialized tab nodes for layout / gallery paint.
    pub children: Vec<WidgetNode<M>>,
}

/// Create a [`TabBar`] builder.
pub fn tab_bar<M>(key: impl Into<WidgetKey>) -> TabBar<M> {
    TabBar {
        key: key.into(),
        tabs: Vec::new(),
        selected: 0,
        children: Vec::new(),
    }
}

impl<M> TabBar<M> {
    /// Append a tab by label.
    pub fn tab(mut self, label: impl Into<String>) -> Self {
        self.tabs.push(TabItem::new(label));
        self
    }

    /// Append a fully configured [`TabItem`].
    pub fn item(mut self, item: TabItem<M>) -> Self {
        self.tabs.push(item);
        self
    }

    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }
}

impl<M: Clone> TabBar<M> {
    pub fn build(mut self) -> WidgetNode<M> {
        self.rematerialize();
        WidgetNode::TabBar(self)
    }

    fn rematerialize(&mut self) {
        let key = self.key.as_str();
        let selected = self.selected;
        self.children = self
            .tabs
            .iter()
            .enumerate()
            .map(|(i, tab)| {
                let selected = i == selected;
                materialize_tab_item(key, i, tab, selected)
            })
            .collect();
    }

    pub(crate) fn layout_style(&self) -> LayoutStyle {
        LayoutStyle {
            kind: LayoutKind::Row,
            gap: 4.0,
            height: Length::px(36.0),
            ..Default::default()
        }
    }
}

impl<M: Clone> From<TabBar<M>> for WidgetNode<M> {
    fn from(value: TabBar<M>) -> Self {
        value.build()
    }
}

// ============================================================================
// Breadcrumb / BreadcrumbSegment
// ============================================================================

/// One segment in a [`Breadcrumb`] trail.
#[derive(Clone, Debug, PartialEq)]
pub struct BreadcrumbSegment<M> {
    pub label: String,
    pub message: Option<M>,
}

impl<M> BreadcrumbSegment<M> {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            message: None,
        }
    }

    pub fn on_click(mut self, message: M) -> Self {
        self.message = Some(message);
        self
    }

    pub fn activate(&self) -> Option<&M> {
        self.message.as_ref()
    }
}

/// Hierarchical path trail with separators.
#[derive(Clone, Debug, PartialEq)]
pub struct Breadcrumb<M> {
    pub key: WidgetKey,
    pub segments: Vec<BreadcrumbSegment<M>>,
    /// Separator string between segments. Default `"/"`.
    pub separator: String,
    /// Materialized segment + separator nodes.
    pub children: Vec<WidgetNode<M>>,
}

/// Create a [`Breadcrumb`] builder.
pub fn breadcrumb<M>(key: impl Into<WidgetKey>) -> Breadcrumb<M> {
    Breadcrumb {
        key: key.into(),
        segments: Vec::new(),
        separator: "/".into(),
        children: Vec::new(),
    }
}

impl<M> Breadcrumb<M> {
    /// Append a segment by label.
    pub fn segment(mut self, label: impl Into<String>) -> Self {
        self.segments.push(BreadcrumbSegment::new(label));
        self
    }

    /// Append a fully configured segment.
    pub fn item(mut self, segment: BreadcrumbSegment<M>) -> Self {
        self.segments.push(segment);
        self
    }

    pub fn separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }
}

impl<M: Clone> Breadcrumb<M> {
    pub fn build(mut self) -> WidgetNode<M> {
        self.rematerialize();
        WidgetNode::Breadcrumb(self)
    }

    fn rematerialize(&mut self) {
        let key = self.key.as_str();
        let sep = self.separator.clone();
        let mut nodes = Vec::new();
        for (i, seg) in self.segments.iter().enumerate() {
            if i > 0 {
                nodes.push(label::<M>(format!(" {sep} ")));
            }
            nodes.push(materialize_breadcrumb_segment(key, i, seg));
        }
        self.children = nodes;
    }

    pub(crate) fn layout_style(&self) -> LayoutStyle {
        LayoutStyle {
            kind: LayoutKind::Row,
            gap: 2.0,
            height: Length::px(28.0),
            ..Default::default()
        }
    }
}

impl<M: Clone> From<Breadcrumb<M>> for WidgetNode<M> {
    fn from(value: Breadcrumb<M>) -> Self {
        value.build()
    }
}

// ============================================================================
// Materialization helpers
// ============================================================================

fn short_label(label: &str) -> String {
    label
        .chars()
        .next()
        .map(|c| c.to_string())
        .unwrap_or_default()
}

fn materialize_nav_item<M: Clone>(
    key_prefix: &str,
    index: usize,
    item: &NavItem<M>,
    collapsed: bool,
    selected: bool,
) -> WidgetNode<M> {
    let base = if collapsed {
        item.icon
            .clone()
            .unwrap_or_else(|| short_label(&item.label))
    } else if let Some(icon) = &item.icon {
        format!("{icon} {}", item.label)
    } else {
        item.label.clone()
    };
    let text = if selected {
        format!("• {base}")
    } else {
        base
    };
    let key = format!("{key_prefix}_item_{index}");
    if let Some(msg) = item.message.clone() {
        let mut b = button(key.as_str(), text);
        if item.disabled {
            b = b.disabled(true);
        }
        b.on_click(msg)
    } else {
        label(text)
    }
}

fn materialize_tab_item<M: Clone>(
    key_prefix: &str,
    index: usize,
    tab: &TabItem<M>,
    selected: bool,
) -> WidgetNode<M> {
    let text = if selected {
        format!("[{}]", tab.label)
    } else {
        tab.label.clone()
    };
    let key = format!("{key_prefix}_tab_{index}");
    if let Some(msg) = tab.message.clone() {
        let mut b = button(key.as_str(), text);
        if tab.disabled {
            b = b.disabled(true);
        }
        b.on_click(msg)
    } else {
        label(text)
    }
}

fn materialize_breadcrumb_segment<M: Clone>(
    key_prefix: &str,
    index: usize,
    seg: &BreadcrumbSegment<M>,
) -> WidgetNode<M> {
    let key = format!("{key_prefix}_seg_{index}");
    if let Some(msg) = seg.message.clone() {
        button(key.as_str(), seg.label.clone()).on_click(msg)
    } else {
        label(seg.label.clone())
    }
}
