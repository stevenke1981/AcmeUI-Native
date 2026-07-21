//! Rating component — star rating display/input (1–5 stars).
//!
//! Renders as a [`Row`] of star symbols (★ = filled, ☆ = empty) with
//! an optional "{value}/{max}" text suffix.

use crate::ControlSize;
use acme_core::WidgetKey;
use acme_widgets::*;

/// Builder for a Rating component.
pub struct RatingBuilder<M> {
    pub id: WidgetKey,
    pub value: u32,
    pub max: u32,
    pub size: ControlSize,
    pub readonly: bool,
    pub show_value: bool,
    pub on_change: Option<M>,
}

/// Create a new Rating builder.
pub fn rating<M: Clone + 'static>(id: impl Into<WidgetKey>) -> RatingBuilder<M> {
    RatingBuilder {
        id: id.into(),
        value: 0,
        max: 5,
        size: ControlSize::Md,
        readonly: false,
        show_value: false,
        on_change: None,
    }
}

impl<M: Clone + 'static> RatingBuilder<M> {
    /// Set the current rating value (clamped between 0 and max).
    pub fn value(mut self, v: u32) -> Self {
        self.value = v;
        self
    }

    /// Set the maximum number of stars (default 5).
    pub fn max(mut self, m: u32) -> Self {
        self.max = m;
        self
    }

    /// Set the control size for the star symbols.
    pub fn size(mut self, s: ControlSize) -> Self {
        self.size = s;
        self
    }

    /// Set whether the rating is read-only (display only).
    pub fn readonly(mut self, v: bool) -> Self {
        self.readonly = v;
        self
    }

    /// Set whether to show "{value}/{max}" text after the stars.
    pub fn show_value(mut self, v: bool) -> Self {
        self.show_value = v;
        self
    }

    /// Set the message dispatched when the rating is changed.
    pub fn on_change(mut self, msg: M) -> Self {
        self.on_change = Some(msg);
        self
    }
}

// ---------------------------------------------------------------------------
// From impl
// ---------------------------------------------------------------------------

impl<M: Clone + 'static> From<RatingBuilder<M>> for WidgetNode<M> {
    fn from(b: RatingBuilder<M>) -> Self {
        let mut r = row::<M>().key(b.id).gap(2.0);

        for i in 0..b.max {
            let star_char = if i < b.value { "★" } else { "☆" };
            r = r.child(label::<M>(star_char));
        }

        if b.show_value {
            let value_text = format!("{}/{}", b.value, b.max);
            r = r.child(label::<M>(value_text));
        }

        r.build()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn rating_builder_defaults() {
        let r = rating::<TestMsg>("rate1");
        assert_eq!(r.value, 0);
        assert_eq!(r.max, 5);
        assert!(!r.readonly);
        assert!(!r.show_value);
        assert!(r.on_change.is_none());
    }

    #[test]
    fn rating_builds_into_row() {
        let node: WidgetNode<TestMsg> = rating("rate2").value(3).into();
        let WidgetNode::Row(container) = &node else {
            panic!("expected Row");
        };
        // 5 stars (value=3, max=5, show_value=false)
        assert_eq!(container.children.len(), 5);
    }

    #[test]
    fn rating_star_count_matches_max() {
        let node: WidgetNode<TestMsg> = rating("rate3").max(3).value(1).into();
        let WidgetNode::Row(container) = &node else {
            panic!("expected Row");
        };
        assert_eq!(container.children.len(), 3);
    }

    #[test]
    fn rating_filled_stars_match_value() {
        let node: WidgetNode<TestMsg> = rating("rate4").max(4).value(2).into();
        let WidgetNode::Row(container) = &node else {
            panic!("expected Row");
        };
        assert_eq!(container.children.len(), 4);
        // First two children should be filled stars
        let WidgetNode::Label(l0) = &container.children[0] else {
            panic!("expected Label at index 0");
        };
        assert_eq!(l0.text, "★");
        let WidgetNode::Label(l1) = &container.children[1] else {
            panic!("expected Label at index 1");
        };
        assert_eq!(l1.text, "★");
        // Last two should be empty stars
        let WidgetNode::Label(l2) = &container.children[2] else {
            panic!("expected Label at index 2");
        };
        assert_eq!(l2.text, "☆");
        let WidgetNode::Label(l3) = &container.children[3] else {
            panic!("expected Label at index 3");
        };
        assert_eq!(l3.text, "☆");
    }

    #[test]
    fn rating_show_value_appends_text() {
        let node: WidgetNode<TestMsg> = rating("rate5").value(4).max(5).show_value(true).into();
        let WidgetNode::Row(container) = &node else {
            panic!("expected Row");
        };
        // 5 stars + 1 value text = 6 children
        assert_eq!(container.children.len(), 6);
        let WidgetNode::Label(last) = &container.children[5] else {
            panic!("expected Label at index 5 (value text)");
        };
        assert_eq!(last.text, "4/5");
    }
}
