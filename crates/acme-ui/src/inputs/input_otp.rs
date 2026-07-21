//! InputOTP — one-time password input with individual character boxes.
//! Aligns with shadcn/ui Input OTP.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for an OTP input.
pub struct InputOtpBuilder<M> {
    pub id: WidgetKey,
    pub length: usize,
    pub values: Vec<String>,
    pub on_submit: Option<M>,
}

/// Create an OTP input builder.
pub fn input_otp<M: Clone + 'static>(length: usize) -> InputOtpBuilder<M> {
    InputOtpBuilder {
        id: WidgetKey::from("input_otp"),
        length: length.max(1),
        values: Vec::new(),
        on_submit: None,
    }
}

impl<M: Clone + 'static> InputOtpBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn values(mut self, vals: Vec<impl Into<String>>) -> Self {
        self.values = vals.into_iter().map(Into::into).collect();
        self
    }

    pub fn on_submit(mut self, msg: M) -> Self {
        self.on_submit = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<InputOtpBuilder<M>> for WidgetNode<M> {
    fn from(b: InputOtpBuilder<M>) -> Self {
        let mut row = crate::row::<M>().key(b.id).gap(8.0).padding(4.0);
        for i in 0..b.length {
            let ch = b.values.get(i).cloned().unwrap_or_else(|| "□".to_string());
            let cell = crate::stack::<M>()
                .size(40.0, 48.0)
                .child(crate::label(ch))
                .build();
            row = row.child(cell);
        }
        if let Some(msg) = b.on_submit {
            row = row.on_click(msg);
        }
        row.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Submit,
    }

    #[test]
    fn input_otp_produces_row() {
        let node: WidgetNode<Msg> = input_otp(6).into();
        assert!(matches!(node, WidgetNode::Row(_)));
    }

    #[test]
    fn input_otp_has_correct_cell_count() {
        let node: WidgetNode<Msg> = input_otp(4).into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        assert_eq!(r.children.len(), 4);
    }

    #[test]
    fn input_otp_min_length_one() {
        let b = input_otp::<Msg>(0);
        assert_eq!(b.length, 1);
    }

    #[test]
    fn input_otp_with_values() {
        let node: WidgetNode<Msg> = input_otp(3).values(vec!["1", "2"]).into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        assert_eq!(r.children.len(), 3);
    }
}
