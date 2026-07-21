//! Framework-owned facade over Taffy.
#![forbid(unsafe_op_in_unsafe_fn)]

use acme_core::NodeId;
use acme_text::{FontSystem, TextConstraints, TextStyle, TextWrap};
use std::collections::HashMap;
use taffy::prelude::{
    AvailableSpace, Dimension, Display, FlexDirection, Size, Style, TaffyTree, length, percent,
};
use taffy::style::{Overflow as TaffyOverflow, Position};

/// Controls how text wraps when measured during layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextWrapMode {
    None,
    Word,
    Character,
}

/// Specifies the text content and style for a node that should be intrinsically
/// measured during Taffy layout via `compute_layout_with_measure`.
#[derive(Clone, Debug, PartialEq)]
pub struct TextMeasureSpec {
    pub text: String,
    pub font_size: f32,
    pub line_height: f32,
    pub wrap: TextWrapMode,
    pub max_lines: Option<usize>,
}

/// Typography and sizing context provided by the theme / application so that
/// widget‑to‑layout conversion can produce correct [`TextMeasureSpec`] values
/// without hard‑coding theme constants inside the widget library.
#[derive(Clone, Copy, Debug)]
pub struct WidgetLayoutContext {
    pub body_font_size: f32,
    pub body_line_height: f32,
    pub label_font_size: f32,
    pub control_height: f32,
    pub scale_factor: f32,
}

/// The per-node context stored inside TaffyTree for intrinsic measurement.
type NodeContext = Option<TextMeasureSpec>;

fn map_wrap(mode: TextWrapMode) -> TextWrap {
    match mode {
        TextWrapMode::None => TextWrap::None,
        TextWrapMode::Word => TextWrap::Word,
        TextWrapMode::Character => TextWrap::Glyph,
    }
}

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
    pub margin: Edges,
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
            margin: Edges::default(),
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
    pub id: NodeId,
    pub style: LayoutStyle,
    pub measure: Option<TextMeasureSpec>,
    pub children: Vec<LayoutNode>,
}
impl LayoutNode {
    pub fn leaf(id: NodeId, style: LayoutStyle) -> Self {
        Self {
            id,
            style,
            measure: None,
            children: vec![],
        }
    }
    pub fn text_leaf(id: NodeId, style: LayoutStyle, spec: TextMeasureSpec) -> Self {
        Self {
            id,
            style,
            measure: Some(spec),
            children: vec![],
        }
    }
    pub fn container(id: NodeId, style: LayoutStyle, children: Vec<Self>) -> Self {
        Self {
            id,
            style,
            measure: None,
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
    rects: HashMap<NodeId, LayoutRect>,
    scroll: HashMap<NodeId, ScrollMetrics>,
}
impl LayoutSnapshot {
    pub fn get(&self, id: NodeId) -> Option<&LayoutRect> {
        self.rects.get(&id)
    }
    pub fn scroll_metrics(&self, id: NodeId) -> Option<&ScrollMetrics> {
        self.scroll.get(&id)
    }
    pub fn len(&self) -> usize {
        self.rects.len()
    }
    pub fn is_empty(&self) -> bool {
        self.rects.is_empty()
    }
    /// Iterate over all `(id, rect)` pairs in the snapshot.
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, &LayoutRect)> {
        self.rects.iter().map(|(id, rect)| (*id, rect))
    }
}

#[derive(Debug)]
pub enum LayoutError {
    DuplicateId(NodeId),
    InvalidViewport,
    Engine(String),
}
impl std::fmt::Display for LayoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateId(id) => write!(f, "duplicate layout id {}", id.get()),
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
        let mut tree: TaffyTree<NodeContext> = TaffyTree::new();
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

    /// Run layout with intrinsic text measurement.
    ///
    /// Nodes that carry a [`TextMeasureSpec`] (created via [`LayoutNode::text_leaf`])
    /// are measured by actually shaping the text with `fonts` inside Taffy's measure
    /// function, so wrapping and font metrics are accounted for during layout.
    pub fn compute_with_text(
        &mut self,
        root: &LayoutNode,
        viewport: (f32, f32),
        fonts: &mut FontSystem,
        scale_factor: f32,
    ) -> Result<LayoutSnapshot, LayoutError> {
        if !viewport.0.is_finite()
            || !viewport.1.is_finite()
            || viewport.0 < 0.0
            || viewport.1 < 0.0
        {
            return Err(LayoutError::InvalidViewport);
        }
        let mut tree: TaffyTree<NodeContext> = TaffyTree::new();
        let mut ids = HashMap::new();
        let root_node = build(&mut tree, root, &mut ids, false)?;

        tree.compute_layout_with_measure(
            root_node,
            Size {
                width: AvailableSpace::Definite(viewport.0),
                height: AvailableSpace::Definite(viewport.1),
            },
            |known: Size<Option<f32>>,
             available: Size<AvailableSpace>,
             _node_id: taffy::NodeId,
             ctx: Option<&mut NodeContext>,
             _style: &Style| {
                // Only provide intrinsic size for text-leaf nodes.
                let Some(Some(spec)) = ctx else {
                    return Size::ZERO;
                };
                // Derive the maximum width from available / known dimensions.
                let max_width = match available.width {
                    AvailableSpace::Definite(w) => Some(w),
                    _ => known.width,
                };
                let text_style = TextStyle {
                    font_size: spec.font_size,
                    line_height: spec.line_height,
                    ..TextStyle::default()
                };
                let constraints = TextConstraints {
                    max_width,
                    wrap: map_wrap(spec.wrap),
                };
                let shaped = fonts.shape(&spec.text, &text_style, constraints, scale_factor);
                Size {
                    width: known.width.unwrap_or(shaped.width),
                    height: known.height.unwrap_or(shaped.height.max(spec.line_height)),
                }
            },
        )
        .map_err(|e| LayoutError::Engine(e.to_string()))?;

        let mut out = LayoutSnapshot::default();
        collect(&tree, root, root_node, (0.0, 0.0), &mut out)?;
        Ok(out)
    }
}

fn build(
    tree: &mut TaffyTree<NodeContext>,
    node: &LayoutNode,
    ids: &mut HashMap<NodeId, ()>,
    absolute: bool,
) -> Result<taffy::NodeId, LayoutError> {
    if ids.insert(node.id, ()).is_some() {
        return Err(LayoutError::DuplicateId(node.id));
    }
    // Text leaf nodes: store the measure spec as context so
    // `compute_with_text` can intrinsically size them.
    if node.children.is_empty()
        && let Some(spec) = &node.measure
    {
        let mut style = to_taffy(&node.style);
        if absolute {
            style.position = Position::Absolute;
        }
        return tree
            .new_leaf_with_context(style, Some(spec.clone()))
            .map_err(|e| LayoutError::Engine(e.to_string()));
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
    tree: &TaffyTree<NodeContext>,
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
        margin: taffy::Rect {
            left: length(s.margin.left),
            right: length(s.margin.right),
            top: length(s.margin.top),
            bottom: length(s.margin.bottom),
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

// ---------------------------------------------------------------------------
// Text shaping utilities (intrinsic text layout — P0-02)
// ---------------------------------------------------------------------------

/// A positioned glyph with bounding-box info.
///
/// This is a cache-friendly, simplified representation of a shaped glyph.
/// Coordinates are in logical pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct PositionedGlyph {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// Cached shaped text that survives across frames.
///
/// Created by [`measure_text`] and intended to be stored by callers (widgets,
/// text input) so that shaping only runs when the text or style changes.
#[derive(Clone, Debug)]
pub struct ShapedText {
    pub glyphs: Vec<PositionedGlyph>,
    pub width: f32,
    pub height: f32,
    pub text: String,
    pub font_size: f32,
}

/// Measure (shape) text and return a [`ShapedText`] that can be cached.
///
/// Wraps [`FontSystem::shape`] into a reusable, cache-friendly form.
/// The caller is responsible for storing the result and only calling this
/// again when text or style changes.
pub fn measure_text(
    text: &str,
    fonts: &mut acme_text::FontSystem,
    style: &acme_text::TextStyle,
    scale: f32,
    max_width: Option<f32>,
) -> ShapedText {
    use acme_text::TextConstraints;
    let constraints = TextConstraints {
        max_width,
        wrap: acme_text::TextWrap::None,
    };
    let layout = fonts.shape(text, style, constraints, scale);
    let line_h = style.line_height;
    let glyphs: Vec<PositionedGlyph> = layout
        .glyphs
        .iter()
        .map(|g| PositionedGlyph {
            id: g.byte_range.start as u32,
            x: g.x,
            y: g.y,
            w: g.advance,
            h: line_h,
        })
        .collect();
    ShapedText {
        glyphs,
        width: layout.width,
        height: layout.height,
        text: text.to_string(),
        font_size: style.font_size,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixed(id: u64, w: f32, h: f32) -> LayoutNode {
        LayoutNode::leaf(
            NodeId::new(id),
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
            NodeId::new(1),
            LayoutStyle {
                width: Length::px(100.0),
                height: Length::px(100.0),
                gap: 10.0,
                ..LayoutStyle::column()
            },
            vec![fixed(2, 20.0, 20.0), fixed(3, 20.0, 20.0)],
        );
        let s = LayoutEngine::new().compute(&root, (100.0, 100.0)).unwrap();
        assert_eq!(s.get(NodeId::new(2)).unwrap().y, 0.0);
        assert_eq!(s.get(NodeId::new(3)).unwrap().y, 30.0);
    }
    #[test]
    fn row_places_children_horizontally() {
        let root = LayoutNode::container(
            NodeId::new(1),
            LayoutStyle {
                width: Length::px(100.0),
                height: Length::px(20.0),
                gap: 5.0,
                ..LayoutStyle::row()
            },
            vec![fixed(2, 20.0, 20.0), fixed(3, 20.0, 20.0)],
        );
        let s = LayoutEngine::new().compute(&root, (100.0, 20.0)).unwrap();
        assert_eq!(s.get(NodeId::new(3)).unwrap().x, 25.0);
    }
    #[test]
    fn stack_overlaps_children() {
        let root = LayoutNode::container(
            NodeId::new(1),
            LayoutStyle {
                width: Length::px(100.0),
                height: Length::px(100.0),
                ..LayoutStyle::stack()
            },
            vec![fixed(2, 20.0, 30.0), fixed(3, 40.0, 50.0)],
        );
        let snapshot = LayoutEngine::new().compute(&root, (100.0, 100.0)).unwrap();
        assert_eq!(
            (
                snapshot.get(NodeId::new(2)).unwrap().x,
                snapshot.get(NodeId::new(2)).unwrap().y
            ),
            (0.0, 0.0)
        );
        assert_eq!(
            (
                snapshot.get(NodeId::new(3)).unwrap().x,
                snapshot.get(NodeId::new(3)).unwrap().y
            ),
            (0.0, 0.0)
        );
    }
    #[test]
    fn scroll_reports_overflow() {
        let root = LayoutNode::container(
            NodeId::new(1),
            LayoutStyle {
                width: Length::px(50.0),
                height: Length::px(40.0),
                overflow: Overflow::Scroll,
                ..LayoutStyle::column()
            },
            vec![fixed(2, 50.0, 90.0)],
        );
        let s = LayoutEngine::new().compute(&root, (50.0, 40.0)).unwrap();
        let m = s.scroll_metrics(NodeId::new(1)).unwrap();
        assert_eq!(m.viewport_height, 40.0);
        assert_eq!(m.content_height, 90.0);
    }
    #[test]
    fn duplicate_ids_fail_closed() {
        let root =
            LayoutNode::container(NodeId::new(1), LayoutStyle::row(), vec![fixed(1, 1.0, 1.0)]);
        assert!(matches!(
            LayoutEngine::new().compute(&root, (10.0, 10.0)),
            Err(LayoutError::DuplicateId(id)) if id.get() == 1
        ));
    }

    #[test]
    fn label_has_non_zero_intrinsic_height() {
        let mut fonts = FontSystem::new();
        let style = TextStyle {
            font_size: 24.0,
            line_height: 28.0,
            ..TextStyle::default()
        };
        let result = measure_text("Typography", &mut fonts, &style, 1.0, None);
        assert!(
            result.height >= 24.0,
            "expected height >= 24, got {}",
            result.height
        );
    }

    #[test]
    fn larger_font_has_larger_intrinsic_height() {
        let mut fonts = FontSystem::new();
        let small = TextStyle {
            font_size: 12.0,
            line_height: 16.0,
            ..TextStyle::default()
        };
        let large = TextStyle {
            font_size: 24.0,
            line_height: 28.0,
            ..TextStyle::default()
        };
        let small_result = measure_text("Text", &mut fonts, &small, 1.0, None);
        let large_result = measure_text("Text", &mut fonts, &large, 1.0, None);
        assert!(
            large_result.height > small_result.height,
            "expected 24px height ({}) > 12px height ({})",
            large_result.height,
            small_result.height
        );
    }

    #[test]
    fn narrow_cjk_text_wraps_to_more_lines() {
        let mut fonts = FontSystem::new();
        // 22 CJK characters — wraps at 240 px but fits on a single line at 600 px
        let cjk = "繁體中文內容需要換行測試文字排版與行數計算";
        let style = TextStyle {
            font_size: 16.0,
            line_height: 20.0,
            ..TextStyle::default()
        };
        // Note: measure_text hard-codes TextWrap::None, so we use fonts.shape()
        // directly (the same approach acme-text tests use) to exercise wrapping.
        let narrow = fonts.shape(
            cjk,
            &style,
            TextConstraints {
                max_width: Some(240.0),
                wrap: TextWrap::Word,
            },
            1.0,
        );
        let wide = fonts.shape(
            cjk,
            &style,
            TextConstraints {
                max_width: Some(600.0),
                wrap: TextWrap::Word,
            },
            1.0,
        );
        assert!(
            narrow.height > wide.height,
            "expected narrow height ({}) > wide height ({})",
            narrow.height,
            wide.height,
        );
    }
}
