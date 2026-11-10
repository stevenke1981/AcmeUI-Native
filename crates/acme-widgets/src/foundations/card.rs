use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_layout::{Edges, LayoutKind, LayoutStyle};
use acme_style::Style;
use acme_style::prelude::*;
use acme_theme::ThemeColor;

/// Card style variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CardVariant {
    #[default]
    Plain,
    Outlined,
    Elevated,
    Interactive,
    Muted,
}

/// A card container widget.
#[derive(Clone, Debug, PartialEq)]
pub struct Card<M> {
    pub key: Option<WidgetKey>,
    pub variant: CardVariant,
    pub children: Vec<WidgetNode<M>>,
    pub gap: f32,
    pub padding: Edges,
    /// Optional explicit background color override.
    /// When `None`, the renderer derives the color from `variant`.
    pub background_color: Option<ThemeColor>,
    /// Optional explicit border radius override.
    /// When `None`, the renderer uses the default card radius.
    pub border_radius: Option<f32>,
    /// Accumulated GPUI‑inspired / Tailwind‑style styling.
    pub style: Style,
}

impl<M> Card<M> {
    pub(crate) fn layout(&self) -> LayoutStyle {
        let mut layout = LayoutStyle {
            kind: LayoutKind::Column,
            gap: self.gap,
            padding: self.padding,
            ..Default::default()
        };
        self.style.clone().apply_to_layout(&mut layout);
        layout
    }
}

/// Builder for Card widgets.
pub struct CardBuilder<M> {
    card: Card<M>,
}
impl<M> CardBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.card.key = Some(key.into());
        self
    }
    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.card.children.push(child.into());
        self
    }
    pub fn gap(mut self, value: f32) -> Self {
        self.card.gap = crate::finite(value);
        self
    }
    pub fn padding(mut self, value: f32) -> Self {
        self.card.padding = Edges::all(value);
        self
    }
    pub fn variant(mut self, value: CardVariant) -> Self {
        self.card.variant = value;
        self
    }
    /// Set an explicit background color override.
    pub fn background_color(mut self, color: ThemeColor) -> Self {
        self.card.background_color = Some(color);
        self
    }
    /// Set an explicit border radius override.
    pub fn border_radius(mut self, radius: f32) -> Self {
        self.card.border_radius = Some(crate::finite(radius));
        self
    }
    pub fn build(self) -> WidgetNode<M> {
        WidgetNode::Card(self.card)
    }
}
impl<M> From<CardBuilder<M>> for WidgetNode<M> {
    fn from(value: CardBuilder<M>) -> Self {
        value.build()
    }
}

/// Create a card widget builder.
pub fn card<M>() -> CardBuilder<M> {
    CardBuilder {
        card: Card {
            key: None,
            variant: CardVariant::Plain,
            children: vec![],
            gap: 0.0,
            padding: Edges::default(),
            background_color: None,
            border_radius: None,
            style: Style::new(),
        },
    }
}

impl<M> Styled for CardBuilder<M> {
    fn style(&self) -> &Style {
        &self.card.style
    }
    fn style_mut(&mut self) -> &mut Style {
        &mut self.card.style
    }
}

#[cfg(test)]
mod layout_tests {
    use super::*;
    use acme_core::NodeId;
    use acme_layout::{LayoutEngine, LayoutKind, Length, WidgetLayoutContext};

    #[test]
    fn card_existing_style_is_honoured_in_both_layout_paths_and_empty_style_keeps_defaults() {
        let node = card::<()>()
            .padding(8.0)
            .gap(4.0)
            .w(Length::px(180.0))
            .h(Length::px(120.0))
            .min_w(Length::px(100.0))
            .max_w(Length::px(240.0))
            .p(10.0)
            .flex_shrink(0.0)
            .build();
        let context = WidgetLayoutContext {
            body_font_size: 14.0,
            body_line_height: 20.0,
            label_font_size: 13.0,
            control_height: 36.0,
            scale_factor: 1.0,
        };
        for layout in [
            node.to_layout(NodeId::new(1)),
            node.to_layout_with_context(NodeId::new(1), &context),
        ] {
            assert_eq!(layout.style.width, Length::px(180.0));
            assert_eq!(layout.style.height, Length::px(120.0));
            assert_eq!(layout.style.min_width, Length::px(100.0));
            assert_eq!(layout.style.max_width, Length::px(240.0));
            assert_eq!(layout.style.padding, Edges::all(10.0));
            assert_eq!(layout.style.gap, 4.0);
            assert_eq!(layout.style.flex_shrink, 0.0);
            let snapshot = LayoutEngine::new()
                .compute(&layout, (400.0, 300.0))
                .unwrap();
            let r = snapshot.get(layout.id).unwrap();
            assert_eq!((r.width, r.height), (180.0, 120.0));
        }
        let plain = card::<()>()
            .padding(8.0)
            .gap(4.0)
            .build()
            .to_layout(NodeId::new(1));
        assert_eq!(plain.style.kind, LayoutKind::Column);
        assert_eq!(plain.style.padding, Edges::all(8.0));
        assert_eq!(plain.style.gap, 4.0);
        assert_eq!(plain.style.width, Length::Auto);
    }
}
