//! Mobile action — tappable action row with icon, label, and destructive flag.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a mobile action row.
pub struct MobileActionBuilder<M> {
    pub id: WidgetKey,
    pub label: String,
    pub icon: Option<crate::IconName>,
    pub destructive: bool,
    pub disabled: bool,
    pub on_press: Option<M>,
}

/// Create a mobile action builder.
pub fn mobile_action<M: Clone + 'static>(label: impl Into<String>) -> MobileActionBuilder<M> {
    MobileActionBuilder {
        id: WidgetKey::from("mobile_action"),
        label: label.into(),
        icon: None,
        destructive: false,
        disabled: false,
        on_press: None,
    }
}

impl<M: Clone + 'static> MobileActionBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn icon(mut self, value: crate::IconName) -> Self {
        self.icon = Some(value);
        self
    }

    pub fn destructive(mut self, value: bool) -> Self {
        self.destructive = value;
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    pub fn on_press(mut self, msg: M) -> Self {
        self.on_press = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<MobileActionBuilder<M>> for WidgetNode<M> {
    fn from(b: MobileActionBuilder<M>) -> Self {
        let mut row = crate::row::<M>().key(b.id).gap(12.0).padding(14.0);

        if let Some(icon_name) = b.icon {
            row = row.child(crate::icon::<M>(icon_name).build());
        }
        row = row.child(crate::label(b.label));

        if let Some(msg) = b.on_press {
            row = row.on_click(msg);
        }
        row.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Pressed,
    }

    #[test]
    fn mobile_action_produces_row() {
        let node: WidgetNode<Msg> = mobile_action("Delete").into();
        assert!(matches!(node, WidgetNode::Row(_)));
    }

    #[test]
    fn mobile_action_with_icon() {
        let node: WidgetNode<Msg> = mobile_action("Share")
            .icon(crate::IconName::Star)
            .into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        assert_eq!(r.children.len(), 2);
    }

    #[test]
    fn mobile_action_on_press() {
        let node: WidgetNode<Msg> = mobile_action("Go").on_press(Msg::Pressed).into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        assert_eq!(r.message, Some(Msg::Pressed));
    }

    #[test]
    fn mobile_action_destructive_flag() {
        let b = mobile_action::<Msg>("Remove").destructive(true);
        assert!(b.destructive);
    }
}
