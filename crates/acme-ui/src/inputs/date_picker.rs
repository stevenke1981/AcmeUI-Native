//! DatePicker component — a calendar grid date selector.
//!
//! Renders a Column with a year/month header (◀ ▶ buttons), a row of weekday
//! labels (S M T W T F S), and a grid of up to 42 day cells (Cards). The
//! selected day uses the Interactive card variant. When closed, the picker
//! collapses to a single Card showing the selected date string or placeholder.

use crate::*;

/// Builder for a DatePicker component.
pub struct DatePickerBuilder<M> {
    pub id: WidgetKey,
    pub year: u32,
    pub month: u32,
    pub selected_day: Option<u32>,
    pub open: bool,
    pub placeholder: String,
    /// Message sent when a day cell is clicked (not yet wired per-cell).
    pub on_change: Option<M>,
    /// Message sent when the ◀ (prev month) button is clicked.
    pub on_prev_month: Option<M>,
    /// Message sent when the ▶ (next month) button is clicked.
    pub on_next_month: Option<M>,
}

/// Create a new DatePicker builder. Defaults to January 2025, closed.
pub fn date_picker<M: Clone + 'static>(id: impl Into<WidgetKey>) -> DatePickerBuilder<M> {
    DatePickerBuilder {
        id: id.into(),
        year: 2025,
        month: 1,
        selected_day: None,
        open: false,
        placeholder: String::new(),
        on_change: None,
        on_prev_month: None,
        on_next_month: None,
    }
}

impl<M: Clone + 'static> DatePickerBuilder<M> {
    /// Set the displayed year.
    pub fn year(mut self, value: u32) -> Self {
        self.year = value;
        self
    }

    /// Set the displayed month (1-12).
    pub fn month(mut self, value: u32) -> Self {
        self.month = value;
        self
    }

    /// Set the selected day (1-31), or None for no selection.
    pub fn selected_day(mut self, value: Option<u32>) -> Self {
        self.selected_day = value;
        self
    }

    /// Set whether the calendar popup is open.
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }

    /// Set the placeholder text shown when no date is selected.
    pub fn placeholder(mut self, value: impl Into<String>) -> Self {
        self.placeholder = value.into();
        self
    }

    /// Set the message dispatched when the selection changes.
    pub fn on_change(mut self, msg: M) -> Self {
        self.on_change = Some(msg);
        self
    }

    /// Set the message dispatched when the ◀ (prev month) button is clicked.
    pub fn on_prev_month(mut self, msg: M) -> Self {
        self.on_prev_month = Some(msg);
        self
    }

    /// Set the message dispatched when the ▶ (next month) button is clicked.
    pub fn on_next_month(mut self, msg: M) -> Self {
        self.on_next_month = Some(msg);
        self
    }

    /// Validate and normalize year/month to safe ranges.
    /// Clamps month to 1-12 and year to >= 1.
    pub fn normalize(mut self) -> Self {
        self.month = self.month.clamp(1, 12);
        if self.year < 1 {
            self.year = 1;
        }
        self
    }
}

/// Number of days in the given month (1-indexed). Handles leap years.
#[allow(clippy::manual_is_multiple_of)]
fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            let leap = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
            if leap { 29 } else { 28 }
        }
        _ => 30,
    }
}

/// Day-of-week using Tomohiko Sakamoto's algorithm.
/// Returns 0=Sunday..=6=Saturday.
fn sakamoto_weekday(y: u32, m: u32, d: u32) -> u32 {
    let (y, m) = if m < 3 {
        (y.saturating_sub(1), m + 12)
    } else {
        (y, m)
    };
    let y_mod = y % 100;
    let c = y / 100;
    let h = (d + (13 * (m + 1)) / 5 + y_mod + y_mod / 4 + c / 4 + 5 * c) % 7;
    // Zeller: h=0 => Saturday. Remap so Sunday=0.
    (h + 6) % 7
}

/// Day-of-week for the first day of `month`/`year`. Returns 0=Sunday..=6=Saturday.
fn month_start_weekday(year: u32, month: u32) -> u32 {
    sakamoto_weekday(year, month, 1)
}

impl<M: Clone + 'static> From<DatePickerBuilder<M>> for WidgetNode<M> {
    fn from(b: DatePickerBuilder<M>) -> Self {
        let b = b.normalize();
        // Clamp selected_day to valid range [1, days_in_month].
        let dim = days_in_month(b.year, b.month);
        let selected_day = match b.selected_day {
            Some(d) if d >= 1 && d <= dim => Some(d),
            _ => None,
        };
        let b = DatePickerBuilder { selected_day, ..b };
        // Closed: show the selected date string or placeholder in a Card.
        if !b.open {
            let display = b
                .selected_day
                .map(|d| format!("{:04}-{:02}-{:02}", b.year, b.month, d))
                .unwrap_or_else(|| b.placeholder.clone());
            let card = card::<M>()
                .key(b.id)
                .variant(CardVariant::Outlined)
                .padding(8.0)
                .child(label::<M>(display))
                .build();
            return if let Some(msg) = b.on_change {
                // Wrap the card in a column that carries the click message.
                column::<M>().on_click(msg).child(card).build()
            } else {
                card
            };
        }

        // Open: header + weekdays + day grid.
        let prev_key = format!("{}-prev", b.id.as_str());
        let next_key = format!("{}-next", b.id.as_str());
        let prev_btn: WidgetNode<M> = if let Some(ref msg) = b.on_prev_month {
            button::<M>(prev_key.as_str(), "◀")
                .variant(ButtonVariant::Ghost)
                .on_click(msg.clone())
        } else {
            button::<M>(prev_key.as_str(), "◀")
                .variant(ButtonVariant::Ghost)
                .into()
        };
        let next_btn: WidgetNode<M> = if let Some(ref msg) = b.on_next_month {
            button::<M>(next_key.as_str(), "▶")
                .variant(ButtonVariant::Ghost)
                .on_click(msg.clone())
        } else {
            button::<M>(next_key.as_str(), "▶")
                .variant(ButtonVariant::Ghost)
                .into()
        };
        let header = row::<M>()
            .gap(8.0)
            .child(prev_btn)
            .child(label::<M>(format!("{:04}-{:02}", b.year, b.month)))
            .child(next_btn);

        let weekdays = ["S", "M", "T", "W", "T", "F", "S"];
        let mut weekday_row = row::<M>().gap(4.0);
        for w in weekdays {
            weekday_row = weekday_row.child(label::<M>(w));
        }

        let dim = days_in_month(b.year, b.month);
        let offset = month_start_weekday(b.year, b.month);
        let total_cells = {
            let needed = offset + dim;
            if needed <= 35 { 35 } else { 42 }
        };
        let id_str = b.id.as_str().to_string();

        let mut grid_rows: Vec<WidgetNode<M>> = Vec::new();
        let mut day = 1u32;
        let mut row_builder = row::<M>().gap(4.0);
        let mut cells_in_row = 0usize;

        for i in 0..total_cells {
            let cell: WidgetNode<M> = if i as u32 >= offset && day <= dim {
                let is_selected = b.selected_day == Some(day);
                let cell_card = card::<M>()
                    .key(format!("{}-d{}", id_str, day).as_str())
                    .padding(4.0)
                    .variant(if is_selected {
                        CardVariant::Interactive
                    } else {
                        CardVariant::Plain
                    })
                    .child(label::<M>(day.to_string()));
                day += 1;
                cell_card.build()
            } else {
                // Empty filler cell so the grid stays aligned.
                card::<M>()
                    .key(format!("{}-e{}", id_str, i).as_str())
                    .padding(4.0)
                    .variant(CardVariant::Plain)
                    .child(label::<M>(""))
                    .build()
            };

            row_builder = row_builder.child(cell);
            cells_in_row += 1;
            if cells_in_row == 7 {
                let built = row_builder.build();
                grid_rows.push(built);
                row_builder = row::<M>().gap(4.0);
                cells_in_row = 0;
            }
        }

        // Flush any partial trailing row (shouldn't happen with 35 cells, but be safe).
        if cells_in_row > 0 {
            grid_rows.push(row_builder.build());
        }

        let mut day_grid = column::<M>().gap(2.0);
        for r in grid_rows {
            day_grid = day_grid.child(r);
        }

        let mut outer = column::<M>()
            .key(b.id)
            .gap(4.0)
            .child(header)
            .child(weekday_row)
            .child(day_grid.build());

        // Wire on_change to the outermost column so the message is
        // dispatched when the calendar area is clicked.
        if let Some(msg) = b.on_change {
            outer = outer.on_click(msg);
        }

        outer.build()
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
    enum TestMsg {
        Prev,
        Next,
        Change,
    }

    #[test]
    fn date_picker_builder_defaults() {
        let dp = date_picker::<TestMsg>("dp");
        assert_eq!(dp.year, 2025);
        assert_eq!(dp.month, 1);
        assert!(dp.selected_day.is_none());
        assert!(!dp.open);
        assert!(dp.placeholder.is_empty());
        assert!(dp.on_change.is_none());
        assert!(dp.on_prev_month.is_none());
        assert!(dp.on_next_month.is_none());
    }

    #[test]
    fn date_picker_closed_renders_card() {
        let node: WidgetNode<TestMsg> = date_picker("dp").placeholder("Pick a date").into();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant when closed");
        };
        assert_eq!(c.variant, CardVariant::Outlined);
        let WidgetNode::Label(l) = &c.children[0] else {
            panic!("expected Label child");
        };
        assert_eq!(l.text, "Pick a date");
    }

    #[test]
    fn date_picker_closed_shows_selected_date() {
        let node: WidgetNode<TestMsg> = date_picker("dp").open(false).selected_day(Some(15)).into();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card variant");
        };
        let WidgetNode::Label(l) = &c.children[0] else {
            panic!("expected Label");
        };
        assert_eq!(l.text, "2025-01-15");
    }

    #[test]
    fn date_picker_open_renders_column_with_header_weekdays_grid() {
        let node: WidgetNode<TestMsg> = date_picker("dp").year(2025).month(1).open(true).into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant when open");
        };
        // header row + weekday row + day-grid column = 3 children
        assert_eq!(col.children.len(), 3);
        let WidgetNode::Row(header) = &col.children[0] else {
            panic!("expected header Row");
        };
        // prev button + label + next button
        assert_eq!(header.children.len(), 3);
        let WidgetNode::Row(weekdays) = &col.children[1] else {
            panic!("expected weekday Row");
        };
        assert_eq!(weekdays.children.len(), 7);
        let WidgetNode::Column(grid) = &col.children[2] else {
            panic!("expected day grid Column");
        };
        // At least 5 rows (5 or 6 weeks depending on month)
        assert!(grid.children.len() >= 5);
    }

    #[test]
    fn date_picker_selected_day_uses_interactive_variant() {
        let node: WidgetNode<TestMsg> = date_picker("dp")
            .year(2025)
            .month(1)
            .selected_day(Some(1))
            .open(true)
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        let WidgetNode::Column(grid) = &col.children[2] else {
            panic!("expected grid Column");
        };
        let WidgetNode::Row(first_row) = &grid.children[0] else {
            panic!("expected first grid Row");
        };
        // January 2025 starts on Wednesday (offset 3): first 3 cells are empty.
        // The 4th cell (index 3) is day 1 — the selected day.
        let WidgetNode::Card(day_card) = &first_row.children[3] else {
            panic!("expected Card for day 1");
        };
        assert_eq!(day_card.variant, CardVariant::Interactive);
        let WidgetNode::Label(l) = &day_card.children[0] else {
            panic!("expected Label inside day Card");
        };
        assert_eq!(l.text, "1");
    }

    #[test]
    fn date_picker_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = date_picker("dp").open(true).year(2025).month(2).into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert!(!layout.children.is_empty());
    }

    #[test]
    fn date_picker_days_in_month_leap_year() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2025, 2), 28);
        assert_eq!(days_in_month(2025, 1), 31);
        assert_eq!(days_in_month(2025, 4), 30);
    }

    #[test]
    fn date_picker_month_start_weekday_jan_2025_is_wednesday() {
        // Jan 1, 2025 is a Wednesday (0=Sunday => 3).
        assert_eq!(month_start_weekday(2025, 1), 3);
    }

    #[test]
    fn days_in_march_2025_has_31_days() {
        assert_eq!(days_in_month(2025, 3), 31);
    }

    #[test]
    fn feb_2024_leap_year_has_29_days() {
        assert_eq!(days_in_month(2024, 2), 29);
    }

    #[test]
    fn feb_2023_non_leap_has_28_days() {
        assert_eq!(days_in_month(2023, 2), 28);
    }

    #[test]
    fn feb_in_january_year0_does_not_underflow() {
        // year=0, month=2 (February). month_start_weekday internally does
        // checked_sub for the year adjustment. This should not panic.
        let wd = month_start_weekday(0, 2);
        // Any result is acceptable as long as it doesn't underflow.
        assert!(wd < 7);
    }

    #[test]
    fn sakamoto_weekday_jan_1_2025_is_wednesday() {
        // Jan 1, 2025 is Wednesday, which is 3 in our convention (0=Sunday).
        assert_eq!(sakamoto_weekday(2025, 1, 1), 3);
    }

    #[test]
    fn sakamoto_weekday_mar_1_2025_is_saturday() {
        // Mar 1, 2025 is Saturday, which is 6 in our convention (0=Sunday).
        assert_eq!(sakamoto_weekday(2025, 3, 1), 6);
    }

    // -----------------------------------------------------------------------
    // selected_day clamping
    // -----------------------------------------------------------------------

    #[test]
    fn selected_day_clamped_to_days_in_month() {
        // Feb 2025 has 28 days; 31 should be clamped to None.
        let node: WidgetNode<TestMsg> = date_picker("dp")
            .year(2025)
            .month(2)
            .selected_day(Some(31))
            .open(true)
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        let WidgetNode::Column(grid) = &col.children[2] else {
            panic!("expected grid Column");
        };
        let WidgetNode::Row(first_row) = &grid.children[0] else {
            panic!("expected first grid Row");
        };
        // All February day cells should be Plain, not Interactive,
        // since selected_day was clamped to None.
        for child in &first_row.children {
            if let WidgetNode::Card(c) = child {
                assert_eq!(
                    c.variant,
                    CardVariant::Plain,
                    "no day should be Interactive when selected_day is invalid"
                );
            }
        }
    }

    #[test]
    fn selected_day_zero_clamped_to_none() {
        let node: WidgetNode<TestMsg> = date_picker("dp")
            .year(2025)
            .month(1)
            .selected_day(Some(0))
            .open(true)
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        let WidgetNode::Column(grid) = &col.children[2] else {
            panic!("expected grid Column");
        };
        let WidgetNode::Row(first_row) = &grid.children[0] else {
            panic!("expected first grid Row");
        };
        for child in &first_row.children {
            if let WidgetNode::Card(c) = child {
                assert_eq!(c.variant, CardVariant::Plain);
            }
        }
    }

    #[test]
    fn selected_day_valid_day_kept() {
        // Jan 2025 has 31 days; day 15 is valid — should be Interactive.
        let node: WidgetNode<TestMsg> = date_picker("dp")
            .year(2025)
            .month(1)
            .selected_day(Some(15))
            .open(true)
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        let WidgetNode::Column(grid) = &col.children[2] else {
            panic!("expected grid Column");
        };
        // Jan 2025 starts Wednesday (offset 3). Grid layout:
        // Row 0: [empty, empty, empty, d1, d2, d3, d4]
        // Row 1: [d5, d6, d7, d8, d9, d10, d11]
        // Row 2: [d12, d13, d14, d15, d16, d17, d18]
        // So day 15 is at grid row index 2, column index 3.
        let WidgetNode::Row(third_row) = &grid.children[2] else {
            panic!("expected third grid Row");
        };
        let WidgetNode::Card(day_card) = &third_row.children[3] else {
            panic!("expected Card for day 15");
        };
        assert_eq!(day_card.variant, CardVariant::Interactive);
    }

    #[test]
    fn selected_day_leap_year_feb_29_valid() {
        // 2024 is a leap year — Feb 29 is valid.
        let node: WidgetNode<TestMsg> = date_picker("dp")
            .year(2024)
            .month(2)
            .selected_day(Some(29))
            .open(true)
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        let WidgetNode::Column(grid) = &col.children[2] else {
            panic!("expected grid Column");
        };
        // Feb 2024 starts Thursday (offset 4). Day 29 is in the 5th row.
        let WidgetNode::Row(fifth_row) = &grid.children[4] else {
            panic!("expected fifth grid Row");
        };
        // Feb 2024: offset=4, so row 0 cells 4-6 = days 1-3.
        // Row 1: days 4-10, Row 2: days 11-17, Row 3: days 18-24, Row 4: days 25-29
        // Day 29 is at index 4 in row 4 (25=0, 26=1, 27=2, 28=3, 29=4)
        let WidgetNode::Card(day_card) = &fifth_row.children[4] else {
            panic!("expected Card for day 29");
        };
        assert_eq!(day_card.variant, CardVariant::Interactive);
    }

    #[test]
    fn closed_view_clamps_selected_day() {
        // Feb 31 → invalid → None → shows placeholder.
        let node: WidgetNode<TestMsg> = date_picker("dp")
            .year(2025)
            .month(2)
            .selected_day(Some(31))
            .placeholder("no date")
            .open(false)
            .into();
        let WidgetNode::Card(c) = &node else {
            panic!("expected Card");
        };
        let WidgetNode::Label(l) = &c.children[0] else {
            panic!("expected Label");
        };
        assert_eq!(l.text, "no date");
    }

    // -----------------------------------------------------------------------
    // on_prev_month / on_next_month builder
    // -----------------------------------------------------------------------

    #[test]
    fn date_picker_on_prev_month_default_none() {
        let dp = date_picker::<TestMsg>("dp");
        assert!(dp.on_prev_month.is_none());
    }

    #[test]
    fn date_picker_on_next_month_default_none() {
        let dp = date_picker::<TestMsg>("dp");
        assert!(dp.on_next_month.is_none());
    }

    #[test]
    fn date_picker_on_prev_month_stores_message() {
        let dp = date_picker::<TestMsg>("dp").on_prev_month(TestMsg::Prev);
        assert_eq!(dp.on_prev_month, Some(TestMsg::Prev));
    }

    #[test]
    fn date_picker_on_next_month_stores_message() {
        let dp = date_picker::<TestMsg>("dp").on_next_month(TestMsg::Next);
        assert_eq!(dp.on_next_month, Some(TestMsg::Next));
    }

    #[test]
    fn date_picker_prev_button_wired_when_message_set() {
        let node: WidgetNode<TestMsg> = date_picker("dp")
            .year(2025)
            .month(1)
            .open(true)
            .on_prev_month(TestMsg::Prev)
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        let WidgetNode::Row(header) = &col.children[0] else {
            panic!("expected header Row");
        };
        // First child: prev button with message
        let WidgetNode::Button(btn) = &header.children[0] else {
            panic!("expected Button for prev");
        };
        assert_eq!(btn.label, "◀");
        assert_eq!(btn.activate(), Some(&TestMsg::Prev));
    }

    #[test]
    fn date_picker_next_button_wired_when_message_set() {
        let node: WidgetNode<TestMsg> = date_picker("dp")
            .year(2025)
            .month(1)
            .open(true)
            .on_next_month(TestMsg::Next)
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        let WidgetNode::Row(header) = &col.children[0] else {
            panic!("expected header Row");
        };
        // Third child: next button with message
        let WidgetNode::Button(btn) = &header.children[2] else {
            panic!("expected Button for next");
        };
        assert_eq!(btn.label, "▶");
        assert_eq!(btn.activate(), Some(&TestMsg::Next));
    }

    #[test]
    fn date_picker_prev_button_disabled_when_no_message() {
        let node: WidgetNode<TestMsg> = date_picker("dp").year(2025).month(1).open(true).into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        let WidgetNode::Row(header) = &col.children[0] else {
            panic!("expected header Row");
        };
        let WidgetNode::Button(btn) = &header.children[0] else {
            panic!("expected Button for prev");
        };
        assert_eq!(btn.label, "◀");
        assert!(
            btn.activate().is_none(),
            "prev button should have no message when not wired"
        );
    }

    #[test]
    fn date_picker_next_button_disabled_when_no_message() {
        let node: WidgetNode<TestMsg> = date_picker("dp").year(2025).month(1).open(true).into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        let WidgetNode::Row(header) = &col.children[0] else {
            panic!("expected header Row");
        };
        let WidgetNode::Button(btn) = &header.children[2] else {
            panic!("expected Button for next");
        };
        assert_eq!(btn.label, "▶");
        assert!(
            btn.activate().is_none(),
            "next button should have no message when not wired"
        );
    }

    // -----------------------------------------------------------------------
    // on_change message wiring
    // -----------------------------------------------------------------------

    #[test]
    fn date_picker_closed_on_change_wraps_in_column() {
        let node: WidgetNode<TestMsg> = date_picker("dp")
            .placeholder("Pick")
            .on_change(TestMsg::Change)
            .into();
        // When on_change is set on a closed picker, the card is wrapped
        // in a Column that carries the message.
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column wrapper when on_change is set on closed picker");
        };
        assert_eq!(col.message, Some(TestMsg::Change));
        assert_eq!(col.children.len(), 1);
        let WidgetNode::Card(_inner) = &col.children[0] else {
            panic!("expected Card child inside wrapper Column");
        };
    }

    #[test]
    fn date_picker_closed_no_on_change_returns_card() {
        let node: WidgetNode<TestMsg> = date_picker("dp").placeholder("Pick").into();
        // Without on_change the closed picker is still a plain Card.
        let WidgetNode::Card(_c) = &node else {
            panic!("expected Card variant when on_change is not set");
        };
    }

    #[test]
    fn date_picker_open_on_change_wired() {
        let node: WidgetNode<TestMsg> = date_picker("dp")
            .open(true)
            .on_change(TestMsg::Change)
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant when open");
        };
        assert_eq!(col.message, Some(TestMsg::Change));
        // Structure unchanged: header + weekday row + day grid
        assert_eq!(col.children.len(), 3);
    }

    #[test]
    fn date_picker_open_no_on_change_no_message() {
        let node: WidgetNode<TestMsg> = date_picker("dp").open(true).into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant when open");
        };
        assert_eq!(col.message, None);
        assert_eq!(col.children.len(), 3);
    }
}
