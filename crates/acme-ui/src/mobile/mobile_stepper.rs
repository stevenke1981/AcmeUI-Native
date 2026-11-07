//! Mobile stepper — minus/plus buttons flanking a value display.

use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_layout::Length;
use acme_widgets::style::prelude::*;

/// Builder for a mobile stepper control.
pub struct MobileStepperBuilder<M> {
    pub id: WidgetKey,
    pub value: i64,
    pub min: i64,
    pub max: i64,
    pub step: i64,
    pub disabled: bool,
    pub on_decrement: Option<M>,
    pub on_increment: Option<M>,
    /// Maps the new, clamped value to a message (ROADMAP 014).
    pub on_change: Option<fn(i64) -> M>,
}

/// Create a mobile stepper builder.
pub fn mobile_stepper<M: Clone + 'static>(value: i64) -> MobileStepperBuilder<M> {
    MobileStepperBuilder {
        id: WidgetKey::from("mobile_stepper"),
        value,
        min: 0,
        max: 100,
        step: 1,
        disabled: false,
        on_decrement: None,
        on_increment: None,
        on_change: None,
    }
}

impl<M: Clone + 'static> MobileStepperBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    /// Set a range endpoint. Inverted endpoints are sorted at build time.
    pub fn min(mut self, value: i64) -> Self {
        self.min = value;
        self
    }

    pub fn max(mut self, value: i64) -> Self {
        self.max = value;
        self
    }

    /// Set the positive increment; non-positive values are treated as one.
    pub fn step(mut self, value: i64) -> Self {
        self.step = value;
        self
    }

    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    pub fn on_decrement(mut self, msg: M) -> Self {
        self.on_decrement = Some(msg);
        self
    }

    pub fn on_increment(mut self, msg: M) -> Self {
        self.on_increment = Some(msg);
        self
    }

    /// Emit `f(new_value)` with the value clamped to `[min, max]`. An explicit
    /// [`Self::on_decrement`] or [`Self::on_increment`] wins for its button.
    pub fn on_change(mut self, f: fn(i64) -> M) -> Self {
        self.on_change = Some(f);
        self
    }
}

impl<M: Clone + 'static> From<MobileStepperBuilder<M>> for WidgetNode<M> {
    fn from(b: MobileStepperBuilder<M>) -> Self {
        let min = b.min.min(b.max);
        let max = b.min.max(b.max);
        let clamped = b.value.clamp(min, max);
        let step = b.step.max(1);
        let id = b.id.as_str();
        let minus_btn = crate::button(format!("{id}__dec"), "−")
            .w(Length::px(44.0))
            .min_w(Length::px(44.0))
            .h(Length::px(44.0))
            .min_h(Length::px(44.0))
            .disabled(b.disabled || clamped <= min);
        let plus_btn = crate::button(format!("{id}__inc"), "+")
            .w(Length::px(44.0))
            .min_w(Length::px(44.0))
            .h(Length::px(44.0))
            .min_h(Length::px(44.0))
            .disabled(b.disabled || clamped >= max);
        let decrement = b.on_decrement.or_else(|| {
            b.on_change
                .map(|f| f(clamped.saturating_sub(step).clamp(min, max)))
        });
        let increment = b.on_increment.or_else(|| {
            b.on_change
                .map(|f| f(clamped.saturating_add(step).clamp(min, max)))
        });

        let minus_node = if let Some(msg) = decrement {
            minus_btn.on_click(msg)
        } else {
            minus_btn.into()
        };

        let plus_node = if let Some(msg) = increment {
            plus_btn.on_click(msg)
        } else {
            plus_btn.into()
        };

        crate::row::<M>()
            .key(b.id)
            .gap(16.0)
            .padding(8.0)
            .child(minus_node)
            .child(crate::label(clamped.to_string()))
            .child(plus_node)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Dec,
        Inc,
    }

    #[test]
    fn mobile_stepper_produces_row() {
        let node: WidgetNode<Msg> = mobile_stepper(5).into();
        assert!(matches!(node, WidgetNode::Row(_)));
    }

    #[test]
    fn mobile_stepper_has_three_children() {
        let node: WidgetNode<Msg> = mobile_stepper(3).into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        assert_eq!(r.children.len(), 3);
    }

    #[test]
    fn mobile_stepper_clamps_value() {
        let node: WidgetNode<Msg> = mobile_stepper(200).max(10).into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        // Middle child is the value label
        let WidgetNode::Label(l) = &r.children[1] else {
            panic!("expected Label");
        };
        assert_eq!(l.text, "10");
    }

    #[test]
    fn mobile_stepper_stores_increment_and_decrement_messages() {
        let b = mobile_stepper::<Msg>(5)
            .on_decrement(Msg::Dec)
            .on_increment(Msg::Inc);
        assert_eq!(b.on_decrement, Some(Msg::Dec));
        assert_eq!(b.on_increment, Some(Msg::Inc));
    }
}

#[cfg(test)]
#[path = "mobile_stepper_tests.rs"]
mod interaction_tests;
