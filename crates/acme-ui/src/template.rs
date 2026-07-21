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

/// Apple-inspired application shell with restrained spacing and hierarchy.
///
/// The template keeps all colors semantic and delegates material, typography,
/// and motion decisions to the active theme/renderer. It is suitable as a
/// calm macOS/iOS-style starting point without exposing platform types.
#[derive(Clone, Debug, PartialEq)]
pub struct AppleTemplate<M> {
    title: String,
    subtitle: Option<String>,
    children: Vec<WidgetNode<M>>,
}

impl<M> AppleTemplate<M> {
    /// Create an Apple-inspired template with a title.
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

    /// Build the stable Apple-style declarative root node.
    pub fn build(self) -> WidgetNode<M> {
        let mut root = column::<M>()
            .key("acmeui-apple-template")
            .gap(12.0)
            .padding(20.0)
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

/// Start the Apple-inspired AcmeUI application template.
pub fn apple_template<M>(title: impl Into<String>) -> AppleTemplate<M> {
    AppleTemplate::new(title)
}

macro_rules! platform_template {
    ($name:ident, $constructor:ident, $key:literal, $gap:literal, $padding:literal, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name<M> {
            title: String,
            subtitle: Option<String>,
            children: Vec<WidgetNode<M>>,
        }

        impl<M> $name<M> {
            /// Create a platform-inspired template with a title.
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

            /// Build the stable platform-style declarative root node.
            pub fn build(self) -> WidgetNode<M> {
                let mut root = column::<M>()
                    .key($key)
                    .gap($gap)
                    .padding($padding)
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

        /// Start the platform-inspired AcmeUI application template.
        pub fn $constructor<M>(title: impl Into<String>) -> $name<M> {
            $name::new(title)
        }
    };
}

platform_template!(
    Windows11Template,
    windows11_template,
    "acmeui-windows11-template",
    16.0,
    24.0,
    "Windows 11-inspired shell with layered spacing and a clear content hierarchy."
);
platform_template!(
    Ubuntu25Template,
    ubuntu25_template,
    "acmeui-ubuntu25-template",
    12.0,
    24.0,
    "Ubuntu 25-inspired shell with compact rhythm and efficient workspace density."
);

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

    #[test]
    fn apple_template_has_stable_root_and_content() {
        let node = apple_template::<()>("Acme App")
            .subtitle("Native and calm")
            .child(label("Content"))
            .build();
        assert_eq!(
            node.key().expect("template key").as_str(),
            "acmeui-apple-template"
        );
        assert_eq!(node.children().len(), 3);
    }

    #[test]
    fn platform_templates_have_stable_keys() {
        let windows = windows11_template::<()>("Windows")
            .child(label("Content"))
            .build();
        let ubuntu = ubuntu25_template::<()>("Ubuntu")
            .child(label("Content"))
            .build();
        assert_eq!(
            windows.key().expect("template key").as_str(),
            "acmeui-windows11-template"
        );
        assert_eq!(
            ubuntu.key().expect("template key").as_str(),
            "acmeui-ubuntu25-template"
        );
    }
}
