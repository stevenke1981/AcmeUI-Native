//! Interaction and geometry tests for `mobile_toggle`.

use super::*;
use crate::mobile::mobile_toggle::test_support::{Laid, lay_out};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Set(bool),
    Fixed,
}

fn laid(builder: MobileToggleBuilder<Msg>) -> Laid<Msg> {
    lay_out(builder.key("wifi").into())
}

#[test]
fn press_emits_the_new_value() {
    let off = laid(mobile_toggle("Wi-Fi").on_toggle(Msg::Set));
    assert_eq!(off.press_button(0), Some(Msg::Set(true)));
    let on = laid(mobile_toggle("Wi-Fi").checked(true).on_toggle(Msg::Set));
    assert_eq!(on.press_button(0), Some(Msg::Set(false)));
}

#[test]
fn pressing_the_track_or_the_label_toggles() {
    let l = laid(mobile_toggle("Wi-Fi").on_toggle(Msg::Set));
    let [tx, ty, tw, th] = l.rect_of_key("wifi__track");
    assert_eq!(
        l.press_at(tx + tw / 2.0, ty + th / 2.0),
        Some(Msg::Set(true))
    );
    let [hx, hy, _, hh] = l.hits()[0].rect;
    assert_eq!(l.press_at(hx + 4.0, hy + hh / 2.0), Some(Msg::Set(true)));
}

#[test]
fn fixed_on_change_message_still_works_and_on_toggle_wins() {
    let fixed = laid(mobile_toggle("X").on_change(Msg::Fixed));
    assert_eq!(fixed.press_button(0), Some(Msg::Fixed));
    let both = laid(mobile_toggle("X").on_change(Msg::Fixed).on_toggle(Msg::Set));
    assert_eq!(both.press_button(0), Some(Msg::Set(true)));
}

#[test]
fn disabled_emits_nothing() {
    let l = laid(
        mobile_toggle("X")
            .disabled(true)
            .on_toggle(Msg::Set)
            .on_change(Msg::Fixed),
    );
    assert_eq!(l.press_button(0), None);
    let [tx, ty, tw, th] = l.rect_of_key("wifi__track");
    assert_eq!(l.press_at(tx + tw / 2.0, ty + th / 2.0), None);
}

#[test]
fn no_message_means_nothing_to_press() {
    assert_eq!(laid(mobile_toggle("X")).press_button(0), None);
}

#[test]
fn track_and_thumb_geometry_reflects_the_checked_state() {
    let off = laid(mobile_toggle("Wi-Fi").on_toggle(Msg::Set));
    let on = laid(mobile_toggle("Wi-Fi").checked(true).on_toggle(Msg::Set));
    for (l, is_on) in [(&off, false), (&on, true)] {
        let [tx, ty, tw, th] = l.rect_of_key("wifi__track");
        let [bx, by, bw, bh] = l.rect_of_key("wifi__thumb");
        assert_eq!((tw, th), (52.0, 32.0), "track size");
        assert_eq!((bw, bh), (24.0, 24.0), "thumb size");
        assert!(bx >= tx && bx + bw <= tx + tw, "thumb inside track");
        assert!(by >= ty && by + bh <= ty + th, "thumb inside track");
        let inset_left = bx - tx;
        let inset_right = tx + tw - (bx + bw);
        if is_on {
            assert!(inset_right < inset_left, "on: thumb at the end");
        } else {
            assert!(inset_left < inset_right, "off: thumb at the start");
        }
    }
    // The track sits in the trailing part of the control.
    let [hx, _, hw, _] = off.hits()[0].rect;
    let [tx, _, tw, _] = off.rect_of_key("wifi__track");
    assert!(tx + tw <= hx + hw && tx > hx + hw / 2.0);
}

#[test]
fn touch_target_is_at_least_44_px() {
    let l = laid(mobile_toggle("Wi-Fi").on_toggle(Msg::Set));
    let [_, _, _, h] = l.hits()[0].rect;
    assert!(h >= 44.0, "height {h}");
    let [_, _, _, root_h] = {
        let r = l.snapshot.get(l.layout.id).copied().unwrap();
        [r.x, r.y, r.width, r.height]
    };
    assert!(root_h >= 44.0, "control height {root_h}");
}

#[test]
fn checked_state_is_in_the_accessible_name() {
    for (checked, state) in [(false, "Off"), (true, "On")] {
        let node: WidgetNode<Msg> = mobile_toggle("Wi-Fi").checked(checked).into();
        let button = find_button(&node).expect("a button hosts the toggle");
        assert!(button.label.starts_with("Wi-Fi"), "{}", button.label);
        assert!(button.label.ends_with(state), "{}", button.label);
    }
}

#[test]
fn track_colours_are_semantic_theme_tokens() {
    use acme_widgets::style::ColorToken;
    for (checked, expected) in [(false, ColorToken::Muted), (true, ColorToken::Primary)] {
        let node: WidgetNode<Msg> = mobile_toggle("Wi-Fi").checked(checked).into();
        let WidgetNode::Stack(root) = node else {
            panic!("expected Stack")
        };
        let WidgetNode::Row(overlay) = &root.children[1] else {
            panic!("expected overlay")
        };
        let WidgetNode::Row(track) = &overlay.children[1] else {
            panic!("expected track")
        };
        assert_eq!(track.style.background, Some(expected));
        let WidgetNode::Row(thumb) = track.children.last().unwrap() else {
            panic!("expected thumb")
        };
        assert_eq!(thumb.style.background, Some(ColorToken::Surface));
    }
}

fn find_button<M>(node: &WidgetNode<M>) -> Option<&crate::Button<M>> {
    if let WidgetNode::Button(b) = node {
        return Some(b);
    }
    node.children().iter().find_map(find_button)
}
