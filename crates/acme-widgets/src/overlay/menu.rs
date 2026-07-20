use crate::WidgetNode;
use acme_core::WidgetKey;

/// A menu item.
#[derive(Clone, Debug, PartialEq)]
pub struct MenuItem<M> {
    pub key: WidgetKey,
    pub label: String,
    pub disabled: bool,
    pub message: Option<M>,
    pub separator: bool,
    pub children: Vec<MenuItem<M>>,
}
impl<M> MenuItem<M> {
    pub fn activate(&self) -> Option<&M> {
        if self.disabled || self.separator {
            None
        } else {
            self.message.as_ref()
        }
    }
}

/// A menu widget.
#[derive(Clone, Debug, PartialEq)]
pub struct Menu<M> {
    pub key: WidgetKey,
    pub items: Vec<MenuItem<M>>,
    pub open: bool,
}

/// Create a menu builder.
pub fn menu<M>(key: impl Into<WidgetKey>) -> Menu<M> {
    Menu {
        key: key.into(),
        items: vec![],
        open: false,
    }
}

impl<M> Menu<M> {
    pub fn item(mut self, item: MenuItem<M>) -> Self {
        self.items.push(item);
        self
    }
    pub fn separator(mut self) -> Self {
        self.items.push(MenuItem {
            key: WidgetKey::from(""),
            label: String::new(),
            disabled: true,
            message: None,
            separator: true,
            children: vec![],
        });
        self
    }
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }
    pub fn build(self) -> WidgetNode<M> {
        WidgetNode::Menu(self)
    }
}

impl<M> From<Menu<M>> for WidgetNode<M> {
    fn from(value: Menu<M>) -> Self {
        WidgetNode::Menu(value)
    }
}

/// Create a menu item builder.
pub fn menu_item<M>(key: impl Into<WidgetKey>, label: impl Into<String>) -> MenuItem<M> {
    MenuItem {
        key: key.into(),
        label: label.into(),
        disabled: false,
        message: None,
        separator: false,
        children: vec![],
    }
}

impl<M> MenuItem<M> {
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    pub fn on_click(mut self, message: M) -> Self {
        self.message = Some(message);
        self
    }
    pub fn submenu(mut self, child: MenuItem<M>) -> Self {
        self.children.push(child);
        self
    }
}
