//! BottomNavigation — desktop bottom navigation bar.
//! Aligns with MUI Bottom Navigation component.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// A single bottom navigation action.
#[derive(Clone, Debug)]
pub struct BottomNavigationAction {
    pub label: String,
    pub icon: Option<crate::IconName>,
}

impl BottomNavigationAction {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
        }
    }

    pub fn icon(mut self, value: crate::IconName) -> Self {
        self.icon = Some(value);
        self
    }
}

/// Builder for a bottom navigation bar.
pub struct BottomNavigationBuilder<M> {
    pub id: WidgetKey,
    pub actions: Vec<BottomNavigationAction>,
    pub selected_index: usize,
    pub show_labels: bool,
    pub on_select: Option<fn(usize) -> M>,
}

/// Create a bottom navigation builder.
pub fn bottom_navigation<M: Clone + 'static>() -> BottomNavigationBuilder<M> {
    BottomNavigationBuilder {
        id: WidgetKey::from("bottom_navigation"),
        actions: Vec::new(),
        selected_index: 0,
        show_labels: true,
        on_select: None,
    }
}

impl<M: Clone + 'static> BottomNavigationBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn action(mut self, action: BottomNavigationAction) -> Self {
        self.actions.push(action);
        self
    }

    pub fn selected_index(mut self, index: usize) -> Self {
        self.selected_index = index;
        self
    }

    pub fn show_labels(mut self, value: bool) -> Self {
        self.show_labels = value;
        self
    }

    pub fn on_select(mut self, f: fn(usize) -> M) -> Self {
        self.on_select = Some(f);
        self
    }
}

impl<M: Clone + 'static> From<BottomNavigationBuilder<M>> for WidgetNode<M> {
    fn from(b: BottomNavigationBuilder<M>) -> Self {
        let mut row = crate::row::<M>()
            .key(b.id)
            .gap(8.0)
            .padding(8.0)
            .height(56.0);

        for (i, action) in b.actions.iter().enumerate() {
            let is_selected = i == b.selected_index;
            let icon_str = action
                .icon
                .map(|ic| ic.char().to_string())
                .unwrap_or_default();
            let text = if b.show_labels {
                let marker = if is_selected { "▣" } else { "▢" };
                format!("{} {} {}", icon_str, marker, action.label)
            } else {
                icon_str
            };
            row = row.child(crate::label(text));
        }
        row.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Selected(usize),
    }

    fn select_msg(i: usize) -> Msg {
        Msg::Selected(i)
    }

    #[test]
    fn bottom_navigation_produces_row() {
        let node: WidgetNode<Msg> = bottom_navigation()
            .action(BottomNavigationAction::new("Home"))
            .into();
        assert!(matches!(node, WidgetNode::Row(_)));
    }

    #[test]
    fn bottom_navigation_child_count() {
        let node: WidgetNode<Msg> = bottom_navigation()
            .action(BottomNavigationAction::new("Home").icon(crate::IconName::User))
            .action(BottomNavigationAction::new("Search").icon(crate::IconName::Search))
            .action(BottomNavigationAction::new("Settings").icon(crate::IconName::Settings))
            .into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        assert_eq!(r.children.len(), 3);
    }

    #[test]
    fn bottom_navigation_selected_index() {
        let b = bottom_navigation::<Msg>().selected_index(2);
        assert_eq!(b.selected_index, 2);
    }
}
