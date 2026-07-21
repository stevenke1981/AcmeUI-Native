//! Progress component — a track bar with a proportional fill indicator.
//!
//! Renders a horizontal track (Row) with a fill `Card` whose width is proportional
//! to `value` / 100. A spacer occupies the remaining width. Optionally shows a
//! percentage label.

use crate::WidgetNode;

/// Builder for a progress bar widget.
pub struct ProgressBuilder<M> {
    pub value: f32, // 0.0 – 100.0
    pub size: crate::ControlSize,
    pub show_label: bool,
    pub animated: bool,
    pub tone: crate::Tone,
    /// Total visual width of the track in pixels.
    /// The fill width = `track_width * (value / 100.0)`.
    pub track_width: f32,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a progress bar builder.
pub fn progress<M>(value: f32) -> ProgressBuilder<M> {
    ProgressBuilder {
        value: value.clamp(0.0, 100.0),
        size: crate::ControlSize::Md,
        show_label: false,
        animated: false,
        tone: crate::Tone::Primary,
        track_width: 200.0,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> ProgressBuilder<M> {
    /// Set the track size (controls height).
    pub fn size(mut self, value: crate::ControlSize) -> Self {
        self.size = value;
        self
    }

    /// Show or hide the percentage label.
    pub fn show_label(mut self, value: bool) -> Self {
        self.show_label = value;
        self
    }

    /// Enable or disable stripe animation.
    pub fn animated(mut self, value: bool) -> Self {
        self.animated = value;
        self
    }

    /// Set the tone (affects fill color).
    pub fn tone(mut self, value: crate::Tone) -> Self {
        self.tone = value;
        self
    }

    /// Set the total track width in pixels (default 200.0).
    /// The fill width = `track_width * (value / 100.0)`.
    pub fn track_width(mut self, value: f32) -> Self {
        self.track_width = value;
        self
    }

    /// Build the progress bar widget.
    ///
    /// Returns a `Row` containing:
    /// - A fill `Card` with width = `track_width * (value / 100.0)`
    /// - An empty spacer `Column` for the remaining width
    /// - Optionally a percentage `Label`
    pub fn build(self) -> WidgetNode<M> {
        let track_px: f32 = match self.size {
            crate::ControlSize::Xs => 22.0,
            crate::ControlSize::Sm => 28.0,
            crate::ControlSize::Md => 34.0,
            crate::ControlSize::Lg => 40.0,
            crate::ControlSize::Xl => 48.0,
        };
        let track_height = (track_px * 0.5_f32).max(8.0_f32);
        let fill_ratio = (self.value / 100.0).clamp(0.0, 1.0);
        let fill_width = self.track_width * fill_ratio;
        let empty_width = self.track_width - fill_width;

        // Build the fill portion
        let fill_card = crate::card()
            .padding(0.0)
            .variant(crate::CardVariant::Interactive)
            .build();
        let fill = crate::column()
            .width(fill_width)
            .height(track_height)
            .child(fill_card)
            .build();

        // Build the track row
        let mut track = crate::row()
            .width(self.track_width)
            .height(track_height)
            .gap(0.0)
            .padding(0.0)
            .child(fill);

        // Add empty spacer for remaining width (only if there's remaining space)
        if empty_width > 0.5 {
            let spacer = crate::column()
                .width(empty_width)
                .height(track_height)
                .build();
            track = track.child(spacer);
        }

        if self.show_label {
            let pct = format!("{:.0}%", self.value);
            track = track.child(crate::label(pct));
        }

        track.build()
    }
}

impl<M: Clone + 'static> From<ProgressBuilder<M>> for WidgetNode<M> {
    fn from(b: ProgressBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn progress_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = progress(50.0).build();
        let WidgetNode::Row(r) = &node else {
            panic!(
                "expected Row variant, got {:?}",
                std::mem::discriminant(&node)
            );
        };
        assert_eq!(r.width, Some(200.0), "track width should be 200px");
        assert!(!r.children.is_empty(), "progress should have children");
    }

    #[test]
    fn progress_fill_width_proportional_to_value() {
        let node_75: WidgetNode<TestMsg> = progress(75.0).build();
        let WidgetNode::Row(r) = &node_75 else {
            panic!("expected Row variant");
        };
        // First child should be the fill column with width = 150px (75% of 200)
        let WidgetNode::Column(fill) = &r.children[0] else {
            panic!("expected Column as first child");
        };
        assert_eq!(fill.width, Some(150.0), "75% fill should be 150px");

        let node_25: WidgetNode<TestMsg> = progress(25.0).track_width(400.0).build();
        let WidgetNode::Row(r2) = &node_25 else {
            panic!("expected Row variant");
        };
        let WidgetNode::Column(fill2) = &r2.children[0] else {
            panic!("expected Column as first child");
        };
        assert_eq!(fill2.width, Some(100.0), "25% of 400px should be 100px");
    }

    #[test]
    fn progress_zero_value_has_zero_fill() {
        let node: WidgetNode<TestMsg> = progress(0.0).build();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row variant");
        };
        let WidgetNode::Column(fill) = &r.children[0] else {
            panic!("expected Column as first child");
        };
        assert_eq!(fill.width, Some(0.0), "0% fill should be 0px");
    }

    #[test]
    fn progress_full_value_has_no_spacer() {
        let node: WidgetNode<TestMsg> = progress(100.0).build();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row variant");
        };
        // At 100%, only the fill child exists (no spacer)
        assert_eq!(r.children.len(), 1, "100% should have only fill, no spacer");
    }

    #[test]
    fn progress_displays_label() {
        let node: WidgetNode<TestMsg> = progress(75.0).show_label(true).build();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row variant");
        };
        // With label, there should be 3 children: fill, spacer, label
        // At 75%: fill(150px) + spacer(50px) + label
        assert_eq!(r.children.len(), 3, "should have fill + spacer + label");
        let WidgetNode::Label(lbl) = &r.children[2] else {
            panic!("expected Label as third child");
        };
        assert_eq!(lbl.text, "75%");
    }

    #[test]
    fn progress_default_value_is_clamped() {
        let b = progress::<TestMsg>(150.0);
        assert_eq!(b.value, 100.0);
        let b2 = progress::<TestMsg>(-10.0);
        assert_eq!(b2.value, 0.0);
    }

    #[test]
    fn progress_from_trait() {
        let node: WidgetNode<TestMsg> = progress(50.0).into();
        let WidgetNode::Row(_) = &node else {
            panic!("expected Row variant");
        };
    }

    #[test]
    fn progress_custom_track_width() {
        let node: WidgetNode<TestMsg> = progress(50.0).track_width(400.0).build();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row variant");
        };
        assert_eq!(r.width, Some(400.0));
    }

    #[test]
    fn progress_all_sizes_build() {
        for size in &[
            crate::ControlSize::Xs,
            crate::ControlSize::Sm,
            crate::ControlSize::Md,
            crate::ControlSize::Lg,
            crate::ControlSize::Xl,
        ] {
            let node: WidgetNode<TestMsg> = progress(50.0).size(*size).build();
            let WidgetNode::Row(_) = &node else {
                panic!("expected Row variant for size {:?}", size);
            };
        }
    }
}
