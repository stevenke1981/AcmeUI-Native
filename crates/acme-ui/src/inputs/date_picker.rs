//! DatePicker component — a calendar grid date selector.
//!
//! Renders a Column with a year/month header (◀ ▶ buttons), a row of weekday
//! labels (S M T W T F S), and a grid of up to 35 day cells (Cards). The
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
    pub on_change: Option<M>,
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

/// Day-of-week for the first day of `month`/`year`. Returns 0=Sunday..=6=Saturday.
/// Uses Zeller's congruence (Gregorian).
fn first_weekday(year: u32, month: u32) -> u32 {
    let (y, m) = if month < 3 {
        (year - 1, month + 12)
    } else {
        (year, month)
    };
    let k = y % 100;
    let j = y / 100;
    let h = (1 + (13 * (m + 1)) / 5 + k + k / 4 + j / 4 + 5 * j) % 7;
    // Zeller: h=0 => Saturday. Remap so Sunday=0.
    (h + 6) % 7
}

impl<M: Clone + 'static> From<DatePickerBuilder<M>> for WidgetNode<M> {
    fn from(b: DatePickerBuilder<M>) -> Self {
        // Closed: show the selected date string or placeholder in a Card.
        if !b.open {
            let display = b
                .selected_day
                .map(|d| format!("{:04}-{:02}-{:02}", b.year, b.month, d))
                .unwrap_or_else(|| b.placeholder.clone());
            return card::<M>()
                .key(b.id)
                .variant(CardVariant::Outlined)
                .padding(8.0)
                .child(label::<M>(display))
                .build();
        }

        // Open: header + weekdays + day grid.
        let header = row::<M>()
            .gap(8.0)
            .child(button::<M>(format!("{}-prev", b.id.as_str()).as_str(), "◀"))
            .child(label::<M>(format!("{:04}-{:02}", b.year, b.month)))
            .child(button::<M>(format!("{}-next", b.id.as_str()).as_str(), "▶"));

        let weekdays = ["S", "M", "T", "W", "T", "F", "S"];
        let mut weekday_row = row::<M>().gap(4.0);
        for w in weekdays {
            weekday_row = weekday_row.child(label::<M>(w));
        }

        let dim = days_in_month(b.year, b.month);
        let offset = first_weekday(b.year, b.month);
        let total_cells = 35usize;
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

        column::<M>()
            .key(b.id)
            .gap(4.0)
            .child(header)
            .child(weekday_row)
            .child(day_grid.build())
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
    fn date_picker_builder_defaults() {
        let dp = date_picker::<TestMsg>("dp");
        assert_eq!(dp.year, 2025);
        assert_eq!(dp.month, 1);
        assert!(dp.selected_day.is_none());
        assert!(!dp.open);
        assert!(dp.placeholder.is_empty());
        assert!(dp.on_change.is_none());
    }

    #[test]
    fn date_picker_closed_renders_card() {
        let node: WidgetNode<TestMsg> = date_picker("dp")
            .placeholder("Pick a date")
            .into();
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
        let node: WidgetNode<TestMsg> = date_picker("dp")
            .open(false)
            .selected_day(Some(15))
            .into();
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
        let node: WidgetNode<TestMsg> = date_picker("dp")
            .year(2025)
            .month(1)
            .open(true)
            .into();
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
        // 35 / 7 = 5 rows
        assert_eq!(grid.children.len(), 5);
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
        let node: WidgetNode<TestMsg> = date_picker("dp")
            .open(true)
            .year(2025)
            .month(2)
            .into();
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
    fn date_picker_first_weekday_jan_2025_is_wednesday() {
        // Jan 1, 2025 is a Wednesday (0=Sunday => 3).
        assert_eq!(first_weekday(2025, 1), 3);
    }
}
