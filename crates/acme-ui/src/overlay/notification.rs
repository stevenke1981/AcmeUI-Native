//! Notification — desktop notification overlay with title, description, and actions.
//! Aligns with Ant Design Notification component.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Notification placement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NotificationPlacement {
    #[default]
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

/// Notification type/tone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NotificationType {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

impl NotificationType {
    pub fn icon_char(&self) -> &'static str {
        match self {
            Self::Info => "ℹ",
            Self::Success => "✓",
            Self::Warning => "⚠",
            Self::Error => "✗",
        }
    }
}

/// Builder for a desktop notification.
pub struct NotificationBuilder<M> {
    pub id: WidgetKey,
    pub title: String,
    pub description: Option<String>,
    pub notification_type: NotificationType,
    pub placement: NotificationPlacement,
    pub duration_secs: Option<u32>,
    pub on_close: Option<M>,
    pub action_label: Option<String>,
    pub on_action: Option<M>,
}

/// Create a notification builder.
pub fn notification<M: Clone + 'static>(title: impl Into<String>) -> NotificationBuilder<M> {
    NotificationBuilder {
        id: WidgetKey::from("notification"),
        title: title.into(),
        description: None,
        notification_type: NotificationType::default(),
        placement: NotificationPlacement::default(),
        duration_secs: Some(5),
        on_close: None,
        action_label: None,
        on_action: None,
    }
}

impl<M: Clone + 'static> NotificationBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }

    pub fn notification_type(mut self, value: NotificationType) -> Self {
        self.notification_type = value;
        self
    }

    pub fn placement(mut self, value: NotificationPlacement) -> Self {
        self.placement = value;
        self
    }

    pub fn duration(mut self, secs: u32) -> Self {
        self.duration_secs = Some(secs);
        self
    }

    pub fn on_close(mut self, msg: M) -> Self {
        self.on_close = Some(msg);
        self
    }

    pub fn action(mut self, label: impl Into<String>, msg: M) -> Self {
        self.action_label = Some(label.into());
        self.on_action = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<NotificationBuilder<M>> for WidgetNode<M> {
    fn from(b: NotificationBuilder<M>) -> Self {
        let icon = crate::label(b.notification_type.icon_char());

        let mut content = crate::column::<M>().gap(4.0).child(crate::label(b.title));
        if let Some(desc) = b.description {
            content = content.child(crate::label(desc));
        }

        let mut card = crate::card::<M>()
            .key(b.id)
            .variant(acme_widgets::CardVariant::Elevated)
            .padding(16.0)
            .gap(12.0);

        let body = crate::row::<M>()
            .gap(12.0)
            .child(icon)
            .child(content.build())
            .build();
        card = card.child(body);

        if let (Some(label), Some(msg)) = (b.action_label, b.on_action) {
            let btn = crate::button("notif_action", label).on_click(msg);
            card = card.child(btn);
        }
        if let Some(msg) = b.on_close {
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
        Close,
        Act,
    }

    #[test]
    fn notification_produces_card() {
        let node: WidgetNode<Msg> = notification("Alert").into();
        assert!(matches!(node, WidgetNode::Card(_)));
    }

    #[test]
    fn notification_with_description() {
        let node: WidgetNode<Msg> = notification("Title")
            .description("Details here")
            .into();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card");
        };
        assert!(!c.children.is_empty());
    }

    #[test]
    fn notification_with_action_and_close() {
        let node: WidgetNode<Msg> = notification("N")
            .action("Undo", Msg::Act)
            .on_close(Msg::Close)
            .into();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card");
        };
        // body + action + close = 3
        assert_eq!(c.children.len(), 3);
    }

    #[test]
    fn notification_type_icons() {
        assert_eq!(NotificationType::Success.icon_char(), "✓");
        assert_eq!(NotificationType::Error.icon_char(), "✗");
    }
}
