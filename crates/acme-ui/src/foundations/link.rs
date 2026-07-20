//! Link component — a styled text element that looks like a hyperlink.
//!
//! Renders as a primary-colored Label when no click handler is provided,
//! or as a Ghost-variant Button when `on_click` is set.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a link widget.
pub struct LinkBuilder<M> {
    pub id: WidgetKey,
    pub text: String,
    pub underline: bool,
    pub disabled: bool,
    pub on_click: Option<M>,
}

/// Create a link builder.
///
/// Defaults: primary color, no underline, not disabled, no click handler.
pub fn link<M>(id: impl Into<WidgetKey>, text: impl Into<String>) -> LinkBuilder<M> {
    LinkBuilder {
        id: id.into(),
        text: text.into(),
        underline: false,
        disabled: false,
        on_click: None,
    }
}

impl<M: Clone + 'static> LinkBuilder<M> {
    /// Add underline decoration to the link text.
    pub fn underline(mut self, value: bool) -> Self {
        self.underline = value;
        self
    }

    /// Mark the link as disabled (greyed out, non-interactive).
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    /// Set the click handler. When set, the link renders as a Ghost-variant
    /// button to support click interaction.
    pub fn on_click(mut self, message: M) -> Self {
        self.on_click = Some(message);
        self
    }

    /// Build the link widget node.
    ///
    /// - When `on_click` is `Some`: produces a `WidgetNode::Button` with
    ///   `ButtonVariant::Ghost` and primary foreground color.
    /// - When `on_click` is `None`: produces a `WidgetNode::Label` with
    ///   primary foreground color.
    pub fn build(self) -> WidgetNode<M> {
        let theme = acme_theme::Theme::light();
        let primary_color = theme.colors.primary;

        match self.on_click {
            Some(msg) => {
                // Interactive link → Button with Ghost variant
                crate::button(self.id, self.text.as_str())
                    .variant(crate::ButtonVariant::Ghost)
                    .disabled(self.disabled)
                    .on_click(msg)
            }
            None => {
                // Static link → Label with primary color
                let mut lbl = crate::label_builder::<M>(self.text.as_str());
                if self.disabled {
                    lbl = lbl.color(theme.colors.disabled_text);
                } else {
                    lbl = lbl.color(primary_color);
                }
                lbl.build()
            }
        }
    }
}

impl<M: Clone + 'static> From<LinkBuilder<M>> for WidgetNode<M> {
    fn from(b: LinkBuilder<M>) -> Self {
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
    enum TestMsg {
        Clicked,
    }

    #[test]
    fn link_builder_defaults() {
        let builder = link::<TestMsg>("home", "Home");
        assert!(!builder.underline, "underline should default to false");
        assert!(!builder.disabled, "disabled should default to false");
        assert!(builder.on_click.is_none(), "on_click should default to None");
        assert_eq!(builder.id.as_str(), "home");
        assert_eq!(builder.text, "Home");
    }

    #[test]
    fn link_with_underline() {
        let builder = link::<TestMsg>("about", "About Us").underline(true);
        assert!(builder.underline);
    }

    #[test]
    fn link_disabled_flag() {
        let builder = link::<TestMsg>("del", "Delete").disabled(true);
        assert!(builder.disabled);
    }

    #[test]
    fn link_without_on_click_creates_label() {
        let node: WidgetNode<TestMsg> = link("home", "Home").build();
        let WidgetNode::Label(l) = &node else {
            panic!("expected Label variant when no on_click");
        };
        assert_eq!(l.text, "Home");
        assert!(l.color.is_some(), "label should have a primary color set");
    }

    #[test]
    fn link_with_on_click_creates_button() {
        let node: WidgetNode<TestMsg> =
            link("about", "About Us").on_click(TestMsg::Clicked).build();
        let WidgetNode::Button(b) = &node else {
            panic!("expected Button variant when on_click is set");
        };
        assert_eq!(b.label, "About Us");
        assert_eq!(b.key.as_str(), "about");
        assert_eq!(b.variant, ButtonVariant::Ghost);
        assert!(!b.disabled);
    }

    #[test]
    fn link_disabled_propagates_to_button() {
        let node: WidgetNode<TestMsg> =
            link("del", "Delete").disabled(true).on_click(TestMsg::Clicked).build();
        let WidgetNode::Button(b) = &node else {
            panic!("expected Button variant");
        };
        assert!(b.disabled, "disabled should propagate to Button");
    }

    #[test]
    fn link_from_conversion() {
        let node: WidgetNode<TestMsg> = link("home", "Home").into();
        let WidgetNode::Label(l) = &node else {
            panic!("expected Label variant via From conversion");
        };
        assert_eq!(l.text, "Home");
    }

    #[test]
    fn link_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = link("home", "Home").build();
        let ctx = test_context();
        let layout = node.to_layout_with_context(NodeId::new(1), &ctx);
        let mut fonts = acme_text::FontSystem::new();
        let snapshot = LayoutEngine::new()
            .compute_with_text(&layout, (800.0, 600.0), &mut fonts, 1.0)
            .unwrap();
        let rect = snapshot.get(NodeId::new(1)).unwrap();
        assert!(rect.width > 0.0, "link width should be > 0");
        assert!(rect.height > 0.0, "link height should be > 0");
    }
}
