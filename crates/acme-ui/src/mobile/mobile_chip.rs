//! Mobile chip — compact tag with optional close icon.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a mobile chip.
pub struct MobileChipBuilder<M> {
    pub id: WidgetKey,
    pub text: String,
    pub selected: bool,
    pub on_dismiss: Option<M>,
    pub on_tap: Option<M>,
}

/// Create a mobile chip builder.
pub fn mobile_chip<M: Clone + 'static>(text: impl Into<String>) -> MobileChipBuilder<M> {
    MobileChipBuilder {
        id: WidgetKey::from("mobile_chip"),
        text: text.into(),
        selected: false,
        on_dismiss: None,
        on_tap: None,
    }
}

impl<M: Clone + 'static> MobileChipBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn selected(mut self, value: bool) -> Self {
        self.selected = value;
        self
    }

    pub fn on_dismiss(mut self, msg: M) -> Self {
        self.on_dismiss = Some(msg);
        self
    }

    pub fn on_tap(mut self, msg: M) -> Self {
        self.on_tap = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<MobileChipBuilder<M>> for WidgetNode<M> {
    fn from(b: MobileChipBuilder<M>) -> Self {
        let prefix = if b.selected { "✓ " } else { "" };
        let text = format!("{}{}", prefix, b.text);
        let mut row = crate::row::<M>()
            .key(b.id)
            .gap(4.0)
            .padding(6.0)
            .child(crate::label(text));

        if let Some(msg) = b.on_dismiss {
            let close = crate::button("chip_close", "✕").on_click(msg);
            row = row.child(close);
        }
        if let Some(msg) = b.on_tap {
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
        Dismiss,
        Tap,
    }

    #[test]
    fn mobile_chip_produces_row() {
        let node: WidgetNode<Msg> = mobile_chip("Tag").into();
        assert!(matches!(node, WidgetNode::Row(_)));
    }

    #[test]
    fn mobile_chip_with_dismiss_has_close_button() {
        let node: WidgetNode<Msg> = mobile_chip("X").on_dismiss(Msg::Dismiss).into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        assert_eq!(r.children.len(), 2);
    }

    #[test]
    fn mobile_chip_selected_prefix() {
        let b = mobile_chip::<Msg>("Filter").selected(true);
        assert!(b.selected);
    }
}
