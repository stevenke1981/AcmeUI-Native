//! ZoomView component — a content container with zoom controls.

use acme_core::WidgetKey;
use acme_widgets::*;

/// Builder for a ZoomView (content with zoom controls).
pub struct ZoomViewBuilder<M> {
    pub id: WidgetKey,
    pub zoom_level: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub show_controls: bool,
    pub child: Option<WidgetNode<M>>,
    pub on_zoom_change: Option<M>,
}

/// Create a new ZoomView builder.
pub fn zoom_view<M: Clone + 'static>(id: impl Into<WidgetKey>) -> ZoomViewBuilder<M> {
    ZoomViewBuilder {
        id: id.into(),
        zoom_level: 1.0,
        min_zoom: 0.25,
        max_zoom: 4.0,
        show_controls: true,
        child: None,
        on_zoom_change: None,
    }
}

impl<M: Clone + 'static> ZoomViewBuilder<M> {
    /// Set the current zoom level (1.0 = 100%).
    pub fn zoom_level(mut self, value: f32) -> Self {
        self.zoom_level = value;
        self
    }

    /// Set the minimum allowed zoom level.
    pub fn min_zoom(mut self, value: f32) -> Self {
        self.min_zoom = value;
        self
    }

    /// Set the maximum allowed zoom level.
    pub fn max_zoom(mut self, value: f32) -> Self {
        self.max_zoom = value;
        self
    }

    /// Show or hide the zoom control buttons.
    pub fn show_controls(mut self, value: bool) -> Self {
        self.show_controls = value;
        self
    }

    /// Set the child content to display inside the zoom view.
    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.child = Some(child.into());
        self
    }

    /// Set the message dispatched when zoom buttons are clicked.
    pub fn on_zoom_change(mut self, msg: M) -> Self {
        self.on_zoom_change = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<ZoomViewBuilder<M>> for WidgetNode<M> {
    fn from(b: ZoomViewBuilder<M>) -> Self {
        let mut col = crate::column::<M>().gap(8.0);

        // Zoom controls row
        if b.show_controls {
            let zoom_pct = format!("{}%", (b.zoom_level * 100.0).round() as i32);

            let zoom_out_btn: WidgetNode<M> = if let Some(ref msg) = b.on_zoom_change {
                crate::button("zoom-out", "−")
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Small)
                    .on_click(msg.clone())
            } else {
                crate::button("zoom-out", "−")
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Small)
                    .into()
            };

            let zoom_in_btn: WidgetNode<M> = if let Some(ref msg) = b.on_zoom_change {
                crate::button("zoom-in", "+")
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Small)
                    .on_click(msg.clone())
            } else {
                crate::button("zoom-in", "+")
                    .variant(ButtonVariant::Secondary)
                    .size(ButtonSize::Small)
                    .into()
            };

            let controls = crate::row::<M>()
                .gap(4.0)
                .child(zoom_out_btn)
                .child(crate::label::<M>(zoom_pct))
                .child(zoom_in_btn)
                .build();

            col = col.child(controls);
        }

        // Child content wrapped in a Card
        if let Some(child) = b.child {
            col = col.child(
                crate::card::<M>()
                    .variant(CardVariant::Outlined)
                    .child(child)
                    .padding(8.0)
                    .build(),
            );
        }

        crate::card::<M>()
            .key(b.id)
            .child(col.build())
            .padding(8.0)
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
    fn zoom_view_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = zoom_view("zoom")
            .child(crate::label::<TestMsg>("Content"))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Wrapper Card -> Column with [controls Row, Card(child)]
        assert!(!layout.children.is_empty());
    }

    #[test]
    fn zoom_view_builder_defaults() {
        let zv = zoom_view::<TestMsg>("zoom");
        assert!((zv.zoom_level - 1.0).abs() < f32::EPSILON);
        assert!((zv.min_zoom - 0.25).abs() < f32::EPSILON);
        assert!((zv.max_zoom - 4.0).abs() < f32::EPSILON);
        assert!(zv.show_controls);
        assert!(zv.child.is_none());
        assert!(zv.on_zoom_change.is_none());
    }

    #[test]
    fn zoom_view_shows_zoom_label() {
        let node: WidgetNode<TestMsg> = zoom_view("zoom")
            .zoom_level(1.5)
            .child(crate::label::<TestMsg>("Content"))
            .into();
        let WidgetNode::Card(card) = &node else {
            panic!("expected Card wrapper");
        };
        // Card -> Column -> [Row, Card]
        let WidgetNode::Column(col) = &card.children[0] else {
            panic!("expected Column inside Card");
        };
        // First child is the controls Row
        let WidgetNode::Row(controls) = &col.children[0] else {
            panic!("expected controls Row as first child");
        };
        // controls has: [zoom-out btn, zoom label, zoom-in btn] = 3 children
        assert_eq!(controls.children.len(), 3);
        // Second child in controls is the zoom level label
        let WidgetNode::Label(lbl) = &controls.children[1] else {
            panic!("expected Label for zoom level");
        };
        assert_eq!(lbl.text, "150%");
    }
}
