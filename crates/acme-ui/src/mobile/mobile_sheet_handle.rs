//! Mobile sheet handle — drag indicator bar for bottom sheets.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a mobile sheet drag handle.
pub struct MobileSheetHandleBuilder<M> {
    pub id: WidgetKey,
    pub width: f32,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a mobile sheet handle builder.
pub fn mobile_sheet_handle<M: Clone + 'static>() -> MobileSheetHandleBuilder<M> {
    MobileSheetHandleBuilder {
        id: WidgetKey::from("mobile_sheet_handle"),
        width: 36.0,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> MobileSheetHandleBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn width(mut self, value: f32) -> Self {
        self.width = value;
        self
    }
}

impl<M: Clone + 'static> From<MobileSheetHandleBuilder<M>> for WidgetNode<M> {
    fn from(b: MobileSheetHandleBuilder<M>) -> Self {
        crate::row::<M>()
            .key(b.id)
            .size(b.width, 4.0)
            .padding(0.0)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn mobile_sheet_handle_produces_row() {
        let node: WidgetNode<Msg> = mobile_sheet_handle().into();
        assert!(matches!(node, WidgetNode::Row(_)));
    }

    #[test]
    fn mobile_sheet_handle_default_width() {
        let b = mobile_sheet_handle::<Msg>();
        assert_eq!(b.width, 36.0);
    }

    #[test]
    fn mobile_sheet_handle_custom_width() {
        let b = mobile_sheet_handle::<Msg>().width(48.0);
        assert_eq!(b.width, 48.0);
    }
}
