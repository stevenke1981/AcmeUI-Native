//! Opinionated, usable application shells for AcmeUI applications.

use acme_widgets::{WidgetNode, column, label, label_with_size};

use crate::native::{NativePlatform, NativeProfile};

fn build_profile_shell<M>(
    key: &'static str,
    title: String,
    subtitle: Option<String>,
    children: Vec<WidgetNode<M>>,
    profile: NativeProfile,
) -> WidgetNode<M> {
    let metrics = profile.metrics;
    let mut root = column::<M>()
        .key(key)
        .gap(metrics.content_gap)
        .padding(metrics.page_padding)
        .child(label_with_size(title, metrics.title_font_size));
    if let Some(subtitle) = subtitle {
        root = root.child(label_with_size(subtitle, metrics.body_font_size));
    }
    for child in children {
        root = root.child(child);
    }
    root.build()
}

/// A composable default application template.
///
/// The default template remains deterministic and platform-neutral. Applications
/// that want automatic operating-system adaptation should use [`native_template`].
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

/// Start the deterministic default AcmeUI application template.
pub fn default_template<M>(title: impl Into<String>) -> DefaultTemplate<M> {
    DefaultTemplate::new(title)
}

/// A target-aware application shell backed by [`NativeProfile`].
///
/// The shell adapts page rhythm and typography while keeping colors semantic.
/// Use the same profile with `native_theme` and `native_layout_context` so the
/// component tree, theme, and layout engine share one platform contract.
#[derive(Clone, Debug, PartialEq)]
pub struct NativeTemplate<M> {
    title: String,
    subtitle: Option<String>,
    children: Vec<WidgetNode<M>>,
    profile: NativeProfile,
}

impl<M> NativeTemplate<M> {
    /// Create a shell for the current compilation target.
    pub fn new(title: impl Into<String>) -> Self {
        Self::with_profile(title, NativeProfile::current())
    }

    /// Create a shell for an explicit platform profile.
    pub fn with_profile(title: impl Into<String>, profile: NativeProfile) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            children: Vec::new(),
            profile,
        }
    }

    /// Replace the profile used to build this shell.
    pub fn profile(mut self, profile: NativeProfile) -> Self {
        self.profile = profile;
        self
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

    /// Return the profile selected for this shell.
    pub const fn native_profile(&self) -> NativeProfile {
        self.profile
    }

    /// Build the stable target-aware root node.
    pub fn build(self) -> WidgetNode<M> {
        build_profile_shell(
            "acmeui-native-template",
            self.title,
            self.subtitle,
            self.children,
            self.profile,
        )
    }
}

/// Start a target-aware application shell for the current compilation target.
pub fn native_template<M>(title: impl Into<String>) -> NativeTemplate<M> {
    NativeTemplate::new(title)
}

/// Start a deterministic target-aware shell for previews and tests.
pub fn native_template_for<M>(
    platform: NativePlatform,
    title: impl Into<String>,
) -> NativeTemplate<M> {
    NativeTemplate::with_profile(title, NativeProfile::for_platform(platform))
}

macro_rules! platform_template {
    (
        $name:ident,
        $constructor:ident,
        $platform:expr,
        $key:literal,
        $doc:literal
    ) => {
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
                build_profile_shell(
                    $key,
                    self.title,
                    self.subtitle,
                    self.children,
                    NativeProfile::for_platform($platform),
                )
            }
        }

        /// Start the platform-inspired AcmeUI application template.
        pub fn $constructor<M>(title: impl Into<String>) -> $name<M> {
            $name::new(title)
        }
    };
}

platform_template!(
    AppleTemplate,
    apple_template,
    NativePlatform::MacOs,
    "acmeui-apple-template",
    "Apple-inspired shell driven by the shared macOS native profile."
);
platform_template!(
    Windows11Template,
    windows11_template,
    NativePlatform::Windows,
    "acmeui-windows11-template",
    "Windows 11-inspired shell driven by the shared Windows native profile."
);
platform_template!(
    Ubuntu25Template,
    ubuntu25_template,
    NativePlatform::Linux,
    "acmeui-ubuntu25-template",
    "Ubuntu-inspired shell driven by the shared Linux native profile."
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
    fn native_template_uses_explicit_profile_and_stable_key() {
        let template = native_template_for::<()>(NativePlatform::Android, "Acme App")
            .subtitle("Touch first")
            .child(label("Content"));
        assert_eq!(
            template.native_profile(),
            NativeProfile::for_platform(NativePlatform::Android)
        );
        let node = template.build();
        assert_eq!(
            node.key().expect("template key").as_str(),
            "acmeui-native-template"
        );
        assert_eq!(node.children().len(), 3);
    }

    #[test]
    fn platform_templates_have_stable_keys() {
        let apple = apple_template::<()>("Apple")
            .child(label("Content"))
            .build();
        let windows = windows11_template::<()>("Windows")
            .child(label("Content"))
            .build();
        let ubuntu = ubuntu25_template::<()>("Ubuntu")
            .child(label("Content"))
            .build();
        assert_eq!(
            apple.key().expect("template key").as_str(),
            "acmeui-apple-template"
        );
        assert_eq!(
            windows.key().expect("template key").as_str(),
            "acmeui-windows11-template"
        );
        assert_eq!(
            ubuntu.key().expect("template key").as_str(),
            "acmeui-ubuntu25-template"
        );
    }

    #[test]
    fn platform_templates_use_different_native_geometry() {
        let apple = apple_template::<()>("Apple")
            .build()
            .to_layout(acme_core::NodeId::new(1));
        let windows = windows11_template::<()>("Windows")
            .build()
            .to_layout(acme_core::NodeId::new(1));
        assert_ne!(apple.style.padding, windows.style.padding);
        assert_ne!(apple.style.gap, windows.style.gap);
    }
}
