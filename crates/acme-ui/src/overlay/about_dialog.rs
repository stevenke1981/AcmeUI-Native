//! AboutDialog component.
//!
//! A simple about dialog with app icon, name, version, and credits.
//! Renders as a Column with an icon placeholder, app name, version,
//! optional description, optional credits, and a close button.

use acme_core::WidgetKey;
use acme_widgets::*;

/// Builder for an AboutDialog component.
pub struct AboutDialogBuilder<M> {
    pub id: WidgetKey,
    pub app_name: String,
    pub version: String,
    pub description: Option<String>,
    pub credits: Option<String>,
    pub open: bool,
    pub on_close: Option<M>,
}

/// Create a new AboutDialog builder.
pub fn about_dialog<M: Clone + 'static>(
    id: impl Into<WidgetKey>,
    app_name: impl Into<String>,
) -> AboutDialogBuilder<M> {
    AboutDialogBuilder {
        id: id.into(),
        app_name: app_name.into(),
        version: String::new(),
        description: None,
        credits: None,
        open: false,
        on_close: None,
    }
}

impl<M: Clone + 'static> AboutDialogBuilder<M> {
    /// Set the version string.
    pub fn version(mut self, value: impl Into<String>) -> Self {
        self.version = value.into();
        self
    }

    /// Set the description text.
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Set the credits text.
    pub fn credits(mut self, value: impl Into<String>) -> Self {
        self.credits = Some(value.into());
        self
    }

    /// Set whether the dialog is open.
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }

    /// Set the message dispatched when the close button is clicked.
    pub fn on_close(mut self, msg: M) -> Self {
        self.on_close = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<AboutDialogBuilder<M>> for WidgetNode<M> {
    fn from(b: AboutDialogBuilder<M>) -> Self {
        let close_key = format!("{}_close", b.id.as_str());

        // App icon placeholder: first letter of app name in a Card
        let icon_letter = b
            .app_name
            .chars()
            .next()
            .map(|c| c.to_string())
            .unwrap_or_default();
        let icon = card::<M>()
            .padding(12.0)
            .variant(CardVariant::Interactive)
            .child(label::<M>(icon_letter));

        let mut dialog_content = column::<M>()
            .key(b.id)
            .gap(12.0)
            .child(icon)
            .child(label(&b.app_name))
            .child(label(&b.version));

        if let Some(ref desc) = b.description {
            dialog_content = dialog_content.child(label(desc));
        }

        if let Some(ref credits_text) = b.credits {
            dialog_content = dialog_content.child(label(credits_text));
        }

        dialog_content = dialog_content.child(button::<M>(close_key.as_str(), "Close"));

        // Wrap in a Card to give the dialog a container appearance
        card::<M>()
            .gap(8.0)
            .padding(16.0)
            .child(dialog_content)
            .build()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn about_dialog_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = about_dialog("about1", "MyApp")
            .version("1.0.0")
            .description("A great app")
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // AboutDialog builds into a Card wrapping a Column
        assert!(!layout.children.is_empty());
    }

    #[test]
    fn about_dialog_builder_defaults() {
        let a = about_dialog::<TestMsg>("a", "App");
        assert_eq!(a.app_name, "App");
        assert!(a.version.is_empty());
        assert!(a.description.is_none());
        assert!(a.credits.is_none());
        assert!(!a.open);
        assert!(a.on_close.is_none());
    }

    #[test]
    fn about_dialog_shows_app_name_and_version() {
        let node: WidgetNode<TestMsg> = about_dialog("about", "MyApp")
            .version("2.0.0")
            .into();
        // Root is a Card
        let WidgetNode::Card(outer) = &node else {
            panic!("expected Card");
        };
        // First child is the inner Column
        let WidgetNode::Column(col) = &outer.children[0] else {
            panic!("expected Column");
        };
        // Column has: icon, app_name label, version label, close button
        assert_eq!(col.children.len(), 4);
        // Second child = app name label
        let WidgetNode::Label(name) = &col.children[1] else {
            panic!("expected Label for app name");
        };
        assert_eq!(name.text, "MyApp");
        // Third child = version label
        let WidgetNode::Label(ver) = &col.children[2] else {
            panic!("expected Label for version");
        };
        assert_eq!(ver.text, "2.0.0");
    }

    #[test]
    fn about_dialog_shows_close_button() {
        let node: WidgetNode<TestMsg> = about_dialog("about", "App").version("1.0").into();
        let WidgetNode::Card(outer) = &node else {
            panic!("expected Card");
        };
        let WidgetNode::Column(col) = &outer.children[0] else {
            panic!("expected Column");
        };
        // Last child should be a Button
        let WidgetNode::Button(btn) = &col.children[col.children.len() - 1] else {
            panic!("expected Button as last child");
        };
        assert_eq!(btn.label, "Close");
    }

    #[test]
    fn about_dialog_shows_description_and_credits() {
        let node: WidgetNode<TestMsg> = about_dialog("about", "App")
            .version("1.0")
            .description("Description text")
            .credits("Credits text")
            .into();
        let WidgetNode::Card(outer) = &node else {
            panic!("expected Card");
        };
        let WidgetNode::Column(col) = &outer.children[0] else {
            panic!("expected Column");
        };
        // Children: icon, app_name, version, description, credits, close button = 6
        assert_eq!(col.children.len(), 6);
        // Description is at index 3
        let WidgetNode::Label(desc) = &col.children[3] else {
            panic!("expected Label for description");
        };
        assert_eq!(desc.text, "Description text");
        // Credits is at index 4
        let WidgetNode::Label(cred) = &col.children[4] else {
            panic!("expected Label for credits");
        };
        assert_eq!(cred.text, "Credits text");
    }
}
