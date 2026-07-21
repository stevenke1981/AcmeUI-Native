//! The [`Styled`] trait — GPUI‑inspired chainable styling API.
//!
//! Any type that implements `Styled` (typically a widget builder or widget data)
//! gains tailwind‑like utility methods:
//!
//! ```ignore
//! use acme_style::prelude::*;
//!
//! row()
//!     .w_full()
//!     .p_4()
//!     .gap_3()
//!     .bg_surface()
//!     .rounded_lg()
//!     .build()
//! ```

use crate::color::ColorToken;
use crate::style::{FontWeight, ShadowDef, Style, TextAlign};
use acme_layout::{Edges, LayoutKind, Length, Overflow};

/// Trait that provides chainable, Tailwind‑inspired style methods.
///
/// Implement `style()` / `style_mut()` on your widget builder, then all
/// methods below become available.
pub trait Styled: Sized {
    /// Borrow the accumulated [`Style`].
    fn style(&self) -> &Style;
    /// Mutably borrow the accumulated [`Style`].
    fn style_mut(&mut self) -> &mut Style;

    // ═══════════════════════════════════════════════════════════════
    // Sizing
    // ═══════════════════════════════════════════════════════════════

    /// Set explicit width.
    fn w(mut self, value: impl Into<Length>) -> Self {
        self.style_mut().width = Some(value.into());
        self
    }
    /// Set explicit height.
    fn h(mut self, value: impl Into<Length>) -> Self {
        self.style_mut().height = Some(value.into());
        self
    }
    /// Set minimum width.
    fn min_w(mut self, value: impl Into<Length>) -> Self {
        self.style_mut().min_width = Some(value.into());
        self
    }
    /// Set minimum height.
    fn min_h(mut self, value: impl Into<Length>) -> Self {
        self.style_mut().min_height = Some(value.into());
        self
    }
    /// Set maximum width.
    fn max_w(mut self, value: impl Into<Length>) -> Self {
        self.style_mut().max_width = Some(value.into());
        self
    }
    /// Set maximum height.
    fn max_h(mut self, value: impl Into<Length>) -> Self {
        self.style_mut().max_height = Some(value.into());
        self
    }

    // ── Tailwind‑style w_ / h_ shortcuts ─────────────────────────

    /// `width: 100%`
    fn w_full(self) -> Self {
        self.w(Length::Percent(100.0))
    }
    /// `width: auto`
    fn w_auto(self) -> Self {
        self.w(Length::Auto)
    }
    /// `height: 100%`
    fn h_full(self) -> Self {
        self.h(Length::Percent(100.0))
    }
    /// `height: auto`
    fn h_auto(self) -> Self {
        self.h(Length::Auto)
    }

    // ═══════════════════════════════════════════════════════════════
    // Padding
    // ═══════════════════════════════════════════════════════════════

    /// Uniform padding on all sides (`value` in logical pixels).
    fn p(mut self, value: f32) -> Self {
        self.style_mut().padding = Some(Edges::all(value.max(0.0)));
        self
    }
    /// Horizontal padding (left + right).
    fn px(mut self, value: f32) -> Self {
        let v = value.max(0.0);
        let e = self.style().padding.unwrap_or_default();
        self.style_mut().padding = Some(Edges {
            left: v,
            right: v,
            ..e
        });
        self
    }
    /// Vertical padding (top + bottom).
    fn py(mut self, value: f32) -> Self {
        let v = value.max(0.0);
        let e = self.style().padding.unwrap_or_default();
        self.style_mut().padding = Some(Edges {
            top: v,
            bottom: v,
            ..e
        });
        self
    }
    /// Padding top.
    fn pt(mut self, value: f32) -> Self {
        let mut e = self.style().padding.unwrap_or_default();
        e.top = value.max(0.0);
        self.style_mut().padding = Some(e);
        self
    }
    /// Padding bottom.
    fn pb(mut self, value: f32) -> Self {
        let mut e = self.style().padding.unwrap_or_default();
        e.bottom = value.max(0.0);
        self.style_mut().padding = Some(e);
        self
    }
    /// Padding left.
    fn pl(mut self, value: f32) -> Self {
        let mut e = self.style().padding.unwrap_or_default();
        e.left = value.max(0.0);
        self.style_mut().padding = Some(e);
        self
    }
    /// Padding right.
    fn pr(mut self, value: f32) -> Self {
        let mut e = self.style().padding.unwrap_or_default();
        e.right = value.max(0.0);
        self.style_mut().padding = Some(e);
        self
    }

    // ═══════════════════════════════════════════════════════════════
    // Margin
    // ═══════════════════════════════════════════════════════════════

    /// Uniform margin on all sides (`value` in logical pixels).
    fn m(mut self, value: f32) -> Self {
        self.style_mut().margin = Some(Edges::all(value.max(0.0)));
        self
    }
    /// Horizontal margin (left + right).
    fn mx(mut self, value: f32) -> Self {
        let v = value.max(0.0);
        let e = self.style().margin.unwrap_or_default();
        self.style_mut().margin = Some(Edges {
            left: v,
            right: v,
            ..e
        });
        self
    }
    /// Vertical margin (top + bottom).
    fn my(mut self, value: f32) -> Self {
        let v = value.max(0.0);
        let e = self.style().margin.unwrap_or_default();
        self.style_mut().margin = Some(Edges {
            top: v,
            bottom: v,
            ..e
        });
        self
    }
    /// Margin top.
    fn mt(mut self, value: f32) -> Self {
        let mut e = self.style().margin.unwrap_or_default();
        e.top = value.max(0.0);
        self.style_mut().margin = Some(e);
        self
    }
    /// Margin bottom.
    fn mb(mut self, value: f32) -> Self {
        let mut e = self.style().margin.unwrap_or_default();
        e.bottom = value.max(0.0);
        self.style_mut().margin = Some(e);
        self
    }
    /// Margin left.
    fn ml(mut self, value: f32) -> Self {
        let mut e = self.style().margin.unwrap_or_default();
        e.left = value.max(0.0);
        self.style_mut().margin = Some(e);
        self
    }
    /// Margin right.
    fn mr(mut self, value: f32) -> Self {
        let mut e = self.style().margin.unwrap_or_default();
        e.right = value.max(0.0);
        self.style_mut().margin = Some(e);
        self
    }

    // ═══════════════════════════════════════════════════════════════
    // Gap
    // ═══════════════════════════════════════════════════════════════

    /// Gap between flex children (logical pixels).
    fn gap(mut self, value: f32) -> Self {
        self.style_mut().gap = Some(value.max(0.0));
        self
    }

    // ═══════════════════════════════════════════════════════════════
    // Flex
    // ═══════════════════════════════════════════════════════════════

    /// Set `flex-grow`.
    fn flex_grow(mut self, value: f32) -> Self {
        self.style_mut().flex_grow = Some(value.max(0.0));
        self
    }
    /// Set `flex-shrink`.
    fn flex_shrink(mut self, value: f32) -> Self {
        self.style_mut().flex_shrink = Some(value.max(0.0));
        self
    }
    /// Shortcut: `flex-grow: 1`.
    fn flex_1(self) -> Self {
        self.flex_grow(1.0)
    }

    // ═══════════════════════════════════════════════════════════════
    // Background
    // ═══════════════════════════════════════════════════════════════

    /// Set the background colour.
    fn bg(mut self, color: impl Into<ColorToken>) -> Self {
        self.style_mut().background = Some(color.into());
        self
    }

    // ── Tailwind‑style bg_ shortcuts ─────────────────────────────

    /// `background: var(--primary)`
    fn bg_primary(self) -> Self {
        self.bg(ColorToken::Primary)
    }
    /// `background: var(--surface)`
    fn bg_surface(self) -> Self {
        self.bg(ColorToken::Surface)
    }
    /// `background: var(--muted)`
    fn bg_muted(self) -> Self {
        self.bg(ColorToken::Muted)
    }
    /// `background: var(--accent)`
    fn bg_accent(self) -> Self {
        self.bg(ColorToken::Accent)
    }
    /// `background: var(--secondary)`
    fn bg_secondary(self) -> Self {
        self.bg(ColorToken::Secondary)
    }
    /// `background: var(--success)`
    fn bg_success(self) -> Self {
        self.bg(ColorToken::Success)
    }
    /// `background: var(--warning)`
    fn bg_warning(self) -> Self {
        self.bg(ColorToken::Warning)
    }
    /// `background: var(--danger)`
    fn bg_danger(self) -> Self {
        self.bg(ColorToken::Danger)
    }

    // ═══════════════════════════════════════════════════════════════
    // Border
    // ═══════════════════════════════════════════════════════════════

    /// Set border width + colour.
    fn border(mut self, width: f32, color: impl Into<ColorToken>) -> Self {
        self.style_mut().border_width = Some(width.max(0.0));
        self.style_mut().border_color = Some(color.into());
        self
    }
    /// Set border colour only.
    fn border_color(mut self, color: impl Into<ColorToken>) -> Self {
        self.style_mut().border_color = Some(color.into());
        self
    }
    /// Set border width only.
    fn border_width(mut self, value: f32) -> Self {
        self.style_mut().border_width = Some(value.max(0.0));
        self
    }

    // ═══════════════════════════════════════════════════════════════
    // Border Radius
    // ═══════════════════════════════════════════════════════════════

    /// Set corner radius (logical pixels).
    fn rounded(mut self, value: f32) -> Self {
        self.style_mut().border_radius = Some(value.max(0.0));
        self
    }
    /// `border-radius: 0`
    fn rounded_none(self) -> Self {
        self.rounded(0.0)
    }
    /// `border-radius: 4px`
    fn rounded_sm(self) -> Self {
        self.rounded(4.0)
    }
    /// `border-radius: 6px`
    fn rounded_md(self) -> Self {
        self.rounded(6.0)
    }
    /// `border-radius: 8px`
    fn rounded_lg(self) -> Self {
        self.rounded(8.0)
    }
    /// `border-radius: 12px`
    fn rounded_xl(self) -> Self {
        self.rounded(12.0)
    }
    /// `border-radius: 9999px` (pill shape)
    fn rounded_full(self) -> Self {
        self.rounded(9999.0)
    }

    // ═══════════════════════════════════════════════════════════════
    // Shadow
    // ═══════════════════════════════════════════════════════════════

    /// Set a custom shadow definition.
    fn shadow(mut self, def: ShadowDef) -> Self {
        self.style_mut().shadow = Some(def);
        self
    }

    // ═══════════════════════════════════════════════════════════════
    // Typography
    // ═══════════════════════════════════════════════════════════════

    /// Set font size (logical pixels).
    fn font_size(mut self, value: f32) -> Self {
        self.style_mut().font_size = Some(value.max(0.0));
        self
    }
    /// Set font weight.
    fn font_weight(mut self, value: FontWeight) -> Self {
        self.style_mut().font_weight = Some(value);
        self
    }
    /// Set line height (as a ratio × font_size, e.g. `1.5`).
    fn line_height(mut self, value: f32) -> Self {
        self.style_mut().line_height = Some(value);
        self
    }
    /// Set text colour.
    fn text_color(mut self, color: impl Into<ColorToken>) -> Self {
        self.style_mut().text_color = Some(color.into());
        self
    }
    /// Set text alignment.
    fn text_align(mut self, align: TextAlign) -> Self {
        self.style_mut().text_align = Some(align);
        self
    }

    // ── Tailwind‑style text_ shortcuts ───────────────────────────

    /// `color: var(--foreground)`
    fn text_foreground(self) -> Self {
        self.text_color(ColorToken::Foreground)
    }
    /// `color: var(--muted-foreground)`
    fn text_muted(self) -> Self {
        self.text_color(ColorToken::MutedForeground)
    }
    /// `color: var(--primary)`
    fn text_primary(self) -> Self {
        self.text_color(ColorToken::Primary)
    }
    /// `color: var(--accent-foreground)`
    fn text_accent(self) -> Self {
        self.text_color(ColorToken::AccentForeground)
    }

    // ═══════════════════════════════════════════════════════════════
    // Opacity
    // ═══════════════════════════════════════════════════════════════

    /// Set opacity (0.0 – 1.0).
    fn opacity(mut self, value: f32) -> Self {
        self.style_mut().opacity = Some(value.clamp(0.0, 1.0));
        self
    }

    // ═══════════════════════════════════════════════════════════════
    // Overflow
    // ═══════════════════════════════════════════════════════════════

    /// Set overflow behaviour.
    fn overflow(mut self, value: Overflow) -> Self {
        self.style_mut().overflow = Some(value);
        self
    }
    /// `overflow: clip`
    fn overflow_clip(self) -> Self {
        self.overflow(Overflow::Clip)
    }
    /// `overflow: scroll`
    fn overflow_scroll(self) -> Self {
        self.overflow(Overflow::Scroll)
    }

    // ═══════════════════════════════════════════════════════════════
    // Display / Layout kind
    // ═══════════════════════════════════════════════════════════════

    /// Set the layout kind (row / column / stack).
    fn layout_kind(mut self, kind: LayoutKind) -> Self {
        self.style_mut().layout_kind = Some(kind);
        self
    }
    /// `flex-direction: row`
    fn flex_row(self) -> Self {
        self.layout_kind(LayoutKind::Row)
    }
    /// `flex-direction: column`
    fn flex_col(self) -> Self {
        self.layout_kind(LayoutKind::Column)
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal struct that implements `Styled` for testing.
    #[derive(Default)]
    struct TestWidget {
        style: Style,
    }
    impl Styled for TestWidget {
        fn style(&self) -> &Style {
            &self.style
        }
        fn style_mut(&mut self) -> &mut Style {
            &mut self.style
        }
    }

    #[test]
    fn chaining_sets_expected_fields() {
        let w = TestWidget::default()
            .w(Length::Px(320.0))
            .h(Length::Px(240.0))
            .p(16.0)
            .bg_primary()
            .rounded_lg()
            .shadow(ShadowDef::new(
                0.0,
                4.0,
                12.0,
                ColorToken::Direct(acme_theme::ThemeColor::rgba(0.0, 0.0, 0.0, 0.1)),
            ))
            .text_foreground()
            .font_size(16.0);

        assert_eq!(w.style.width, Some(Length::Px(320.0)));
        assert_eq!(w.style.height, Some(Length::Px(240.0)));
        assert_eq!(w.style.padding, Some(Edges::all(16.0)));
        assert_eq!(w.style.background, Some(ColorToken::Primary));
        assert_eq!(w.style.border_radius, Some(8.0));
        assert!(w.style.shadow.is_some());
        assert_eq!(w.style.text_color, Some(ColorToken::Foreground));
        assert_eq!(w.style.font_size, Some(16.0));
    }

    #[test]
    fn px_and_py_merge_with_existing_padding() {
        let w = TestWidget::default().p(16.0).px(8.0);
        let p = w.style.padding.unwrap();
        assert_eq!(p.left, 8.0);
        assert_eq!(p.right, 8.0);
        assert_eq!(p.top, 16.0);
        assert_eq!(p.bottom, 16.0);
    }
}
