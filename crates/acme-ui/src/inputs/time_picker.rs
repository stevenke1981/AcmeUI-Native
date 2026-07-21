//! TimePicker component — hour/minute selector with spinners.
//!
//! When open, renders a Row of three Columns: an hour spinner, a ":", and a
//! minute spinner. Each spinner is a Column with a ▲ button, the current value
//! Card, and a ▼ button. When closed, collapses to a single Card showing
//! "HH:MM".

use crate::*;

/// Builder for a TimePicker component.
pub struct TimePickerBuilder<M> {
    pub id: WidgetKey,
    pub hour: u32,
    pub minute: u32,
    pub open: bool,
    pub on_change: Option<M>,
}

/// Create a new TimePicker builder. Defaults to 00:00, closed.
pub fn time_picker<M: Clone + 'static>(id: impl Into<WidgetKey>) -> TimePickerBuilder<M> {
    TimePickerBuilder {
        id: id.into(),
        hour: 0,
        minute: 0,
        open: false,
        on_change: None,
    }
}

impl<M: Clone + 'static> TimePickerBuilder<M> {
    /// Set the hour (0-23).
    pub fn hour(mut self, value: u32) -> Self {
        self.hour = value;
        self
    }

    /// Set the minute (0-59).
    pub fn minute(mut self, value: u32) -> Self {
        self.minute = value;
        self
    }

    /// Set whether the spinner popup is open.
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }

    /// Set the message dispatched when the time changes.
    pub fn on_change(mut self, msg: M) -> Self {
        self.on_change = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<TimePickerBuilder<M>> for WidgetNode<M> {
    fn from(b: TimePickerBuilder<M>) -> Self {
        let id_str = b.id.as_str().to_string();

        // Closed: single Card showing "HH:MM".
        if !b.open {
            let display = format!("{:02}:{:02}", b.hour, b.minute);
            return card::<M>()
                .key(b.id)
                .variant(CardVariant::Outlined)
                .padding(8.0)
                .child(label::<M>(display))
                .build();
        }

        // Open: hour spinner, ":", minute spinner.
        let hour_spinner = column::<M>()
            .gap(2.0)
            .child(button::<M>(format!("{}-hour-up", id_str).as_str(), "▲"))
            .child(
                card::<M>()
                    .padding(4.0)
                    .variant(CardVariant::Outlined)
                    .child(label::<M>(format!("{:02}", b.hour))),
            )
            .child(button::<M>(format!("{}-hour-down", id_str).as_str(), "▼"));

        let colon = column::<M>().child(label::<M>(":"));

        let minute_spinner = column::<M>()
            .gap(2.0)
            .child(button::<M>(format!("{}-min-up", id_str).as_str(), "▲"))
            .child(
                card::<M>()
                    .padding(4.0)
                    .variant(CardVariant::Outlined)
                    .child(label::<M>(format!("{:02}", b.minute))),
            )
            .child(button::<M>(format!("{}-min-down", id_str).as_str(), "▼"));

        row::<M>()
            .key(b.id)
            .gap(8.0)
            .child(hour_spinner)
            .child(colon)
            .child(minute_spinner)
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
    use acme_layout::LayoutKind;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn time_picker_builder_defaults() {
        let tp = time_picker::<TestMsg>("tp");
        assert_eq!(tp.hour, 0);
        assert_eq!(tp.minute, 0);
        assert!(!tp.open);
        assert!(tp.on_change.is_none());
    }

    #[test]
    fn time_picker_closed_renders_card() {
        let node: WidgetNode<TestMsg> = time_picker("tp").hour(9).minute(5).into();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant when closed");
        };
        assert_eq!(c.variant, CardVariant::Outlined);
        let WidgetNode::Label(l) = &c.children[0] else {
            panic!("expected Label");
        };
        assert_eq!(l.text, "09:05");
    }

    #[test]
    fn time_picker_open_renders_row_with_three_spinners() {
        let node: WidgetNode<TestMsg> = time_picker("tp").hour(14).minute(30).open(true).into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row variant when open");
        };
        // hour spinner + colon + minute spinner = 3 children
        assert_eq!(r.children.len(), 3);

        // Hour spinner is a Column: ▲ button, value Card, ▼ button
        let WidgetNode::Column(hour) = &r.children[0] else {
            panic!("expected hour spinner Column");
        };
        assert_eq!(hour.children.len(), 3);
        let WidgetNode::Button(_) = &hour.children[0] else {
            panic!("expected ▲ Button");
        };
        let WidgetNode::Card(val) = &hour.children[1] else {
            panic!("expected value Card");
        };
        let WidgetNode::Label(l) = &val.children[0] else {
            panic!("expected Label inside value Card");
        };
        assert_eq!(l.text, "14");

        // Colon is a Column with a single Label ":"
        let WidgetNode::Column(colon_col) = &r.children[1] else {
            panic!("expected colon Column");
        };
        let WidgetNode::Label(cl) = &colon_col.children[0] else {
            panic!("expected colon Label");
        };
        assert_eq!(cl.text, ":");

        // Minute spinner value should be "30"
        let WidgetNode::Column(minute) = &r.children[2] else {
            panic!("expected minute spinner Column");
        };
        let WidgetNode::Card(mval) = &minute.children[1] else {
            panic!("expected minute value Card");
        };
        let WidgetNode::Label(ml) = &mval.children[0] else {
            panic!("expected minute Label");
        };
        assert_eq!(ml.text, "30");
    }

    #[test]
    fn time_picker_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = time_picker("tp").open(true).into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Row);
        assert!(!layout.children.is_empty());
    }
}
