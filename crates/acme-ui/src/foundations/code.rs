//! Code — inline and block code display.
//! Aligns with Radix Themes Code component.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Code display variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CodeVariant {
    #[default]
    Soft,
    Solid,
    Outline,
    Ghost,
}

/// Builder for a code display.
pub struct CodeBuilder<M> {
    pub id: WidgetKey,
    pub text: String,
    pub variant: CodeVariant,
    pub block: bool,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a code builder.
pub fn code<M: Clone + 'static>(text: impl Into<String>) -> CodeBuilder<M> {
    CodeBuilder {
        id: WidgetKey::from("code"),
        text: text.into(),
        variant: CodeVariant::default(),
        block: false,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> CodeBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn variant(mut self, value: CodeVariant) -> Self {
        self.variant = value;
        self
    }

    pub fn block(mut self, value: bool) -> Self {
        self.block = value;
        self
    }
}

impl<M: Clone + 'static> From<CodeBuilder<M>> for WidgetNode<M> {
    fn from(b: CodeBuilder<M>) -> Self {
        if b.block {
            crate::card::<M>()
                .key(b.id)
                .variant(acme_widgets::CardVariant::Muted)
                .padding(12.0)
                .child(crate::label(b.text))
                .build()
        } else {
            let mut node = crate::label(format!("`{}`", b.text));
            if let WidgetNode::Label(ref mut l) = node {
                l.font_size = Some(13.0);
            }
            node
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn code_inline_produces_label() {
        let node: WidgetNode<Msg> = code("let x = 1").into();
        assert!(matches!(node, WidgetNode::Label(_)));
    }

    #[test]
    fn code_inline_wraps_backticks() {
        let node: WidgetNode<Msg> = code("foo").into();
        let WidgetNode::Label(l) = &node else {
            panic!("expected Label");
        };
        assert_eq!(l.text, "`foo`");
    }

    #[test]
    fn code_block_produces_card() {
        let node: WidgetNode<Msg> = code("fn main() {}").block(true).into();
        assert!(matches!(node, WidgetNode::Card(_)));
    }

    #[test]
    fn code_default_variant_soft() {
        let b = code::<Msg>("x");
        assert_eq!(b.variant, CodeVariant::Soft);
    }
}
