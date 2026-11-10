//! Toast/Snackbar — wrapping message with optional typed actions.
//!
//! Stateless composition: the host owns visibility, bottom placement and expiry.
//! `duration_ms` is scheduling metadata; building a widget starts no timer.

use crate::{Tone, WidgetNode};
use acme_core::WidgetKey;
use acme_layout::Length;
use acme_widgets::style::Styled;

/// Builder for a toast notification.
pub struct ToastBuilder<M> {
    pub id: WidgetKey,
    pub action: Option<(String, M)>,
    pub on_dismiss: Option<M>,
    pub message: String,
    pub tone: Tone,
    pub duration_ms: u64,
    pub dismissible: bool,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a toast builder.
pub fn toast<M: Clone + 'static>(message: impl Into<String>) -> ToastBuilder<M> {
    ToastBuilder {
        id: WidgetKey::from("toast"),
        action: None,
        on_dismiss: None,
        message: message.into(),
        tone: Tone::Neutral,
        duration_ms: 3000,
        dismissible: true,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone> ToastBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    /// Add an independent action (for example Undo).
    pub fn action(mut self, label: impl Into<String>, message: M) -> Self {
        self.action = Some((label.into(), message));
        self
    }

    /// Message emitted by the close button when `dismissible` is true.
    pub fn on_dismiss(mut self, message: M) -> Self {
        self.on_dismiss = Some(message);
        self
    }

    /// Set the semantic tone (affects the icon).
    pub fn tone(mut self, value: Tone) -> Self {
        self.tone = value;
        self
    }

    /// Preferred display duration in milliseconds; the host schedules expiry.
    pub fn duration_ms(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    /// Set whether the toast can be dismissed.
    pub fn dismissible(mut self, value: bool) -> Self {
        self.dismissible = value;
        self
    }

    /// Build the widget node tree.
    ///
    /// Renders a `Card` with an icon label, message label, and optional close button.
    pub fn build(self) -> WidgetNode<M> {
        let icon_char = match self.tone {
            Tone::Neutral => "ℹ",
            Tone::Primary => "✓",
            Tone::Success => "✔",
            Tone::Warning => "⚠",
            Tone::Danger => "✕",
            Tone::Info => "ℹ",
        };

        let id = self.id.as_str();
        let body = crate::row::<M>()
            .key(format!("{id}__content"))
            .w(Length::Percent(1.0))
            .gap(8.0)
            .child(
                crate::label_builder(icon_char)
                    .w(Length::px(20.0))
                    .flex_shrink(0.0)
                    .build(),
            )
            .child(
                crate::label_builder(self.message)
                    .wrap(crate::LabelWrap::Character)
                    .min_w(Length::px(0.0))
                    .flex_1()
                    .flex_shrink(1.0)
                    .build(),
            );
        let mut card = crate::card::<M>()
            .key(self.id.clone())
            .variant(crate::CardVariant::Outlined)
            .w(Length::Percent(1.0))
            .padding(8.0)
            .gap(8.0)
            .child(body);
        if self.action.is_some() || self.dismissible {
            let mut footer = crate::row::<M>()
                .key(format!("{id}__actions"))
                .w(Length::Percent(1.0))
                .gap(8.0);
            if let Some((label, message)) = self.action {
                footer = footer.child(
                    crate::button(format!("{id}__action"), label)
                        .variant(crate::ButtonVariant::Ghost)
                        .h(Length::px(44.0))
                        .min_w(Length::px(0.0))
                        .max_w(Length::Percent(1.0))
                        .flex_shrink(1.0)
                        .on_click(message),
                );
            }
            if self.dismissible {
                let close = crate::button(format!("{id}__dismiss"), "✕")
                    .variant(crate::ButtonVariant::Ghost)
                    .w(Length::px(44.0))
                    .h(Length::px(44.0))
                    .flex_shrink(0.0);
                footer = footer.child(if let Some(message) = self.on_dismiss {
                    close.on_click(message)
                } else {
                    close.into()
                });
            }
            card = card.child(footer);
        }
        card.build()
    }
}

impl<M: Clone + 'static> From<ToastBuilder<M>> for WidgetNode<M> {
    fn from(b: ToastBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
#[path = "toast_tests.rs"]
mod acceptance_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WidgetNode;
    use acme_core::NodeId;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn toast_has_non_zero_layout_rect() {
        let node: WidgetNode<Msg> = toast::<Msg>("File saved")
            .tone(Tone::Success)
            .duration_ms(5000)
            .dismissible(true)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Body and footer: icon/message row, then independent close button.
        assert_eq!(layout.children.len(), 2);
        let row = &layout.children[0];
        assert_eq!(row.children.len(), 2);
        assert_eq!(layout.children[1].children.len(), 1);
    }

    #[test]
    fn toast_non_dismissible() {
        let node: WidgetNode<Msg> = toast::<Msg>("Info").dismissible(false).into();
        let layout = node.to_layout(NodeId::new(1));
        let row = &layout.children[0];
        // Icon + message only = 2 children
        assert_eq!(row.children.len(), 2);
    }

    #[test]
    fn toast_default_duration() {
        let t = toast::<Msg>("Hello");
        assert_eq!(t.duration_ms, 3000);
    }

    #[test]
    fn toast_default_tone() {
        let t = toast::<Msg>("Hi");
        assert_eq!(t.tone, Tone::Neutral);
    }
}
