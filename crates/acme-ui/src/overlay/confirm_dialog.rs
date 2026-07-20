//! ConfirmDialog component — a modal dialog with title, message, and confirm/cancel buttons.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a confirm dialog.
pub struct ConfirmDialogBuilder<M> {
    pub id: WidgetKey,
    pub title: String,
    pub message: String,
    pub confirm_label: String,
    pub cancel_label: String,
    pub open: bool,
    pub on_confirm: Option<M>,
    pub on_cancel: Option<M>,
}

/// Create a confirm dialog builder.
pub fn confirm_dialog<M: Clone + 'static>(id: impl Into<WidgetKey>) -> ConfirmDialogBuilder<M> {
    ConfirmDialogBuilder {
        id: id.into(),
        title: String::new(),
        message: String::new(),
        confirm_label: "Confirm".into(),
        cancel_label: "Cancel".into(),
        open: false,
        on_confirm: None,
        on_cancel: None,
    }
}

impl<M: Clone> ConfirmDialogBuilder<M> {
    /// Set the dialog title.
    pub fn title(mut self, value: impl Into<String>) -> Self {
        self.title = value.into();
        self
    }

    /// Set the dialog body message.
    pub fn message(mut self, value: impl Into<String>) -> Self {
        self.message = value.into();
        self
    }

    /// Set the confirm button label.
    pub fn confirm_label(mut self, value: impl Into<String>) -> Self {
        self.confirm_label = value.into();
        self
    }

    /// Set the cancel button label.
    pub fn cancel_label(mut self, value: impl Into<String>) -> Self {
        self.cancel_label = value.into();
        self
    }

    /// Open or close the dialog.
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }

    /// Set the message fired on confirm.
    pub fn on_confirm(mut self, msg: M) -> Self {
        self.on_confirm = Some(msg);
        self
    }

    /// Set the message fired on cancel.
    pub fn on_cancel(mut self, msg: M) -> Self {
        self.on_cancel = Some(msg);
        self
    }

    /// Build the widget node tree.
    ///
    /// Uses `acme_widgets::dialog()` internally and wraps title, message,
    /// and confirm/cancel buttons as content.
    pub fn build(self) -> WidgetNode<M> {
        let msg_label = crate::label::<M>(self.message);

        let id_prefix = self.id.as_str();

        // Build button row
        let mut button_row = crate::row::<M>().gap(8.0);

        // Cancel button
        let cancel_key = format!("{id_prefix}_cancel");
        if let Some(ref msg) = self.on_cancel {
            let b =
                crate::button(cancel_key.as_str(), self.cancel_label.clone()).on_click(msg.clone());
            button_row = button_row.child(b);
        } else {
            button_row = button_row.child(crate::button(
                cancel_key.as_str(),
                self.cancel_label.clone(),
            ));
        }

        // Confirm button
        let confirm_key = format!("{id_prefix}_confirm");
        if let Some(ref msg) = self.on_confirm {
            let b = crate::button(confirm_key.as_str(), self.confirm_label.clone())
                .primary()
                .on_click(msg.clone());
            button_row = button_row.child(b);
        } else {
            button_row = button_row
                .child(crate::button(confirm_key.as_str(), self.confirm_label.clone()).primary());
        }

        let buttons = button_row.build();

        // Content column: message + buttons
        let content = crate::column::<M>()
            .child(msg_label)
            .child(buttons)
            .gap(16.0)
            .build();

        let mut dlg = crate::dialog::<M>(self.id.clone(), content)
            .title(self.title)
            .open(self.open);

        dlg = dlg.width(400.0);

        dlg.build()
    }
}

impl<M: Clone + 'static> From<ConfirmDialogBuilder<M>> for WidgetNode<M> {
    fn from(b: ConfirmDialogBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WidgetNode;
    use acme_core::NodeId;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Confirm,
        Cancel,
    }

    #[test]
    fn confirm_dialog_has_non_zero_layout_rect() {
        let node: WidgetNode<Msg> = confirm_dialog::<Msg>("dlg")
            .title("Save?")
            .message("Do you want to save changes?")
            .confirm_label("Save")
            .cancel_label("Don't Save")
            .open(true)
            .on_confirm(Msg::Confirm)
            .on_cancel(Msg::Cancel)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Dialog layout leaf
        assert_eq!(layout.style.kind, acme_layout::LayoutKind::Leaf);
        assert_eq!(layout.style.width, crate::Length::px(400.0));
    }

    #[test]
    fn confirm_dialog_default_labels() {
        let dlg = confirm_dialog::<Msg>("d").title("Ask").message("Proceed?");
        assert_eq!(dlg.confirm_label, "Confirm");
        assert_eq!(dlg.cancel_label, "Cancel");
    }

    #[test]
    fn confirm_dialog_key_is_stored() {
        let dlg = confirm_dialog::<Msg>("my-dialog");
        assert_eq!(dlg.id.as_str(), "my-dialog");
    }
}
