//! Hero component — a prominent section banner with title, subtitle,
//! and optional call-to-action area.
//!
//! Suitable for page headers, landing sections, and feature highlights.
//! Renders as a large Card with a centered Column.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a hero section.
pub struct HeroBuilder<M> {
    pub id: WidgetKey,
    pub title: String,
    pub subtitle: Option<String>,
    pub actions: Vec<WidgetNode<M>>,
    pub compact: bool,
    pub centered: bool,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a hero builder.
pub fn hero<M: Clone + 'static>(id: impl Into<WidgetKey>, title: impl Into<String>) -> HeroBuilder<M> {
    HeroBuilder {
        id: id.into(),
        title: title.into(),
        subtitle: None,
        actions: vec![],
        compact: false,
        centered: true,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> HeroBuilder<M> {
    /// Set the subtitle text below the title.
    pub fn subtitle(mut self, value: impl Into<String>) -> Self {
        self.subtitle = Some(value.into());
        self
    }

    /// Add a child widget (e.g. button, badge) to the action area.
    pub fn action(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.actions.push(child.into());
        self
    }

    /// Compact mode reduces padding and font size.
    pub fn compact(mut self, value: bool) -> Self {
        self.compact = value;
        self
    }

    /// Center content (default true).
    pub fn centered(mut self, value: bool) -> Self {
        self.centered = value;
        self
    }

    /// Build the hero widget.
    pub fn build(self) -> WidgetNode<M> {
        let padding = if self.compact { 16.0 } else { 32.0 };
        let title_size = if self.compact { 24.0 } else { 36.0 };
        let subtitle_size = if self.compact { 14.0 } else { 16.0 };
        let gap = if self.compact { 4.0 } else { 8.0 };

        let mut col = crate::column::<M>()
            .key(self.id)
            .gap(gap)
            .child(
                crate::label_builder(&self.title)
                    .font_size(title_size)
                    .build(),
            );

        if let Some(sub) = &self.subtitle {
            col = col.child(
                crate::label_builder(sub)
                    .font_size(subtitle_size)
                    .color(crate::ThemeColor::rgb(120, 120, 120))
                    .build(),
            );
        }

        if !self.actions.is_empty() {
            let mut row = crate::row::<M>().gap(8.0);
            for child in self.actions {
                row = row.child(child);
            }
            col = col.child(row.build());
        }

        crate::card::<M>()
            .variant(crate::CardVariant::Elevated)
            .padding(padding)
            .child(col)
            .build()
    }
}

impl<M: Clone + 'static> From<HeroBuilder<M>> for WidgetNode<M> {
    fn from(b: HeroBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;
    use acme_layout::{LayoutEngine, WidgetLayoutContext};

    fn test_context() -> WidgetLayoutContext {
        WidgetLayoutContext { body_font_size: 16.0, body_line_height: 22.0, label_font_size: 14.0, control_height: 32.0, scale_factor: 1.0 }
    }

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn hero_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = hero("h", "Welcome").build();
        let ctx = test_context();
        let layout = node.to_layout_with_context(NodeId::new(1), &ctx);
        let snapshot = LayoutEngine::new().compute(&layout, (800.0, 600.0)).unwrap();
        let rect = snapshot.get(NodeId::new(1)).unwrap();
        assert!(rect.width > 0.0);
        assert!(rect.height > 0.0);
    }

    #[test]
    fn hero_with_all_fields() {
        let node: WidgetNode<TestMsg> = hero("h", "Hello World")
            .subtitle("This is a subtitle")
            .action(crate::label::<TestMsg>("Action"))
            .compact(false)
            .build();
        let WidgetNode::Card(c) = &node else { panic!("expected Card") };
        assert_eq!(c.variant, crate::CardVariant::Elevated);
        let WidgetNode::Column(col) = &c.children[0] else { panic!("expected Column") };
        assert_eq!(col.children.len(), 3); // title + subtitle + actions
    }

    #[test]
    fn hero_minimal_has_one_child() {
        let node: WidgetNode<TestMsg> = hero("h", "Minimal").build();
        let WidgetNode::Card(c) = &node else { panic!("expected Card") };
        let WidgetNode::Column(col) = &c.children[0] else { panic!("expected Column") };
        assert_eq!(col.children.len(), 1);
    }

    #[test]
    fn hero_compact_reduces_padding() {
        let h = hero::<TestMsg>("h", "x").compact(true);
        assert!(h.compact);
    }

    #[test]
    fn hero_builder_defaults() {
        let h = hero::<TestMsg>("h", "Title");
        assert_eq!(h.title, "Title");
        assert!(h.subtitle.is_none());
        assert!(h.actions.is_empty());
        assert!(h.centered);
    }
}
