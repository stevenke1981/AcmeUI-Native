//! Mobile progress — determinate progress bar with optional label.

use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_layout::Length;
use acme_widgets::style::Styled;

const TRACK_HEIGHT: f32 = 6.0;

fn valid_total(total: f32) -> f32 {
    if total.is_finite() && total > 0.0 {
        total
    } else {
        1.0
    }
}

/// Builder for a mobile progress bar.
pub struct MobileProgressBuilder<M> {
    pub id: WidgetKey,
    pub value: f32,
    pub total: f32,
    pub label: Option<String>,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a mobile progress bar builder.
///
/// Nonpositive or non-finite totals use one. Values are clamped for display;
/// NaN means zero and infinities saturate to the corresponding endpoint.
pub fn mobile_progress<M: Clone + 'static>(value: f32, total: f32) -> MobileProgressBuilder<M> {
    MobileProgressBuilder {
        id: WidgetKey::from("mobile_progress"),
        value,
        total: valid_total(total),
        label: None,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> MobileProgressBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }

    /// Finite normalized fraction in [0.0, 1.0], including after public field edits.
    pub fn fraction(&self) -> f32 {
        if self.value.is_nan() {
            0.0
        } else {
            (self.value / valid_total(self.total)).clamp(0.0, 1.0)
        }
    }
}

impl<M: Clone + 'static> From<MobileProgressBuilder<M>> for WidgetNode<M> {
    fn from(b: MobileProgressBuilder<M>) -> Self {
        let fraction = b.fraction();
        let pct = (fraction * 100.0) as u32;
        let id = b.id.as_str();
        let fill = crate::row::<M>()
            .key(format!("{id}__fill"))
            .w(Length::Percent(fraction))
            .height(TRACK_HEIGHT)
            .flex_shrink(0.0)
            .bg_primary()
            .rounded(TRACK_HEIGHT / 2.0);
        let track = crate::row::<M>()
            .key(format!("{id}__track"))
            .height(TRACK_HEIGHT)
            .flex_shrink(0.0)
            .bg_muted()
            .rounded(TRACK_HEIGHT / 2.0)
            .child(fill);

        let mut col = crate::column::<M>()
            .key(b.id)
            .w(Length::Percent(1.0))
            .gap(4.0)
            .padding(8.0);
        if let Some(text) = b.label {
            col = col.child(crate::label(text));
        }
        // Text stays outside the thin track and uses the theme foreground.
        col.child(track)
            .child(crate::label(format!("{pct}%")))
            .build()
    }
}

#[cfg(test)]
#[path = "mobile_progress_tests.rs"]
mod tests;
