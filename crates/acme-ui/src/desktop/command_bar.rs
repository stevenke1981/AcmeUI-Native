//! CommandBar — a desktop-style command/search bar.
//!
//! A horizontal bar with a search text input, action buttons, and optional
//! separators between groups. Inspired by VS Code's command palette and
//! browser toolbar patterns.
//!
//! Built as an outlined [`Card`] containing a [`Row`] with the search
//! input followed by the action items.

use crate::*;

/// Builder for a CommandBar component.
pub struct CommandBarBuilder<M> {
    pub search_placeholder: String,
    pub search_value: String,
    pub on_search: Option<M>,
    pub items: Vec<WidgetNode<M>>,
}

/// Create a new CommandBar builder.
pub fn command_bar<M>() -> CommandBarBuilder<M> {
    CommandBarBuilder {
        search_placeholder: String::new(),
        search_value: String::new(),
        on_search: None,
        items: vec![],
    }
}

impl<M: Clone + 'static> CommandBarBuilder<M> {
    /// Set the placeholder text for the search input.
    pub fn search_placeholder(mut self, text: impl Into<String>) -> Self {
        self.search_placeholder = text.into();
        self
    }

    /// Set the current value of the search input.
    pub fn search_value(mut self, text: impl Into<String>) -> Self {
        self.search_value = text.into();
        self
    }

    /// Set the message dispatched when the search input text changes.
    pub fn on_search(mut self, msg: M) -> Self {
        self.on_search = Some(msg);
        self
    }

    /// Add an action button with the given icon, label, and click message.
    pub fn action(mut self, icon: impl Into<String>, label: impl Into<String>, msg: M) -> Self {
        let key = format!("cmd-action-{}", self.items.len());
        let btn = button(key, label).leading_icon(icon).on_click(msg);
        self.items.push(btn);
        self
    }

    /// Add a visual separator between groups of actions.
    pub fn separator(mut self) -> Self {
        self.items.push(crate::separator());
        self
    }
}

impl<M: Clone + 'static> From<CommandBarBuilder<M>> for WidgetNode<M> {
    fn from(b: CommandBarBuilder<M>) -> Self {
        // Build the inner row: search input followed by action items
        let mut inner_row = row::<M>().gap(4.0);

        // Search text input
        let search = text_input("cmd-search")
            .placeholder(b.search_placeholder)
            .value(b.search_value);

        let search_node: WidgetNode<M> = match b.on_search {
            Some(msg) => search.on_change(msg),
            None => search.build(),
        };
        inner_row = inner_row.child(search_node);

        // Action items (buttons and separators)
        for item in b.items {
            inner_row = inner_row.child(item);
        }

        // Wrap in an outlined Card
        card()
            .key("command-bar")
            .variant(CardVariant::Outlined)
            .padding(4.0)
            .child(inner_row.build())
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

    /// Empty message enum for builder-only tests.
    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    /// Message enum for tests with actions and messages.
    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Find,
        Filter,
        Edit,
        Save,
    }

    #[test]
    fn command_bar_has_search() {
        let node: WidgetNode<TestMsg> = command_bar()
            .search_placeholder("Search commands...")
            .search_value("")
            .into();
        let layout = node.to_layout(NodeId::new(1));

        // Card is a Column layout
        assert_eq!(layout.style.kind, LayoutKind::Column);
        // Card has one child: the inner Row
        assert_eq!(layout.children.len(), 1);

        let row = &layout.children[0];
        assert_eq!(row.style.kind, LayoutKind::Row);
        // At minimum: the search TextInput
        assert!(!row.children.is_empty());
    }

    #[test]
    fn command_bar_with_actions() {
        let node: WidgetNode<Msg> = command_bar()
            .search_placeholder("Find...")
            .action("search", "Find", Msg::Find)
            .action("filter", "Filter", Msg::Filter)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        let row = &layout.children[0];

        // search + 2 action buttons = 3 children
        assert_eq!(row.children.len(), 3);
    }

    #[test]
    fn command_bar_separators() {
        let node: WidgetNode<Msg> = command_bar()
            .action("edit", "Edit", Msg::Edit)
            .separator()
            .action("save", "Save", Msg::Save)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        let row = &layout.children[0];

        // search + action + separator + action = 4 children
        assert_eq!(row.children.len(), 4);
    }

    #[test]
    fn command_bar_from_trait() {
        let builder = command_bar::<TestMsg>()
            .search_placeholder("Go...")
            .search_value("init");
        let node: WidgetNode<TestMsg> = builder.into();
        assert!(matches!(node, WidgetNode::Card(_)));
    }

    #[test]
    fn command_bar_on_search_sets_message() {
        let builder = command_bar::<Msg>()
            .search_placeholder("Type")
            .on_search(Msg::Find);
        assert_eq!(builder.on_search, Some(Msg::Find));
    }
}
