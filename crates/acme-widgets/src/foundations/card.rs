use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_layout::Edges;
use acme_style::Style;
use acme_style::prelude::*;
use acme_theme::ThemeColor;

/// Card style variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CardVariant {
    #[default]
    Plain,
    Outlined,
    Elevated,
    Interactive,
    Muted,
}

/// A card container widget.
#[derive(Clone, Debug, PartialEq)]
pub struct Card<M> {
    pub key: Option<WidgetKey>,
    pub variant: CardVariant,
    pub children: Vec<WidgetNode<M>>,
    pub gap: f32,
    pub padding: Edges,
    /// Optional explicit background color override.
    /// When `None`, the renderer derives the color from `variant`.
    pub background_color: Option<ThemeColor>,
    /// Optional explicit border radius override.
    /// When `None`, the renderer uses the default card radius.
    pub border_radius: Option<f32>,
    /// Accumulated GPUI‑inspired / Tailwind‑style styling.
    pub style: Style,
}

/// Builder for Card widgets.
pub struct CardBuilder<M> {
    card: Card<M>,
}
impl<M> CardBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.card.key = Some(key.into());
        self
    }
    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.card.children.push(child.into());
        self
    }
    pub fn gap(mut self, value: f32) -> Self {
        self.card.gap = crate::finite(value);
        self
    }
    pub fn padding(mut self, value: f32) -> Self {
        self.card.padding = Edges::all(value);
        self
    }
    pub fn variant(mut self, value: CardVariant) -> Self {
        self.card.variant = value;
        self
    }
    /// Set an explicit background color override.
    pub fn background_color(mut self, color: ThemeColor) -> Self {
        self.card.background_color = Some(color);
        self
    }
    /// Set an explicit border radius override.
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.card.border_radius = Some(crate::finite(radius));
        self
    }
    pub fn build(self) -> WidgetNode<M> {
        WidgetNode::Card(self.card)
    }
}
impl<M> From<CardBuilder<M>> for WidgetNode<M> {
    fn from(value: CardBuilder<M>) -> Self {
        value.build()
    }
}

/// Create a card widget builder.
pub fn card<M>() -> CardBuilder<M> {
    CardBuilder {
        card: Card {
            key: None,
            variant: CardVariant::Plain,
            children: vec![],
            gap: 0.0,
            padding: Edges::default(),
            background_color: None,
            border_radius: None,
            style: Style::new(),
        },
    }
}

impl<M> Styled for CardBuilder<M> {
    fn style(&self) -> &Style {
        &self.card.style
    }
    fn style_mut(&mut self) -> &mut Style {
        &mut self.card.style
    }
}
