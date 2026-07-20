//! NumberInput component.
//!
//! Renders as a Row with a value label, decrement button ("−"), and
//! increment button ("+") in a horizontal row with 4 px gap.

use acme_core::WidgetKey;
use acme_widgets::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Format an `f64` value as a clean display string — whole numbers omit the
/// decimal part, fractional values show up to two decimal places.
fn display_value(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{:.0}", v)
    } else {
        format!("{:.2}", v)
    }
}

// ---------------------------------------------------------------------------
// Builder
// ---------------------------------------------------------------------------

/// Builder for a NumberInput component.
pub struct NumberInputBuilder<M> {
    pub id: WidgetKey,
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub disabled: bool,
    pub size: crate::ControlSize,
    pub on_change: Option<M>,
}

/// Create a new NumberInput builder.
///
/// Defaults: value=0, min=0, max=100, step=1.
pub fn number_input<M: Clone + 'static>(id: impl Into<WidgetKey>) -> NumberInputBuilder<M> {
    NumberInputBuilder {
        id: id.into(),
        value: 0.0,
        min: 0.0,
        max: 100.0,
        step: 1.0,
        disabled: false,
        size: crate::ControlSize::Md,
        on_change: None,
    }
}

impl<M: Clone + 'static> NumberInputBuilder<M> {
    /// Set the current numeric value.
    pub fn value(mut self, v: f64) -> Self {
        self.value = v;
        self
    }

    /// Set the minimum value (default 0).
    pub fn min(mut self, v: f64) -> Self {
        self.min = v;
        self
    }

    /// Set the maximum value (default 100).
    pub fn max(mut self, v: f64) -> Self {
        self.max = v;
        self
    }

    /// Set the step increment (default 1).
    pub fn step(mut self, v: f64) -> Self {
        self.step = v;
        self
    }

    /// Set whether the input is disabled.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    /// Set the control size (default `Md`).
    pub fn size(mut self, value: crate::ControlSize) -> Self {
        self.size = value;
        self
    }

    /// Set the message dispatched when the value changes.
    pub fn on_change(mut self, msg: M) -> Self {
        self.on_change = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<NumberInputBuilder<M>> for WidgetNode<M> {
    fn from(b: NumberInputBuilder<M>) -> Self {
        let dec_key = format!("{}-dec", b.id.as_str());
        let inc_key = format!("{}-inc", b.id.as_str());
        let value_text = display_value(b.value);

        row::<M>()
            .key(b.id)
            .gap(4.0)
            .child(label::<M>(value_text))
            .child(button::<M>(dec_key.as_str(), "−"))
            .child(button::<M>(inc_key.as_str(), "+"))
            .build()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;
    use acme_layout::{LayoutKind, Length};

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {
        Changed,
    }

    #[test]
    fn number_input_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> =
            number_input("ni1").value(42.0).min(0.0).max(100.0).into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Row);
        // Three children: value label, decrement button, increment button
        assert_eq!(layout.children.len(), 3);
        // Label child has non-zero min_height
        let label_leaf = &layout.children[0];
        assert!(label_leaf.children.is_empty());
        assert_ne!(label_leaf.style.min_height, Length::px(0.0));
        // Button leaves have non-zero height
        for i in 1..=2 {
            assert_ne!(layout.children[i].style.height, Length::px(0.0));
        }
    }

    #[test]
    fn number_input_builder_defaults() {
        let n = number_input::<TestMsg>("n");
        assert_eq!(n.value, 0.0);
        assert_eq!(n.min, 0.0);
        assert_eq!(n.max, 100.0);
        assert_eq!(n.step, 1.0);
        assert!(!n.disabled);
        assert!(n.on_change.is_none());
    }

    #[test]
    fn number_input_structure() {
        let node: WidgetNode<TestMsg> =
            number_input("n").value(50.0).min(0.0).max(200.0).step(5.0).into();
        let WidgetNode::Row(container) = &node else {
            panic!("expected Row");
        };
        assert_eq!(container.children.len(), 3);
        // First child is the value label
        let WidgetNode::Label(lbl) = &container.children[0] else {
            panic!("expected Label");
        };
        assert_eq!(lbl.text, "50");
        // Second child is the decrement button ("−")
        let WidgetNode::Button(dec) = &container.children[1] else {
            panic!("expected Button (dec)");
        };
        assert_eq!(dec.label, "−");
        // Third child is the increment button ("+")
        let WidgetNode::Button(inc) = &container.children[2] else {
            panic!("expected Button (inc)");
        };
        assert_eq!(inc.label, "+");
    }

    #[test]
    fn number_input_on_change() {
        let n = number_input::<TestMsg>("n")
            .value(10.0)
            .on_change(TestMsg::Changed);
        assert_eq!(n.value, 10.0);
        assert!(n.on_change.is_some());
    }

    #[test]
    fn number_input_displays_fractional_value() {
        let node: WidgetNode<TestMsg> =
            number_input("nf").value(3.5).min(0.0).max(10.0).step(0.5).into();
        let WidgetNode::Row(container) = &node else {
            panic!("expected Row");
        };
        let WidgetNode::Label(lbl) = &container.children[0] else {
            panic!("expected Label");
        };
        assert_eq!(lbl.text, "3.50");
    }
}
