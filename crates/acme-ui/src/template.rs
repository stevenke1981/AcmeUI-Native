//! Opinionated, usable application shell for new AcmeUI applications.

use acme_widgets::{WidgetNode, column, label};

/// A composable default application template.
///
/// The template is intentionally declarative: callers provide their content
/// nodes, while the shell supplies stable keys, spacing and a title region.
/// Visual colors remain in `acme-theme` semantic tokens and are resolved by the
/// renderer/application theme.
#[derive(Clone, Debug, PartialEq)]
pub struct DefaultTemplate<M> {
    title: String,
    subtitle: Option<String>,
    children: Vec<WidgetNode<M>>,
}

impl<M> DefaultTemplate<M> {
    /// Create a template with a title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            children: Vec::new(),
        }
    }

    /// Add supporting text below the title.
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Append a page/content node.
    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Build the stable declarative root node.
    pub fn build(self) -> WidgetNode<M> {
        let mut root = column::<M>()
            .key("acmeui-default-template")
            .gap(16.0)
            .padding(24.0)
            .child(label(self.title));
        if let Some(subtitle) = self.subtitle {
            root = root.child(label(subtitle));
        }
        for child in self.children {
            root = root.child(child);
        }
        root.build()
    }
}

/// Start the default AcmeUI application template.
pub fn default_template<M>(title: impl Into<String>) -> DefaultTemplate<M> {
    DefaultTemplate::new(title)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_has_stable_root_and_content() {
        let node = default_template::<()>("Acme App")
            .subtitle("A modern native desktop app")
            .child(label("Content"))
            .build();
        assert_eq!(
            node.key().expect("template key").as_str(),
            "acmeui-default-template"
        );
        assert_eq!(node.children().len(), 3);
    }
}
