use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_layout::{Edges, LayoutKind, LayoutStyle, Length};
use acme_style::Style;
use acme_style::prelude::*;

/// A container holds children and arranges them according to the layout kind.
///
/// The container can carry an optional click message for hit-testing
/// without affecting layout behaviour.
#[derive(Clone, Debug, PartialEq)]
pub struct Container<M> {
    pub key: Option<WidgetKey>,
    pub children: Vec<WidgetNode<M>>,
    pub gap: f32,
    pub padding: Edges,
    /// Optional explicit width. When set, the container uses this width
    /// instead of relying on content-intrinsic sizing.
    pub width: Option<f32>,
    /// Optional explicit height. When set, the container uses this height
    /// instead of relying on content-intrinsic sizing.
    pub height: Option<f32>,
    /// GPUI‑inspired / Tailwind‑style accumulated styling.
    pub style: Style,
    /// Optional message dispatched when the container region is clicked.
    pub message: Option<M>,
}
impl<M> Default for Container<M> {
    fn default() -> Self {
        Self {
            key: None,
            children: vec![],
            gap: 0.0,
            padding: Edges::default(),
            width: None,
            height: None,
            style: Style::new(),
            message: None,
        }
    }
}

impl<M> Container<M> {
    pub fn new() -> Self {
        Self::default()
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
        self.gap = crate::finite(value);
        self
    }
    pub fn padding(mut self, value: f32) -> Self {
        self.padding = Edges::all(value);
        self
    }
    pub(crate) fn layout(&self, kind: LayoutKind) -> LayoutStyle {
        let mut base = LayoutStyle {
            kind,
            gap: self.gap,
            padding: self.padding,
            width: self.width.map_or(Length::Auto, Length::px),
            height: self.height.map_or(Length::Auto, Length::px),
            ..Default::default()
        };
        // Apply accumulated style overrides on top of explicit fields.
        if !self.style.is_empty() {
            self.style.clone().apply_to_layout(&mut base);
        }
        base
    }
}

/// Builder for container widgets (row, column, stack).
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
    /// Set an explicit width for the container.
    pub fn width(mut self, value: f32) -> Self {
        self.container.width = Some(crate::finite(value));
        self
    }
    /// Set an explicit height for the container.
    pub fn height(mut self, value: f32) -> Self {
        self.container.height = Some(crate::finite(value));
        self
    }
    /// Set both explicit width and height.
    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.container.width = Some(crate::finite(w));
        self.container.height = Some(crate::finite(h));
        self
    }
    /// Attach a click message to the container.
    ///
    /// The message is carried through the widget tree for hit-testing.
    /// This does **not** affect layout behaviour.
    pub fn on_click(mut self, msg: M) -> Self {
        self.container.message = Some(msg);
        self
    }
    pub fn build(self) -> WidgetNode<M> {
        match self.kind {
            LayoutKind::Row => WidgetNode::Row(self.container),
            LayoutKind::Column => WidgetNode::Column(self.container),
            LayoutKind::Stack => WidgetNode::Stack(self.container),
            LayoutKind::Leaf => WidgetNode::Card(crate::Card {
                key: self.container.key,
                children: self.container.children,
                gap: self.container.gap,
                padding: self.container.padding,
                variant: crate::CardVariant::Plain,
                background_color: None,
                border_radius: None,
                style: self.container.style,
            }),
        }
    }
}
impl<M> From<ContainerBuilder<M>> for WidgetNode<M> {
    fn from(value: ContainerBuilder<M>) -> Self {
        value.build()
    }
}

/// Implement the `Styled` trait for `ContainerBuilder` so users can chain
/// tailwind‑style utility methods when building containers.
impl<M> Styled for ContainerBuilder<M> {
    fn style(&self) -> &Style {
        &self.container.style
    }
    fn style_mut(&mut self) -> &mut Style {
        &mut self.container.style
    }
}

/// Create a row container builder.
pub fn row<M>() -> ContainerBuilder<M> {
    ContainerBuilder {
        container: Container::new(),
        kind: LayoutKind::Row,
    }
}

/// Create a column container builder.
pub fn column<M>() -> ContainerBuilder<M> {
    ContainerBuilder {
        container: Container::new(),
        kind: LayoutKind::Column,
    }
}

/// Create a stack container builder.
pub fn stack<M>() -> ContainerBuilder<M> {
    ContainerBuilder {
        container: Container::new(),
        kind: LayoutKind::Stack,
    }
}
