//! Mobile toggle — switch control with label and checked state.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a mobile toggle switch.
pub struct MobileToggleBuilder<M> {
    pub id: WidgetKey,
    pub label: String,
    pub checked: bool,
    pub disabled: bool,
    pub on_change: Option<M>,
}

/// Create a mobile toggle builder.
pub fn mobile_toggle<M: Clone + 'static>(label: impl Into<String>) -> MobileToggleBuilder<M> {
    MobileToggleBuilder {
        id: WidgetKey::from("mobile_toggle"),
        label: label.into(),
        checked: false,
        disabled: false,
        on_change: None,
    }
}

impl<M: Clone + 'static> MobileToggleBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn checked(mut self, value: bool) -> Self {
        self.checked = value;
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    pub fn on_change(mut self, msg: M) -> Self {
        self.on_change = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<MobileToggleBuilder<M>> for WidgetNode<M> {
    fn from(b: MobileToggleBuilder<M>) -> Self {
        let indicator = if b.checked { "●" } else { "○" };
        let mut row = crate::row::<M>()
            .key(b.id)
            .gap(12.0)
            .padding(12.0)
            .child(crate::label(b.label))
            .child(crate::label(indicator));
        if let Some(msg) = b.on_change {
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
        Toggled,
    }

    #[test]
    fn mobile_toggle_produces_row() {
        let node: WidgetNode<Msg> = mobile_toggle("Wi-Fi").into();
        assert!(matches!(node, WidgetNode::Row(_)));
    }

    #[test]
    fn mobile_toggle_has_label_and_indicator() {
        let node: WidgetNode<Msg> = mobile_toggle("BT").checked(true).into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        assert_eq!(r.children.len(), 2);
    }

    #[test]
    fn mobile_toggle_on_click_sets_message() {
        let node: WidgetNode<Msg> = mobile_toggle("X").on_change(Msg::Toggled).into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        assert_eq!(r.message, Some(Msg::Toggled));
    }
}
