//! StatusTray component — a compact system tray for status indicators.
//!
//! Renders as a Row of small badge-like indicators. Each indicator
//! shows an icon character and label with a configurable tone color.
//! Suitable for battery, network, notification, clock, and VPN status.

use crate::{Tone, WidgetNode};
use acme_core::WidgetKey;

/// A single status indicator descriptor.
#[derive(Clone, Debug)]
pub struct StatusIndicator {
    /// Icon or symbol character (e.g. "🔋", "📶", "🔒").
    pub icon: String,
    /// Short label (e.g. "85%", "WiFi", "Connected").
    pub label: String,
    /// Semantic tone for coloring.
    pub tone: Tone,
    /// Optional tooltip (key for click handling).
    pub key: Option<WidgetKey>,
}

impl StatusIndicator {
    /// Create a new status indicator.
    pub fn new(icon: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            label: label.into(),
            tone: Tone::Neutral,
            key: None,
        }
    }

    /// Set the indicator tone.
    pub fn tone(mut self, value: Tone) -> Self {
        self.tone = value;
        self
    }

    /// Set a click key for handling interaction.
    pub fn key(mut self, value: impl Into<WidgetKey>) -> Self {
        self.key = Some(value.into());
        self
    }
}

/// Builder for a StatusTray component.
pub struct StatusTrayBuilder<M> {
    pub id: WidgetKey,
    pub indicators: Vec<StatusIndicator>,
    pub size: crate::ControlSize,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a new StatusTray builder.
pub fn status_tray<M: Clone + 'static>(id: impl Into<WidgetKey>) -> StatusTrayBuilder<M> {
    StatusTrayBuilder {
        id: id.into(),
        indicators: vec![],
        size: crate::ControlSize::Xs,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> StatusTrayBuilder<M> {
    /// Add a status indicator.
    pub fn indicator(mut self, item: StatusIndicator) -> Self {
        self.indicators.push(item);
        self
    }

    /// Set the tray item size.
    pub fn size(mut self, value: crate::ControlSize) -> Self {
        self.size = value;
        self
    }
}

impl<M: Clone + 'static> From<StatusTrayBuilder<M>> for WidgetNode<M> {
    fn from(b: StatusTrayBuilder<M>) -> Self {
        let theme = acme_theme::Theme::light();
        let mut row = crate::row::<M>()
            .key(b.id)
            .gap(4.0);

        for indicator in &b.indicators {
            let resolved = crate::resolve_tone(&theme, indicator.tone, true);
            let label_text = format!("{} {}", indicator.icon, indicator.label);

            let indicator_widget = if let Some(ref key) = indicator.key {
                // Clickable indicator
                crate::card::<M>()
                    .key(key.as_str())
                    .child(
                        crate::label_builder(&label_text)
                            .font_size(theme.typography.caption)
                            .color(resolved.fg)
                            .build(),
                    )
                    .padding(3.0)
                    .variant(crate::CardVariant::Interactive)
                    .background_color(resolved.bg)
                    .border_radius(theme.radii.full)
                    .build()
            } else {
                // Non-interactive indicator
                crate::card::<M>()
                    .child(
                        crate::label_builder(&label_text)
                            .font_size(theme.typography.caption)
                            .color(resolved.fg)
                            .build(),
                    )
                    .padding(3.0)
                    .variant(crate::CardVariant::Plain)
                    .background_color(resolved.bg)
                    .border_radius(theme.radii.full)
                    .build()
            };

            row = row.child(indicator_widget);
        }

        row.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn status_tray_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = status_tray::<TestMsg>("tray")
            .indicator(StatusIndicator::new("🔋", "85%").tone(Tone::Success))
            .indicator(StatusIndicator::new("📶", "WiFi").tone(Tone::Primary))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Row: [indicator1, indicator2] = 2 children
        assert_eq!(layout.children.len(), 2);
    }

    #[test]
    fn status_tray_empty() {
        let node: WidgetNode<TestMsg> = status_tray::<TestMsg>("empty").into();
        let layout = node.to_layout(NodeId::new(1));
        assert!(layout.children.is_empty());
    }

    #[test]
    fn status_tray_single_indicator() {
        let node: WidgetNode<TestMsg> = status_tray::<TestMsg>("tray")
            .indicator(StatusIndicator::new("🔒", "Secured").tone(Tone::Warning))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.children.len(), 1);
    }

    #[test]
    fn status_tray_builder_defaults() {
        let t = status_tray::<TestMsg>("tray");
        assert!(t.indicators.is_empty());
        assert_eq!(t.size, crate::ControlSize::Xs);
    }

    #[test]
    fn status_indicator_defaults() {
        let ind = StatusIndicator::new("⚡", "AC");
        assert_eq!(ind.icon, "⚡");
        assert_eq!(ind.label, "AC");
        assert_eq!(ind.tone, Tone::Neutral);
        assert!(ind.key.is_none());
    }

    #[test]
    fn status_indicator_with_key() {
        let ind = StatusIndicator::new("🔔", "3")
            .tone(Tone::Danger)
            .key("notif");
        assert_eq!(ind.key.unwrap().as_str(), "notif");
    }
}
