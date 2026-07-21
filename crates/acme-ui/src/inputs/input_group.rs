//! InputGroup — input with leading/trailing addons (prefix, suffix, buttons).
//! Absorbs gpui-component's input-with-addons strength.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for an input group.
pub struct InputGroupBuilder<M> {
    pub id: WidgetKey,
    pub placeholder: String,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub leading: Option<WidgetNode<M>>,
    pub trailing: Option<WidgetNode<M>>,
}

/// Create an input group builder.
pub fn input_group<M: Clone + 'static>(placeholder: impl Into<String>) -> InputGroupBuilder<M> {
    InputGroupBuilder {
        id: WidgetKey::from("input_group"),
        placeholder: placeholder.into(),
        prefix: None,
        suffix: None,
        leading: None,
        trailing: None,
    }
}

impl<M: Clone + 'static> InputGroupBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn prefix(mut self, text: impl Into<String>) -> Self {
        self.prefix = Some(text.into());
        self
    }

    pub fn suffix(mut self, text: impl Into<String>) -> Self {
        self.suffix = Some(text.into());
        self
    }

    pub fn leading(mut self, node: WidgetNode<M>) -> Self {
        self.leading = Some(node);
        self
    }

    pub fn trailing(mut self, node: WidgetNode<M>) -> Self {
        self.trailing = Some(node);
        self
    }
}

impl<M: Clone + 'static> From<InputGroupBuilder<M>> for WidgetNode<M> {
    fn from(b: InputGroupBuilder<M>) -> Self {
        let mut row = crate::row::<M>().key(b.id).gap(4.0).padding(4.0);

        if let Some(leading) = b.leading {
            row = row.child(leading);
        }
        if let Some(prefix) = b.prefix {
            row = row.child(crate::label(prefix));
        }
        row = row.child(crate::label(b.placeholder));
        if let Some(suffix) = b.suffix {
            row = row.child(crate::label(suffix));
        }
        if let Some(trailing) = b.trailing {
            row = row.child(trailing);
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
    fn input_group_produces_row() {
        let node: WidgetNode<Msg> = input_group("amount").into();
        assert!(matches!(node, WidgetNode::Row(_)));
    }

    #[test]
    fn input_group_with_prefix_suffix() {
        let node: WidgetNode<Msg> = input_group("0.00")
            .prefix("$")
            .suffix("USD")
            .into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        // prefix + placeholder + suffix = 3
        assert_eq!(r.children.len(), 3);
    }

    #[test]
    fn input_group_with_leading_trailing() {
        let node: WidgetNode<Msg> = input_group("search")
            .leading(crate::label("⌕"))
            .trailing(crate::label("✕"))
            .into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        // leading + placeholder + trailing = 3
        assert_eq!(r.children.len(), 3);
    }
}
