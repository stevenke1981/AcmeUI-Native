use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_layout::{LayoutKind, LayoutStyle, Length, Overflow};
use acme_style::Style;
use acme_style::prelude::*;

/// A scrollable viewport.
///
/// The viewport can shrink in a constrained parent when it has flex growth,
/// a percentage height, or the default automatic height. These cases use
/// `flex_shrink = 1` and `min_height = 0` unless overridden by the caller.
#[derive(Clone, Debug, PartialEq)]
pub struct ScrollView<M> {
    pub key: WidgetKey,
    pub children: Vec<WidgetNode<M>>,
    pub viewport_height: Length,
    /// Accumulated GPUI‑inspired / Tailwind‑style styling.
    pub style: Style,
}
impl<M> ScrollView<M> {
    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.children.push(child.into());
        self
    }
    /// Set the preferred viewport height in logical pixels.
    ///
    /// The viewport keeps this height by default, even in a shorter column.
    /// Use `.flex_1()` or set `.flex_shrink(...)` to allow it to shrink.
    pub fn viewport_height(mut self, value: f32) -> Self {
        self.viewport_height = Length::px(value);
        self
    }
    pub fn build(self) -> WidgetNode<M> {
        WidgetNode::ScrollView(self)
    }
    pub(crate) fn layout(&self) -> LayoutStyle {
        let mut layout = LayoutStyle {
            kind: LayoutKind::Column,
            height: self.viewport_height,
            overflow: Overflow::Scroll,
            ..Default::default()
        };
        let height = self.style.height.unwrap_or(self.viewport_height);
        if self.style.flex_grow.unwrap_or_default() > 0.0
            || matches!(height, Length::Percent(_))
            || (height == Length::Auto && self.viewport_height == Length::Auto)
        {
            layout.flex_shrink = 1.0;
            layout.min_height = Length::px(0.0);
        }
        self.style.clone().apply_to_layout(&mut layout);
        layout
    }
}

/// Create a scroll view builder.
pub fn scroll_view<M>(key: impl Into<WidgetKey>) -> ScrollView<M> {
    ScrollView {
        key: key.into(),
        children: vec![],
        viewport_height: Length::Auto,
        style: Style::new(),
    }
}

impl<M> Styled for ScrollView<M> {
    fn style(&self) -> &Style {
        &self.style
    }
    fn style_mut(&mut self) -> &mut Style {
        &mut self.style
    }
}
