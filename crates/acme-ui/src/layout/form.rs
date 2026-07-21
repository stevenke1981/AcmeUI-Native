//! Form component — a vertical layout of labeled fields with help text
//! and validation messages.
//!
//! # Example
//!
//! ```ignore
//! form("user_form")
//!     .field(form_field("name", "Name")
//!         .help("Enter your full name")
//!         .content(text_input("name_input")))
//!     .field(form_field("email", "Email")
//!         .required(true)
//!         .content(text_input("email_input")))
//!     .field(form_field("bio", "Bio")
//!         .help("Tell us about yourself")
//!         .content(textarea("bio_input")));
//! ```

use crate::WidgetNode;
use acme_core::WidgetKey;

/// A single form field with label, optional help text, error message,
/// and content widget.
pub struct FormField<M> {
    pub id: WidgetKey,
    pub label: String,
    pub help_text: Option<String>,
    pub required: bool,
    pub content: Option<WidgetNode<M>>,
    pub error: Option<String>,
}

impl<M> FormField<M> {
    /// Set help text displayed below the content widget.
    pub fn help(mut self, value: impl Into<String>) -> Self {
        self.help_text = Some(value.into());
        self
    }

    /// Mark the field as required (appends `*` to the label).
    pub fn required(mut self, value: bool) -> Self {
        self.required = value;
        self
    }

    /// Set the content widget for this field.
    pub fn content(mut self, widget: impl Into<WidgetNode<M>>) -> Self {
        self.content = Some(widget.into());
        self
    }

    /// Set an error message for validation feedback (shown below the content).
    pub fn error(mut self, value: impl Into<String>) -> Self {
        self.error = Some(value.into());
        self
    }
}

/// Builder for a form container that holds multiple fields.
pub struct FormBuilder<M> {
    pub id: WidgetKey,
    pub fields: Vec<FormField<M>>,
}

/// Create a new Form builder.
pub fn form<M: Clone + 'static>(id: impl Into<WidgetKey>) -> FormBuilder<M> {
    FormBuilder {
        id: id.into(),
        fields: vec![],
    }
}

/// Create a new FormField.
pub fn form_field<M>(id: impl Into<WidgetKey>, label: impl Into<String>) -> FormField<M> {
    FormField {
        id: id.into(),
        label: label.into(),
        help_text: None,
        required: false,
        content: None,
        error: None,
    }
}

impl<M: Clone + 'static> FormBuilder<M> {
    /// Add a field to the form.
    pub fn field(mut self, field: FormField<M>) -> Self {
        self.fields.push(field);
        self
    }

    /// Build the widget node tree.
    ///
    /// Each field renders as a `Column` containing (top to bottom):
    /// - The label text (with a `*` suffix when `required`)
    /// - The content widget (if present)
    /// - Help text as a small 12px label (if present)
    /// - Error message as a Danger-tone `form_message` with icon (if present)
    pub fn build(self) -> WidgetNode<M> {
        let mut col = crate::column::<M>().gap(12.0);

        for field in self.fields {
            let mut field_col = crate::column::<M>().gap(4.0);

            // Label — append asterisk for required fields
            let label_text = if field.required {
                format!("{} *", field.label)
            } else {
                field.label.clone()
            };
            field_col = field_col.child(crate::label::<M>(label_text));

            // Content widget
            if let Some(content) = field.content {
                field_col = field_col.child(content);
            }

            // Help text — small muted label (12 px)
            if let Some(ref help) = field.help_text {
                field_col = field_col.child(crate::label_with_size::<M>(help.as_str(), 12.0));
            }

            // Error message — Danger-tone form_message with icon
            if let Some(ref err) = field.error {
                let msg = crate::inputs::form_message::form_message()
                    .text(err.as_str())
                    .danger()
                    .with_icon(true);
                field_col = field_col.child(msg);
            }

            col = col.child(field_col.build());
        }

        col.build()
    }
}

impl<M: Clone + 'static> From<FormBuilder<M>> for WidgetNode<M> {
    fn from(b: FormBuilder<M>) -> Self {
        b.build()
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

    // ------------------------------------------------------------------
    // FormField builder tests
    // ------------------------------------------------------------------

    #[test]
    fn form_field_defaults() {
        let f = form_field::<TestMsg>("name", "Name");
        assert_eq!(f.id.as_str(), "name");
        assert_eq!(f.label, "Name");
        assert!(!f.required);
        assert!(f.help_text.is_none());
        assert!(f.content.is_none());
        assert!(f.error.is_none());
    }

    #[test]
    fn form_field_builder_methods() {
        let f = form_field::<TestMsg>("email", "Email")
            .help("Enter your email address")
            .required(true)
            .content(crate::label("hello"))
            .error("Invalid email");
        assert_eq!(f.label, "Email");
        assert_eq!(f.help_text.as_deref(), Some("Enter your email address"));
        assert!(f.required);
        assert!(f.content.is_some());
        assert_eq!(f.error.as_deref(), Some("Invalid email"));
    }

    // ------------------------------------------------------------------
    // Form structural tests
    // ------------------------------------------------------------------

    #[test]
    fn form_renders_field_count() {
        let node: WidgetNode<TestMsg> = form("user_form")
            .field(form_field("name", "Name").content(crate::label("Alice")))
            .field(form_field("email", "Email").content(crate::label("alice@example.com")))
            .field(form_field("bio", "Bio").content(crate::label("Hello world")))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Outer column — 3 fields
        assert_eq!(layout.children.len(), 3);
        // Each field column: label + content = 2 children
        for child in &layout.children {
            assert_eq!(child.style.kind, LayoutKind::Column);
            assert_eq!(child.children.len(), 2);
        }
    }

    #[test]
    fn form_field_shows_help_text() {
        let node: WidgetNode<TestMsg> = form("f")
            .field(
                form_field("name", "Name")
                    .help("Enter your full name")
                    .content(crate::label("Bob")),
            )
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        assert_eq!(col.children.len(), 1);
        let WidgetNode::Column(field_col) = &col.children[0] else {
            panic!("expected field Column");
        };
        // label + content + help text = 3 children
        assert_eq!(field_col.children.len(), 3);
    }

    #[test]
    fn form_field_shows_error_message() {
        let node: WidgetNode<TestMsg> = form("f")
            .field(
                form_field("email", "Email")
                    .content(crate::label("x"))
                    .error("Must be a valid email"),
            )
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        let WidgetNode::Column(field_col) = &col.children[0] else {
            panic!("expected field Column");
        };
        // label + content + error message = 3 children
        assert_eq!(field_col.children.len(), 3);
    }

    #[test]
    fn form_field_required_shows_asterisk() {
        let node: WidgetNode<TestMsg> = form("f")
            .field(
                form_field("email", "Email")
                    .required(true)
                    .content(crate::label("a@b.com")),
            )
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        let WidgetNode::Column(field_col) = &col.children[0] else {
            panic!("expected field Column");
        };
        // First child should be the label with asterisk
        let WidgetNode::Label(lbl) = &field_col.children[0] else {
            panic!("expected Label");
        };
        assert_eq!(lbl.text, "Email *");
    }

    #[test]
    fn form_without_fields_is_empty() {
        let node: WidgetNode<TestMsg> = form("empty_form").build();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert!(layout.children.is_empty());
    }

    #[test]
    fn form_from_conversion() {
        let f = form::<TestMsg>("user_form");
        let node: WidgetNode<TestMsg> = f.into();
        assert!(matches!(node, WidgetNode::Column(_)));
    }
}
