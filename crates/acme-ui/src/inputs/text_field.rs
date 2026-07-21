//! TextField — multi-variant text input (outlined/filled/standard).
//! Aligns with MUI TextField component.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// TextField variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextFieldVariant {
    #[default]
    Outlined,
    Filled,
    Standard,
}

/// TextField size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextFieldSize {
    Small,
    #[default]
    Medium,
}

/// Builder for a text field.
pub struct TextFieldBuilder<M> {
    pub id: WidgetKey,
    pub label: Option<String>,
    pub placeholder: Option<String>,
    pub helper_text: Option<String>,
    pub error: Option<String>,
    pub variant: TextFieldVariant,
    pub size: TextFieldSize,
    pub disabled: bool,
    pub required: bool,
    pub on_change: Option<M>,
}

/// Create a text field builder.
pub fn text_field<M: Clone + 'static>() -> TextFieldBuilder<M> {
    TextFieldBuilder {
        id: WidgetKey::from("text_field"),
        label: None,
        placeholder: None,
        helper_text: None,
        error: None,
        variant: TextFieldVariant::default(),
        size: TextFieldSize::default(),
        disabled: false,
        required: false,
        on_change: None,
    }
}

impl<M: Clone + 'static> TextFieldBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }

    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = Some(text.into());
        self
    }

    pub fn helper_text(mut self, text: impl Into<String>) -> Self {
        self.helper_text = Some(text.into());
        self
    }

    pub fn error(mut self, text: impl Into<String>) -> Self {
        self.error = Some(text.into());
        self
    }

    pub fn variant(mut self, value: TextFieldVariant) -> Self {
        self.variant = value;
        self
    }

    pub fn size(mut self, value: TextFieldSize) -> Self {
        self.size = value;
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    pub fn required(mut self, value: bool) -> Self {
        self.required = value;
        self
    }

    pub fn on_change(mut self, msg: M) -> Self {
        self.on_change = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<TextFieldBuilder<M>> for WidgetNode<M> {
    fn from(b: TextFieldBuilder<M>) -> Self {
        let mut col = crate::column::<M>().key(b.id).gap(4.0);

        if let Some(label) = b.label {
            let suffix = if b.required { " *" } else { "" };
            col = col.child(crate::label(format!("{}{}", label, suffix)));
        }

        let placeholder = b.placeholder.unwrap_or_default();
        let input = crate::label(format!("[ {} ]", placeholder));
        col = col.child(input);

        if let Some(err) = b.error {
            col = col.child(crate::label(format!("⚠ {}", err)));
        } else if let Some(helper) = b.helper_text {
            col = col.child(crate::label(helper));
        }
        col.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Changed,
    }

    #[test]
    fn text_field_produces_column() {
        let node: WidgetNode<Msg> = text_field().into();
        assert!(matches!(node, WidgetNode::Column(_)));
    }

    #[test]
    fn text_field_with_label_and_helper() {
        let node: WidgetNode<Msg> = text_field()
            .label("Email")
            .helper_text("We never share")
            .into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column");
        };
        // label + input + helper = 3
        assert_eq!(c.children.len(), 3);
    }

    #[test]
    fn text_field_error_replaces_helper() {
        let node: WidgetNode<Msg> = text_field()
            .label("Age")
            .helper_text("ignored")
            .error("Must be a number")
            .into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column");
        };
        // label + input + error = 3
        assert_eq!(c.children.len(), 3);
    }

    #[test]
    fn text_field_default_variant_outlined() {
        let b = text_field::<Msg>();
        assert_eq!(b.variant, TextFieldVariant::Outlined);
    }
}
