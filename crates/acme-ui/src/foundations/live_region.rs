//! LiveRegion — aria-live region for screen reader announcements.
//! Aligns with Radix UI Announce / accessibility live region pattern.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Live region politeness level.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LiveRegionPoliteness {
    #[default]
    Polite,
    Assertive,
    Off,
}

/// Builder for a live region.
pub struct LiveRegionBuilder<M> {
    pub id: WidgetKey,
    pub message: String,
    pub politeness: LiveRegionPoliteness,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a live region builder.
pub fn live_region<M: Clone + 'static>(message: impl Into<String>) -> LiveRegionBuilder<M> {
    LiveRegionBuilder {
        id: WidgetKey::from("live_region"),
        message: message.into(),
        politeness: LiveRegionPoliteness::default(),
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> LiveRegionBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn politeness(mut self, value: LiveRegionPoliteness) -> Self {
        self.politeness = value;
        self
    }
}

impl<M: Clone + 'static> From<LiveRegionBuilder<M>> for WidgetNode<M> {
    fn from(b: LiveRegionBuilder<M>) -> Self {
        // Visually hidden label announced to screen readers.
        let mut node = crate::label(b.message);
        if let WidgetNode::Label(ref mut l) = node {
            l.font_size = Some(0.0);
        }
        node
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn live_region_produces_label() {
        let node: WidgetNode<Msg> = live_region("Item added").into();
        assert!(matches!(node, WidgetNode::Label(_)));
    }

    #[test]
    fn live_region_default_polite() {
        let b = live_region::<Msg>("msg");
        assert_eq!(b.politeness, LiveRegionPoliteness::Polite);
    }

    #[test]
    fn live_region_assertive() {
        let b = live_region::<Msg>("Error!").politeness(LiveRegionPoliteness::Assertive);
        assert_eq!(b.politeness, LiveRegionPoliteness::Assertive);
    }
}
