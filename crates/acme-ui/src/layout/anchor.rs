//! Anchor — page anchor navigation for scrolling to sections.
//! Aligns with Ant Design Anchor component.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// A single anchor link item.
#[derive(Clone, Debug)]
pub struct AnchorItem {
    pub href: String,
    pub title: String,
    pub children: Vec<AnchorItem>,
}

impl AnchorItem {
    pub fn new(href: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            href: href.into(),
            title: title.into(),
            children: Vec::new(),
        }
    }

    pub fn child(mut self, item: AnchorItem) -> Self {
        self.children.push(item);
        self
    }
}

/// Builder for an anchor navigation.
pub struct AnchorBuilder<M> {
    pub id: WidgetKey,
    pub items: Vec<AnchorItem>,
    pub active_href: Option<String>,
    pub on_navigate: Option<fn(&str) -> M>,
}

/// Create an anchor navigation builder.
pub fn anchor<M: Clone + 'static>() -> AnchorBuilder<M> {
    AnchorBuilder {
        id: WidgetKey::from("anchor"),
        items: Vec::new(),
        active_href: None,
        on_navigate: None,
    }
}

impl<M: Clone + 'static> AnchorBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn item(mut self, item: AnchorItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn active_href(mut self, href: impl Into<String>) -> Self {
        self.active_href = Some(href.into());
        self
    }

    /// Build a navigation message from each rendered link's href. Without
    /// this callback links remain passive labels. The host owns active_href
    /// and target scrolling; this conversion only binds typed messages.
    pub fn on_navigate(mut self, f: fn(&str) -> M) -> Self {
        self.on_navigate = Some(f);
        self
    }
}

impl<M: Clone + 'static> From<AnchorBuilder<M>> for WidgetNode<M> {
    fn from(b: AnchorBuilder<M>) -> Self {
        let action_prefix = b.id.as_str().to_owned();
        let mut col = crate::column::<M>().key(b.id).gap(4.0).padding(8.0);
        for (index, item) in b.items.iter().enumerate() {
            let marker = if b.active_href.as_deref() == Some(&item.href) {
                "▸ "
            } else {
                "  "
            };
            let text = format!("{}{}", marker, item.title);
            let key = format!("{action_prefix}:link:{index}");
            let link = if let Some(navigate) = b.on_navigate {
                crate::button(key, text).on_click(navigate(&item.href))
            } else {
                crate::label(text)
            };
            col = col.child(link);
            for (child_index, child) in item.children.iter().enumerate() {
                let text = format!("    · {}", child.title);
                let link = if let Some(navigate) = b.on_navigate {
                    crate::button(format!("{action_prefix}:link:{index}:{child_index}"), text)
                        .on_click(navigate(&child.href))
                } else {
                    crate::label(text)
                };
                col = col.child(link);
            }
        }
        col.build()
    }
}

#[cfg(test)]
#[path = "anchor_tests.rs"]
mod conversion_tests;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Nav(String),
    }

    fn nav_msg(href: &str) -> Msg {
        Msg::Nav(href.to_string())
    }

    #[test]
    fn anchor_produces_column() {
        let node: WidgetNode<Msg> = anchor()
            .item(AnchorItem::new("#intro", "Introduction"))
            .into();
        assert!(matches!(node, WidgetNode::Column(_)));
    }

    #[test]
    fn anchor_with_nested_items() {
        let node: WidgetNode<Msg> = anchor()
            .item(
                AnchorItem::new("#api", "API")
                    .child(AnchorItem::new("#api-get", "GET"))
                    .child(AnchorItem::new("#api-post", "POST")),
            )
            .into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column");
        };
        // parent + 2 children = 3
        assert_eq!(c.children.len(), 3);
    }

    #[test]
    fn anchor_active_href() {
        let b = anchor::<Msg>().active_href("#intro");
        assert_eq!(b.active_href, Some("#intro".to_string()));
    }

    #[test]
    fn anchor_on_navigate_emits_nav_with_href() {
        let b = anchor::<Msg>().on_navigate(nav_msg);
        assert_eq!(
            b.on_navigate.map(|f| f("#api")),
            Some(Msg::Nav("#api".to_string()))
        );
    }
}
