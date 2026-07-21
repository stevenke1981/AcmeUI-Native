//! KbdCombo — keyboard shortcut combination display (e.g. Ctrl+Shift+P).
//! Absorbs gpui-component's keyboard shortcut display strength.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a keyboard shortcut combination.
pub struct KbdComboBuilder<M> {
    pub id: WidgetKey,
    pub keys: Vec<String>,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a keyboard combo builder.
pub fn kbd_combo<M: Clone + 'static>(keys: Vec<impl Into<String>>) -> KbdComboBuilder<M> {
    KbdComboBuilder {
        id: WidgetKey::from("kbd_combo"),
        keys: keys.into_iter().map(Into::into).collect(),
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> KbdComboBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }
}

impl<M: Clone + 'static> From<KbdComboBuilder<M>> for WidgetNode<M> {
    fn from(b: KbdComboBuilder<M>) -> Self {
        let mut row = crate::row::<M>().key(b.id).gap(2.0).padding(2.0);
        for (i, key) in b.keys.iter().enumerate() {
            if i > 0 {
                row = row.child(crate::label("+"));
            }
            row = row.child(crate::label(format!("[{}]", key)));
        }
        row.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn kbd_combo_produces_row() {
        let node: WidgetNode<Msg> = kbd_combo(vec!["Ctrl", "C"]).into();
        assert!(matches!(node, WidgetNode::Row(_)));
    }

    #[test]
    fn kbd_combo_child_count() {
        let node: WidgetNode<Msg> = kbd_combo(vec!["Ctrl", "Shift", "P"]).into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        // 3 keys + 2 separators = 5
        assert_eq!(r.children.len(), 5);
    }

    #[test]
    fn kbd_combo_single_key() {
        let node: WidgetNode<Msg> = kbd_combo(vec!["Esc"]).into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        assert_eq!(r.children.len(), 1);
    }
}
