//! Mobile toggle — switch control with label and checked state.
//!
//! Layout: a 44 px press target (a Button) with a 52 x 32 track and a 24 px
//! thumb on top. Colours come from theme tokens (primary when on, muted when off).

use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_widgets::style::prelude::*;

/// Minimum touch target height, as `mobile_button` md.
const TOUCH_HEIGHT: f32 = 44.0;
const TRACK_WIDTH: f32 = 52.0;
const TRACK_HEIGHT: f32 = 32.0;
const TRACK_INSET: f32 = 4.0;
const THUMB: f32 = 24.0;
/// Free width of the track around the thumb.
const THUMB_TRAVEL: f32 = TRACK_WIDTH - 2.0 * TRACK_INSET - THUMB;

/// Builder for a mobile toggle switch.
pub struct MobileToggleBuilder<M> {
    pub id: WidgetKey,
    pub label: String,
    pub checked: bool,
    pub disabled: bool,
    pub on_change: Option<M>,
    /// Maps the new checked value to a message (ROADMAP 014).
    pub on_toggle: Option<fn(bool) -> M>,
}

/// Create a mobile toggle builder.
pub fn mobile_toggle<M: Clone + 'static>(label: impl Into<String>) -> MobileToggleBuilder<M> {
    MobileToggleBuilder {
        id: WidgetKey::from("mobile_toggle"),
        label: label.into(),
        checked: false,
        disabled: false,
        on_change: None,
        on_toggle: None,
    }
}

impl<M: Clone + 'static> MobileToggleBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn checked(mut self, value: bool) -> Self {
        self.checked = value;
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    pub fn on_change(mut self, msg: M) -> Self {
        self.on_change = Some(msg);
        self
    }

    /// Emit `f(new_value)` when pressed, where `new_value` is the opposite of
    /// the current checked state. Takes precedence over [`Self::on_change`].
    pub fn on_toggle(mut self, f: fn(bool) -> M) -> Self {
        self.on_toggle = Some(f);
        self
    }
}

impl<M: Clone + 'static> From<MobileToggleBuilder<M>> for WidgetNode<M> {
    /// A `Stack` holding a full-width, 44 px `Button` (the press target and the
    /// accessible name, which ends in the state: "Wi-Fi · On") and, on top of
    /// it, a switch track with a thumb at the start (off) or the end (on).
    /// The press emits `on_toggle(!checked)`, else the fixed `on_change`
    /// message; a disabled toggle emits nothing.
    fn from(b: MobileToggleBuilder<M>) -> Self {
        let id = b.id.as_str().to_string();
        let state = if b.checked { "On" } else { "Off" };
        let message = b.on_toggle.map(|f| f(!b.checked)).or(b.on_change);

        let mut host = crate::button(format!("{id}__host"), format!("{} · {state}", b.label))
            .variant(crate::ButtonVariant::Ghost)
            .full_width(true)
            .disabled(b.disabled);
        host.style.height = Some(acme_layout::Length::px(TOUCH_HEIGHT));
        let host: WidgetNode<M> = match message {
            Some(msg) => host.on_click(msg),
            None => host.into(),
        };

        let thumb = crate::row::<M>()
            .key(format!("{id}__thumb"))
            .size(THUMB, THUMB)
            .bg_surface()
            .rounded(THUMB / 2.0);
        let mut track = crate::row::<M>()
            .key(format!("{id}__track"))
            .size(TRACK_WIDTH, TRACK_HEIGHT)
            .padding(TRACK_INSET)
            .rounded(TRACK_HEIGHT / 2.0);
        track = if b.checked {
            track.bg_primary()
        } else {
            track.bg_muted()
        };
        if b.checked {
            // The thumb travels the free width of the track: off is at the
            // start, on is at the end.
            track = track.child(crate::row::<M>().size(THUMB_TRAVEL, THUMB));
        }
        let track = track.child(thumb);

        let mut grow = crate::row::<M>();
        grow.style_mut().flex_grow = Some(1.0);
        let overlay = crate::row::<M>()
            .padding((TOUCH_HEIGHT - TRACK_HEIGHT) / 2.0)
            .w_full()
            .height(TOUCH_HEIGHT)
            .child(grow)
            .child(track);

        crate::stack::<M>()
            .key(b.id)
            .w_full()
            .height(TOUCH_HEIGHT)
            .child(host)
            .child(overlay)
            .build()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Toggled,
    }

    #[test]
    fn mobile_toggle_produces_stack() {
        let node: WidgetNode<Msg> = mobile_toggle("Wi-Fi").into();
        assert!(matches!(node, WidgetNode::Stack(_)));
    }

    #[test]
    fn mobile_toggle_has_press_host_and_track_overlay() {
        let node: WidgetNode<Msg> = mobile_toggle("BT").checked(true).into();
        let WidgetNode::Stack(r) = &node else {
            panic!("expected Stack");
        };
        assert_eq!(r.children.len(), 2);
        let WidgetNode::Button(host) = &r.children[0] else {
            panic!("expected Button");
        };
        assert_eq!(host.label, "BT · On");
        assert!(matches!(&r.children[1], WidgetNode::Row(_)));
    }

    #[test]
    fn mobile_toggle_on_click_sets_message() {
        let node: WidgetNode<Msg> = mobile_toggle("X").on_change(Msg::Toggled).into();
        let WidgetNode::Stack(r) = &node else {
            panic!("expected Stack");
        };
        let WidgetNode::Button(host) = &r.children[0] else {
            panic!("expected Button");
        };
        assert_eq!(host.activate(), Some(&Msg::Toggled));
    }
}

#[cfg(test)]
#[path = "mobile_toggle_tests.rs"]
mod interaction_tests;
#[cfg(test)]
#[path = "mobile_test_support.rs"]
pub(crate) mod test_support;
