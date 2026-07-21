//! The central [`Style`] struct that accumulates all visual and layout properties.
//!
//! A `Style` is built by chaining methods from the [`Styled`](crate::Styled) trait
//! and then applied to a widget during layout / rendering.

use crate::color::ColorToken;
use acme_layout::{Edges, LayoutKind, LayoutStyle, Length, Overflow};

/// Shadow descriptor (offset + blur + color).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowDef {
    /// Horizontal offset in logical pixels.
    pub offset_x: f32,
    /// Vertical offset in logical pixels.
    pub offset_y: f32,
    /// Blur radius in logical pixels.
    pub blur: f32,
    /// Shadow colour (theme‑aware).
    pub color: ColorToken,
}

impl ShadowDef {
    /// Create a new shadow definition.
    pub const fn new(offset_x: f32, offset_y: f32, blur: f32, color: ColorToken) -> Self {
        Self {
            offset_x,
            offset_y,
            blur,
            color,
        }
    }
}

/// Represents the display behaviour of a widget — analogous to CSS `display`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Display {
    /// The default flex behaviour (row / column / stack).
    #[default]
    Flex,
    /// The element is not displayed and takes no space.
    None,
}

/// Font weight — analogous to CSS `font-weight`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FontWeight {
    /// Thin (100).
    Thin,
    /// ExtraLight (200).
    ExtraLight,
    /// Light (300).
    Light,
    /// Normal (400).
    #[default]
    Normal,
    /// Medium (500).
    Medium,
    /// SemiBold (600).
    SemiBold,
    /// Bold (700).
    Bold,
    /// ExtraBold (800).
    ExtraBold,
    /// Black (900).
    Black,
}

/// Text alignment — analogous to CSS `text-align`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextAlign {
    #[default]
    Left,
    Center,
    Right,
}

/// Accumulated visual and layout properties.
///
/// All fields are `Option` so that a default `Style` is empty; only explicitly
/// set properties override the widget's built‑in defaults.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Style {
    // ── Sizing ───────────────────────────────────────────────────
    /// Explicit width override.
    pub width: Option<Length>,
    /// Explicit height override.
    pub height: Option<Length>,
    /// Minimum width.
    pub min_width: Option<Length>,
    /// Minimum height.
    pub min_height: Option<Length>,
    /// Maximum width.
    pub max_width: Option<Length>,
    /// Maximum height.
    pub max_height: Option<Length>,

    // ── Spacing ──────────────────────────────────────────────────
    /// Padding — inner spacing on all four sides.
    pub padding: Option<Edges>,
    /// Margin — outer spacing on all four sides.
    pub margin: Option<Edges>,
    /// Gap between children (for flex containers).
    pub gap: Option<f32>,

    // ── Flex ─────────────────────────────────────────────────────
    /// Flex grow factor.
    pub flex_grow: Option<f32>,
    /// Flex shrink factor.
    pub flex_shrink: Option<f32>,

    // ── Display / Layout kind ────────────────────────────────────
    /// The layout kind (row / column / stack).
    pub display: Option<Display>,
    /// The flex layout kind when display is `Flex`.
    pub layout_kind: Option<LayoutKind>,

    // ── Visual ───────────────────────────────────────────────────
    /// Background colour / fill.
    pub background: Option<ColorToken>,
    /// Opacity (0.0 – 1.0).
    pub opacity: Option<f32>,

    // ── Border ───────────────────────────────────────────────────
    /// Border colour.
    pub border_color: Option<ColorToken>,
    /// Border width in logical pixels.
    pub border_width: Option<f32>,
    /// Corner radius in logical pixels.
    pub border_radius: Option<f32>,

    // ── Shadow ───────────────────────────────────────────────────
    /// Drop shadow.
    pub shadow: Option<ShadowDef>,

    // ── Typography ───────────────────────────────────────────────
    /// Font size in logical pixels.
    pub font_size: Option<f32>,
    /// Font weight.
    pub font_weight: Option<FontWeight>,
    /// Line height (ratio × font_size).
    pub line_height: Option<f32>,
    /// Text colour.
    pub text_color: Option<ColorToken>,
    /// Text alignment.
    pub text_align: Option<TextAlign>,

    // ── Overflow ─────────────────────────────────────────────────
    pub overflow: Option<Overflow>,
}

impl Style {
    /// Create an empty `Style` (equivalent to `Default`).
    pub fn new() -> Self {
        Self::default()
    }

    /// Merge `other` into `self`.  Where `other` has a `Some` value, it
    /// overwrites `self`.
    pub fn merge(&mut self, other: &Self) {
        macro_rules! merge_opt {
            ($field:ident) => {
                if let Some(v) = other.$field {
                    self.$field = Some(v);
                }
            };
        }
        merge_opt!(width);
        merge_opt!(height);
        merge_opt!(min_width);
        merge_opt!(min_height);
        merge_opt!(max_width);
        merge_opt!(max_height);
        merge_opt!(padding);
        merge_opt!(margin);
        merge_opt!(gap);
        merge_opt!(flex_grow);
        merge_opt!(flex_shrink);
        merge_opt!(display);
        merge_opt!(layout_kind);
        merge_opt!(background);
        merge_opt!(opacity);
        merge_opt!(border_color);
        merge_opt!(border_width);
        merge_opt!(border_radius);
        merge_opt!(shadow);
        merge_opt!(font_size);
        merge_opt!(font_weight);
        merge_opt!(line_height);
        merge_opt!(text_color);
        merge_opt!(text_align);
        merge_opt!(overflow);
    }

    /// Apply this style's layout‑relevant fields to a [`LayoutStyle`], taking
    /// `self` by value (consume‑and‑apply).
    ///
    /// Each `Some` field in `self` overrides the corresponding field in `base`.
    pub fn apply_to_layout(self, base: &mut LayoutStyle) {
        macro_rules! apply_opt {
            ($field:ident) => {
                if let Some(v) = self.$field {
                    base.$field = v;
                }
            };
        }
        apply_opt!(width);
        apply_opt!(height);
        apply_opt!(min_width);
        apply_opt!(min_height);
        apply_opt!(max_width);
        apply_opt!(max_height);
        apply_opt!(gap);
        apply_opt!(flex_grow);
        apply_opt!(flex_shrink);
        apply_opt!(overflow);

        if let Some(p) = self.padding {
            base.padding = p;
        }
        if let Some(m) = self.margin {
            base.margin = m;
        }
        if let Some(kind) = self.layout_kind {
            base.kind = kind;
        }
    }

    /// Return the resolved padding, or `Edges::default()`.
    pub fn resolved_padding(&self) -> Edges {
        self.padding.unwrap_or_default()
    }

    /// Return the resolved margin, or `Edges::default()`.
    pub fn resolved_margin(&self) -> Edges {
        self.margin.unwrap_or_default()
    }

    /// Return `true` if this style has no overrides set.
    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

// ── Builder helpers ─────────────────────────────────────────────────

impl Style {
    /// Convenience: create a `Style` with `padding` set.
    pub fn with_padding(mut self, edges: Edges) -> Self {
        self.padding = Some(edges);
        self
    }

    /// Convenience: create a `Style` with uniform padding.
    pub fn with_padding_all(value: f32) -> Self {
        Self {
            padding: Some(Edges::all(value)),
            ..Self::default()
        }
    }

    /// Convenience: create a `Style` with `gap` set.
    pub fn with_gap(value: f32) -> Self {
        Self {
            gap: Some(value),
            ..Self::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_style_is_empty() {
        assert!(Style::new().is_empty());
    }

    #[test]
    fn merge_overwrites_fields() {
        let mut s = Style::new();
        s.width = Some(Length::px(100.0));
        let other = Style {
            height: Some(Length::px(200.0)),
            ..Style::default()
        };
        s.merge(&other);
        assert_eq!(s.width, Some(Length::px(100.0)));
        assert_eq!(s.height, Some(Length::px(200.0)));
    }

    #[test]
    fn apply_to_layout_sets_fields() {
        let s = Style {
            width: Some(Length::px(320.0)),
            padding: Some(Edges::all(16.0)),
            ..Style::default()
        };
        let mut ls = LayoutStyle::default();
        s.apply_to_layout(&mut ls);
        assert_eq!(ls.width, Length::px(320.0));
        assert_eq!(ls.padding, Edges::all(16.0));
    }
}
