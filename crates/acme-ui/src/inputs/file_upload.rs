//! File upload trigger component.

use crate::WidgetNode;

/// Builder for a file upload trigger. The application decides how to open a
/// picker and receives the configured message through the normal event path.
pub struct FileUploadBuilder<M> {
    label: String,
    hint: Option<String>,
    message: Option<M>,
}

/// Create a file upload trigger.
pub fn file_upload<M: Clone + 'static>(label: impl Into<String>) -> FileUploadBuilder<M> {
    FileUploadBuilder {
        label: label.into(),
        hint: None,
        message: None,
    }
}

impl<M: Clone + 'static> FileUploadBuilder<M> {
    /// Set supporting hint text.
    pub fn hint(mut self, value: impl Into<String>) -> Self {
        self.hint = Some(value.into());
        self
    }

    /// Set the message emitted when the trigger is activated.
    pub fn on_upload(mut self, message: M) -> Self {
        self.message = Some(message);
        self
    }

    /// Build the upload trigger.
    pub fn build(self) -> WidgetNode<M> {
        let mut content = crate::column::<M>().gap(4.0);
        let trigger = crate::button("file-upload", self.label);
        let trigger = if let Some(message) = self.message {
            trigger.on_click(message)
        } else {
            trigger.into()
        };
        content = content.child(trigger);
        if let Some(hint) = self.hint {
            content = content.child(crate::label(hint));
        }
        content.build()
    }
}

impl<M: Clone + 'static> From<FileUploadBuilder<M>> for WidgetNode<M> {
    fn from(value: FileUploadBuilder<M>) -> Self {
        value.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_upload_builds_trigger_and_hint() {
        let node = file_upload::<()>("Choose file").hint("PNG or JPG").build();
        assert_eq!(node.children().len(), 2);
    }
}
