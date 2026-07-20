//! HoverCard component — a card overlay triggered by hover.
//!
//! Renders a trigger widget and, when `open` is true, overlays a Card with
//! rich content on top of the trigger using a Stack.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a hover card with a trigger and rich content overlay.
pub struct HoverCardBuilder<M> {
    pub id: WidgetKey,
    pub trigger: Option<WidgetNode<M>>,
    pub content: Option<WidgetNode<M>>,
    pub open: bool,
}

/// Create a hover card builder.
pub fn hover_card<M: Clone + 'static>(id: impl Into<WidgetKey>) -> HoverCardBuilder<M> {
    HoverCardBuilder {
        id: id.into(),
        trigger: None,
        content: None,
        open: false,
    }
}

impl<M: Clone> HoverCardBuilder<M> {
    /// Set the trigger widget (the element that the user hovers over).
    pub fn trigger(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.trigger = Some(child.into());
        self
    }

    /// Set the card content shown when open.
    pub fn content(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.content = Some(child.into());
        self
    }

    /// Open or close the hover card.
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }

    /// Build the widget node tree.
    ///
    /// When closed: renders just the trigger widget.
    /// When open: renders a `Stack` with the trigger and a `Card` overlay.
    pub fn build(self) -> WidgetNode<M> {
        if !self.open {
            // Closed: render only the trigger (or an empty label placeholder)
            return self.trigger.unwrap_or_else(|| crate::label::<M>(""));
        }

        let trigger = self.trigger.unwrap_or_else(|| crate::label::<M>(""));
        let card_content = self.content.unwrap_or_else(|| crate::label::<M>(""));

        let card = crate::card::<M>()
            .variant(crate::CardVariant::Elevated)
            .child(card_content)
            .padding(8.0)
            .build();

        crate::stack::<M>()
            .child(trigger)
            .child(card)
            .build()
    }
}

impl<M: Clone + 'static> From<HoverCardBuilder<M>> for WidgetNode<M> {
    fn from(b: HoverCardBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WidgetNode;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn hover_card_closed_renders_trigger() {
        let node: WidgetNode<Msg> = hover_card("hc1")
            .trigger(crate::label("Hover me"))
            .content(
                crate::card::<Msg>()
                    .child(crate::label("Details here..."))
                    .build(),
            )
            .open(false)
            .into();
        // Closed: should render the trigger (Label)
        assert!(matches!(node, WidgetNode::Label(_)));
    }

    #[test]
    fn hover_card_open_renders_stack() {
        let node: WidgetNode<Msg> = hover_card("hc2")
            .trigger(crate::label("Hover me"))
            .content(
                crate::card::<Msg>()
                    .child(crate::label("Details here..."))
                    .build(),
            )
            .open(true)
            .build();
        // Open: Stack containing trigger + Card
        assert!(matches!(node, WidgetNode::Stack(_)));
    }

    #[test]
    fn hover_card_open_stack_has_two_children() {
        let node: WidgetNode<Msg> = hover_card("hc3")
            .trigger(crate::label("Trigger"))
            .content(
                crate::card::<Msg>()
                    .child(crate::label("Content"))
                    .build(),
            )
            .open(true)
            .build();
        let WidgetNode::Stack(s) = &node else {
            panic!("expected Stack variant");
        };
        assert_eq!(s.children.len(), 2);
    }

    #[test]
    fn hover_card_closed_no_content() {
        let node: WidgetNode<Msg> = hover_card("hc4")
            .trigger(crate::label("Just trigger"))
            .open(false)
            .build();
        assert!(matches!(node, WidgetNode::Label(_)));
    }

    #[test]
    fn hover_card_open_content_is_card() {
        let node: WidgetNode<Msg> = hover_card("hc5")
            .trigger(crate::label("Trigger"))
            .content(
                crate::card::<Msg>()
                    .child(crate::label("Content"))
                    .build(),
            )
            .open(true)
            .build();
        let WidgetNode::Stack(s) = &node else {
            panic!("expected Stack variant");
        };
        assert_eq!(s.children.len(), 2);
        // Second child is the Card overlay
        assert!(matches!(&s.children[1], WidgetNode::Card(_)));
    }

    #[test]
    fn hover_card_default_state_is_closed() {
        let b = hover_card::<Msg>("hc6");
        assert!(!b.open);
    }

    #[test]
    fn hover_card_trigger_defaults_to_none() {
        let b = hover_card::<Msg>("hc7");
        assert!(b.trigger.is_none());
    }

    #[test]
    fn hover_card_from_trait_builds() {
        let node: WidgetNode<Msg> = hover_card("hc8")
            .trigger(crate::label("Test"))
            .open(true)
            .into();
        assert!(matches!(node, WidgetNode::Stack(_)));
    }
}
