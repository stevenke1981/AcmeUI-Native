//! Interaction tests for `mobile_stepper`.

use super::*;
use crate::mobile::mobile_toggle::test_support::{Laid, lay_out};

#[derive(Clone, Debug, PartialEq)]
enum Msg {
    Value(i64),
    Dec,
    Inc,
}

fn laid(builder: MobileStepperBuilder<Msg>) -> Laid<Msg> {
    lay_out(builder.key("qty").into())
}

const MINUS: usize = 0;
const PLUS: usize = 1;

#[test]
fn minus_and_plus_emit_the_new_value() {
    let l = laid(
        mobile_stepper(5)
            .min(0)
            .max(10)
            .step(2)
            .on_change(Msg::Value),
    );
    assert_eq!(l.press_button(MINUS), Some(Msg::Value(3)));
    assert_eq!(l.press_button(PLUS), Some(Msg::Value(7)));
}

#[test]
fn default_step_is_one() {
    let l = laid(mobile_stepper(5).on_change(Msg::Value));
    assert_eq!(l.press_button(MINUS), Some(Msg::Value(4)));
    assert_eq!(l.press_button(PLUS), Some(Msg::Value(6)));
}

#[test]
fn minus_is_disabled_at_min_and_plus_at_max() {
    let at_min = laid(mobile_stepper(0).min(0).max(10).on_change(Msg::Value));
    assert_eq!(at_min.press_button(MINUS), None);
    assert_eq!(at_min.press_button(PLUS), Some(Msg::Value(1)));
    let at_max = laid(mobile_stepper(10).min(0).max(10).on_change(Msg::Value));
    assert_eq!(at_max.press_button(MINUS), Some(Msg::Value(9)));
    assert_eq!(at_max.press_button(PLUS), None);
}

#[test]
fn a_step_larger_than_the_range_clamps() {
    let l = laid(
        mobile_stepper(3)
            .min(0)
            .max(10)
            .step(50)
            .on_change(Msg::Value),
    );
    assert_eq!(l.press_button(MINUS), Some(Msg::Value(0)));
    assert_eq!(l.press_button(PLUS), Some(Msg::Value(10)));
}

#[test]
fn min_equal_to_max_disables_both() {
    let l = laid(mobile_stepper(4).min(4).max(4).on_change(Msg::Value));
    assert_eq!(l.press_button(MINUS), None);
    assert_eq!(l.press_button(PLUS), None);
}

#[test]
fn inverted_bounds_do_not_panic() {
    // min > max is treated as the range [max, min].
    let l = laid(mobile_stepper(5).min(10).max(0).on_change(Msg::Value));
    assert_eq!(l.press_button(MINUS), Some(Msg::Value(4)));
    assert_eq!(l.press_button(PLUS), Some(Msg::Value(6)));
}

#[test]
fn a_value_outside_the_range_is_clamped_first() {
    let high = laid(mobile_stepper(200).min(0).max(10).on_change(Msg::Value));
    assert_eq!(high.press_button(PLUS), None);
    assert_eq!(high.press_button(MINUS), Some(Msg::Value(9)));
    let low = laid(mobile_stepper(-50).min(0).max(10).on_change(Msg::Value));
    assert_eq!(low.press_button(MINUS), None);
    assert_eq!(low.press_button(PLUS), Some(Msg::Value(1)));
}

#[test]
fn a_zero_or_negative_step_is_treated_as_one() {
    for step in [0, -3] {
        let l = laid(mobile_stepper(5).step(step).on_change(Msg::Value));
        assert_eq!(l.press_button(MINUS), Some(Msg::Value(4)), "step {step}");
        assert_eq!(l.press_button(PLUS), Some(Msg::Value(6)), "step {step}");
    }
}

#[test]
fn extreme_values_do_not_overflow() {
    let l = laid(
        mobile_stepper(i64::MAX)
            .min(i64::MIN)
            .max(i64::MAX)
            .step(i64::MAX)
            .on_change(Msg::Value),
    );
    assert_eq!(l.press_button(PLUS), None);
    assert!(l.press_button(MINUS).is_some());
}

#[test]
fn explicit_messages_win_for_their_button() {
    let l = laid(
        mobile_stepper(5)
            .on_decrement(Msg::Dec)
            .on_increment(Msg::Inc)
            .on_change(Msg::Value),
    );
    assert_eq!(l.press_button(MINUS), Some(Msg::Dec));
    assert_eq!(l.press_button(PLUS), Some(Msg::Inc));
    let only_inc = laid(
        mobile_stepper(5)
            .on_increment(Msg::Inc)
            .on_change(Msg::Value),
    );
    assert_eq!(only_inc.press_button(MINUS), Some(Msg::Value(4)));
    assert_eq!(only_inc.press_button(PLUS), Some(Msg::Inc));
}

#[test]
fn no_message_means_nothing_to_press() {
    let l = laid(mobile_stepper(5));
    assert_eq!(l.press_button(MINUS), None);
    assert_eq!(l.press_button(PLUS), None);
}

#[test]
fn disabled_suppresses_both_computed_and_explicit_messages() {
    let l = laid(
        mobile_stepper(5)
            .disabled(true)
            .on_change(Msg::Value)
            .on_decrement(Msg::Dec)
            .on_increment(Msg::Inc),
    );
    assert_eq!(l.press_button(MINUS), None);
    assert_eq!(l.press_button(PLUS), None);
}

#[test]
fn arithmetic_saturates_before_clamping() {
    let low = laid(
        mobile_stepper(i64::MIN + 1)
            .min(i64::MIN)
            .max(i64::MAX)
            .step(10)
            .on_change(Msg::Value),
    );
    assert_eq!(low.press_button(MINUS), Some(Msg::Value(i64::MIN)));
    let high = laid(
        mobile_stepper(i64::MAX - 1)
            .min(i64::MIN)
            .max(i64::MAX)
            .step(10)
            .on_change(Msg::Value),
    );
    assert_eq!(high.press_button(PLUS), Some(Msg::Value(i64::MAX)));
}

#[test]
fn buttons_flank_the_value_and_are_44_px_touch_targets() {
    let l = laid(mobile_stepper(5).on_change(Msg::Value));
    let hits = l.hits();
    let [mx, _, mw, mh] = hits[MINUS].rect;
    let [px, _, pw, ph] = hits[PLUS].rect;
    assert!(mh >= 44.0 && ph >= 44.0, "heights {mh} {ph}");
    assert!(mw >= 44.0 && pw >= 44.0, "widths {mw} {pw}");
    assert!(mx + mw <= px, "minus is left of plus");
}
