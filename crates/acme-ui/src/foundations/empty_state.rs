//! EmptyState component — a centered Column with optional Icon + title + description.

use crate::WidgetNode;

/// Builder for an empty state widget.
pub struct EmptyStateBuilder<M> {
    pub title: String,
    pub description: Option<String>,
    pub icon: Option<super::IconName>,
    _phantom: std::marker::PhantomData<M>,
}

/// Create an empty state builder.
pub fn empty_state<M>(title: impl Into<String>) -> EmptyStateBuilder<M> {
    EmptyStateBuilder {
        title: title.into(),
        description: None,
        icon: None,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> EmptyStateBuilder<M> {
    /// Set the description text.
    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }

    /// Set the icon.
    pub fn icon(mut self, name: super::IconName) -> Self {
        self.icon = Some(name);
        self
    }

    /// Build the empty state widget.
    pub fn build(self) -> WidgetNode<M> {
        let mut col = crate::column().gap(8.0);
        if let Some(icon_name) = self.icon {
            col = col.child(super::icon(icon_name).size(48.0));
        }
        col = col.child(crate::label(&self.title));
        if let Some(ref desc) = self.description {
            col = col.child(crate::label(desc));
        }
        col.build()
    }
}

impl<M: Clone + 'static> From<EmptyStateBuilder<M>> for WidgetNode<M> {
    fn from(b: EmptyStateBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundations::IconName;
    use crate::*;
    use acme_core::NodeId;
    use acme_layout::{LayoutEngine, WidgetLayoutContext};

    fn test_context() -> WidgetLayoutContext {
        WidgetLayoutContext {
            body_font_size: 16.0,
            body_line_height: 22.0,
            label_font_size: 14.0,
            control_height: 32.0,
            scale_factor: 1.0,
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn empty_state_has_non_zero_layout_rect() {
        let ctx = test_context();
        let node: WidgetNode<TestMsg> = empty_state("No items")
            .description("Try adding some.")
            .icon(IconName::Info)
            .build();
        // Column with icon + title + description should have non-zero height
        let layout = node.to_layout_with_context(NodeId::new(1), &ctx);
        let mut fonts = acme_text::FontSystem::new();
        let snapshot = LayoutEngine::new()
            .compute_with_text(&layout, (800.0, 600.0), &mut fonts, 1.0)
            .unwrap();
        let rect = snapshot.get(NodeId::new(1)).unwrap();
        assert!(rect.width > 0.0, "empty_state width should be > 0");
        assert!(rect.height > 0.0, "empty_state height should be > 0");
    }

    #[test]
    fn empty_state_displays_label_text() {
        let node: WidgetNode<TestMsg> = empty_state("No items")
            .description("Try adding some items.")
            .icon(IconName::Folder)
            .build();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column variant");
        };
        // First child is icon, second is title, third is description
        assert_eq!(c.children.len(), 3);
        let WidgetNode::Label(l) = &c.children[1] else {
            panic!("expected Label as second child");
        };
        assert_eq!(l.text, "No items");
    }
}
