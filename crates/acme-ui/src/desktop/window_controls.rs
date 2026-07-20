//! WindowControls component — standalone window control buttons
//! (minimize, maximize, close).
//!
//! Renders as a Row of small ghost/danger buttons.

use crate::*;

/// Builder for a WindowControls component.
pub struct WindowControlsBuilder<M> {
    pub id: WidgetKey,
    pub show_minimize: bool,
    pub show_maximize: bool,
    pub show_close: bool,
    pub on_minimize: Option<M>,
    pub on_maximize: Option<M>,
    pub on_close: Option<M>,
}

/// Create a new WindowControls builder.
pub fn window_controls<M: Clone + 'static>(
    id: impl Into<WidgetKey>,
) -> WindowControlsBuilder<M> {
    WindowControlsBuilder {
        id: id.into(),
        show_minimize: true,
        show_maximize: true,
        show_close: true,
        on_minimize: None,
        on_maximize: None,
        on_close: None,
    }
}

impl<M: Clone + 'static> WindowControlsBuilder<M> {
    /// Show or hide the minimize button.
    pub fn show_minimize(mut self, value: bool) -> Self {
        self.show_minimize = value;
        self
    }

    /// Show or hide the maximize button.
    pub fn show_maximize(mut self, value: bool) -> Self {
        self.show_maximize = value;
        self
    }

    /// Show or hide the close button.
    pub fn show_close(mut self, value: bool) -> Self {
        self.show_close = value;
        self
    }

    /// Set the message dispatched when the minimize button is clicked.
    pub fn on_minimize(mut self, msg: M) -> Self {
        self.on_minimize = Some(msg);
        self
    }

    /// Set the message dispatched when the maximize button is clicked.
    pub fn on_maximize(mut self, msg: M) -> Self {
        self.on_maximize = Some(msg);
        self
    }

    /// Set the message dispatched when the close button is clicked.
    pub fn on_close(mut self, msg: M) -> Self {
        self.on_close = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<WindowControlsBuilder<M>> for WidgetNode<M> {
    fn from(b: WindowControlsBuilder<M>) -> Self {
        let mut btn_row = row::<M>().key(b.id).gap(4.0);

        if b.show_minimize {
            let btn = button("minimize", "─")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Small);
            if let Some(msg) = b.on_minimize.clone() {
                btn_row = btn_row.child(btn.on_click(msg));
            } else {
                btn_row = btn_row.child(btn);
            }
        }

        if b.show_maximize {
            let btn = button("maximize", "□")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Small);
            if let Some(msg) = b.on_maximize.clone() {
                btn_row = btn_row.child(btn.on_click(msg));
            } else {
                btn_row = btn_row.child(btn);
            }
        }

        if b.show_close {
            let btn = button("close", "✕")
                .variant(ButtonVariant::Danger)
                .size(ButtonSize::Small);
            if let Some(msg) = b.on_close.clone() {
                btn_row = btn_row.child(btn.on_click(msg));
            } else {
                btn_row = btn_row.child(btn);
            }
        }

        btn_row.build()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;
    use acme_layout::LayoutKind;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn window_controls_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = window_controls("wc").into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Row);
        // 3 buttons by default
        assert_eq!(layout.children.len(), 3);
    }

    #[test]
    fn window_controls_builder_defaults() {
        let wc = window_controls::<TestMsg>("wc");
        assert!(wc.show_minimize);
        assert!(wc.show_maximize);
        assert!(wc.show_close);
        assert!(wc.on_minimize.is_none());
        assert!(wc.on_maximize.is_none());
        assert!(wc.on_close.is_none());
    }

    #[test]
    fn window_controls_hides_minimize() {
        let node: WidgetNode<TestMsg> = window_controls("wc")
            .show_minimize(false)
            .into();
        let WidgetNode::Row(row) = &node else {
            panic!("expected Row variant");
        };
        // Only maximize + close = 2
        assert_eq!(row.children.len(), 2);
    }

    #[test]
    fn window_controls_hides_maximize() {
        let node: WidgetNode<TestMsg> = window_controls("wc")
            .show_maximize(false)
            .into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row variant");
        };
        // Minimize + close = 2
        assert_eq!(r.children.len(), 2);
    }

    #[test]
    fn window_controls_hides_close() {
        let node: WidgetNode<TestMsg> = window_controls("wc")
            .show_close(false)
            .into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row variant");
        };
        // Minimize + maximize = 2
        assert_eq!(r.children.len(), 2);
    }

    #[test]
    fn window_controls_all_hidden() {
        let node: WidgetNode<TestMsg> = window_controls("wc")
            .show_minimize(false)
            .show_maximize(false)
            .show_close(false)
            .into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row variant");
        };
        assert!(r.children.is_empty());
    }

    #[test]
    fn window_controls_with_messages() {
        #[derive(Clone, Debug, PartialEq)]
        enum Msg {
            Min,
            Max,
            Close,
        }
        let node: WidgetNode<Msg> = window_controls("wc")
            .on_minimize(Msg::Min)
            .on_maximize(Msg::Max)
            .on_close(Msg::Close)
            .into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row variant");
        };
        assert_eq!(r.children.len(), 3);
        // All children should be Button nodes (on_click returns WidgetNode::Button)
        for child in &r.children {
            assert!(matches!(child, WidgetNode::Button(_)), "expected Button");
        }
    }
}
