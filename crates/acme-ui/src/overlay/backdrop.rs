//! Backdrop — overlay scrim behind modals and dialogs.
//! Aligns with MUI Backdrop component.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a backdrop overlay.
pub struct BackdropBuilder<M> {
    pub id: WidgetKey,
    pub visible: bool,
    pub child: Option<WidgetNode<M>>,
    pub on_click: Option<M>,
}

/// Create a backdrop builder.
pub fn backdrop<M: Clone + 'static>() -> BackdropBuilder<M> {
    BackdropBuilder {
        id: WidgetKey::from("backdrop"),
        visible: true,
        child: None,
        on_click: None,
    }
}

impl<M: Clone + 'static> BackdropBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn visible(mut self, value: bool) -> Self {
        self.visible = value;
        self
    }

    pub fn child(mut self, node: WidgetNode<M>) -> Self {
        self.child = Some(node);
        self
    }

    pub fn on_click(mut self, msg: M) -> Self {
        self.on_click = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<BackdropBuilder<M>> for WidgetNode<M> {
    fn from(b: BackdropBuilder<M>) -> Self {
        if !b.visible {
            return crate::label("");
        }
        let mut stack = crate::stack::<M>().key(b.id);
        if let Some(child) = b.child {
            stack = stack.child(child);
        }
        if let Some(msg) = b.on_click {
            stack = stack.on_click(msg);
        }
        stack.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Clicked,
    }

    #[test]
    fn backdrop_visible_produces_stack() {
        let node: WidgetNode<Msg> = backdrop().into();
        assert!(matches!(node, WidgetNode::Stack(_)));
    }

    #[test]
    fn backdrop_hidden_produces_empty_label() {
        let node: WidgetNode<Msg> = backdrop().visible(false).into();
        assert!(matches!(node, WidgetNode::Label(_)));
    }

    #[test]
    fn backdrop_with_child() {
        let node: WidgetNode<Msg> = backdrop()
            .child(crate::label("Loading…"))
            .into();
        let WidgetNode::Stack(s) = &node else {
            panic!("expected Stack");
        };
        assert_eq!(s.children.len(), 1);
    }

    #[test]
    fn backdrop_on_click() {
        let node: WidgetNode<Msg> = backdrop().on_click(Msg::Clicked).into();
        let WidgetNode::Stack(s) = &node else {
            panic!("expected Stack");
        };
        assert_eq!(s.message, Some(Msg::Clicked));
    }
}
