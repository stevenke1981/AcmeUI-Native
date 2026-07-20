//! FormMessage component.
//!
//! A helper text component for forms that shows validation messages with
//! tone-specific icons and default styling.
//!
//! Renders as a Row with an optional tone-icon and a small label.

use acme_widgets::*;
use std::marker::PhantomData;

/// Mapping from tone to a short icon character.
fn tone_icon(tone: crate::Tone) -> &'static str {
    match tone {
        crate::Tone::Neutral => "\u{2139}",      // ℹ
        crate::Tone::Primary => "\u{2139}",       // ℹ
        crate::Tone::Success => "\u{2713}",       // ✓
        crate::Tone::Warning => "\u{26a0}",       // ⚠
        crate::Tone::Danger => "\u{2715}",        // ✕
        crate::Tone::Info => "\u{2139}",          // ℹ
    }
}

/// Builder for a FormMessage component.
pub struct FormMessageBuilder<M> {
    pub text: String,
    pub tone: crate::Tone,
    pub size: crate::ControlSize,
    pub icon: bool,
    _phantom: PhantomData<M>,
}

/// Create a new FormMessage builder.
///
/// Defaults: tone=Neutral, size=Md, icon=false.
pub fn form_message<M: Clone + 'static>() -> FormMessageBuilder<M> {
    FormMessageBuilder {
        text: String::new(),
        tone: crate::Tone::Neutral,
        size: crate::ControlSize::Md,
        icon: false,
        _phantom: PhantomData,
    }
}

impl<M: Clone + 'static> FormMessageBuilder<M> {
    /// Set the message text.
    pub fn text(mut self, value: impl Into<String>) -> Self {
        self.text = value.into();
        self
    }

    /// Set the tone explicitly.
    pub fn tone(mut self, value: crate::Tone) -> Self {
        self.tone = value;
        self
    }

    /// Convenience: set tone to Success.
    pub fn success(mut self) -> Self {
        self.tone = crate::Tone::Success;
        self
    }

    /// Convenience: set tone to Warning.
    pub fn warning(mut self) -> Self {
        self.tone = crate::Tone::Warning;
        self
    }

    /// Convenience: set tone to Danger.
    pub fn danger(mut self) -> Self {
        self.tone = crate::Tone::Danger;
        self
    }

    /// Set the control size (default `Md`).
    pub fn size(mut self, value: crate::ControlSize) -> Self {
        self.size = value;
        self
    }

    /// Show or hide the tone icon before the message text.
    pub fn with_icon(mut self, value: bool) -> Self {
        self.icon = value;
        self
    }
}

impl<M: Clone + 'static> From<FormMessageBuilder<M>> for WidgetNode<M> {
    fn from(b: FormMessageBuilder<M>) -> Self {
        let mut row = row::<M>().gap(4.0);

        // Optional tone icon
        if b.icon {
            let icon_str = tone_icon(b.tone);
            // Use a small label for the icon
            row = row.child(label_with_size::<M>(icon_str, 12.0));
        }

        // Message text — use a slightly smaller font (12 px ≈ caption)
        row = row.child(label_with_size::<M>(b.text, 12.0));

        row.build()
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
    enum TestMsg {}

    #[test]
    fn form_message_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> =
            form_message().text("This is a message").into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Row);
        assert!(!layout.children.is_empty());
        for child in &layout.children {
            assert!(child.children.is_empty());
            assert_ne!(child.style.min_height, Length::px(0.0));
        }
    }

    #[test]
    fn form_message_builder_defaults() {
        let f = form_message::<TestMsg>();
        assert!(f.text.is_empty());
        assert_eq!(f.tone, crate::Tone::Neutral);
        assert!(!f.icon);
    }

    #[test]
    fn form_message_without_icon_has_single_label() {
        let node: WidgetNode<TestMsg> =
            form_message().text("Required field").into();
        let WidgetNode::Row(container) = &node else {
            panic!("expected Row");
        };
        // No icon, so only the message label
        assert_eq!(container.children.len(), 1);
        let WidgetNode::Label(lbl) = &container.children[0] else {
            panic!("expected Label");
        };
        assert_eq!(lbl.text, "Required field");
        assert_eq!(lbl.font_size, Some(12.0));
    }

    #[test]
    fn form_message_with_icon_shows_tone_icon_and_text() {
        let node: WidgetNode<TestMsg> =
            form_message().text("Warning!").warning().with_icon(true).into();
        let WidgetNode::Row(container) = &node else {
            panic!("expected Row");
        };
        assert_eq!(container.children.len(), 2);
        // First child is the tone icon
        let WidgetNode::Label(icon) = &container.children[0] else {
            panic!("expected Label icon");
        };
        assert_eq!(icon.text, "\u{26a0}"); // ⚠
        assert_eq!(icon.font_size, Some(12.0));
        // Second child is the message text
        let WidgetNode::Label(lbl) = &container.children[1] else {
            panic!("expected Label text");
        };
        assert_eq!(lbl.text, "Warning!");
    }

    #[test]
    fn form_message_tone_helpers() {
        let f_success = form_message::<TestMsg>().success();
        assert_eq!(f_success.tone, crate::Tone::Success);

        let f_warning = form_message::<TestMsg>().warning();
        assert_eq!(f_warning.tone, crate::Tone::Warning);

        let f_danger = form_message::<TestMsg>().danger();
        assert_eq!(f_danger.tone, crate::Tone::Danger);
    }

    #[test]
    fn form_message_tone_icons() {
        use crate::Tone;

        // Neutral → ℹ
        let node: WidgetNode<TestMsg> =
            form_message().text("info").tone(Tone::Neutral).with_icon(true).into();
        let WidgetNode::Row(r) = &node else { panic!("expected Row") };
        let WidgetNode::Label(icon) = &r.children[0] else { panic!("expected Label") };
        assert_eq!(icon.text, "\u{2139}");

        // Success → ✓
        let node: WidgetNode<TestMsg> =
            form_message().text("ok").success().with_icon(true).into();
        let WidgetNode::Row(r) = &node else { panic!("expected Row") };
        let WidgetNode::Label(icon) = &r.children[0] else { panic!("expected Label") };
        assert_eq!(icon.text, "\u{2713}");

        // Danger → ✕
        let node: WidgetNode<TestMsg> =
            form_message().text("err").danger().with_icon(true).into();
        let WidgetNode::Row(r) = &node else { panic!("expected Row") };
        let WidgetNode::Label(icon) = &r.children[0] else { panic!("expected Label") };
        assert_eq!(icon.text, "\u{2715}");
    }
}
