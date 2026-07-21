//! Banner component — a top-of-page Card for persistent notifications or announcements.
//!
//! Dismissible, with tone-based coloring and optional action buttons.
//! Uses the V2 tone system for semantic background coloring (soft variant).

use crate::WidgetNode;
use acme_theme::Theme;

/// Builder for a banner widget.
pub struct BannerBuilder<M> {
    message: String,
    description: Option<String>,
    tone: crate::Tone,
    dismissible: bool,
    icon_override: Option<crate::IconName>,
    actions: Vec<(String, M)>,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a banner builder.
pub fn banner<M>(message: impl Into<String>) -> BannerBuilder<M> {
    BannerBuilder {
        message: message.into(),
        description: None,
        tone: crate::Tone::Neutral,
        dismissible: false,
        icon_override: None,
        actions: Vec::new(),
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> BannerBuilder<M> {
    /// Set an optional description text shown below the message.
    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }

    /// Set the banner tone.
    pub fn tone(mut self, tone: crate::Tone) -> Self {
        self.tone = tone;
        self
    }

    /// Make the banner dismissible (shows a close button).
    pub fn dismissible(mut self) -> Self {
        self.dismissible = true;
        self
    }

    /// Override the default icon derived from the tone.
    pub fn icon(mut self, name: crate::IconName) -> Self {
        self.icon_override = Some(name);
        self
    }

    /// Add an action button with a text label and a message.
    pub fn action(mut self, label: impl Into<String>, msg: M) -> Self {
        self.actions.push((label.into(), msg));
        self
    }

    /// Build the banner widget using the V2 tone system.
    ///
    /// Resolves the tone's soft background via `crate::resolve_tone`
    /// and applies it as the Card's background color.
    pub fn build(self, theme: &Theme) -> WidgetNode<M> {
        // Resolve tone colors — use soft variant for muted backgrounds
        let tone_colors = crate::resolve_tone(theme, self.tone, false);

        let icon_name = self.icon_override.unwrap_or(match self.tone {
            crate::Tone::Neutral => crate::IconName::Info,
            crate::Tone::Primary => crate::IconName::Info,
            crate::Tone::Success => crate::IconName::Success,
            crate::Tone::Warning => crate::IconName::Warning,
            crate::Tone::Danger => crate::IconName::Error,
            crate::Tone::Info => crate::IconName::Info,
        });

        // Content column: message label + optional description
        let content_col = {
            let mut col = crate::column().gap(4.0).child(crate::label(&self.message));
            if let Some(ref desc) = self.description {
                col = col.child(crate::label(desc));
            }
            col.build()
        };

        // Main row: icon + content column + action buttons + close button
        let mut main_row = crate::row()
            .gap(8.0)
            .child(crate::icon(icon_name).size(20.0))
            .child(content_col);

        // Action buttons
        for (i, (label, msg)) in self.actions.iter().enumerate() {
            let key = format!("banner-action-{i}");
            main_row = main_row.child(crate::button(key, label).on_click(msg.clone()));
        }

        // Optional close button
        if self.dismissible {
            main_row = main_row.child(crate::button("banner-close", "✕"));
        }

        // Wrap in a Card with tone-based background
        let mut card = crate::card().gap(0.0).padding(12.0).child(main_row.build());

        if let Some(bg) = tone_colors.soft_bg {
            card = card.background_color(bg);
        }

        card.build()
    }
}

impl<M: Clone + 'static> BannerBuilder<M> {
    /// Build without a theme (uses default resolution).
    /// Prefer `build(&theme)` when a `Theme` is available for proper V2 colors.
    pub fn build_default(self) -> WidgetNode<M> {
        let theme = acme_theme::Theme::light();
        self.build(&theme)
    }
}

impl<M: Clone + 'static> From<BannerBuilder<M>> for WidgetNode<M> {
    fn from(b: BannerBuilder<M>) -> Self {
        b.build(&Theme::light())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use acme_theme::Theme;

    fn test_theme() -> Theme {
        Theme::light()
    }

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {
        Retry,
    }

    #[test]
    fn banner_has_message() {
        let node: WidgetNode<TestMsg> = banner("Updates available").build(&test_theme());
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        // Card children: one Row
        assert_eq!(c.children.len(), 1);
        let WidgetNode::Row(r) = &c.children[0] else {
            panic!("expected Row child");
        };
        // Row children: icon (Label) + content Column
        let WidgetNode::Column(col) = &r.children[1] else {
            panic!("expected Column as second child");
        };
        // Column first child is the message label
        let WidgetNode::Label(l) = &col.children[0] else {
            panic!("expected Label as first child of Column");
        };
        assert_eq!(l.text, "Updates available");
    }

    #[test]
    fn banner_dismissible_has_close() {
        let node: WidgetNode<TestMsg> = banner("Notice").dismissible().build(&test_theme());
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        let WidgetNode::Row(r) = &c.children[0] else {
            panic!("expected Row child");
        };
        // Row: icon + Column + close button = 3 children
        assert_eq!(r.children.len(), 3);
        let WidgetNode::Button(_) = &r.children[2] else {
            panic!("expected Button as third child");
        };
    }

    #[test]
    fn banner_with_action() {
        let node: WidgetNode<TestMsg> = banner("Error")
            .action("Retry", TestMsg::Retry)
            .build(&test_theme());
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        let WidgetNode::Row(r) = &c.children[0] else {
            panic!("expected Row child");
        };
        // Row: icon + Column + action button = 3 children
        assert_eq!(r.children.len(), 3);
        let WidgetNode::Button(b) = &r.children[2] else {
            panic!("expected Button as third child");
        };
        assert_eq!(b.label, "Retry");
    }

    #[test]
    fn banner_with_description() {
        let node: WidgetNode<TestMsg> = banner("Warning")
            .description("This is a description.")
            .build(&test_theme());
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        let WidgetNode::Row(r) = &c.children[0] else {
            panic!("expected Row child");
        };
        let WidgetNode::Column(col) = &r.children[1] else {
            panic!("expected Column child");
        };
        // Column should have 2 children: message label + description label
        assert_eq!(col.children.len(), 2);
        let WidgetNode::Label(l) = &col.children[1] else {
            panic!("expected Label as second child of Column");
        };
        assert_eq!(l.text, "This is a description.");
    }

    #[test]
    fn banner_from_trait() {
        let node: WidgetNode<TestMsg> = banner("Hello").into();
        // The From impl calls build_default which uses Theme::light()
        let WidgetNode::Card(_) = &node else {
            panic!("expected Card variant");
        };
    }
}
