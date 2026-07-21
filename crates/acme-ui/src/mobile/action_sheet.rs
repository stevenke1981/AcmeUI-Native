//! ActionSheet component — a mobile-style bottom action sheet that
//! presents a list of action options.
//!
//! Renders as a Card with action rows. Each action shows a label
//! and optional icon. A cancel button is optionally rendered at the bottom.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// A single action in the action sheet.
#[derive(Clone, Debug)]
pub struct ActionItem {
    pub label: String,
    pub destructive: bool,
    pub disabled: bool,
}

impl ActionItem {
    /// Create a normal action item.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            destructive: false,
            disabled: false,
        }
    }

    /// Mark this action as destructive (red styling).
    pub fn destructive(mut self, value: bool) -> Self {
        self.destructive = value;
        self
    }

    /// Mark this action as disabled.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
}

/// Builder for an action sheet.
pub struct ActionSheetBuilder<M> {
    pub id: WidgetKey,
    pub title: Option<String>,
    pub actions: Vec<ActionItem>,
    pub show_cancel: bool,
    pub cancel_label: String,
    pub on_select: Option<fn(usize) -> M>,
    _phantom: std::marker::PhantomData<M>,
}

/// Create an action sheet builder.
pub fn action_sheet<M: Clone + 'static>(id: impl Into<WidgetKey>) -> ActionSheetBuilder<M> {
    ActionSheetBuilder {
        id: id.into(),
        title: None,
        actions: vec![],
        show_cancel: true,
        cancel_label: "Cancel".to_string(),
        on_select: None,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> ActionSheetBuilder<M> {
    /// Set the action sheet title.
    pub fn title(mut self, value: impl Into<String>) -> Self {
        self.title = Some(value.into());
        self
    }

    /// Add an action item.
    pub fn action(mut self, item: ActionItem) -> Self {
        self.actions.push(item);
        self
    }

    /// Show/hide the cancel button.
    pub fn show_cancel(mut self, value: bool) -> Self {
        self.show_cancel = value;
        self
    }

    /// Set the cancel button label.
    pub fn cancel_label(mut self, value: impl Into<String>) -> Self {
        self.cancel_label = value.into();
        self
    }

    /// Set the callback for action selection (index → message).
    pub fn on_select(mut self, callback: fn(usize) -> M) -> Self {
        self.on_select = Some(callback);
        self
    }

    /// Build the action sheet widget.
    pub fn build(self) -> WidgetNode<M> {
        let mut col = crate::column::<M>().key(self.id.clone()).gap(4.0);

        // Optional title
        if let Some(t) = &self.title {
            col = col.child(
                crate::label_builder(t)
                    .font_size(14.0)
                    .color(crate::ThemeColor::rgb(140, 140, 140))
                    .build(),
            );
        }

        // Action rows — use label as button since Button takes key+label
        for (i, action) in self.actions.iter().enumerate() {
            let btn_key = format!("{}_action_{}", self.id.as_str(), i);
            let btn = crate::button(btn_key.as_str(), &action.label);
            let btn = if action.destructive {
                btn.primary()
            } else {
                btn
            };
            col = col.child(btn);
        }

        // Cancel button (separated)
        if self.show_cancel {
            let cancel_key = format!("{}_cancel", self.id.as_str());
            col = col.child(crate::button(cancel_key.as_str(), &self.cancel_label));
        }

        crate::card::<M>()
            .variant(crate::CardVariant::Elevated)
            .padding(12.0)
            .child(col)
            .build()
    }
}

impl<M: Clone + 'static> From<ActionSheetBuilder<M>> for WidgetNode<M> {
    fn from(b: ActionSheetBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;
    use acme_layout::{LayoutEngine, WidgetLayoutContext};

    fn test_context() -> WidgetLayoutContext {
        WidgetLayoutContext {
            body_font_size: 16.0,
            body_line_height: 22.0,
            label_font_size: 14.0,
            control_height: 32.0,
            scale_factor: 1.0,
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn action_sheet_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = action_sheet("as").build();
        let ctx = test_context();
        let layout = node.to_layout_with_context(NodeId::new(1), &ctx);
        let snapshot = LayoutEngine::new()
            .compute(&layout, (800.0, 600.0))
            .unwrap();
        let rect = snapshot.get(NodeId::new(1)).unwrap();
        assert!(rect.width > 0.0);
        assert!(rect.height > 0.0);
    }

    #[test]
    fn action_sheet_with_actions() {
        let node: WidgetNode<TestMsg> = action_sheet("as")
            .title("Options")
            .action(ActionItem::new("Edit"))
            .action(ActionItem::new("Delete").destructive(true))
            .action(ActionItem::new("Share"))
            .build();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card")
        };
        let WidgetNode::Column(col) = &c.children[0] else {
            panic!("expected Column")
        };
        // title + 3 action buttons + cancel = 5
        assert_eq!(col.children.len(), 5);
    }

    #[test]
    fn action_sheet_no_title_no_cancel() {
        let node: WidgetNode<TestMsg> = action_sheet("as")
            .show_cancel(false)
            .action(ActionItem::new("Save"))
            .build();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card")
        };
        let WidgetNode::Column(col) = &c.children[0] else {
            panic!("expected Column")
        };
        assert_eq!(col.children.len(), 1);
    }

    #[test]
    fn action_sheet_destructive_flag() {
        let a = ActionItem::new("Delete").destructive(true);
        assert!(a.destructive);
        let a2 = ActionItem::new("Save");
        assert!(!a2.destructive);
    }

    #[test]
    fn action_sheet_builder_defaults() {
        let a = action_sheet::<TestMsg>("as");
        assert!(a.title.is_none());
        assert!(a.show_cancel);
        assert_eq!(a.cancel_label, "Cancel");
        assert!(a.actions.is_empty());
    }

    #[test]
    fn action_sheet_cancel_label_custom() {
        let a = action_sheet::<TestMsg>("as").cancel_label("Close");
        assert_eq!(a.cancel_label, "Close");
    }
}
