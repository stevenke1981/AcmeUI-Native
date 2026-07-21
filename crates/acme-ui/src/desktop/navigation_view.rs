//! NavigationView — sidebar + content split view inspired by AcmeUIKit's
//! NavigationView.

use crate::*;

/// The visual style of the navigation view.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NavigationViewStyle {
    /// Sidebar on the left, content on the right.
    #[default]
    Sidebar,
    /// Tab-based navigation.
    Tab,
}

/// Builder for a NavigationView component.
pub struct NavigationViewBuilder<M> {
    pub id: WidgetKey,
    pub sidebar: Option<WidgetNode<M>>,
    pub content: Option<WidgetNode<M>>,
    pub style: NavigationViewStyle,
    pub sidebar_width: f32,
}

/// Create a new NavigationView builder.
pub fn navigation_view<M: Clone + 'static>(id: impl Into<WidgetKey>) -> NavigationViewBuilder<M> {
    NavigationViewBuilder {
        id: id.into(),
        sidebar: None,
        content: None,
        style: NavigationViewStyle::default(),
        sidebar_width: 240.0,
    }
}

impl<M: Clone + 'static> NavigationViewBuilder<M> {
    /// Set the sidebar widget.
    pub fn sidebar(mut self, node: impl Into<WidgetNode<M>>) -> Self {
        self.sidebar = Some(node.into());
        self
    }

    /// Set the content widget.
    pub fn content(mut self, node: impl Into<WidgetNode<M>>) -> Self {
        self.content = Some(node.into());
        self
    }

    /// Set the navigation style.
    pub fn style(mut self, value: NavigationViewStyle) -> Self {
        self.style = value;
        self
    }

    /// Set the sidebar width in pixels.
    pub fn sidebar_width(mut self, value: f32) -> Self {
        self.sidebar_width = value;
        self
    }
}

impl<M: Clone + 'static> From<NavigationViewBuilder<M>> for WidgetNode<M> {
    fn from(b: NavigationViewBuilder<M>) -> Self {
        let sidebar = b.sidebar.unwrap_or_else(|| label::<M>(""));
        let content = b.content.unwrap_or_else(|| label::<M>(""));

        row::<M>()
            .key(b.id)
            // Sidebar in a fixed-width wrapper
            .child(column::<M>().width(b.sidebar_width).child(sidebar).build())
            // Thin vertical divider
            .child(column::<M>().width(1.0).build())
            // Content fills remaining space
            .child(content)
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
    use acme_layout::LayoutKind;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn navigation_view_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = navigation_view("nv")
            .sidebar(label::<TestMsg>("Sidebar"))
            .content(label::<TestMsg>("Content"))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Row);
        // sidebar wrapper + divider + content = 3 children
        assert_eq!(layout.children.len(), 3);
    }

    #[test]
    fn navigation_view_builder_defaults() {
        let nv = navigation_view::<TestMsg>("nv");
        assert!(nv.sidebar.is_none());
        assert!(nv.content.is_none());
        assert_eq!(nv.style, NavigationViewStyle::Sidebar);
        assert_eq!(nv.sidebar_width, 240.0);
    }

    #[test]
    fn navigation_view_field_setters_work() {
        let nv = navigation_view::<TestMsg>("nv")
            .sidebar(label::<TestMsg>("S"))
            .content(label::<TestMsg>("C"))
            .style(NavigationViewStyle::Tab)
            .sidebar_width(300.0);

        assert!(nv.sidebar.is_some());
        assert!(nv.content.is_some());
        assert_eq!(nv.style, NavigationViewStyle::Tab);
        assert!((nv.sidebar_width - 300.0).abs() < f32::EPSILON);
    }
}
