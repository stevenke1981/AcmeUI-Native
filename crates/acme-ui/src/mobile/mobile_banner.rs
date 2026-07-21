//! Mobile banner — full-width alert with icon, text, and optional action.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Tone for the banner alert.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MobileBannerTone {
    #[default]
    Info,
    Success,
    Warning,
    Danger,
}

impl MobileBannerTone {
    pub fn icon_char(&self) -> &'static str {
        match self {
            Self::Info => "ℹ",
            Self::Success => "✓",
            Self::Warning => "⚠",
            Self::Danger => "✗",
        }
    }
}

/// Builder for a mobile banner.
pub struct MobileBannerBuilder<M> {
    pub id: WidgetKey,
    pub text: String,
    pub tone: MobileBannerTone,
    pub action_label: Option<String>,
    pub on_action: Option<M>,
    pub on_dismiss: Option<M>,
}

/// Create a mobile banner builder.
pub fn mobile_banner<M: Clone + 'static>(text: impl Into<String>) -> MobileBannerBuilder<M> {
    MobileBannerBuilder {
        id: WidgetKey::from("mobile_banner"),
        text: text.into(),
        tone: MobileBannerTone::default(),
        action_label: None,
        on_action: None,
        on_dismiss: None,
    }
}

impl<M: Clone + 'static> MobileBannerBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn tone(mut self, value: MobileBannerTone) -> Self {
        self.tone = value;
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

impl<M: Clone + 'static> From<MobileBannerBuilder<M>> for WidgetNode<M> {
    fn from(b: MobileBannerBuilder<M>) -> Self {
        let icon = crate::label(b.tone.icon_char());
        let mut row = crate::row::<M>()
            .key(b.id)
            .gap(8.0)
            .padding(12.0)
            .child(icon)
            .child(crate::label(b.text));

        if let (Some(label), Some(msg)) = (b.action_label, b.on_action) {
            let btn = crate::button("banner_action", label).on_click(msg);
            row = row.child(btn);
        }
        if let Some(msg) = b.on_dismiss {
            let close = crate::button("banner_close", "✕").on_click(msg);
            row = row.child(close);
        }
        row.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Act,
        Close,
    }

    #[test]
    fn mobile_banner_produces_row() {
        let node: WidgetNode<Msg> = mobile_banner("Update available").into();
        assert!(matches!(node, WidgetNode::Row(_)));
    }

    #[test]
    fn mobile_banner_has_icon_and_text() {
        let node: WidgetNode<Msg> = mobile_banner("Hello").into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        assert_eq!(r.children.len(), 2);
    }

    #[test]
    fn mobile_banner_with_action_and_dismiss() {
        let node: WidgetNode<Msg> = mobile_banner("Err")
            .tone(MobileBannerTone::Danger)
            .action("Retry", Msg::Act)
            .on_dismiss(Msg::Close)
            .into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        assert_eq!(r.children.len(), 4);
    }
}
