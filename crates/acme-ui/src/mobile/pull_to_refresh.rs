//! Pull-to-refresh indicator with spinner/arrow and label.
//!
//! Renders as a [`Column`] with a refresh indicator [`Row`] above the child content.

use acme_core::WidgetKey;
use acme_widgets::*;

/// Builder for a pull-to-refresh component.
pub struct PullToRefreshBuilder<M> {
    pub id: WidgetKey,
    pub refreshing: bool,
    pub progress: f32,
    pub child: Option<WidgetNode<M>>,
    pub on_refresh: Option<M>,
}

/// Create a new pull-to-refresh builder.
pub fn pull_to_refresh<M: Clone + 'static>(id: impl Into<WidgetKey>) -> PullToRefreshBuilder<M> {
    PullToRefreshBuilder {
        id: id.into(),
        refreshing: false,
        progress: 0.0,
        child: None,
        on_refresh: None,
    }
}

impl<M: Clone + 'static> PullToRefreshBuilder<M> {
    /// Set whether the refresh is in progress.
    pub fn refreshing(mut self, value: bool) -> Self {
        self.refreshing = value;
        self
    }

    /// Set the pull progress (0.0 to 1.0).
    pub fn progress(mut self, value: f32) -> Self {
        self.progress = value.max(0.0).min(1.0);
        self
    }

    /// Set the child content below the refresh indicator.
    pub fn child(mut self, value: impl Into<WidgetNode<M>>) -> Self {
        self.child = Some(value.into());
        self
    }

    /// Set the message dispatched when a refresh is triggered.
    pub fn on_refresh(mut self, msg: M) -> Self {
        self.on_refresh = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<PullToRefreshBuilder<M>> for WidgetNode<M> {
    fn from(b: PullToRefreshBuilder<M>) -> Self {
        let mut col = column::<M>().key(b.id).gap(8.0);

        // Refresh indicator Row
        let indicator_icon = if b.refreshing { "↻" } else { "↓" };
        let indicator_text = if b.refreshing {
            "Refreshing..."
        } else {
            "Pull to refresh"
        };

        col = col.child(
            row::<M>()
                .gap(8.0)
                .padding(4.0)
                .child(crate::label::<M>(indicator_icon))
                .child(crate::label::<M>(indicator_text))
                .build(),
        );

        // Optional child content
        if let Some(child) = b.child {
            col = col.child(child);
        }

        col.build()
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
    fn pull_to_refresh_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = pull_to_refresh("ptr")
            .child(crate::label::<TestMsg>("Content"))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert!(!layout.children.is_empty());
        // Column has indicator Row + content = 2 children
        assert_eq!(layout.children.len(), 2);
        // First child is the indicator Row
        assert_eq!(layout.children[0].style.kind, LayoutKind::Row);
        // Second child is the content label (leaf)
        assert_eq!(layout.children[1].style.kind, LayoutKind::Leaf);
    }

    #[test]
    fn pull_to_refresh_builder_defaults() {
        let ptr = pull_to_refresh::<TestMsg>("ptr");
        assert!(!ptr.refreshing);
        assert_eq!(ptr.progress, 0.0);
        assert!(ptr.child.is_none());
        assert!(ptr.on_refresh.is_none());
    }

    #[test]
    fn pull_to_refresh_shows_indicators() {
        let node: WidgetNode<TestMsg> = pull_to_refresh("ptr")
            .refreshing(true)
            .child(crate::label::<TestMsg>("Items"))
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        // Column has 2 children: indicator Row + content
        assert_eq!(col.children.len(), 2);
        // First child is the indicator Row
        let WidgetNode::Row(indicator) = &col.children[0] else {
            panic!("expected Row for indicator");
        };
        assert_eq!(indicator.children.len(), 2);
        // Labels inside the indicator
        let WidgetNode::Label(icon_lbl) = &indicator.children[0] else {
            panic!("expected Label for icon");
        };
        // When refreshing, show the refresh icon
        assert_eq!(icon_lbl.text, "↻");
        let WidgetNode::Label(text_lbl) = &indicator.children[1] else {
            panic!("expected Label for text");
        };
        assert_eq!(text_lbl.text, "Refreshing...");
    }
}
