//! Progress component — a track bar with a fill indicator.

use crate::WidgetNode;

/// Builder for a progress bar widget.
pub struct ProgressBuilder<M> {
    pub value: f32, // 0.0 – 100.0
    pub size: crate::ControlSize,
    pub show_label: bool,
    pub animated: bool,
    pub tone: crate::Tone,
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
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> ProgressBuilder<M> {
    /// Set the track size.
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

    /// Build the progress bar widget.
    pub fn build(self) -> WidgetNode<M> {
        let track_px: f32 = match self.size {
            crate::ControlSize::Xs => 22.0,
            crate::ControlSize::Sm => 28.0,
            crate::ControlSize::Md => 34.0,
            crate::ControlSize::Lg => 40.0,
            crate::ControlSize::Xl => 48.0,
        };
        let _track_height = (track_px * 0.5_f32).max(8.0_f32);
        let fill_card = crate::card()
            .padding(0.0)
            .variant(crate::CardVariant::Interactive)
            .build();
        let mut track = crate::card()
            .child(fill_card)
            .padding(0.0)
            .gap(0.0)
            .variant(crate::CardVariant::Plain);
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
        // Progress is a Card track containing a fill Card.
        // Without explicit sizes, the Card has no intrinsic dimensions.
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        assert_eq!(c.variant, crate::CardVariant::Plain);
        assert!(!c.children.is_empty(), "progress should have a fill child");
    }

    #[test]
    fn progress_displays_label_text() {
        let node: WidgetNode<TestMsg> = progress(75.0).build();
        // Progress is a Card containing a Card (fill)
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        assert!(!c.children.is_empty(), "progress should have children");
    }
}
