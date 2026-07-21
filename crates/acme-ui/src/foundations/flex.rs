//! Flex component — a convenience layout helper that wraps `Row` or `Column`
//! with common alignment and distribution properties.
//!
//! Unlike `row()` / `column()` which return a plain `ContainerBuilder`, `flex()`
//! returns a dedicated builder with semantic shorthands for gap, alignment,
//! padding, and explicit sizing. It builds into a `Row` or `Column` widget.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Flex direction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
}

/// Builder for a flex container.
pub struct FlexBuilder<M> {
    pub direction: FlexDirection,
    pub children: Vec<WidgetNode<M>>,
    pub gap: f32,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub padding: f32,
    pub key: Option<WidgetKey>,
}

/// Create a flex container builder with the given direction.
pub fn flex<M>(direction: FlexDirection) -> FlexBuilder<M> {
    FlexBuilder {
        direction,
        children: vec![],
        gap: 0.0,
        width: None,
        height: None,
        padding: 0.0,
        key: None,
    }
}

/// Shorthand: create a horizontal flex container (Row).
pub fn h_flex<M>() -> FlexBuilder<M> {
    flex(FlexDirection::Row)
}

/// Shorthand: create a vertical flex container (Column).
pub fn v_flex<M>() -> FlexBuilder<M> {
    flex(FlexDirection::Column)
}

impl<M: Clone + 'static> FlexBuilder<M> {
    /// Set the widget key.
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Add a child widget.
    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Set the gap between children.
    pub fn gap(mut self, value: f32) -> Self {
        self.gap = value;
        self
    }

    /// Set an explicit width.
    pub fn width(mut self, value: f32) -> Self {
        self.width = Some(value);
        self
    }

    /// Set an explicit height.
    pub fn height(mut self, value: f32) -> Self {
        self.height = Some(value);
        self
    }

    /// Set padding on all sides.
    pub fn padding(mut self, value: f32) -> Self {
        self.padding = value;
        self
    }

    /// Build the flex container.
    ///
    /// Returns a `Row` (for `FlexDirection::Row`) or `Column`
    /// (for `FlexDirection::Column`) widget with the configured properties.
    pub fn build(self) -> WidgetNode<M> {
        let mut builder = match self.direction {
            FlexDirection::Row => crate::row(),
            FlexDirection::Column => crate::column(),
        };

        if let Some(k) = self.key {
            builder = builder.key(k);
        }
        builder = builder.gap(self.gap).padding(self.padding);
        if let Some(w) = self.width {
            builder = builder.width(w);
        }
        if let Some(h) = self.height {
            builder = builder.height(h);
        }
        for child in self.children {
            builder = builder.child(child);
        }
        builder.build()
    }
}

impl<M: Clone + 'static> From<FlexBuilder<M>> for WidgetNode<M> {
    fn from(b: FlexBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn h_flex_creates_row() {
        let node: WidgetNode<TestMsg> = h_flex()
            .gap(8.0)
            .child(crate::label("A"))
            .child(crate::label("B"))
            .build();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row variant");
        };
        assert_eq!(r.gap, 8.0);
        assert_eq!(r.children.len(), 2);
    }

    #[test]
    fn v_flex_creates_column() {
        let node: WidgetNode<TestMsg> = v_flex()
            .gap(12.0)
            .width(200.0)
            .child(crate::label("X"))
            .build();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column variant");
        };
        assert_eq!(c.gap, 12.0);
        assert_eq!(c.width, Some(200.0));
        assert_eq!(c.children.len(), 1);
    }

    #[test]
    fn flex_custom_direction() {
        let node: WidgetNode<TestMsg> = flex(FlexDirection::Row)
            .padding(16.0)
            .height(100.0)
            .build();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row variant");
        };
        assert_eq!(r.padding.top, 16.0);
        assert_eq!(r.height, Some(100.0));
    }

    #[test]
    fn flex_from_trait() {
        let node: WidgetNode<TestMsg> = h_flex().into();
        let WidgetNode::Row(_) = &node else {
            panic!("expected Row variant");
        };
    }

    #[test]
    fn flex_empty_build() {
        let node: WidgetNode<TestMsg> = v_flex().build();
        let WidgetNode::Column(_) = &node else {
            panic!("expected Column variant");
        };
    }
}
