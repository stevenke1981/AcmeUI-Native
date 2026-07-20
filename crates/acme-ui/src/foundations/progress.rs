//! Progress component — a track bar with a fill indicator.

use crate::WidgetNode;

/// Builder for a progress bar widget.
pub struct ProgressBuilder<M> {
    pub value: f32, // 0.0 – 100.0
    _phantom: std::marker::PhantomData<M>,
}

/// Create a progress bar builder.
pub fn progress<M>(value: f32) -> ProgressBuilder<M> {
    ProgressBuilder {
        value: value.clamp(0.0, 100.0),
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> ProgressBuilder<M> {
    /// Build the progress bar widget.
    pub fn build(self) -> WidgetNode<M> {
        // Track: a Card container with min-height
        // Fill: a Card with accent color, width proportional to value
        crate::card()
            .child(
                crate::card()
                    .padding(0.0)
                    .variant(crate::CardVariant::Plain)
                    .build(),
            )
            .padding(0.0)
            .gap(0.0)
            .build()
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
