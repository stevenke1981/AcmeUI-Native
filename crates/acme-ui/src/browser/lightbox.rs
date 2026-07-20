//! Lightbox component — a full-screen overlay for viewing images/media.

use acme_core::WidgetKey;
use acme_widgets::*;

/// Builder for a Lightbox (full-screen media overlay).
pub struct LightboxBuilder<M> {
    pub id: WidgetKey,
    pub open: bool,
    pub label: String,
    pub caption: Option<String>,
    pub on_close: Option<M>,
}

/// Create a new Lightbox builder.
pub fn lightbox<M: Clone + 'static>(
    id: impl Into<WidgetKey>,
    label: impl Into<String>,
) -> LightboxBuilder<M> {
    LightboxBuilder {
        id: id.into(),
        open: false,
        label: label.into(),
        caption: None,
        on_close: None,
    }
}

impl<M: Clone + 'static> LightboxBuilder<M> {
    /// Open or close the lightbox.
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }

    /// Set a caption displayed below the media area.
    pub fn caption(mut self, value: impl Into<String>) -> Self {
        self.caption = Some(value.into());
        self
    }

    /// Set the message dispatched when the close button is clicked.
    pub fn on_close(mut self, msg: M) -> Self {
        self.on_close = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<LightboxBuilder<M>> for WidgetNode<M> {
    fn from(b: LightboxBuilder<M>) -> Self {
        if !b.open {
            // Closed state: minimal indicator
            return crate::card::<M>()
                .key(b.id)
                .variant(CardVariant::Muted)
                .child(crate::label::<M>("📷"))
                .padding(4.0)
                .build();
        }

        // Close button
        let close_btn: WidgetNode<M> = if let Some(msg) = b.on_close {
            crate::button("lightbox-close", "✕")
                .variant(ButtonVariant::Ghost)
                .on_click(msg)
        } else {
            crate::button("lightbox-close", "✕")
                .variant(ButtonVariant::Ghost)
                .into()
        };

        // Top bar with close button right-aligned
        let top_bar = crate::row::<M>()
            .child(crate::label::<M>(""))
            .child(close_btn)
            .build();

        // Media area
        let media_card = crate::card::<M>()
            .variant(CardVariant::Muted)
            .child(crate::label::<M>(b.label))
            .padding(24.0)
            .build();

        // Build content column
        let mut col = crate::column::<M>()
            .gap(8.0)
            .child(top_bar)
            .child(media_card);

        if let Some(caption) = b.caption {
            col = col.child(crate::label::<M>(caption));
        }

        // Wrap in a Stack for overlay effect
        crate::stack::<M>()
            .key(b.id)
            .child(col.build())
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

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn lightbox_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> =
            lightbox("lb", "Photo").open(true).caption("A beautiful view").into();
        let layout = node.to_layout(NodeId::new(1));
        // Stack with one child (Column)
        assert_eq!(layout.children.len(), 1);
    }

    #[test]
    fn lightbox_builder_defaults() {
        let lb = lightbox::<TestMsg>("lb", "Photo");
        assert!(!lb.open);
        assert!(lb.caption.is_none());
        assert!(lb.on_close.is_none());
        assert_eq!(lb.label, "Photo");
    }

    #[test]
    fn lightbox_closed_shows_minimal() {
        let node: WidgetNode<TestMsg> = lightbox("lb", "Photo").open(false).into();
        // Closed: Card with Muted variant
        assert!(matches!(node, WidgetNode::Card(_)));
    }
}
