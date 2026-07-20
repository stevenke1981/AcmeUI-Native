use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_layout::Length;

/// A dialog/modal widget.
#[derive(Clone, Debug, PartialEq)]
pub struct Dialog<M> {
    pub key: WidgetKey,
    pub title: String,
    pub content: Box<WidgetNode<M>>,
    pub open: bool,
    pub modal: bool,
    pub width: Length,
    pub height: Length,
}

/// Create a dialog builder.
pub fn dialog<M>(key: impl Into<WidgetKey>, content: impl Into<WidgetNode<M>>) -> Dialog<M> {
    Dialog {
        key: key.into(),
        title: String::new(),
        content: Box::new(content.into()),
        open: false,
        modal: true,
        width: Length::px(480.0),
        height: Length::Auto,
    }
}

impl<M> Dialog<M> {
    pub fn title(mut self, value: impl Into<String>) -> Self {
        self.title = value.into();
        self
    }
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }
    pub fn modal(mut self, value: bool) -> Self {
        self.modal = value;
        self
    }
    pub fn width(mut self, value: f32) -> Self {
        self.width = Length::px(crate::finite(value));
        self
    }
    pub fn height(mut self, value: f32) -> Self {
        self.height = Length::px(crate::finite(value));
        self
    }
    pub fn build(self) -> WidgetNode<M> {
        WidgetNode::Dialog(self)
    }
}

impl<M> From<Dialog<M>> for WidgetNode<M> {
    fn from(value: Dialog<M>) -> Self {
        WidgetNode::Dialog(value)
    }
}
