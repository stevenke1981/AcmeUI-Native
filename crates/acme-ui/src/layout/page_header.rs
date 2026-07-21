//! PageHeader component — a page title area with description and optional actions.
//!
//! V2 design: renders as a Column with the title (large label), an optional
//! description (smaller muted label), and an optional Row of action buttons.

use acme_core::WidgetKey;

/// A single action button descriptor for the page header.
#[derive(Clone, Debug)]
pub struct HeaderAction<M> {
    /// Button label.
    pub label: String,
    /// Unique key for the button.
    pub key: WidgetKey,
    /// Message dispatched on click.
    pub on_click: Option<M>,
    /// Whether this is a primary action (Primary variant).
    pub primary: bool,
}

impl<M> HeaderAction<M> {
    /// Create a primary action button.
    pub fn primary(key: impl Into<WidgetKey>, label: impl Into<String>, msg: M) -> Self {
        Self {
            label: label.into(),
            key: key.into(),
            on_click: Some(msg),
            primary: true,
        }
    }

    /// Create a secondary action button.
    pub fn secondary(key: impl Into<WidgetKey>, label: impl Into<String>, msg: M) -> Self {
        Self {
            label: label.into(),
            key: key.into(),
            on_click: Some(msg),
            primary: false,
        }
    }
}

/// Builder for a PageHeader component.
pub struct PageHeaderBuilder<M> {
    pub id: WidgetKey,
    pub title: String,
    pub description: Option<String>,
    pub actions: Vec<HeaderAction<M>>,
}

/// Create a new PageHeader builder.
pub fn page_header<M: Clone + 'static>(
    id: impl Into<WidgetKey>,
    title: impl Into<String>,
) -> PageHeaderBuilder<M> {
    PageHeaderBuilder {
        id: id.into(),
        title: title.into(),
        description: None,
        actions: vec![],
    }
}

impl<M: Clone + 'static> PageHeaderBuilder<M> {
    /// Set the page description (subtitle).
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Add an action button to the header.
    pub fn action(mut self, action: HeaderAction<M>) -> Self {
        self.actions.push(action);
        self
    }
}

impl<M: Clone + 'static> From<PageHeaderBuilder<M>> for crate::WidgetNode<M> {
    fn from(b: PageHeaderBuilder<M>) -> Self {
        let mut col = crate::column::<M>().key(b.id).gap(4.0);

        // Title
        let theme = acme_theme::Theme::light();
        col = col.child(
            crate::label_builder(&b.title)
                .font_size(theme.typography.h2)
                .build(),
        );

        // Description
        if let Some(ref desc) = b.description {
            col = col.child(
                crate::label_builder(desc)
                    .font_size(theme.typography.body)
                    .color(theme.colors.muted_foreground)
                    .build(),
            );
        }

        // Actions
        if !b.actions.is_empty() {
            let mut action_row = crate::row::<M>().gap(8.0);
            for action in b.actions {
                let btn = crate::button(action.key, action.label);
                let btn = if action.primary {
                    btn.variant(crate::ButtonVariant::Primary)
                } else {
                    btn.variant(crate::ButtonVariant::Secondary)
                };
                let btn_node = if let Some(msg) = action.on_click {
                    btn.on_click(msg)
                } else {
                    btn.into()
                };
                action_row = action_row.child(btn_node);
            }
            col = col.child(action_row.build());
        }

        col.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WidgetNode;
    use acme_core::NodeId;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Save,
        Cancel,
    }

    #[test]
    fn page_header_has_non_zero_layout_rect() {
        let node: WidgetNode<Msg> = page_header::<Msg>("ph", "User Settings").into();
        let layout = node.to_layout(NodeId::new(1));
        // Column: [title] = at least 1 child
        assert!(!layout.children.is_empty());
    }

    #[test]
    fn page_header_with_description() {
        let node: WidgetNode<Msg> = page_header::<Msg>("ph", "Profile")
            .description("Manage your account settings and preferences.")
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Column: [title, description] = 2 children
        assert_eq!(layout.children.len(), 2);
    }

    #[test]
    fn page_header_with_actions() {
        let node: WidgetNode<Msg> = page_header::<Msg>("ph", "Edit")
            .description("Make changes to your profile.")
            .action(HeaderAction::primary("save", "Save", Msg::Save))
            .action(HeaderAction::secondary("cancel", "Cancel", Msg::Cancel))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Column: [title, description, action_row] = 3 children
        assert_eq!(layout.children.len(), 3);
        let action_row = &layout.children[2];
        // action_row is a Row with 2 buttons
        assert_eq!(action_row.children.len(), 2);
    }

    #[test]
    fn page_header_builder_defaults() {
        let h = page_header::<Msg>("ph", "Dashboard");
        assert_eq!(h.title, "Dashboard");
        assert!(h.description.is_none());
        assert!(h.actions.is_empty());
    }
}
