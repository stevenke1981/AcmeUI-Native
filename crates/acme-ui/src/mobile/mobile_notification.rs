//! Mobile notification — toast-style card with title, body, and optional action.

use crate::WidgetNode;
use acme_core::WidgetKey;

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
        let mut content = crate::column::<M>().gap(4.0).child(crate::label(b.title));
        if let Some(body) = b.body {
            content = content.child(crate::label(body));
        }

        let mut card = crate::card::<M>()
            .key(b.id)
            .variant(acme_widgets::CardVariant::Elevated)
            .padding(12.0)
            .gap(8.0)
            .child(content.build());

        if let (Some(label), Some(msg)) = (b.action_label, b.on_action) {
            let btn = crate::button("notif_action", label).on_click(msg);
            card = card.child(btn);
        }
        if let Some(msg) = b.on_dismiss {
            let close = crate::button("notif_close", "✕").on_click(msg);
            card = card.child(close);
        }
        card.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Action,
        Dismiss,
    }

    #[test]
    fn mobile_notification_produces_card() {
        let node: WidgetNode<Msg> = mobile_notification("Alert").into();
        assert!(matches!(node, WidgetNode::Card(_)));
    }

    #[test]
    fn mobile_notification_with_body() {
        let node: WidgetNode<Msg> = mobile_notification("Title").body("Details").into();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card");
        };
        assert!(!c.children.is_empty());
    }

    #[test]
    fn mobile_notification_with_action_and_dismiss() {
        let node: WidgetNode<Msg> = mobile_notification("N")
            .action("Reply", Msg::Action)
            .on_dismiss(Msg::Dismiss)
            .into();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card");
        };
        // content column + action button + dismiss button
        assert_eq!(c.children.len(), 3);
    }
}
