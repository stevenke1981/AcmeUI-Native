//! LiveRegion — aria-live region for screen reader announcements.
//! Aligns with Radix UI Announce / accessibility live region pattern.

use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_widgets::{LivePoliteness, label_builder};

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
        let live = match b.politeness {
            LiveRegionPoliteness::Off => LivePoliteness::Off,
            LiveRegionPoliteness::Polite => LivePoliteness::Polite,
            LiveRegionPoliteness::Assertive => LivePoliteness::Assertive,
        };
        // Visually hidden label announced to screen readers via AccessKit live region.
        label_builder(b.message).font_size(0.0).live(live).build()
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
        let WidgetNode::Label(label) = node else {
            panic!("expected Label");
        };
        assert_eq!(label.live, Some(LivePoliteness::Polite));
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
        let node: WidgetNode<Msg> = b.into();
        let WidgetNode::Label(label) = node else {
            panic!("expected Label");
        };
        assert_eq!(label.live, Some(LivePoliteness::Assertive));
    }

    #[test]
    fn live_region_off_maps_to_off() {
        let node: WidgetNode<Msg> = live_region("Quiet")
            .politeness(LiveRegionPoliteness::Off)
            .into();
        let WidgetNode::Label(label) = node else {
            panic!("expected Label");
        };
        assert_eq!(label.live, Some(LivePoliteness::Off));
    }
}
