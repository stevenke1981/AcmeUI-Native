//! Section component — a titled content section with optional description
//! and actions area. Used to group related content on a page.
//!
//! Renders as a Card with a header (title + optional actions) and a body area.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a page section.
pub struct SectionBuilder<M> {
    pub id: WidgetKey,
    pub title: String,
    pub description: Option<String>,
    pub actions: Vec<WidgetNode<M>>,
    pub children: Vec<WidgetNode<M>>,
    pub collapsible: bool,
    pub borderless: bool,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a section builder.
pub fn section<M: Clone + 'static>(
    id: impl Into<WidgetKey>,
    title: impl Into<String>,
) -> SectionBuilder<M> {
    SectionBuilder {
        id: id.into(),
        title: title.into(),
        description: None,
        actions: vec![],
        children: vec![],
        collapsible: false,
        borderless: false,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> SectionBuilder<M> {
    /// Set the section description (rendered as muted subtitle).
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Add an action widget (button, link, etc.) to the section header.
    pub fn action(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.actions.push(child.into());
        self
    }

    /// Add a child content widget to the section body.
    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Make the section collapsible (header-only for now — expand/collapse
    /// to be controlled by parent state).
    pub fn collapsible(mut self, value: bool) -> Self {
        self.collapsible = value;
        self
    }

    /// Remove the Card border for a flat appearance.
    pub fn borderless(mut self, value: bool) -> Self {
        self.borderless = value;
        self
    }

    /// Build the section widget.
    pub fn build(self) -> WidgetNode<M> {
        let mut content = crate::column::<M>().key(self.id.clone()).gap(12.0);

        // Header row: title + optional actions
        let mut header = crate::row::<M>()
            .gap(8.0)
            .child(crate::label_builder(&self.title).font_size(18.0).build());

        if !self.actions.is_empty() {
            let mut action_row = crate::row::<M>().gap(6.0);
            for a in self.actions {
                action_row = action_row.child(a);
            }
            header = header.child(action_row.build());
        }

        content = content.child(header.build());

        // Optional description
        if let Some(desc) = &self.description {
            content = content.child(
                crate::label_builder(desc)
                    .font_size(14.0)
                    .color(crate::ThemeColor::rgb(120, 120, 120))
                    .build(),
            );
        }

        // Body children
        for child in self.children {
            content = content.child(child);
        }

        let variant = if self.borderless {
            crate::CardVariant::Plain
        } else {
            crate::CardVariant::Outlined
        };

        crate::card::<M>()
            .variant(variant)
            .padding(16.0)
            .child(content)
            .build()
    }
}

impl<M: Clone + 'static> From<SectionBuilder<M>> for WidgetNode<M> {
    fn from(b: SectionBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn section_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = section("s", "Settings").build();
        let ctx = test_context();
        let layout = node.to_layout_with_context(NodeId::new(1), &ctx);
        let snapshot = LayoutEngine::new()
            .compute(&layout, (800.0, 600.0))
            .unwrap();
        let rect = snapshot.get(NodeId::new(1)).unwrap();
        assert!(rect.width > 0.0);
        assert!(rect.height > 0.0);
    }

    #[test]
    fn section_with_title_and_description() {
        let node: WidgetNode<TestMsg> = section("s", "Profile")
            .description("Manage your profile settings")
            .child(crate::label::<TestMsg>("Content"))
            .build();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card")
        };
        assert_eq!(c.variant, crate::CardVariant::Outlined);
        let WidgetNode::Column(col) = &c.children[0] else {
            panic!("expected Column")
        };
        // header row + desc + 1 child = 3
        assert_eq!(col.children.len(), 3);
    }

    #[test]
    fn section_minimal() {
        let node: WidgetNode<TestMsg> = section("s", "Minimal").build();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card")
        };
        let WidgetNode::Column(col) = &c.children[0] else {
            panic!("expected Column")
        };
        assert_eq!(col.children.len(), 1); // just header row
    }

    #[test]
    fn section_borderless_uses_plain_variant() {
        let node: WidgetNode<TestMsg> = section("s", "Flat").borderless(true).build();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card")
        };
        assert_eq!(c.variant, crate::CardVariant::Plain);
    }

    #[test]
    fn section_collapsible_flag() {
        let s = section::<TestMsg>("s", "Collapse").collapsible(true);
        assert!(s.collapsible);
    }

    #[test]
    fn section_builder_defaults() {
        let s = section::<TestMsg>("s", "Default");
        assert_eq!(s.title, "Default");
        assert!(s.description.is_none());
        assert!(s.children.is_empty());
    }
}
