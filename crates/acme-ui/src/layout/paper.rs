//! Paper — elevated surface container.
//! Aligns with MUI Paper component.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Paper elevation level (0–24).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PaperElevation {
    E0,
    #[default]
    E1,
    E2,
    E4,
    E8,
    E16,
    E24,
}

impl PaperElevation {
    pub fn radius(&self) -> f32 {
        match self {
            Self::E0 => 0.0,
            Self::E1 => 2.0,
            Self::E2 => 4.0,
            Self::E4 => 8.0,
            Self::E8 => 12.0,
            Self::E16 => 16.0,
            Self::E24 => 24.0,
        }
    }
}

/// Builder for a paper surface.
pub struct PaperBuilder<M> {
    pub id: WidgetKey,
    pub elevation: PaperElevation,
    pub outlined: bool,
    pub children: Vec<WidgetNode<M>>,
}

/// Create a paper builder.
pub fn paper<M: Clone + 'static>() -> PaperBuilder<M> {
    PaperBuilder {
        id: WidgetKey::from("paper"),
        elevation: PaperElevation::default(),
        outlined: false,
        children: Vec::new(),
    }
}

impl<M: Clone + 'static> PaperBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn elevation(mut self, value: PaperElevation) -> Self {
        self.elevation = value;
        self
    }

    pub fn outlined(mut self, value: bool) -> Self {
        self.outlined = value;
        self
    }

    pub fn child(mut self, node: WidgetNode<M>) -> Self {
        self.children.push(node);
        self
    }
}

impl<M: Clone + 'static> From<PaperBuilder<M>> for WidgetNode<M> {
    fn from(b: PaperBuilder<M>) -> Self {
        let variant = if b.outlined {
            acme_widgets::CardVariant::Outlined
        } else {
            acme_widgets::CardVariant::Elevated
        };
        let mut card = crate::card::<M>()
            .key(b.id)
            .variant(variant)
            .border_radius(b.elevation.radius())
            .padding(16.0);
        for child in b.children {
            card = card.child(child);
        }
        card.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn paper_produces_card() {
        let node: WidgetNode<Msg> = paper().into();
        assert!(matches!(node, WidgetNode::Card(_)));
    }

    #[test]
    fn paper_with_children() {
        let node: WidgetNode<Msg> = paper()
            .child(crate::label("A"))
            .child(crate::label("B"))
            .into();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card");
        };
        assert_eq!(c.children.len(), 2);
    }

    #[test]
    fn paper_elevation_radius() {
        assert_eq!(PaperElevation::E0.radius(), 0.0);
        assert_eq!(PaperElevation::E24.radius(), 24.0);
    }

    #[test]
    fn paper_outlined_variant() {
        let node: WidgetNode<Msg> = paper().outlined(true).into();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card");
        };
        assert_eq!(c.variant, acme_widgets::CardVariant::Outlined);
    }
}
