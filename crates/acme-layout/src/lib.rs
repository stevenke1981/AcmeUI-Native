//! Framework-owned facade over Taffy.
#![forbid(unsafe_op_in_unsafe_fn)]

use std::collections::HashMap;
use taffy::prelude::{
    AvailableSpace, Dimension, Display, FlexDirection, Size, Style, TaffyTree, length, percent,
};
use taffy::style::{Overflow as TaffyOverflow, Position};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Length {
    Auto,
    Px(f32),
    Percent(f32),
}
impl Length {
    pub fn px(value: f32) -> Self {
        Self::Px(normalize(value))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Edges {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}
impl Edges {
    pub fn all(value: f32) -> Self {
        let value = normalize(value);
        Self {
            left: value,
            right: value,
            top: value,
            bottom: value,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutKind {
    Row,
    Column,
    Stack,
    Leaf,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overflow {
    Visible,
    Clip,
    Scroll,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayoutStyle {
    pub kind: LayoutKind,
    pub width: Length,
    pub height: Length,
    pub min_width: Length,
    pub min_height: Length,
    pub max_width: Length,
    pub max_height: Length,
    pub padding: Edges,
    pub gap: f32,
    pub flex_grow: f32,
    pub flex_shrink: f32,
    pub overflow: Overflow,
}
impl Default for LayoutStyle {
    fn default() -> Self {
        Self {
            kind: LayoutKind::Leaf,
            width: Length::Auto,
            height: Length::Auto,
            min_width: Length::Auto,
            min_height: Length::Auto,
            max_width: Length::Auto,
            max_height: Length::Auto,
            padding: Edges::default(),
            gap: 0.0,
            flex_grow: 0.0,
            flex_shrink: 0.0,
            overflow: Overflow::Visible,
        }
    }
}
impl LayoutStyle {
    pub fn row() -> Self {
        Self {
            kind: LayoutKind::Row,
            ..Self::default()
        }
    }
    pub fn column() -> Self {
        Self {
            kind: LayoutKind::Column,
            ..Self::default()
        }
    }
    pub fn stack() -> Self {
        Self {
            kind: LayoutKind::Stack,
            ..Self::default()
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayoutNode {
    pub id: u64,
    pub style: LayoutStyle,
    pub children: Vec<LayoutNode>,
}
impl LayoutNode {
    pub fn leaf(id: u64, style: LayoutStyle) -> Self {
        Self {
            id,
            style,
            children: vec![],
        }
    }
    pub fn container(id: u64, style: LayoutStyle, children: Vec<Self>) -> Self {
        Self {
            id,
            style,
            children,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct LayoutRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ScrollMetrics {
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub content_width: f32,
    pub content_height: f32,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayoutSnapshot {
    rects: HashMap<u64, LayoutRect>,
    scroll: HashMap<u64, ScrollMetrics>,
}
impl LayoutSnapshot {
    pub fn get(&self, id: u64) -> Option<&LayoutRect> {
        self.rects.get(&id)
    }
    pub fn scroll_metrics(&self, id: u64) -> Option<&ScrollMetrics> {
        self.scroll.get(&id)
    }
    pub fn len(&self) -> usize {
        self.rects.len()
    }
    pub fn is_empty(&self) -> bool {
        self.rects.is_empty()
    }
    /// Iterate over all `(id, rect)` pairs in the snapshot.
    pub fn iter(&self) -> impl Iterator<Item = (u64, &LayoutRect)> {
        self.rects.iter().map(|(id, rect)| (*id, rect))
    }
}

#[derive(Debug)]
pub enum LayoutError {
    DuplicateId(u64),
    InvalidViewport,
    Engine(String),
}
impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateId(id) => write!(f, "duplicate layout id {id}"),
            Self::InvalidViewport => f.write_str("viewport must be finite and non-negative"),
            Self::Engine(e) => f.write_str(e),
        }
    }
}
impl std::error::Error for LayoutError {}

#[derive(Default)]
pub struct LayoutEngine;
impl LayoutEngine {
    pub fn new() -> Self {
        Self
    }
    pub fn compute(
        &mut self,
        root: &LayoutNode,
        viewport: (f32, f32),
    ) -> Result<LayoutSnapshot, LayoutError> {
        if !viewport.0.is_finite()
            || !viewport.1.is_finite()
            || viewport.0 < 0.0
            || viewport.1 < 0.0
        {
            return Err(LayoutError::InvalidViewport);
        }
        let mut tree: TaffyTree<u64> = TaffyTree::new();
        let mut ids = HashMap::new();
        let root_node = build(&mut tree, root, &mut ids, false)?;
        tree.compute_layout(
            root_node,
            Size {
                width: AvailableSpace::Definite(viewport.0),
                height: AvailableSpace::Definite(viewport.1),
            },
        )
        .map_err(|e| LayoutError::Engine(e.to_string()))?;
        let mut out = LayoutSnapshot::default();
        collect(&tree, root, root_node, (0.0, 0.0), &mut out)?;
        Ok(out)
    }
}

fn build(
    tree: &mut TaffyTree<u64>,
    node: &LayoutNode,
    ids: &mut HashMap<u64, ()>,
    absolute: bool,
) -> Result<taffy::NodeId, LayoutError> {
    if ids.insert(node.id, ()).is_some() {
        return Err(LayoutError::DuplicateId(node.id));
    }
    let mut children = Vec::with_capacity(node.children.len());
    for child in &node.children {
        children.push(build(
            tree,
            child,
            ids,
            node.style.kind == LayoutKind::Stack,
        )?);
    }
    let mut style = to_taffy(&node.style);
    if absolute {
        style.position = Position::Absolute;
    }
    tree.new_with_children(style, &children)
        .map_err(|e| LayoutError::Engine(e.to_string()))
}
fn collect(
    tree: &TaffyTree<u64>,
    node: &LayoutNode,
    tid: taffy::NodeId,
    parent: (f32, f32),
    out: &mut LayoutSnapshot,
) -> Result<(), LayoutError> {
    let l = tree
        .layout(tid)
        .map_err(|e| LayoutError::Engine(e.to_string()))?;
    let x = parent.0 + l.location.x;
    let y = parent.1 + l.location.y;
    out.rects.insert(
        node.id,
        LayoutRect {
            x,
            y,
            width: l.size.width,
            height: l.size.height,
        },
    );
    let children = tree
        .children(tid)
        .map_err(|e| LayoutError::Engine(e.to_string()))?;
    let mut max_x = 0.0f32;
    let mut max_y = 0.0f32;
    for (child, ctid) in node.children.iter().zip(children) {
        collect(tree, child, ctid, (x, y), out)?;
        let cl = tree
            .layout(ctid)
            .map_err(|e| LayoutError::Engine(e.to_string()))?;
        max_x = max_x.max(cl.location.x + cl.size.width);
        max_y = max_y.max(cl.location.y + cl.size.height);
    }
    if node.style.overflow == Overflow::Scroll {
        let (descendant_right, descendant_bottom) = descendant_extent(node, out, x, y);
        out.scroll.insert(
            node.id,
            ScrollMetrics {
                viewport_width: l.size.width,
                viewport_height: l.size.height,
                content_width: max_x.max(descendant_right).max(l.size.width),
                content_height: max_y.max(descendant_bottom).max(l.size.height),
            },
        );
    }
    Ok(())
}
fn descendant_extent(node: &LayoutNode, snapshot: &LayoutSnapshot, x: f32, y: f32) -> (f32, f32) {
    let mut right = 0.0_f32;
    let mut bottom = 0.0_f32;
    for child in &node.children {
        if let Some(rect) = snapshot.get(child.id) {
            right = right.max(rect.x + rect.width - x);
            bottom = bottom.max(rect.y + rect.height - y);
        }
        let nested = descendant_extent(child, snapshot, x, y);
        right = right.max(nested.0);
        bottom = bottom.max(nested.1);
    }
    (right, bottom)
}
fn to_taffy(s: &LayoutStyle) -> Style {
    let mut style = Style {
        display: Display::Flex,
        flex_direction: match s.kind {
            LayoutKind::Row => FlexDirection::Row,
            _ => FlexDirection::Column,
        },
        size: Size {
            width: dim(s.width),
            height: dim(s.height),
        },
        min_size: Size {
            width: dim(s.min_width),
            height: dim(s.min_height),
        },
        max_size: Size {
            width: dim(s.max_width),
            height: dim(s.max_height),
        },
        padding: taffy::Rect {
            left: length(s.padding.left),
            right: length(s.padding.right),
            top: length(s.padding.top),
            bottom: length(s.padding.bottom),
        },
        gap: Size {
            width: length(s.gap),
            height: length(s.gap),
        },
        flex_grow: normalize(s.flex_grow),
        flex_shrink: normalize(s.flex_shrink),
        overflow: taffy::Point {
            x: map_overflow(s.overflow),
            y: map_overflow(s.overflow),
        },
        ..Default::default()
    };
    if s.kind == LayoutKind::Stack {
        style.display = Display::Block;
    }
    style
}
fn map_overflow(value: Overflow) -> TaffyOverflow {
    match value {
        Overflow::Visible => TaffyOverflow::Visible,
        Overflow::Clip => TaffyOverflow::Clip,
        Overflow::Scroll => TaffyOverflow::Scroll,
    }
}
fn dim(v: Length) -> Dimension {
    match v {
        Length::Auto => Dimension::auto(),
        Length::Px(v) => length(normalize(v)),
        Length::Percent(v) => percent(normalize(v)),
    }
}
fn normalize(v: f32) -> f32 {
    if v.is_finite() { v.max(0.0) } else { 0.0 }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixed(id: u64, w: f32, h: f32) -> LayoutNode {
        LayoutNode::leaf(
            id,
            LayoutStyle {
                width: Length::px(w),
                height: Length::px(h),
                ..Default::default()
            },
        )
    }
    #[test]
    fn row_and_column_snapshots() {
        let root = LayoutNode::container(
            1,
            LayoutStyle {
                width: Length::px(100.0),
                height: Length::px(100.0),
                gap: 10.0,
                ..LayoutStyle::column()
            },
            vec![fixed(2, 20.0, 20.0), fixed(3, 20.0, 20.0)],
        );
        let s = LayoutEngine::new().compute(&root, (100.0, 100.0)).unwrap();
        assert_eq!(s.get(2).unwrap().y, 0.0);
        assert_eq!(s.get(3).unwrap().y, 30.0);
    }
    #[test]
    fn row_places_children_horizontally() {
        let root = LayoutNode::container(
            1,
            LayoutStyle {
                width: Length::px(100.0),
                height: Length::px(20.0),
                gap: 5.0,
                ..LayoutStyle::row()
            },
            vec![fixed(2, 20.0, 20.0), fixed(3, 20.0, 20.0)],
        );
        let s = LayoutEngine::new().compute(&root, (100.0, 20.0)).unwrap();
        assert_eq!(s.get(3).unwrap().x, 25.0);
    }
    #[test]
    fn stack_overlaps_children() {
        let root = LayoutNode::container(
            1,
            LayoutStyle {
                width: Length::px(100.0),
                height: Length::px(100.0),
                ..LayoutStyle::stack()
            },
            vec![fixed(2, 20.0, 30.0), fixed(3, 40.0, 50.0)],
        );
        let snapshot = LayoutEngine::new().compute(&root, (100.0, 100.0)).unwrap();
        assert_eq!(
            (snapshot.get(2).unwrap().x, snapshot.get(2).unwrap().y),
            (0.0, 0.0)
        );
        assert_eq!(
            (snapshot.get(3).unwrap().x, snapshot.get(3).unwrap().y),
            (0.0, 0.0)
        );
    }
    #[test]
    fn scroll_reports_overflow() {
        let root = LayoutNode::container(
            1,
            LayoutStyle {
                width: Length::px(50.0),
                height: Length::px(40.0),
                overflow: Overflow::Scroll,
                ..LayoutStyle::column()
            },
            vec![fixed(2, 50.0, 90.0)],
        );
        let s = LayoutEngine::new().compute(&root, (50.0, 40.0)).unwrap();
        let m = s.scroll_metrics(1).unwrap();
        assert_eq!(m.viewport_height, 40.0);
        assert_eq!(m.content_height, 90.0);
    }
    #[test]
    fn duplicate_ids_fail_closed() {
        let root = LayoutNode::container(1, LayoutStyle::row(), vec![fixed(1, 1.0, 1.0)]);
        assert!(matches!(
            LayoutEngine::new().compute(&root, (10.0, 10.0)),
            Err(LayoutError::DuplicateId(1))
        ));
    }
}
