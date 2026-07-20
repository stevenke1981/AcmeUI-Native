//! Toast component — a notification card with tone-based styling.

use crate::{Tone, WidgetNode};

/// Builder for a toast notification.
pub struct ToastBuilder<M> {
    pub message: String,
    pub tone: Tone,
    pub duration_ms: u64,
    pub dismissible: bool,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a toast builder.
pub fn toast<M: Clone + 'static>(message: impl Into<String>) -> ToastBuilder<M> {
    ToastBuilder {
        message: message.into(),
        tone: Tone::Neutral,
        duration_ms: 3000,
        dismissible: true,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone> ToastBuilder<M> {
    /// Set the semantic tone (affects icon / border color).
    pub fn tone(mut self, value: Tone) -> Self {
        self.tone = value;
        self
    }

    /// Set the display duration in milliseconds.
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

        let icon_label = crate::label::<M>(icon_char);
        let msg_label = crate::label::<M>(self.message);

        let mut row = crate::row::<M>()
            .child(icon_label)
            .child(msg_label)
            .gap(8.0);

        if self.dismissible {
            row = row.child(crate::button("toast_close", "✕"));
        }

        let inner = row.build();

        crate::card::<M>()
            .variant(crate::CardVariant::Outlined)
            .child(inner)
            .padding(8.0)
            .build()
    }
}

impl<M: Clone + 'static> From<ToastBuilder<M>> for WidgetNode<M> {
    fn from(b: ToastBuilder<M>) -> Self {
        b.build()
    }
}

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
        // Card contains a Row with icon + message + close = 3 children
        assert_eq!(layout.children.len(), 1);
        let row = &layout.children[0];
        assert_eq!(row.children.len(), 3);
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
