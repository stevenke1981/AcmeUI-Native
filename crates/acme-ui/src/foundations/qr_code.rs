//! QRCode — QR code display placeholder.
//! Aligns with Ant Design QRCode component.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a QR code display.
pub struct QRCodeBuilder<M> {
    pub id: WidgetKey,
    pub value: String,
    pub size: f32,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a QR code builder.
pub fn qr_code<M: Clone + 'static>(value: impl Into<String>) -> QRCodeBuilder<M> {
    QRCodeBuilder {
        id: WidgetKey::from("qr_code"),
        value: value.into(),
        size: 128.0,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> QRCodeBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn size(mut self, value: f32) -> Self {
        self.size = value;
        self
    }
}

impl<M: Clone + 'static> From<QRCodeBuilder<M>> for WidgetNode<M> {
    fn from(b: QRCodeBuilder<M>) -> Self {
        // Placeholder: renders a bordered box with the value text
        crate::stack::<M>()
            .key(b.id)
            .size(b.size, b.size)
            .child(crate::label(format!("⬜ {}", b.value)))
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn qr_code_produces_stack() {
        let node: WidgetNode<Msg> = qr_code("https://example.com").into();
        assert!(matches!(node, WidgetNode::Stack(_)));
    }

    #[test]
    fn qr_code_default_size() {
        let b = qr_code::<Msg>("test");
        assert_eq!(b.size, 128.0);
    }

    #[test]
    fn qr_code_custom_size() {
        let b = qr_code::<Msg>("x").size(256.0);
        assert_eq!(b.size, 256.0);
    }
}
