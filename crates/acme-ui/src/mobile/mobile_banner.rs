//! Mobile banner — full-width alert with icon, text, and optional action.
//!
//! Text wraps at character boundaries; tone surfaces resolve from the live theme.

use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_layout::Length;
use acme_widgets::style::{ColorToken, Styled};

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
        let id = b.id.as_str();
        let background = match b.tone {
            MobileBannerTone::Info => ColorToken::InfoSoft,
            MobileBannerTone::Success => ColorToken::SuccessSoft,
            MobileBannerTone::Warning => ColorToken::WarningSoft,
            MobileBannerTone::Danger => ColorToken::DangerSoft,
        };
        let icon = crate::label_builder(b.tone.icon_char())
            .w(Length::px(20.0))
            .flex_shrink(0.0)
            .build();
        let mut row = crate::row::<M>()
            .key(b.id.clone())
            .w(Length::Percent(1.0))
            .bg(background)
            .rounded(8.0)
            .gap(8.0)
            .padding(12.0)
            .child(icon)
            .child(
                crate::label_builder(b.text)
                    .wrap(crate::LabelWrap::Character)
                    .min_w(Length::px(0.0))
                    .flex_1()
                    .flex_shrink(1.0)
                    .build(),
            );

        if let (Some(label), Some(msg)) = (b.action_label, b.on_action) {
            let btn = crate::button(format!("{id}__action"), label)
                .variant(crate::ButtonVariant::Ghost)
                .h(Length::px(44.0))
                .min_h(Length::px(44.0))
                .flex_shrink(0.0)
                .on_click(msg);
            row = row.child(btn);
        }
        if let Some(msg) = b.on_dismiss {
            let close = crate::button(format!("{id}__dismiss"), "✕")
                .variant(crate::ButtonVariant::Ghost)
                .w(Length::px(44.0))
                .h(Length::px(44.0))
                .min_h(Length::px(44.0))
                .flex_shrink(0.0)
                .on_click(msg);
            row = row.child(close);
        }
        row.build()
    }
}

#[cfg(test)]
#[path = "mobile_banner_tests.rs"]
mod tests;
