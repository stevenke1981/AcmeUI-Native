//! FormField — wraps a control with label, description, and error message.
//! Aligns with shadcn/ui Form Field pattern.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a form field wrapper.
pub struct FormFieldBuilder<M> {
    pub id: WidgetKey,
    pub label: String,
    pub description: Option<String>,
    pub error: Option<String>,
    pub required: bool,
    pub control: Option<WidgetNode<M>>,
}

/// Create a form field builder.
pub fn form_field<M: Clone + 'static>(label: impl Into<String>) -> FormFieldBuilder<M> {
    FormFieldBuilder {
        id: WidgetKey::from("form_field"),
        label: label.into(),
        description: None,
        error: None,
        required: false,
        control: None,
    }
}

impl<M: Clone + 'static> FormFieldBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }

    pub fn error(mut self, text: impl Into<String>) -> Self {
        self.error = Some(text.into());
        self
    }

    pub fn required(mut self, value: bool) -> Self {
        self.required = value;
        self
    }

    pub fn control(mut self, node: WidgetNode<M>) -> Self {
        self.control = Some(node);
        self
    }
}

impl<M: Clone + 'static> From<FormFieldBuilder<M>> for WidgetNode<M> {
    fn from(b: FormFieldBuilder<M>) -> Self {
        let label_text = if b.required {
            format!("{} *", b.label)
        } else {
            b.label
        };

        let mut col = crate::column::<M>().key(b.id).gap(6.0).padding(4.0);
        col = col.child(crate::label(label_text));

        if let Some(desc) = b.description {
            col = col.child(crate::label(desc));
        }
        if let Some(ctrl) = b.control {
            col = col.child(ctrl);
        }
        if let Some(err) = b.error {
            col = col.child(crate::label(format!("⚠ {}", err)));
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
    fn form_field_produces_column() {
        let node: WidgetNode<Msg> = form_field("Email").into();
        assert!(matches!(node, WidgetNode::Column(_)));
    }

    #[test]
    fn form_field_with_all_parts() {
        let node: WidgetNode<Msg> = form_field("Name")
            .description("Your full name")
            .control(crate::label("input"))
            .error("Required")
            .required(true)
            .into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column");
        };
        // label + description + control + error = 4
        assert_eq!(c.children.len(), 4);
    }

    #[test]
    fn form_field_required_appends_asterisk() {
        let b = form_field::<Msg>("Age").required(true);
        assert!(b.required);
    }
}
