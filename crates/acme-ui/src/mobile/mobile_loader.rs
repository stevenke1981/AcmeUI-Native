//! Mobile loader — indeterminate spinner indicator with optional message.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a mobile loading indicator.
pub struct MobileLoaderBuilder<M> {
    pub id: WidgetKey,
    pub message: Option<String>,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a mobile loader builder.
pub fn mobile_loader<M: Clone + 'static>() -> MobileLoaderBuilder<M> {
    MobileLoaderBuilder {
        id: WidgetKey::from("mobile_loader"),
        message: None,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> MobileLoaderBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn message(mut self, text: impl Into<String>) -> Self {
        self.message = Some(text.into());
        self
    }
}

impl<M: Clone + 'static> From<MobileLoaderBuilder<M>> for WidgetNode<M> {
    fn from(b: MobileLoaderBuilder<M>) -> Self {
        let mut col = crate::column::<M>()
            .key(b.id)
            .gap(8.0)
            .padding(24.0)
            .child(crate::foundations::spinner::<M>().build());
        if let Some(msg) = b.message {
            col = col.child(crate::label(msg));
        }
        col.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn mobile_loader_produces_column() {
        let node: WidgetNode<Msg> = mobile_loader().into();
        assert!(matches!(node, WidgetNode::Column(_)));
    }

    #[test]
    fn mobile_loader_with_message() {
        let node: WidgetNode<Msg> = mobile_loader().message("Loading…").into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column");
        };
        assert_eq!(c.children.len(), 2);
    }

    #[test]
    fn mobile_loader_without_message() {
        let node: WidgetNode<Msg> = mobile_loader().into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column");
        };
        assert_eq!(c.children.len(), 1);
    }
}
