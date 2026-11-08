//! Mobile card — elevated card with title, optional subtitle and media slot.

use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_widgets::CardVariant;

/// Builder for a mobile card.
pub struct MobileCardBuilder<M> {
    pub id: WidgetKey,
    pub title: String,
    pub subtitle: Option<String>,
    pub media: Option<WidgetNode<M>>,
    pub elevated: bool,
}

/// Create a mobile card builder.
pub fn mobile_card<M: Clone + 'static>(title: impl Into<String>) -> MobileCardBuilder<M> {
    MobileCardBuilder {
        id: WidgetKey::from("mobile_card"),
        title: title.into(),
        subtitle: None,
        media: None,
        elevated: true,
    }
}

impl<M: Clone + 'static> MobileCardBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn subtitle(mut self, text: impl Into<String>) -> Self {
        self.subtitle = Some(text.into());
        self
    }

    pub fn media(mut self, node: WidgetNode<M>) -> Self {
        self.media = Some(node);
        self
    }

    pub fn elevated(mut self, value: bool) -> Self {
        self.elevated = value;
        self
    }
}

impl<M: Clone + 'static> From<MobileCardBuilder<M>> for WidgetNode<M> {
    fn from(b: MobileCardBuilder<M>) -> Self {
        // Media is optional and precedes the text. Keep the text identity
        // independent of its child index when media is inserted or removed.
        let text_key = format!("{}__text", b.id.as_str());
        let variant = if b.elevated {
            CardVariant::Elevated
        } else {
            CardVariant::Outlined
        };
        let mut card = crate::card::<M>()
            .key(b.id)
            .variant(variant)
            .padding(16.0)
            .gap(8.0);

        if let Some(media) = b.media {
            card = card.child(media);
        }

        let mut text_col = crate::column::<M>()
            .key(text_key)
            .gap(4.0)
            .child(crate::label(b.title));
        if let Some(sub) = b.subtitle {
            text_col = text_col.child(crate::label(sub));
        }
        card = card.child(text_col.build());
        card.build()
    }
}

#[cfg(test)]
#[path = "mobile_card_tests.rs"]
mod tests;
