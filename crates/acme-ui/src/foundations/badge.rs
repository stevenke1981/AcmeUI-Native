//! Badge component — a compact pill-shaped label with tone-based coloring.
//!
//! V2 design: uses `crate::Tone` for semantic color, `crate::ControlSize` for sizing,
//! supports soft (default) and solid (`outlined()`) visual variants.

use crate::WidgetNode;

/// Builder for a badge widget.
pub struct BadgeBuilder<M> {
    pub label: String,
    pub tone: crate::Tone,
    pub size: crate::ControlSize,
    pub outlined: bool,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a badge builder. Defaults to Neutral tone, XS size, soft variant.
pub fn badge<M>(label: impl Into<String>) -> BadgeBuilder<M> {
    BadgeBuilder {
        label: label.into(),
        tone: crate::Tone::Neutral,
        size: crate::ControlSize::Xs,
        outlined: false,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> BadgeBuilder<M> {
    /// Set the tone to Primary.
    pub fn primary(mut self) -> Self {
        self.tone = crate::Tone::Primary;
        self
    }

    /// Set the tone to Success.
    pub fn success(mut self) -> Self {
        self.tone = crate::Tone::Success;
        self
    }

    /// Set the tone to Warning.
    pub fn warning(mut self) -> Self {
        self.tone = crate::Tone::Warning;
        self
    }

    /// Set the tone to Danger.
    pub fn danger(mut self) -> Self {
        self.tone = crate::Tone::Danger;
        self
    }

    /// Set the tone explicitly.
    pub fn tone(mut self, tone: crate::Tone) -> Self {
        self.tone = tone;
        self
    }

    /// Set size to XS (default badge size).
    pub fn xs(mut self) -> Self {
        self.size = crate::ControlSize::Xs;
        self
    }

    /// Set size to Small (ControlSize::Sm).
    pub fn small(mut self) -> Self {
        self.size = crate::ControlSize::Sm;
        self
    }

    /// Set size to Medium (ControlSize::Md).
    pub fn medium(mut self) -> Self {
        self.size = crate::ControlSize::Md;
        self
    }

    /// Set size to Large (ControlSize::Lg).
    pub fn large(mut self) -> Self {
        self.size = crate::ControlSize::Lg;
        self
    }

    /// Switch to the solid (filled) visual variant.
    /// By default badges use the soft variant.
    pub fn outlined(mut self) -> Self {
        self.outlined = true;
        self
    }

    /// Build the badge widget.
    ///
    /// Produces a `Card` container with:
    /// - Background color from the resolved tone (soft or solid)
    /// - Label with the tone's foreground color and caption font size
    /// - Pill-shaped border radius
    /// - Padding proportional to the badge size
    pub fn build(self) -> WidgetNode<M> {
        let padding = match self.size {
            crate::ControlSize::Xs => 2.0,
            crate::ControlSize::Sm => 3.0,
            crate::ControlSize::Md => 4.0,
            crate::ControlSize::Lg => 6.0,
            crate::ControlSize::Xl => 8.0,
        };

        // Resolve tone colors using a reference theme for value resolution.
        // Colors are resolved as ThemeColor values (RGBA floats) from the
        // design system's V2 token palette.
        let theme = acme_theme::Theme::light();
        let resolved = crate::resolve_tone(&theme, self.tone, true);

        let (bg, fg) = if self.outlined {
            // Solid variant: use the solid bg/fg pair
            (resolved.bg, resolved.fg)
        } else {
            // Soft variant: use the soft bg/fg pair, falling back to solid
            (
                resolved.soft_bg.unwrap_or(resolved.bg),
                resolved.soft_fg.unwrap_or(resolved.fg),
            )
        };

        // Pill shape
        let radius = theme.radii.full;

        crate::card()
            .child(
                crate::label_builder(&self.label)
                    .font_size(theme.typography.caption)
                    .color(fg)
                    .build(),
            )
            .padding(padding)
            .gap(0.0)
            .variant(crate::CardVariant::Plain)
            .background_color(bg)
            .border_radius(radius)
            .build()
    }
}

impl<M: Clone + 'static> From<BadgeBuilder<M>> for WidgetNode<M> {
    fn from(b: BadgeBuilder<M>) -> Self {
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
    fn badge_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = badge("New").primary().build();
        let ctx = test_context();
        let layout = node.to_layout_with_context(NodeId::new(1), &ctx);
        let snapshot = LayoutEngine::new()
            .compute(&layout, (800.0, 600.0))
            .unwrap();
        let rect = snapshot.get(NodeId::new(1)).unwrap();
        assert!(rect.width > 0.0, "badge width should be > 0");
        assert!(rect.height > 0.0, "badge height should be > 0");
    }

    #[test]
    fn badge_displays_label_text() {
        let node: WidgetNode<TestMsg> = badge("New").primary().build();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        assert_eq!(c.children.len(), 1);
        let WidgetNode::Label(l) = &c.children[0] else {
            panic!("expected Label child");
        };
        assert_eq!(l.text, "New");
    }

    #[test]
    fn badge_soft_uses_plain_variant() {
        let node: WidgetNode<TestMsg> = badge("Beta").build();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        // Soft (default) → Plain variant
        assert_eq!(c.variant, CardVariant::Plain);
    }

    #[test]
    fn badge_sets_background_and_label_color() {
        let node: WidgetNode<TestMsg> = badge("3").danger().build();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        assert!(c.background_color.is_some(), "background_color should be set");
        assert!(c.border_radius.is_some(), "border_radius should be set");
        let WidgetNode::Label(l) = &c.children[0] else {
            panic!("expected Label child");
        };
        assert!(l.color.is_some(), "label color should be set");
        assert_eq!(l.font_size, Some(12.0), "badge label should use caption size");
    }

    #[test]
    fn badge_solid_resolves_tone_colors() {
        let node: WidgetNode<TestMsg> = badge("99+").success().outlined().build();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        assert!(c.background_color.is_some(), "solid badge must have background_color");
        let WidgetNode::Label(l) = &c.children[0] else {
            panic!("expected Label child");
        };
        assert!(l.color.is_some(), "solid badge label must have color");
    }

    #[test]
    fn badge_tone_methods_compile() {
        let _ = badge::<TestMsg>("x").primary().build();
        let _ = badge::<TestMsg>("x").success().build();
        let _ = badge::<TestMsg>("x").warning().build();
        let _ = badge::<TestMsg>("x").danger().build();
        let _ = badge::<TestMsg>("x").tone(Tone::Info).build();
    }

    #[test]
    fn badge_from_conversion() {
        let node: WidgetNode<TestMsg> = badge("converted").into();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        assert!(c.background_color.is_some());
    }
}
