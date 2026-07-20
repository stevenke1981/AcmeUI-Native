use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_layout::Edges;

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
        },
    }
}
