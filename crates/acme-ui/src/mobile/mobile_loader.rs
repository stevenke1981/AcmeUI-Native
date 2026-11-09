//! Mobile loader — indeterminate spinner indicator with optional message.
//!
//! Reuses the foundation spinner's static glyph; no animation timer is scheduled.
//! Messages wrap at character boundaries to fit narrow content areas.

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
            col = col.child(
                crate::label_builder(msg)
                    .wrap(crate::LabelWrap::Character)
                    .build(),
            );
        }
        col.build()
    }
}

#[cfg(test)]
#[path = "mobile_loader_tests.rs"]
mod tests;
