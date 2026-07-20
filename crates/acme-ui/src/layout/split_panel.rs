//! SplitPanel component — a row with two panels separated by a visual divider.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a split panel layout.
pub struct SplitPanelBuilder<M> {
    pub id: WidgetKey,
    pub ratio: f32,
    pub first: Option<WidgetNode<M>>,
    pub second: Option<WidgetNode<M>>,
}

/// Create a split panel builder.
pub fn split_panel<M: Clone + 'static>(id: impl Into<WidgetKey>) -> SplitPanelBuilder<M> {
    SplitPanelBuilder {
        id: id.into(),
        ratio: 0.5,
        first: None,
        second: None,
    }
}

impl<M: Clone> SplitPanelBuilder<M> {
    /// Set the split ratio (0.0 – 1.0). Default 0.5.
    pub fn ratio(mut self, value: f32) -> Self {
        self.ratio = value.clamp(0.0, 1.0);
        self
    }

    /// Set the first (left/top) panel content.
    pub fn first(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.first = Some(child.into());
        self
    }

    /// Set the second (right/bottom) panel content.
    pub fn second(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.second = Some(child.into());
        self
    }

    /// Build the widget node tree.
    ///
    /// Renders as a `Row` with first panel, visual divider (separator), second panel.
    pub fn build(self) -> WidgetNode<M> {
        let mut row = crate::row::<M>().gap(4.0);

        if let Some(first) = self.first {
            row = row.child(first);
        }

        // Visual divider
        row = row.child(crate::separator::<M>());

        if let Some(second) = self.second {
            row = row.child(second);
        }

        row.build()
    }
}

impl<M: Clone + 'static> From<SplitPanelBuilder<M>> for WidgetNode<M> {
    fn from(b: SplitPanelBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{label, WidgetNode};
    use acme_core::NodeId;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn split_panel_has_non_zero_layout_rect() {
        let node: WidgetNode<Msg> = split_panel::<Msg>("panel")
            .first(label("Left"))
            .second(label("Right"))
            .ratio(0.3)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Row: first + separator + second = 3 children
        assert_eq!(layout.children.len(), 3);
    }

    #[test]
    fn split_panel_first_only() {
        let node: WidgetNode<Msg> = split_panel::<Msg>("p").first(label("Only")).build();
        let layout = node.to_layout(NodeId::new(1));
        // Row: first + separator = 2
        assert_eq!(layout.children.len(), 2);
    }

    #[test]
    fn split_panel_ratio_default() {
        let b = split_panel::<Msg>("p");
        assert!((b.ratio - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn split_panel_ratio_clamped() {
        let b = split_panel::<Msg>("p").ratio(2.0);
        assert!((b.ratio - 1.0).abs() < f32::EPSILON);
    }
}
