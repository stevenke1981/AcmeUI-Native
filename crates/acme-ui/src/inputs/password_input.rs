//! PasswordInput component.
//!
//! Renders as a Row with a TextInput (showing either the actual value or
//! a "••••••" mask) and a toggle-visibility button ("👁").

use acme_core::WidgetKey;
use acme_widgets::*;

/// Builder for a PasswordInput component.
pub struct PasswordInputBuilder<M> {
    pub id: WidgetKey,
    pub placeholder: String,
    pub value: String,
    pub visible: bool,
    pub disabled: bool,
    pub size: crate::ControlSize,
    pub on_change: Option<M>,
    pub on_toggle_visibility: Option<M>,
}

/// Create a new PasswordInput builder.
pub fn password_input<M: Clone + 'static>(id: impl Into<WidgetKey>) -> PasswordInputBuilder<M> {
    PasswordInputBuilder {
        id: id.into(),
        placeholder: String::new(),
        value: String::new(),
        visible: false,
        disabled: false,
        size: crate::ControlSize::Md,
        on_change: None,
        on_toggle_visibility: None,
    }
}

impl<M: Clone + 'static> PasswordInputBuilder<M> {
    /// Set the placeholder text.
    pub fn placeholder(mut self, value: impl Into<String>) -> Self {
        self.placeholder = value.into();
        self
    }

    /// Set the current password value.
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }

    /// Show or hide the plaintext (default `false`, i.e. masked).
    pub fn visible(mut self, value: bool) -> Self {
        self.visible = value;
        self
    }

    /// Set whether the input is disabled.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    /// Set the control size (default `Md`).
    pub fn size(mut self, value: crate::ControlSize) -> Self {
        self.size = value;
        self
    }

    /// Set the message dispatched when the text changes.
    pub fn on_change(mut self, msg: M) -> Self {
        self.on_change = Some(msg);
        self
    }

    /// Set the message dispatched when the visibility is toggled.
    pub fn on_toggle_visibility(mut self, msg: M) -> Self {
        self.on_toggle_visibility = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<PasswordInputBuilder<M>> for WidgetNode<M> {
    fn from(b: PasswordInputBuilder<M>) -> Self {
        let input_key = format!("{}_input", b.id.as_str());
        let toggle_key = format!("{}_toggle", b.id.as_str());

        // When visible, show the actual value; otherwise show a mask.
        let display_value = if b.visible {
            b.value.clone()
        } else {
            "\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}".to_string()
        };

        let mut input = text_input::<M>(input_key.as_str())
            .value(display_value.as_str())
            .placeholder(b.placeholder.as_str())
            .disabled(b.disabled);

        // Mark as password for rendering-layer hint when not visible.
        if !b.visible && b.value.is_empty() {
            input = input.password(true);
        }

        row::<M>()
            .key(b.id)
            .gap(4.0)
            .child(input)
            .child(button::<M>(toggle_key.as_str(), "\u{1f441}"))
            .build()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;
    use acme_layout::{LayoutKind, Length};

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {
        Changed,
        Toggled,
    }

    #[test]
    fn password_input_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> =
            password_input("pw1").value("secret").placeholder("Password").into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Row);
        assert_eq!(layout.children.len(), 2);
        // First child = TextInput with non-zero height
        let input = &layout.children[0];
        assert!(input.children.is_empty());
        assert_ne!(input.style.min_height, Length::px(0.0));
        // Second child = Button with non-zero height
        let btn = &layout.children[1];
        assert_ne!(btn.style.height, Length::px(0.0));
    }

    #[test]
    fn password_input_builder_defaults() {
        let p = password_input::<TestMsg>("p");
        assert!(p.placeholder.is_empty());
        assert!(p.value.is_empty());
        assert!(!p.visible);
        assert!(!p.disabled);
        assert!(p.on_change.is_none());
        assert!(p.on_toggle_visibility.is_none());
    }

    #[test]
    fn password_input_shows_mask_when_not_visible() {
        let node: WidgetNode<TestMsg> =
            password_input("pw").value("my-password").into();
        let WidgetNode::Row(container) = &node else {
            panic!("expected Row");
        };
        assert_eq!(container.children.len(), 2);
        let WidgetNode::TextInput(input) = &container.children[0] else {
            panic!("expected TextInput");
        };
        // Value should be masked
        assert_eq!(input.value, "\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}\u{2022}");
        // Second child is the toggle button
        let WidgetNode::Button(btn) = &container.children[1] else {
            panic!("expected Button");
        };
        assert_eq!(btn.label, "\u{1f441}");
    }

    #[test]
    fn password_input_shows_value_when_visible() {
        let node: WidgetNode<TestMsg> =
            password_input("pw").value("hello").visible(true).into();
        let WidgetNode::Row(container) = &node else {
            panic!("expected Row");
        };
        let WidgetNode::TextInput(input) = &container.children[0] else {
            panic!("expected TextInput");
        };
        assert_eq!(input.value, "hello");
    }

    #[test]
    fn password_input_builder_methods() {
        let p = password_input::<TestMsg>("p")
            .placeholder("Enter password")
            .value("s3cr3t")
            .visible(true)
            .disabled(true)
            .size(crate::ControlSize::Lg)
            .on_change(TestMsg::Changed)
            .on_toggle_visibility(TestMsg::Toggled);
        assert_eq!(p.placeholder, "Enter password");
        assert_eq!(p.value, "s3cr3t");
        assert!(p.visible);
        assert!(p.disabled);
        assert_eq!(p.size, crate::ControlSize::Lg);
        assert!(p.on_change.is_some());
        assert!(p.on_toggle_visibility.is_some());
    }
}
