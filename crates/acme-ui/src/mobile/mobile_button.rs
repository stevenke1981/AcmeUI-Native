//! Mobile button — full-width tappable button with size variants.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Size preset for mobile buttons.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MobileButtonSize {
    Sm,
    #[default]
    Md,
    Lg,
}

impl MobileButtonSize {
    pub fn height(&self) -> f32 {
        match self {
            Self::Sm => 36.0,
            Self::Md => 44.0,
            Self::Lg => 52.0,
        }
    }
}

/// Builder for a mobile button.
pub struct MobileButtonBuilder<M> {
    pub id: WidgetKey,
    pub text: String,
    pub size: MobileButtonSize,
    pub disabled: bool,
    pub on_press: Option<M>,
}

/// Create a mobile button builder.
pub fn mobile_button<M: Clone + 'static>(
    key: impl Into<WidgetKey>,
    text: impl Into<String>,
) -> MobileButtonBuilder<M> {
    MobileButtonBuilder {
        id: key.into(),
        text: text.into(),
        size: MobileButtonSize::default(),
        disabled: false,
        on_press: None,
    }
}

impl<M: Clone + 'static> MobileButtonBuilder<M> {
    pub fn size(mut self, value: MobileButtonSize) -> Self {
        self.size = value;
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

impl<M: Clone + 'static> From<MobileButtonBuilder<M>> for WidgetNode<M> {
    fn from(b: MobileButtonBuilder<M>) -> Self {
        let mut btn = crate::button(b.id, b.text)
            .size(acme_widgets::ButtonSize::Large)
            .full_width(true)
            .disabled(b.disabled);
        if let Some(msg) = b.on_press {
            btn = btn.primary();
            return btn.on_click(msg);
        }
        btn.primary().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Pressed,
    }

    #[test]
    fn mobile_button_produces_button_node() {
        let node: WidgetNode<Msg> = mobile_button("btn", "Tap me").on_press(Msg::Pressed).into();
        assert!(matches!(node, WidgetNode::Button(_)));
    }

    #[test]
    fn mobile_button_disabled() {
        let node: WidgetNode<Msg> = mobile_button("btn", "Nope")
            .disabled(true)
            .on_press(Msg::Pressed)
            .into();
        let WidgetNode::Button(b) = &node else {
            panic!("expected Button");
        };
        assert!(b.disabled);
    }

    #[test]
    fn mobile_button_size_default() {
        let b = mobile_button::<Msg>("b", "X");
        assert_eq!(b.size, MobileButtonSize::Md);
    }
}
