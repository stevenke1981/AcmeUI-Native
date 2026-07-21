//! DateRangePicker — calendar-based date range selection.
//! Aligns with shadcn/ui Date Picker (range mode).

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a date range picker.
pub struct DateRangePickerBuilder<M> {
    pub id: WidgetKey,
    pub start_label: String,
    pub end_label: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub on_apply: Option<M>,
}

/// Create a date range picker builder.
pub fn date_range_picker<M: Clone + 'static>() -> DateRangePickerBuilder<M> {
    DateRangePickerBuilder {
        id: WidgetKey::from("date_range_picker"),
        start_label: "Start date".to_string(),
        end_label: "End date".to_string(),
        start_date: None,
        end_date: None,
        on_apply: None,
    }
}

impl<M: Clone + 'static> DateRangePickerBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn start_label(mut self, text: impl Into<String>) -> Self {
        self.start_label = text.into();
        self
    }

    pub fn end_label(mut self, text: impl Into<String>) -> Self {
        self.end_label = text.into();
        self
    }

    pub fn start_date(mut self, date: impl Into<String>) -> Self {
        self.start_date = Some(date.into());
        self
    }

    pub fn end_date(mut self, date: impl Into<String>) -> Self {
        self.end_date = Some(date.into());
        self
    }

    pub fn on_apply(mut self, msg: M) -> Self {
        self.on_apply = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<DateRangePickerBuilder<M>> for WidgetNode<M> {
    fn from(b: DateRangePickerBuilder<M>) -> Self {
        let start_display = b.start_date.unwrap_or_else(|| b.start_label);
        let end_display = b.end_date.unwrap_or_else(|| b.end_label);

        let mut row = crate::row::<M>().key(b.id).gap(8.0).padding(8.0);
        row = row.child(crate::label(format!("📅 {}", start_display)));
        row = row.child(crate::label("→"));
        row = row.child(crate::label(format!("📅 {}", end_display)));

        if let Some(msg) = b.on_apply {
            let btn = crate::button("range_apply", "Apply").on_click(msg);
            row = row.child(btn);
        }
        row.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Apply,
    }

    #[test]
    fn date_range_picker_produces_row() {
        let node: WidgetNode<Msg> = date_range_picker().into();
        assert!(matches!(node, WidgetNode::Row(_)));
    }

    #[test]
    fn date_range_picker_has_start_arrow_end() {
        let node: WidgetNode<Msg> = date_range_picker().into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        assert_eq!(r.children.len(), 3);
    }

    #[test]
    fn date_range_picker_with_dates_and_apply() {
        let node: WidgetNode<Msg> = date_range_picker()
            .start_date("2026-01-01")
            .end_date("2026-12-31")
            .on_apply(Msg::Apply)
            .into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        // start + arrow + end + apply button = 4
        assert_eq!(r.children.len(), 4);
    }
}
