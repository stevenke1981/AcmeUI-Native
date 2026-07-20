use crate::WidgetNode;
use acme_core::WidgetKey;

/// A text input widget with label, description, and validation support.
#[derive(Clone, Debug, PartialEq)]
pub struct TextInput<M> {
    pub key: WidgetKey,
    pub label: Option<String>,
    pub description: Option<String>,
    pub placeholder: Option<String>,
    pub value: String,
    pub clearable: bool,
    pub readonly: bool,
    pub password: bool,
    pub invalid: bool,
    pub validation_message: Option<String>,
    pub disabled: bool,
    pub message: Option<M>,
}

/// Create a text input builder.
pub fn text_input<M>(key: impl Into<WidgetKey>) -> TextInput<M> {
    TextInput {
        key: key.into(),
        label: None,
        description: None,
        placeholder: None,
        value: String::new(),
        clearable: false,
        readonly: false,
        password: false,
        invalid: false,
        validation_message: None,
        disabled: false,
        message: None,
    }
}

impl<M> TextInput<M> {
    pub fn label(mut self, value: impl Into<String>) -> Self {
        self.label = Some(value.into());
        self
    }
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }
    pub fn placeholder(mut self, value: impl Into<String>) -> Self {
        self.placeholder = Some(value.into());
        self
    }
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }
    pub fn clearable(mut self, value: bool) -> Self {
        self.clearable = value;
        self
    }
    pub fn readonly(mut self, value: bool) -> Self {
        self.readonly = value;
        self
    }
    pub fn password(mut self, value: bool) -> Self {
        self.password = value;
        self
    }
    pub fn invalid(mut self, value: bool) -> Self {
        self.invalid = value;
        self
    }
    pub fn validation(mut self, message: impl Into<String>) -> Self {
        self.validation_message = Some(message.into());
        self
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    pub fn on_change(mut self, message: M) -> WidgetNode<M> {
        self.message = Some(message);
        WidgetNode::TextInput(self)
    }
    pub fn build(self) -> WidgetNode<M> {
        WidgetNode::TextInput(self)
    }
}

impl<M> From<TextInput<M>> for WidgetNode<M> {
    fn from(value: TextInput<M>) -> Self {
        WidgetNode::TextInput(value)
    }
}
