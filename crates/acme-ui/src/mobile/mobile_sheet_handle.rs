//! Mobile sheet handle — visual drag indicator bar for bottom sheets.
//!
//! This noninteractive decoration does not install a drag recognizer.

use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_layout::Length;
use acme_widgets::style::{ColorToken, Styled};

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
        let width = if b.width.is_finite() {
            b.width.max(0.0)
        } else {
            36.0
        };
        crate::row::<M>()
            .key(b.id)
            .size(width, 4.0)
            .min_w(Length::px(width))
            .min_h(Length::px(4.0))
            .flex_shrink(0.0)
            .bg(ColorToken::MutedForeground)
            .rounded(2.0)
            .padding(0.0)
            .build()
    }
}

#[cfg(test)]
#[path = "mobile_sheet_handle_tests.rs"]
mod tests;
