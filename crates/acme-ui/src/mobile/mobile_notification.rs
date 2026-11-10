//! Mobile notification — toast-style card with title, body, and optional action.
//!
//! This is a stateless card. Its host owns visibility and any auto-dismiss
//! deadline; rendering the card does not schedule a timer or emit dismissal.

use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_layout::Length;
use acme_widgets::style::Styled;

/// Builder for a mobile notification card.
pub struct MobileNotificationBuilder<M> {
    pub id: WidgetKey,
    pub title: String,
    pub body: Option<String>,
    pub action_label: Option<String>,
    pub on_action: Option<M>,
    pub on_dismiss: Option<M>,
}

/// Create a mobile notification builder.
pub fn mobile_notification<M: Clone + 'static>(
    title: impl Into<String>,
) -> MobileNotificationBuilder<M> {
    MobileNotificationBuilder {
        id: WidgetKey::from("mobile_notification"),
        title: title.into(),
        body: None,
        action_label: None,
        on_action: None,
        on_dismiss: None,
    }
}

impl<M: Clone + 'static> MobileNotificationBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn body(mut self, text: impl Into<String>) -> Self {
        self.body = Some(text.into());
        self
    }

    pub fn action(mut self, label: impl Into<String>, msg: M) -> Self {
        self.action_label = Some(label.into());
        self.on_action = Some(msg);
        self
    }

    pub fn on_dismiss(mut self, msg: M) -> Self {
        self.on_dismiss = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<MobileNotificationBuilder<M>> for WidgetNode<M> {
    fn from(b: MobileNotificationBuilder<M>) -> Self {
        let id = b.id.as_str();
        let mut content = crate::column::<M>()
            .key(format!("{id}__content"))
            .w(Length::Percent(1.0))
            .gap(4.0)
            .child(
                crate::label_builder(b.title)
                    .wrap(crate::LabelWrap::Character)
                    .build(),
            );
        if let Some(body) = b.body {
            content = content.child(
                crate::label_builder(body)
                    .wrap(crate::LabelWrap::Character)
                    .build(),
            );
        }

        let mut card = crate::card::<M>()
            .key(b.id.clone())
            .w(Length::Percent(1.0))
            .variant(acme_widgets::CardVariant::Elevated)
            .border_radius(12.0)
            .padding(12.0)
            .gap(8.0)
            .child(content.build());

        let has_action = b.action_label.is_some() && b.on_action.is_some();
        let has_footer = has_action || b.on_dismiss.is_some();
        let mut footer = crate::row::<M>().key(format!("{id}__actions")).gap(8.0);
        if let (Some(label), Some(msg)) = (b.action_label, b.on_action) {
            footer = footer.child(
                crate::button(format!("{id}__action"), label)
                    .variant(crate::ButtonVariant::Ghost)
                    .h(Length::px(44.0))
                    .min_h(Length::px(44.0))
                    .on_click(msg),
            );
        }
        if let Some(msg) = b.on_dismiss {
            footer = footer.child(
                crate::button(format!("{id}__dismiss"), "Dismiss")
                    .variant(crate::ButtonVariant::Ghost)
                    .h(Length::px(44.0))
                    .min_h(Length::px(44.0))
                    .on_click(msg),
            );
        }
        if has_footer {
            card = card.child(footer);
        }
        card.build()
    }
}

#[cfg(test)]
#[path = "mobile_notification_tests.rs"]
mod tests;
