//! Accordion component — collapsible sections with title Button and content.

use crate::WidgetNode;

/// A single accordion section.
pub struct AccordionSection<M> {
    pub title: String,
    pub open: bool,
    pub content: WidgetNode<M>,
}

impl<M> AccordionSection<M> {
    /// Create a new accordion section.
    pub fn new(title: impl Into<String>, content: WidgetNode<M>) -> Self {
        Self {
            title: title.into(),
            open: false,
            content,
        }
    }

    /// Set the section to be open by default.
    pub fn open(mut self) -> Self {
        self.open = true;
        self
    }
}

/// Builder for an accordion widget.
pub struct AccordionBuilder<M> {
    pub sections: Vec<AccordionSection<M>>,
}

/// Create an accordion builder.
pub fn accordion<M>() -> AccordionBuilder<M> {
    AccordionBuilder { sections: vec![] }
}

impl<M: Clone + 'static> AccordionBuilder<M> {
    /// Add a section to the accordion.
    pub fn section(mut self, section: AccordionSection<M>) -> Self {
        self.sections.push(section);
        self
    }

    /// Build the accordion widget.
    pub fn build(self) -> WidgetNode<M> {
        let mut col = crate::column().gap(0.0);
        for section in self.sections {
            let title_btn = crate::button("accordion-title", &section.title);
            col = col.child(title_btn);
            if section.open {
                col = col.child(section.content);
            }
        }
        col.build()
    }
}

impl<M: Clone + 'static> From<AccordionBuilder<M>> for WidgetNode<M> {
    fn from(b: AccordionBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn accordion_has_non_zero_layout_rect() {
        let content = crate::label("Details go here");
        let section = AccordionSection::new("Section 1", content).open();
        let node: WidgetNode<TestMsg> = accordion().section(section).build();
        let ctx = test_context();
        let layout = node.to_layout_with_context(NodeId::new(1), &ctx);
        let mut fonts = acme_text::FontSystem::new();
        let snapshot = LayoutEngine::new()
            .compute_with_text(&layout, (800.0, 600.0), &mut fonts, 1.0)
            .unwrap();
        let rect = snapshot.get(NodeId::new(1)).unwrap();
        assert!(rect.width > 0.0, "accordion width should be > 0");
        assert!(rect.height > 0.0, "accordion height should be > 0");
    }

    #[test]
    fn accordion_displays_label_text() {
        let content = crate::label("Details go here");
        let section = AccordionSection::new("Section 1", content).open();
        let node: WidgetNode<TestMsg> = accordion().section(section).build();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column variant");
        };
        // Title button + open content = 2 children
        assert_eq!(c.children.len(), 2);
    }

    #[test]
    fn accordion_closed_section_hides_content() {
        let content = crate::label("Hidden content");
        let section = AccordionSection::new("Section 1", content);
        let node: WidgetNode<TestMsg> = accordion().section(section).build();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column variant");
        };
        // Only the title button, content is hidden
        assert_eq!(c.children.len(), 1);
    }
}
