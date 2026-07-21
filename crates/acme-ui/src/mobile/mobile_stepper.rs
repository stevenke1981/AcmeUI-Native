//! Mobile stepper — minus/plus buttons flanking a value display.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a mobile stepper control.
pub struct MobileStepperBuilder<M> {
    pub id: WidgetKey,
    pub value: i64,
    pub min: i64,
    pub max: i64,
    pub step: i64,
    pub on_decrement: Option<M>,
    pub on_increment: Option<M>,
}

/// Create a mobile stepper builder.
pub fn mobile_stepper<M: Clone + 'static>(value: i64) -> MobileStepperBuilder<M> {
    MobileStepperBuilder {
        id: WidgetKey::from("mobile_stepper"),
        value,
        min: 0,
        max: 100,
        step: 1,
        on_decrement: None,
        on_increment: None,
    }
}

impl<M: Clone + 'static> MobileStepperBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn min(mut self, value: i64) -> Self {
        self.min = value;
        self
    }

    pub fn max(mut self, value: i64) -> Self {
        self.max = value;
        self
    }

    pub fn step(mut self, value: i64) -> Self {
        self.step = value;
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
}

impl<M: Clone + 'static> From<MobileStepperBuilder<M>> for WidgetNode<M> {
    fn from(b: MobileStepperBuilder<M>) -> Self {
        let clamped = b.value.clamp(b.min, b.max);
        let minus_btn = crate::button("stepper_dec", "−");
        let plus_btn = crate::button("stepper_inc", "+");

        let minus_node = if let Some(msg) = b.on_decrement {
            minus_btn.disabled(clamped <= b.min).on_click(msg)
        } else {
            minus_btn.disabled(clamped <= b.min).into()
        };

        let plus_node = if let Some(msg) = b.on_increment {
            plus_btn.disabled(clamped >= b.max).on_click(msg)
        } else {
            plus_btn.disabled(clamped >= b.max).into()
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
}
