//! Mobile action — tappable action row with icon, label, and destructive flag.
//!
//! Hosts dispatch the existing Row message; disabled rows expose no message.
//! Generic retained focus and typed action accessibility are not provided by Row.

use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_layout::Length;
use acme_widgets::style::{ColorToken, Styled};

/// Builder for a mobile action row.
pub struct MobileActionBuilder<M> {
    pub id: WidgetKey,
    pub label: String,
    pub icon: Option<crate::IconName>,
    pub destructive: bool,
    pub disabled: bool,
    pub on_press: Option<M>,
}

/// Create a mobile action builder.
pub fn mobile_action<M: Clone + 'static>(label: impl Into<String>) -> MobileActionBuilder<M> {
    MobileActionBuilder {
        id: WidgetKey::from("mobile_action"),
        label: label.into(),
        icon: None,
        destructive: false,
        disabled: false,
        on_press: None,
    }
}

impl<M: Clone + 'static> MobileActionBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn icon(mut self, value: crate::IconName) -> Self {
        self.icon = Some(value);
        self
    }

    pub fn destructive(mut self, value: bool) -> Self {
        self.destructive = value;
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    pub fn on_press(mut self, msg: M) -> Self {
        self.on_press = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<MobileActionBuilder<M>> for WidgetNode<M> {
    fn from(b: MobileActionBuilder<M>) -> Self {
        let background = if b.disabled {
            ColorToken::DisabledBg
        } else if b.destructive {
            ColorToken::DangerSoft
        } else {
            ColorToken::Surface
        };
        let foreground = if b.disabled {
            ColorToken::DisabledText
        } else if b.destructive {
            ColorToken::Danger
        } else {
            ColorToken::Foreground
        };
        let label_key = format!("{}__label", b.id.as_str());
        let mut row = crate::row::<M>()
            .key(b.id)
            .w(Length::Percent(1.0))
            .min_h(Length::px(44.0))
            .bg(background)
            .rounded(8.0)
            .gap(12.0)
            .padding(14.0);
        if let Some(icon) = b.icon {
            row = row.child(
                crate::label_builder(icon.char())
                    .font_size(16.0)
                    .w(Length::px(20.0))
                    .flex_shrink(0.0)
                    .text_color(foreground)
                    .build(),
            );
        }
        // The key keeps the label stable when a leading icon is inserted.
        row = row.child(
            crate::column::<M>()
                .key(label_key)
                .min_w(Length::px(0.0))
                .flex_1()
                .flex_shrink(1.0)
                .child(
                    crate::label_builder(b.label)
                        .wrap(crate::LabelWrap::Character)
                        .text_color(foreground)
                        .build(),
                )
                .build(),
        );
        if !b.disabled {
            if let Some(msg) = b.on_press {
                row = row.on_click(msg);
            }
        }
        row.build()
    }
}

#[cfg(test)]
#[path = "mobile_action_tests.rs"]
mod tests;
