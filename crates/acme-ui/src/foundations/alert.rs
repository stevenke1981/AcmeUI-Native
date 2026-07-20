//! Alert component — a Row with Icon + Label and optional close Button.

use crate::WidgetNode;

/// Builder for an alert widget.
pub struct AlertBuilder<M> {
    pub message: String,
    pub tone: crate::Tone,
    pub dismissible: bool,
    _phantom: std::marker::PhantomData<M>,
}

/// Create an alert builder.
pub fn alert<M>(message: impl Into<String>) -> AlertBuilder<M> {
    AlertBuilder {
        message: message.into(),
        tone: crate::Tone::Neutral,
        dismissible: false,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> AlertBuilder<M> {
    /// Set the alert tone.
    pub fn tone(mut self, tone: crate::Tone) -> Self {
        self.tone = tone;
        self
    }

    /// Make the alert dismissible (shows a close button).
    pub fn dismissible(mut self) -> Self {
        self.dismissible = true;
        self
    }

    /// Build the alert widget.
    pub fn build(self) -> WidgetNode<M> {
        let icon_name = match self.tone {
            crate::Tone::Neutral => super::IconName::Info,
            crate::Tone::Primary => super::IconName::Info,
            crate::Tone::Success => super::IconName::Success,
            crate::Tone::Warning => super::IconName::Warning,
            crate::Tone::Danger => super::IconName::Error,
        };
        let mut r = crate::row()
            .gap(8.0)
            .child(super::icon(icon_name).size(16.0))
            .child(crate::label(&self.message));
        if self.dismissible {
            r = r.child(crate::button("alert-close", "✕"));
        }
        r.build()
    }
}

impl<M: Clone + 'static> From<AlertBuilder<M>> for WidgetNode<M> {
    fn from(b: AlertBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use acme_core::NodeId;
    use acme_layout::{LayoutEngine, WidgetLayoutContext};

    fn test_context() -> WidgetLayoutContext {
        WidgetLayoutContext {
            body_font_size: 16.0,
            body_line_height: 22.0,
            label_font_size: 14.0,
            control_height: 32.0,
            scale_factor: 1.0,
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn alert_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = alert("Something happened").build();
        let ctx = test_context();
        let layout = node.to_layout_with_context(NodeId::new(1), &ctx);
        let mut fonts = acme_text::FontSystem::new();
        let snapshot = LayoutEngine::new()
            .compute_with_text(&layout, (800.0, 600.0), &mut fonts, 1.0)
            .unwrap();
        let rect = snapshot.get(NodeId::new(1)).unwrap();
        assert!(rect.width > 0.0, "alert width should be > 0");
        assert!(rect.height > 0.0, "alert height should be > 0");
    }

    #[test]
    fn alert_displays_label_text() {
        let node: WidgetNode<TestMsg> = alert("Something happened").build();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row variant");
        };
        // First child is icon, second is label
        assert_eq!(r.children.len(), 2);
        let WidgetNode::Label(l) = &r.children[1] else {
            panic!("expected Label as second child");
        };
        assert_eq!(l.text, "Something happened");
    }

    #[test]
    fn alert_dismissible_has_close_button() {
        let node: WidgetNode<TestMsg> = alert("Error").dismissible().build();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row variant");
        };
        assert_eq!(r.children.len(), 3);
        let WidgetNode::Button(_) = &r.children[2] else {
            panic!("expected Button as third child");
        };
    }
}
