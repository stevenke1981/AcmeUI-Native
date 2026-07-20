//! Alert component — a Card with Icon + Title + optional Description and close Button.
//!
//! Uses the V2 tone system for semantic background coloring.

use crate::WidgetNode;
use acme_theme::Theme;

/// Builder for an alert widget.
pub struct AlertBuilder<M> {
    pub title: String,
    pub description: Option<String>,
    pub tone: crate::Tone,
    pub dismissible: bool,
    _phantom: std::marker::PhantomData<M>,
}

/// Create an alert builder.
pub fn alert<M>(title: impl Into<String>) -> AlertBuilder<M> {
    AlertBuilder {
        title: title.into(),
        description: None,
        tone: crate::Tone::Neutral,
        dismissible: false,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> AlertBuilder<M> {
    /// Set the alert tone.
    pub fn tone(mut self, tone: crate::Tone) -> Self {
        self.tone = tone;
        self
    }

    /// Convenience: set tone to Primary.
    pub fn primary(mut self) -> Self {
        self.tone = crate::Tone::Primary;
        self
    }

    /// Convenience: set tone to Success.
    pub fn success(mut self) -> Self {
        self.tone = crate::Tone::Success;
        self
    }

    /// Convenience: set tone to Warning.
    pub fn warning(mut self) -> Self {
        self.tone = crate::Tone::Warning;
        self
    }

    /// Convenience: set tone to Danger.
    pub fn danger(mut self) -> Self {
        self.tone = crate::Tone::Danger;
        self
    }

    /// Convenience: set tone to Info.
    pub fn info(mut self) -> Self {
        self.tone = crate::Tone::Info;
        self
    }

    /// Set an optional description text shown below the title.
    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }

    /// Make the alert dismissible (shows a close button).
    pub fn dismissible(mut self) -> Self {
        self.dismissible = true;
        self
    }

    /// Build the alert widget using the V2 tone system.
    ///
    /// Resolves the tone's soft background via `crate::resolve_tone`
    /// and applies it as the Card's background color.
    pub fn build(self, theme: &Theme) -> WidgetNode<M> {
        // Resolve tone colors — use soft variant for muted backgrounds
        let tone_colors = crate::resolve_tone(theme, self.tone, false);

        let icon_name = match self.tone {
            crate::Tone::Neutral => super::IconName::Info,
            crate::Tone::Primary => super::IconName::Info,
            crate::Tone::Success => super::IconName::Success,
            crate::Tone::Warning => super::IconName::Warning,
            crate::Tone::Danger => super::IconName::Error,
            crate::Tone::Info => super::IconName::Info,
        };

        // Build inner content column: title row + optional description
        let mut title_row = crate::row()
            .gap(8.0)
            .child(super::icon(icon_name).size(16.0))
            .child(crate::label(&self.title));

        if self.dismissible {
            title_row = title_row.child(crate::button("alert-close", "✕"));
        }

        let mut inner = crate::column().gap(4.0).child(title_row.build());

        if let Some(ref desc) = self.description {
            inner = inner.child(crate::label(desc));
        }

        // Wrap in a Card with tone-based background
        let mut card = crate::card()
            .gap(4.0)
            .padding(12.0)
            .child(inner.build());

        if let Some(bg) = tone_colors.soft_bg {
            card = card.background_color(bg);
        }

        card.build()
    }
}

impl<M: Clone + 'static> AlertBuilder<M> {
    /// Build without a theme (uses default resolution).
    /// Prefer `build(&theme)` when a `Theme` is available for proper V2 colors.
    pub fn build_default(self) -> WidgetNode<M> {
        let theme = acme_theme::Theme::light();
        self.build(&theme)
    }
}

impl<M: Clone + 'static> From<AlertBuilder<M>> for WidgetNode<M> {
    fn from(b: AlertBuilder<M>) -> Self {
        b.build(&Theme::light())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use acme_core::NodeId;
    use acme_layout::{LayoutEngine, WidgetLayoutContext};
    use acme_theme::Theme;

    fn test_context() -> WidgetLayoutContext {
        WidgetLayoutContext {
            body_font_size: 16.0,
            body_line_height: 22.0,
            label_font_size: 14.0,
            control_height: 32.0,
            scale_factor: 1.0,
        }
    }

    fn test_theme() -> Theme {
        Theme::light()
    }

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn alert_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = alert("Something happened").build(&test_theme());
        let ctx = test_context();
        let layout = node.to_layout_with_context(NodeId::new(1), &ctx);
        let mut fonts = acme_text::FontSystem::new();
        let snapshot = LayoutEngine::new()
            .compute_with_text(&layout, (800.0, 600.0), &mut fonts, 1.0)
            .unwrap();
        let rect = snapshot.get(NodeId::new(1)).unwrap();
        assert!(rect.width > 0.0, "alert width should be > 0");
        assert!(rect.height > 0.0, "alert height should be > 0");
    }

    #[test]
    fn alert_displays_title_text() {
        let node: WidgetNode<TestMsg> = alert("Something happened").build(&test_theme());
        // Alert builds into a Card wrapping a Column
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        // Card children: one Column (title row + optional description)
        assert_eq!(c.children.len(), 1);
        let WidgetNode::Column(col) = &c.children[0] else {
            panic!("expected Column child");
        };
        // Column children: title row (built Row node)
        assert!(!col.children.is_empty());
        let WidgetNode::Row(r) = &col.children[0] else {
            panic!("expected Row as first child of Column");
        };
        // First child is icon, second is label
        assert_eq!(r.children.len(), 2);
        let WidgetNode::Label(l) = &r.children[1] else {
            panic!("expected Label as second child");
        };
        assert_eq!(l.text, "Something happened");
    }

    #[test]
    fn alert_dismissible_has_close_button() {
        let node: WidgetNode<TestMsg> = alert("Error").dismissible().build(&test_theme());
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        let WidgetNode::Column(col) = &c.children[0] else {
            panic!("expected Column child");
        };
        let WidgetNode::Row(r) = &col.children[0] else {
            panic!("expected Row as first child of Column");
        };
        assert_eq!(r.children.len(), 3);
        let WidgetNode::Button(_) = &r.children[2] else {
            panic!("expected Button as third child");
        };
    }

    #[test]
    fn alert_tone_maps_to_icon() {
        let theme = test_theme();
        let node: WidgetNode<TestMsg> = alert("Test").success().build(&theme);
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        assert!(c.background_color.is_some(), "success tone should set background_color");
    }

    #[test]
    fn alert_neutral_has_no_soft_background() {
        let theme = test_theme();
        let node: WidgetNode<TestMsg> = alert("Test").build(&theme);
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        // Neutral has no soft_bg, so background_color should be None
        assert!(c.background_color.is_none(), "Neutral tone should have no background_color override");
    }

    #[test]
    fn alert_with_description() {
        let node: WidgetNode<TestMsg> = alert("Title")
            .description("This is a description.")
            .build(&test_theme());
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        let WidgetNode::Column(col) = &c.children[0] else {
            panic!("expected Column child");
        };
        // Column should have 2 children: title row + description label
        assert_eq!(col.children.len(), 2);
        let WidgetNode::Label(_) = &col.children[1] else {
            panic!("expected Label as second child of Column");
        };
    }

    #[test]
    fn alert_default_tone() {
        let b = alert::<TestMsg>("Hi");
        assert_eq!(b.tone, Tone::Neutral);
    }

    #[test]
    fn alert_from_trait_uses_default_theme() {
        let node: WidgetNode<TestMsg> = alert("Hello").into();
        // The From impl calls build_default which uses Theme::light()
        let WidgetNode::Card(_) = &node else {
            panic!("expected Card variant");
        };
    }
}
