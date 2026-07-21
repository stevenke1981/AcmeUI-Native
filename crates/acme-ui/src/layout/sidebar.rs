//! Sidebar — collapsible application sidebar with variants.
//! Aligns with shadcn/ui Sidebar.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Sidebar variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SidebarVariant {
    #[default]
    Sidebar,
    Floating,
    Inset,
}

/// Sidebar collapse state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SidebarState {
    #[default]
    Expanded,
    Collapsed,
}

/// A sidebar navigation item.
#[derive(Clone, Debug)]
pub struct SidebarItem {
    pub label: String,
    pub icon: Option<crate::IconName>,
    pub active: bool,
}

impl SidebarItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            active: false,
        }
    }

    pub fn icon(mut self, value: crate::IconName) -> Self {
        self.icon = Some(value);
        self
    }

    pub fn active(mut self, value: bool) -> Self {
        self.active = value;
        self
    }
}

/// Builder for a sidebar.
pub struct SidebarBuilder<M> {
    pub id: WidgetKey,
    pub variant: SidebarVariant,
    pub state: SidebarState,
    pub items: Vec<SidebarItem>,
    pub header: Option<WidgetNode<M>>,
    pub footer: Option<WidgetNode<M>>,
    pub width: f32,
    pub collapsed_width: f32,
}

/// Create a sidebar builder.
pub fn sidebar<M: Clone + 'static>() -> SidebarBuilder<M> {
    SidebarBuilder {
        id: WidgetKey::from("sidebar"),
        variant: SidebarVariant::default(),
        state: SidebarState::default(),
        items: Vec::new(),
        header: None,
        footer: None,
        width: 256.0,
        collapsed_width: 64.0,
    }
}

impl<M: Clone + 'static> SidebarBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn variant(mut self, value: SidebarVariant) -> Self {
        self.variant = value;
        self
    }

    pub fn state(mut self, value: SidebarState) -> Self {
        self.state = value;
        self
    }

    pub fn item(mut self, item: SidebarItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn header(mut self, node: WidgetNode<M>) -> Self {
        self.header = Some(node);
        self
    }

    pub fn footer(mut self, node: WidgetNode<M>) -> Self {
        self.footer = Some(node);
        self
    }

    pub fn width(mut self, value: f32) -> Self {
        self.width = value;
        self
    }

    pub fn collapsed_width(mut self, value: f32) -> Self {
        self.collapsed_width = value;
        self
    }
}

impl<M: Clone + 'static> From<SidebarBuilder<M>> for WidgetNode<M> {
    fn from(b: SidebarBuilder<M>) -> Self {
        let effective_width = match b.state {
            SidebarState::Expanded => b.width,
            SidebarState::Collapsed => b.collapsed_width,
        };

        let mut col = crate::column::<M>()
            .key(b.id)
            .width(effective_width)
            .gap(4.0)
            .padding(8.0);

        if let Some(header) = b.header {
            col = col.child(header);
            col = col.child(crate::separator::<M>());
        }

        for item in &b.items {
            let prefix = if item.active { "▸ " } else { "  " };
            let text = match (&item.icon, b.state) {
                (Some(icon), SidebarState::Expanded) => {
                    format!("{}{} {}", prefix, icon.char(), item.label)
                }
                (Some(icon), SidebarState::Collapsed) => icon.char().to_string(),
                (None, _) => format!("{}{}", prefix, item.label),
            };
            col = col.child(crate::label(text));
        }

        if let Some(footer) = b.footer {
            col = col.child(crate::separator::<M>());
            col = col.child(footer);
        }
        col.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn sidebar_produces_column() {
        let node: WidgetNode<Msg> = sidebar().into();
        assert!(matches!(node, WidgetNode::Column(_)));
    }

    #[test]
    fn sidebar_with_items() {
        let node: WidgetNode<Msg> = sidebar()
            .item(SidebarItem::new("Home").icon(crate::IconName::User))
            .item(SidebarItem::new("Settings").icon(crate::IconName::Settings))
            .into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column");
        };
        assert_eq!(c.children.len(), 2);
    }

    #[test]
    fn sidebar_collapsed_uses_narrow_width() {
        let b = sidebar::<Msg>().state(SidebarState::Collapsed);
        assert_eq!(b.state, SidebarState::Collapsed);
    }

    #[test]
    fn sidebar_with_header_and_footer() {
        let node: WidgetNode<Msg> = sidebar()
            .header(crate::label("Logo"))
            .item(SidebarItem::new("Nav"))
            .footer(crate::label("User"))
            .into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column");
        };
        // header + separator + item + separator + footer = 5
        assert_eq!(c.children.len(), 5);
    }
}
