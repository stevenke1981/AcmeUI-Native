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
        let button_size = match b.size {
            MobileButtonSize::Sm => acme_widgets::ButtonSize::Small,
            MobileButtonSize::Md => acme_widgets::ButtonSize::Medium,
            MobileButtonSize::Lg => acme_widgets::ButtonSize::Large,
        };
        let mut btn = crate::button(b.id, b.text)
            .size(button_size)
            .full_width(true)
            .disabled(b.disabled);
        // Mobile controls keep larger touch targets than the desktop button
        // presets while reusing the same semantic visual states.
        btn.style.height = Some(acme_layout::Length::px(b.size.height()));
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
    use acme_core::NodeId;
    use acme_layout::{Length, WidgetLayoutContext};

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
        assert_eq!(b.activate(), None);
    }

    #[test]
    fn mobile_button_builder_defaults() {
        let b = mobile_button::<Msg>("b", "X");
        assert_eq!(b.size, MobileButtonSize::Md);
        assert!(!b.disabled);
        assert!(b.on_press.is_none());
    }

    #[test]
    fn mobile_button_maps_size_and_touch_height() {
        let cases = [
            (MobileButtonSize::Sm, acme_widgets::ButtonSize::Small, 36.0),
            (MobileButtonSize::Md, acme_widgets::ButtonSize::Medium, 44.0),
            (MobileButtonSize::Lg, acme_widgets::ButtonSize::Large, 52.0),
        ];

        for (mobile_size, button_size, height) in cases {
            let node: WidgetNode<Msg> = mobile_button("btn", "Tap")
                .size(mobile_size)
                .on_press(Msg::Pressed)
                .into();
            let WidgetNode::Button(button) = &node else {
                panic!("expected Button");
            };
            assert_eq!(button.size, button_size);
            assert!(button.full_width);
            assert_eq!(button.activate(), Some(&Msg::Pressed));

            let context = WidgetLayoutContext {
                body_font_size: 14.0,
                body_line_height: 20.0,
                label_font_size: 13.0,
                control_height: 36.0,
                scale_factor: 1.0,
            };
            let layout = node.to_layout_with_context(NodeId::new(1), &context);
            assert_eq!(layout.style.height, Length::px(height));
            assert_eq!(layout.style.width, Length::Percent(100.0));
        }
    }
}
