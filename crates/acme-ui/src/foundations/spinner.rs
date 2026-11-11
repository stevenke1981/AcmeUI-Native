//! Clock-driven spinner composed of themed circular surfaces.
//!
//! Hosts pass elapsed time and their motion preference with `animation` and
//! schedule redraws only while visible and motion is enabled. Without a clock,
//! the indicator stays at its initial pose. No timer is created by this builder.

use crate::WidgetNode;
use acme_layout::Length;
use acme_widgets::style::{ColorToken, Styled};
use std::time::Duration;

/// Builder for a spinner widget.
pub struct SpinnerBuilder<M> {
    pub size: crate::ControlSize,
    pub tone: crate::Tone,
    elapsed: Duration,
    reduced_motion: bool,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a spinner builder.
pub fn spinner<M>() -> SpinnerBuilder<M> {
    SpinnerBuilder {
        size: crate::ControlSize::Md,
        tone: crate::Tone::Neutral,
        elapsed: Duration::ZERO,
        reduced_motion: false,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> SpinnerBuilder<M> {
    /// Set the spinner size using a standard ControlSize.
    pub fn size(mut self, value: crate::ControlSize) -> Self {
        self.size = value;
        self
    }

    /// Set the spinner tone.
    pub fn tone(mut self, value: crate::Tone) -> Self {
        self.tone = value;
        self
    }

    /// Supply the host clock and reduced-motion preference.
    ///
    /// Normal motion rotates once per second. Reduced motion renders the same
    /// initial pose regardless of elapsed time; hosts should stop animation
    /// polling while reduced motion is enabled or the indicator is hidden.
    pub fn animation(mut self, elapsed: Duration, reduced_motion: bool) -> Self {
        self.elapsed = elapsed;
        self.reduced_motion = reduced_motion;
        self
    }

    /// Build a fixed-size Stack of seven circular surfaces.
    pub fn build(self) -> WidgetNode<M> {
        let extent = match self.size {
            crate::ControlSize::Xs => 22.0,
            crate::ControlSize::Sm => 28.0,
            crate::ControlSize::Md => 34.0,
            crate::ControlSize::Lg => 40.0,
            crate::ControlSize::Xl => 48.0,
        } * 0.75;
        let phase = if self.reduced_motion {
            0.0
        } else {
            // Integer seconds are whole rotations. Avoid precision loss (and
            // overflow) even when the host clock has been running for years.
            self.elapsed.subsec_nanos() as f32 / 1_000_000_000.0
        } * std::f32::consts::TAU;
        let color = match self.tone {
            crate::Tone::Neutral => ColorToken::Foreground,
            crate::Tone::Primary => ColorToken::Primary,
            crate::Tone::Success => ColorToken::Success,
            crate::Tone::Warning => ColorToken::Warning,
            crate::Tone::Danger => ColorToken::Danger,
            crate::Tone::Info => ColorToken::Info,
        };
        let mut ring = crate::stack()
            .size(extent, extent)
            .min_w(Length::px(extent))
            .min_h(Length::px(extent))
            .flex_shrink(0.0);
        let radius = extent * 0.37;
        for index in 0..7 {
            let angle =
                phase - std::f32::consts::FRAC_PI_2 - index as f32 * std::f32::consts::FRAC_PI_4;
            let diameter = extent * (0.22 - index as f32 * 0.02);
            let x = extent / 2.0 + radius * angle.cos() - diameter / 2.0;
            let y = extent / 2.0 + radius * angle.sin() - diameter / 2.0;
            ring = ring.child(
                crate::row()
                    .size(diameter, diameter)
                    .flex_shrink(0.0)
                    .ml(x)
                    .mt(y)
                    .bg(color)
                    .rounded(diameter / 2.0),
            );
        }
        ring.build()
    }
}

impl<M: Clone + 'static> From<SpinnerBuilder<M>> for WidgetNode<M> {
    fn from(b: SpinnerBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
#[path = "spinner_tests.rs"]
mod tests;
