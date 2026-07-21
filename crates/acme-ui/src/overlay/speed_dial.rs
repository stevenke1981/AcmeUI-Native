//! SpeedDial — expandable floating action button with multiple actions.
//! Aligns with MUI SpeedDial component.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// A single speed dial action.
#[derive(Clone, Debug)]
pub struct SpeedDialAction {
    pub label: String,
    pub icon: Option<crate::IconName>,
}

impl SpeedDialAction {
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

/// Builder for a speed dial.
pub struct SpeedDialBuilder<M> {
    pub id: WidgetKey,
    pub open: bool,
    pub actions: Vec<SpeedDialAction>,
    pub on_toggle: Option<M>,
    pub on_action: Option<fn(usize) -> M>,
}

/// Create a speed dial builder.
pub fn speed_dial<M: Clone + 'static>() -> SpeedDialBuilder<M> {
    SpeedDialBuilder {
        id: WidgetKey::from("speed_dial"),
        open: false,
        actions: Vec::new(),
        on_toggle: None,
        on_action: None,
    }
}

impl<M: Clone + 'static> SpeedDialBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }

    pub fn action(mut self, action: SpeedDialAction) -> Self {
        self.actions.push(action);
        self
    }

    pub fn on_toggle(mut self, msg: M) -> Self {
        self.on_toggle = Some(msg);
        self
    }

    pub fn on_action(mut self, f: fn(usize) -> M) -> Self {
        self.on_action = Some(f);
        self
    }
}

impl<M: Clone + 'static> From<SpeedDialBuilder<M>> for WidgetNode<M> {
    fn from(b: SpeedDialBuilder<M>) -> Self {
        let mut col = crate::column::<M>().key(b.id).gap(8.0).padding(8.0);

        if b.open {
            for action in &b.actions {
                let icon_str = action
                    .icon
                    .map(|i| format!("{} ", i.char()))
                    .unwrap_or_default();
                col = col.child(crate::label(format!("{}{}", icon_str, action.label)));
            }
        }

        // Main FAB toggle
        let fab_label = if b.open { "✕" } else { "+" };
        let fab = crate::button("speed_dial_fab", fab_label).primary();
        let fab_node = if let Some(msg) = b.on_toggle {
            fab.on_click(msg)
        } else {
            fab.into()
        };
        col = col.child(fab_node);
        col.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Toggle,
        Action(usize),
    }

    fn action_msg(i: usize) -> Msg {
        Msg::Action(i)
    }

    #[test]
    fn speed_dial_closed_shows_only_fab() {
        let node: WidgetNode<Msg> = speed_dial()
            .action(SpeedDialAction::new("Edit"))
            .action(SpeedDialAction::new("Delete"))
            .into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column");
        };
        // Only FAB when closed
        assert_eq!(c.children.len(), 1);
    }

    #[test]
    fn speed_dial_open_shows_actions_and_fab() {
        let node: WidgetNode<Msg> = speed_dial()
            .open(true)
            .action(SpeedDialAction::new("Edit"))
            .action(SpeedDialAction::new("Share"))
            .into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column");
        };
        // 2 actions + FAB = 3
        assert_eq!(c.children.len(), 3);
    }

    #[test]
    fn speed_dial_on_toggle() {
        let node: WidgetNode<Msg> = speed_dial().on_toggle(Msg::Toggle).into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column");
        };
        assert!(!c.children.is_empty());
    }
}
