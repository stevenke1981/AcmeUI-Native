//! FloatButton — floating action button (FAB).
//! Aligns with Ant Design FloatButton component.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Float button shape.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FloatButtonShape {
    #[default]
    Circle,
    Square,
}

/// Float button position on screen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FloatButtonPlacement {
    #[default]
    BottomRight,
    BottomLeft,
    TopRight,
    TopLeft,
}

/// Builder for a floating action button.
pub struct FloatButtonBuilder<M> {
    pub id: WidgetKey,
    pub icon: Option<crate::IconName>,
    pub label: Option<String>,
    pub shape: FloatButtonShape,
    pub placement: FloatButtonPlacement,
    pub on_click: Option<M>,
}

/// Create a float button builder.
pub fn float_button<M: Clone + 'static>() -> FloatButtonBuilder<M> {
    FloatButtonBuilder {
        id: WidgetKey::from("float_button"),
        icon: None,
        label: None,
        shape: FloatButtonShape::default(),
        placement: FloatButtonPlacement::default(),
        on_click: None,
    }
}

impl<M: Clone + 'static> FloatButtonBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn icon(mut self, value: crate::IconName) -> Self {
        self.icon = Some(value);
        self
    }

    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }

    pub fn shape(mut self, value: FloatButtonShape) -> Self {
        self.shape = value;
        self
    }

    pub fn placement(mut self, value: FloatButtonPlacement) -> Self {
        self.placement = value;
        self
    }

    pub fn on_click(mut self, msg: M) -> Self {
        self.on_click = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<FloatButtonBuilder<M>> for WidgetNode<M> {
    fn from(b: FloatButtonBuilder<M>) -> Self {
        let size = 56.0;
        let content = if let Some(icon) = b.icon {
            crate::icon::<M>(icon).build()
        } else if let Some(label) = b.label {
            crate::label(label)
        } else {
            crate::label("+")
        };

        let mut stack = crate::stack::<M>()
            .key(b.id)
            .size(size, size)
            .child(content);

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
    fn float_button_produces_stack() {
        let node: WidgetNode<Msg> = float_button().into();
        assert!(matches!(node, WidgetNode::Stack(_)));
    }

    #[test]
    fn float_button_with_icon() {
        let node: WidgetNode<Msg> = float_button()
            .icon(crate::IconName::Plus)
            .into();
        let WidgetNode::Stack(s) = &node else {
            panic!("expected Stack");
        };
        assert_eq!(s.children.len(), 1);
    }

    #[test]
    fn float_button_on_click() {
        let node: WidgetNode<Msg> = float_button().on_click(Msg::Clicked).into();
        let WidgetNode::Stack(s) = &node else {
            panic!("expected Stack");
        };
        assert_eq!(s.message, Some(Msg::Clicked));
    }

    #[test]
    fn float_button_default_placement() {
        let b = float_button::<Msg>();
        assert_eq!(b.placement, FloatButtonPlacement::BottomRight);
    }
}
