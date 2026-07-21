//! Collapsible component — a single section with a header that toggles visibility
//! of its child content.

use crate::{IconName, icon};
use acme_core::WidgetKey;
use acme_widgets::*;

/// Builder for a Collapsible component.
pub struct CollapsibleBuilder<M> {
    pub id: WidgetKey,
    pub title: String,
    pub open: bool,
    pub disabled: bool,
    pub on_toggle: Option<M>,
    pub child: Option<Box<WidgetNode<M>>>,
}

/// Create a new Collapsible builder.
pub fn collapsible<M: Clone + 'static>(id: impl Into<WidgetKey>) -> CollapsibleBuilder<M> {
    CollapsibleBuilder {
        id: id.into(),
        title: String::new(),
        open: false,
        disabled: false,
        on_toggle: None,
        child: None,
    }
}

impl<M: Clone + 'static> CollapsibleBuilder<M> {
    /// Set the collapsible header title.
    pub fn title(mut self, value: impl Into<String>) -> Self {
        self.title = value.into();
        self
    }

    /// Set whether the collapsible is initially open.
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }

    /// Set whether the collapsible is disabled.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    /// Set the message dispatched when the header is toggled.
    pub fn on_toggle(mut self, msg: M) -> Self {
        self.on_toggle = Some(msg);
        self
    }

    /// Set the child content revealed when open.
    pub fn child(mut self, node: impl Into<WidgetNode<M>>) -> Self {
        self.child = Some(Box::new(node.into()));
        self
    }
}

impl<M: Clone + 'static> From<CollapsibleBuilder<M>> for WidgetNode<M> {
    fn from(b: CollapsibleBuilder<M>) -> Self {
        // Chevron: ChevronDown (▾) when open, ChevronRight (▸) when closed
        let chevron = if b.open {
            icon(IconName::ChevronDown)
        } else {
            icon(IconName::ChevronRight)
        };

        // Header row: [chevron] + [title label]
        let header = row::<M>()
            .gap(6.0)
            .padding(4.0)
            .child(chevron.size(14.0))
            .child(label::<M>(b.title));

        let mut col = column::<M>().key(b.id).gap(0.0).child(header);

        // Child content only visible when open
        if b.open
            && let Some(content) = b.child
        {
            col = col.child(*content);
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
    use acme_layout::{LayoutKind, Length};

    #[derive(Clone, Debug, PartialEq)]
    #[allow(dead_code)]
    enum TestMsg {
        Toggled,
    }

    #[test]
    fn collapsible_builder_defaults() {
        let c = collapsible::<TestMsg>("c1");
        assert!(c.title.is_empty());
        assert!(!c.open);
        assert!(!c.disabled);
        assert!(c.on_toggle.is_none());
        assert!(c.child.is_none());
    }

    #[test]
    fn collapsible_open_shows_child() {
        let node: WidgetNode<TestMsg> = collapsible("sec1")
            .title("Section 1")
            .open(true)
            .child(label::<TestMsg>("Visible content"))
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant");
        };
        // Header row + child = 2 children
        assert_eq!(col.children.len(), 2);
        // First child is the header Row
        assert!(matches!(&col.children[0], WidgetNode::Row(_)));
        // Second child is the content Label
        assert!(matches!(&col.children[1], WidgetNode::Label(_)));
    }

    #[test]
    fn collapsible_closed_hides_child() {
        let node: WidgetNode<TestMsg> = collapsible("sec2")
            .title("Section 2")
            .open(false)
            .child(label::<TestMsg>("Hidden content"))
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant");
        };
        // Only the header row, child is hidden
        assert_eq!(col.children.len(), 1);
    }

    #[test]
    fn collapsible_header_has_chevron_and_title() {
        let node: WidgetNode<TestMsg> = collapsible("s").title("Details").open(true).into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant");
        };
        let WidgetNode::Row(header) = &col.children[0] else {
            panic!("expected Row as header");
        };
        // Chevron icon (Label) + title label = 2 children
        assert_eq!(header.children.len(), 2);
        // Chevron is a Label with chevron character
        let WidgetNode::Label(chevron) = &header.children[0] else {
            panic!("expected Label for chevron");
        };
        // ChevronDown is "▾"
        assert_eq!(chevron.text, "▾");
        // Title label
        let WidgetNode::Label(title) = &header.children[1] else {
            panic!("expected Label for title");
        };
        assert_eq!(title.text, "Details");
    }

    #[test]
    fn collapsible_closed_uses_right_chevron() {
        let node: WidgetNode<TestMsg> = collapsible("s").title("Closed").open(false).into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant");
        };
        let WidgetNode::Row(header) = &col.children[0] else {
            panic!("expected Row as header");
        };
        let WidgetNode::Label(chevron) = &header.children[0] else {
            panic!("expected Label for chevron");
        };
        // ChevronRight is "▸"
        assert_eq!(chevron.text, "▸");
    }

    #[test]
    fn collapsible_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = collapsible("c")
            .title("Info")
            .open(true)
            .child(label::<TestMsg>("Some details here"))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert_eq!(layout.children.len(), 2);
        // Header row child
        assert_eq!(layout.children[0].style.kind, LayoutKind::Row);
        // Content leaf
        assert!(layout.children[1].children.is_empty());
        assert_ne!(layout.children[1].style.min_height, Length::px(0.0));
    }
}
