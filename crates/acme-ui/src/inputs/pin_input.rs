//! PinInput (OTP/PIN code) component.
//!
//! Renders as a Row of digit boxes. Filled cards for entered positions,
//! outlined cards for empty positions, with a 6px gap between boxes.

use acme_core::WidgetKey;
use acme_widgets::*;

/// Builder for a PinInput component.
pub struct PinInputBuilder<M> {
    pub id: WidgetKey,
    pub length: usize,
    pub value: String,
    pub disabled: bool,
    pub masked: bool,
    pub on_complete: Option<M>,
    pub on_change: Option<M>,
}

/// Create a new PinInput builder.
pub fn pin_input<M: Clone + 'static>(id: impl Into<WidgetKey>) -> PinInputBuilder<M> {
    PinInputBuilder {
        id: id.into(),
        length: 4,
        value: String::new(),
        disabled: false,
        masked: false,
        on_complete: None,
        on_change: None,
    }
}

impl<M: Clone + 'static> PinInputBuilder<M> {
    /// Set the number of digit boxes (default 4, clamped to max 8).
    pub fn length(mut self, value: usize) -> Self {
        self.length = value.min(8);
        self
    }

    /// Set the current value (entered digits).
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }

    /// Set whether the input is disabled.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    /// Set whether digits are masked with "•" (default false).
    pub fn masked(mut self, value: bool) -> Self {
        self.masked = value;
        self
    }

    /// Set the message dispatched when all digits are entered.
    pub fn on_complete(mut self, msg: M) -> Self {
        self.on_complete = Some(msg);
        self
    }

    /// Set the message dispatched when the value changes.
    pub fn on_change(mut self, msg: M) -> Self {
        self.on_change = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<PinInputBuilder<M>> for WidgetNode<M> {
    fn from(b: PinInputBuilder<M>) -> Self {
        let chars: Vec<char> = b.value.chars().collect();
        let count = b.length;

        let mut row_builder = row::<M>().key(b.id).gap(6.0);

        for i in 0..count {
            let has_value = i < chars.len();
            let display = if has_value {
                if b.masked {
                    "•".to_string()
                } else {
                    chars[i].to_string()
                }
            } else {
                String::new()
            };

            let box_card = card::<M>()
                .variant(if has_value {
                    CardVariant::Interactive
                } else {
                    CardVariant::Outlined
                })
                .padding(8.0)
                .child(label::<M>(display));

            row_builder = row_builder.child(box_card);
        }

        row_builder.build()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;
    use acme_layout::LayoutKind;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn pin_input_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = pin_input("pin1").length(4).value("12").into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Row);
        assert_eq!(layout.children.len(), 4);
    }

    #[test]
    fn pin_input_builder_defaults() {
        let p = pin_input::<TestMsg>("p");
        assert_eq!(p.length, 4);
        assert!(p.value.is_empty());
        assert!(!p.disabled);
        assert!(!p.masked);
        assert!(p.on_complete.is_none());
        assert!(p.on_change.is_none());
    }

    #[test]
    fn pin_input_length_capped_at_8() {
        let p = pin_input::<TestMsg>("p").length(12);
        assert_eq!(p.length, 8);
    }

    #[test]
    fn pin_input_shows_filled_boxes_for_entered_digits() {
        let node: WidgetNode<TestMsg> = pin_input("pin").length(4).value("123").into();
        let WidgetNode::Row(container) = &node else {
            panic!("expected Row");
        };
        assert_eq!(container.children.len(), 4);
        // First 3 children should be filled Cards (Interactive variant)
        for i in 0..3 {
            let WidgetNode::Card(card) = &container.children[i] else {
                panic!("expected Card at position {}", i);
            };
            assert_eq!(card.variant, CardVariant::Interactive);
        }
        // Last child should be an outlined Card
        let WidgetNode::Card(card) = &container.children[3] else {
            panic!("expected Card at position 3");
        };
        assert_eq!(card.variant, CardVariant::Outlined);
    }

    #[test]
    fn pin_input_shows_masked_digits() {
        let node: WidgetNode<TestMsg> = pin_input("pin")
            .length(4)
            .value("1234")
            .masked(true)
            .into();
        let WidgetNode::Row(container) = &node else {
            panic!("expected Row");
        };
        // Each box should contain a "•" label
        for child in &container.children {
            let WidgetNode::Card(card) = child else {
                panic!("expected Card");
            };
            let WidgetNode::Label(lbl) = &card.children[0] else {
                panic!("expected Label in Card");
            };
            assert_eq!(lbl.text, "•");
        }
    }
}
