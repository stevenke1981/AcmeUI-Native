//! Mobile loader — indeterminate spinner indicator with optional message.
//!
//! Hosts drive the foundation spinner via `animation`; this builder creates no timer.
//! Messages wrap at character boundaries to fit narrow content areas.

use crate::WidgetNode;
use acme_core::WidgetKey;
use std::time::Duration;

/// Builder for a mobile loading indicator.
pub struct MobileLoaderBuilder<M> {
    pub id: WidgetKey,
    pub message: Option<String>,
    elapsed: Duration,
    reduced_motion: bool,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a mobile loader builder.
pub fn mobile_loader<M: Clone + 'static>() -> MobileLoaderBuilder<M> {
    MobileLoaderBuilder {
        id: WidgetKey::from("mobile_loader"),
        message: None,
        elapsed: Duration::ZERO,
        reduced_motion: false,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> MobileLoaderBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    /// Forward the host clock and reduced-motion preference to the spinner.
    /// Stop animation polling while hidden or reduced motion is enabled.
    pub fn animation(mut self, elapsed: Duration, reduced_motion: bool) -> Self {
        self.elapsed = elapsed;
        self.reduced_motion = reduced_motion;
        self
    }

    pub fn message(mut self, text: impl Into<String>) -> Self {
        self.message = Some(text.into());
        self
    }
}

impl<M: Clone + 'static> From<MobileLoaderBuilder<M>> for WidgetNode<M> {
    fn from(b: MobileLoaderBuilder<M>) -> Self {
        let mut col = crate::column::<M>().key(b.id).gap(8.0).padding(24.0).child(
            crate::foundations::spinner::<M>()
                .animation(b.elapsed, b.reduced_motion)
                .build(),
        );
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
