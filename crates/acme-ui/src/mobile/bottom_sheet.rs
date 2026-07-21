//! BottomSheet component — a slide-up panel overlay from the bottom.
//!
//! Renders as a [`Column`] inside a [`Card`] with a handle bar,
//! optional title, and child content.

use acme_core::WidgetKey;
use acme_widgets::*;

/// Builder for a bottom sheet component.
pub struct BottomSheetBuilder<M> {
    pub id: WidgetKey,
    pub title: Option<String>,
    pub open: bool,
    pub child: Option<WidgetNode<M>>,
    pub on_close: Option<M>,
}

/// Create a new bottom sheet builder.
pub fn bottom_sheet<M: Clone + 'static>(id: impl Into<WidgetKey>) -> BottomSheetBuilder<M> {
    BottomSheetBuilder {
        id: id.into(),
        title: None,
        open: false,
        child: None,
        on_close: None,
    }
}

impl<M: Clone + 'static> BottomSheetBuilder<M> {
    /// Set the title displayed in the sheet header.
    pub fn title(mut self, value: impl Into<String>) -> Self {
        self.title = Some(value.into());
        self
    }

    /// Set whether the sheet is open.
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }

    /// Set the child content inside the sheet.
    pub fn child(mut self, value: impl Into<WidgetNode<M>>) -> Self {
        self.child = Some(value.into());
        self
    }

    /// Set the message dispatched when the sheet is dismissed.
    pub fn on_close(mut self, msg: M) -> Self {
        self.on_close = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<BottomSheetBuilder<M>> for WidgetNode<M> {
    fn from(b: BottomSheetBuilder<M>) -> Self {
        let mut col = column::<M>().key(b.id).gap(12.0);

        // Handle bar — small muted Card as a drag indicator
        col = col.child(
            card::<M>()
                .variant(CardVariant::Muted)
                .padding(4.0)
                .child(crate::label::<M>(""))
                .build(),
        );

        // Optional title
        if let Some(title) = &b.title {
            col = col.child(crate::label::<M>(title));
        }

        // Optional child content
        if let Some(child) = b.child {
            col = col.child(child);
        }

        // Wrap everything in a Card for the sheet look
        card::<M>().padding(0.0).child(col).build()
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
    fn bottom_sheet_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = bottom_sheet("sheet")
            .title("Options")
            .child(crate::label::<TestMsg>("Content"))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Card renders as Column container
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert!(!layout.children.is_empty());
        // The outer Card has 1 child (the inner Column)
        assert_eq!(layout.children.len(), 1);
        let inner = &layout.children[0];
        assert_eq!(inner.style.kind, LayoutKind::Column);
        // Inner Column: handle bar + title + content = 3 children
        assert_eq!(inner.children.len(), 3);
    }

    #[test]
    fn bottom_sheet_builder_defaults() {
        let sheet = bottom_sheet::<TestMsg>("sheet");
        assert!(sheet.title.is_none());
        assert!(!sheet.open);
        assert!(sheet.child.is_none());
        assert!(sheet.on_close.is_none());
    }

    #[test]
    fn bottom_sheet_without_child_has_handle_and_title() {
        let node: WidgetNode<TestMsg> = bottom_sheet("sheet").title("Hello").into();
        let WidgetNode::Card(card_node) = &node else {
            panic!("expected Card");
        };
        // Card has one child (the inner Column)
        assert_eq!(card_node.children.len(), 1);
        let WidgetNode::Column(col) = &card_node.children[0] else {
            panic!("expected Column");
        };
        // Column has handle + title = 2 children
        assert_eq!(col.children.len(), 2);
        let WidgetNode::Label(lbl) = &col.children[1] else {
            panic!("expected Label for title");
        };
        assert_eq!(lbl.text, "Hello");
    }
}
