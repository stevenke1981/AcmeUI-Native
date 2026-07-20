use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_layout::{Edges, LayoutKind, LayoutStyle};

/// A container holds children and arranges them according to the layout kind.
#[derive(Clone, Debug, PartialEq)]
pub struct Container<M> {
    pub key: Option<WidgetKey>,
    pub children: Vec<WidgetNode<M>>,
    pub gap: f32,
    pub padding: Edges,
}
impl<M> Default for Container<M> {
    fn default() -> Self {
        Self {
            key: None,
            children: vec![],
            gap: 0.0,
            padding: Edges::default(),
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
        LayoutStyle {
            kind,
            gap: self.gap,
            padding: self.padding,
            ..Default::default()
        }
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
            }),
        }
    }
}
impl<M> From<ContainerBuilder<M>> for WidgetNode<M> {
    fn from(value: ContainerBuilder<M>) -> Self {
        value.build()
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
