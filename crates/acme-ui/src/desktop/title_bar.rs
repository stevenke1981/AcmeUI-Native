//! TitleBar — custom window title bar with icon, title text, and window
//! control buttons (minimize, maximize, close).

use crate::*;

/// Builder for a TitleBar component.
pub struct TitleBarBuilder<M> {
    pub id: WidgetKey,
    pub title: String,
    pub icon: Option<IconName>,
    pub show_controls: bool,
    pub on_close: Option<M>,
    pub on_minimize: Option<M>,
    pub on_maximize: Option<M>,
}

/// Create a new TitleBar builder.
pub fn title_bar<M: Clone + 'static>(
    id: impl Into<WidgetKey>,
    title: impl Into<String>,
) -> TitleBarBuilder<M> {
    TitleBarBuilder {
        id: id.into(),
        title: title.into(),
        icon: None,
        show_controls: true,
        on_close: None,
        on_minimize: None,
        on_maximize: None,
    }
}

impl<M: Clone + 'static> TitleBarBuilder<M> {
    /// Set the icon displayed on the left side of the title bar.
    pub fn icon(mut self, value: IconName) -> Self {
        self.icon = Some(value);
        self
    }

    /// Show or hide the window control buttons (minimize, maximize, close).
    pub fn show_controls(mut self, value: bool) -> Self {
        self.show_controls = value;
        self
    }

    /// Set the message dispatched when the close button is clicked.
    pub fn on_close(mut self, msg: M) -> Self {
        self.on_close = Some(msg);
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
}

impl<M: Clone + 'static> From<TitleBarBuilder<M>> for WidgetNode<M> {
    fn from(b: TitleBarBuilder<M>) -> Self {
        let mut title_row = row::<M>().key(b.id).gap(8.0);

        // Optional icon label on the left
        if let Some(icon_name) = b.icon {
            title_row = title_row.child(icon::<M>(icon_name));
        }

        // Title label (centered area)
        title_row = title_row.child(label::<M>(b.title));

        // Optional window control buttons on the right
        if b.show_controls {
            let mut buttons_row = row::<M>().gap(4.0);
            if let Some(msg) = b.on_minimize {
                buttons_row = buttons_row.child(button("minimize", "─").on_click(msg));
            }
            if let Some(msg) = b.on_maximize {
                buttons_row = buttons_row.child(button("maximize", "□").on_click(msg));
            }
            if let Some(msg) = b.on_close {
                buttons_row = buttons_row.child(button("close", "✕").on_click(msg));
            }
            title_row = title_row.child(buttons_row.build());
        }

        title_row.build()
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
    fn title_bar_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = title_bar("tb", "My App").into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Row);
        // Should have at least 1 child (the title label)
        assert!(!layout.children.is_empty());
    }

    #[test]
    fn title_bar_builder_defaults() {
        let tb = title_bar::<TestMsg>("tb", "App");
        assert_eq!(tb.title, "App");
        assert!(tb.icon.is_none());
        assert!(tb.show_controls);
        assert!(tb.on_close.is_none());
        assert!(tb.on_minimize.is_none());
        assert!(tb.on_maximize.is_none());
    }

    #[test]
    fn title_bar_field_setters_work() {
        #[derive(Clone, Debug, PartialEq)]
        enum Msg {
            Close,
            Min,
            Max,
        }

        let tb = title_bar::<Msg>("tb", "My App")
            .icon(IconName::Settings)
            .show_controls(false)
            .on_close(Msg::Close)
            .on_minimize(Msg::Min)
            .on_maximize(Msg::Max);

        assert_eq!(tb.icon, Some(IconName::Settings));
        assert!(!tb.show_controls);
        assert_eq!(tb.on_close, Some(Msg::Close));
        assert_eq!(tb.on_minimize, Some(Msg::Min));
        assert_eq!(tb.on_maximize, Some(Msg::Max));
    }
}
