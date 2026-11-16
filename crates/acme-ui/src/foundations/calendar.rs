//! Calendar component — a standalone date grid widget.
//!
//! Renders a 7-column grid (Sun–Sat) with day-of-month cells for the given
//! year/month. The current day, selected day or same-month range are highlighted.
//!
//! This is a *structural* placeholder using Labels and Cards, not a true
//! interactive date picker. A future version should add navigation buttons
//! (prev/next month) and click handling.

use crate::WidgetNode;
use acme_layout::Length;
use acme_widgets::style::{ColorToken, Styled, TextAlign};

/// Builder for a calendar grid widget.
pub struct CalendarBuilder<M> {
    pub year: i32,
    pub month: u32, // 1–12
    pub selected_day: Option<u32>,
    pub today_day: Option<u32>,
    pub show_header: bool,
    selected_range: Option<(u32, u32)>,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a calendar builder.
pub fn calendar<M>(year: i32, month: u32) -> CalendarBuilder<M> {
    CalendarBuilder {
        year,
        month: month.clamp(1, 12),
        selected_day: None,
        today_day: None,
        show_header: true,
        selected_range: None,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> CalendarBuilder<M> {
    /// Set the selected day (highlighted).
    pub fn selected_day(mut self, day: u32) -> Self {
        self.selected_day = Some(day);
        self.selected_range = None;
        self
    }

    /// Highlight an inclusive range in the displayed month using theme accents.
    ///
    /// Reversed endpoints are normalized. If either endpoint is outside the
    /// displayed month's valid days, no range is highlighted. Validation uses
    /// the year/month at build time. This replaces the single-day selection;
    /// calling `selected_day` afterward replaces the range. Cells remain passive.
    pub fn selected_range(mut self, start: u32, end: u32) -> Self {
        self.selected_range = Some((start.min(end), start.max(end)));
        self.selected_day = None;
        self
    }

    /// Set today's day (circled / different accent).
    pub fn today_day(mut self, day: u32) -> Self {
        self.today_day = Some(day);
        self
    }

    /// Hide the month/year header row.
    pub fn hide_header(mut self) -> Self {
        self.show_header = false;
        self
    }

    /// Build the calendar widget.
    ///
    /// Returns a `Column` with:
    /// - Optional header row: "Month Year"
    /// - Weekday header row: "Su Mo Tu We Th Fr Sa"
    /// - Day grid rows (up to 6 rows of 7 cells)
    pub fn build(self) -> WidgetNode<M> {
        let mut col = crate::column().gap(2.0).padding(8.0);

        // Month/Year header
        if self.show_header {
            let month_name = month_name(self.month);
            let header_text = format!("{month_name} {}", self.year);
            col = col.child(
                crate::row()
                    .key("calendar_header")
                    .child(crate::label(&header_text)),
            );
        }

        // Weekday header
        let weekday_row = self.build_weekday_header();
        col = col.child(weekday_row);

        // Day grid
        let day_rows = self.build_day_grid();
        for row_node in day_rows {
            col = col.child(row_node);
        }

        col.build()
    }

    fn build_weekday_header(&self) -> WidgetNode<M> {
        let days = ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"];
        let mut row = crate::row().key("calendar_weekdays").gap(2.0).padding(2.0);
        for d in &days {
            row = row.child(
                crate::label_builder(*d)
                    .w(Length::px(28.0))
                    .flex_shrink(0.0)
                    .text_align(TextAlign::Center)
                    .build(),
            );
        }
        row.build()
    }

    fn build_day_grid(&self) -> Vec<WidgetNode<M>> {
        let days_in_month = days_in_month(self.year, self.month);
        let first_weekday = first_weekday_of_month(self.year, self.month); // 0=Sun
        let range = self
            .selected_range
            .filter(|(start, end)| *start >= 1 && *end <= days_in_month);
        let mut weeks: Vec<WidgetNode<M>> = Vec::new();
        let mut day: i32 = 1i32 - first_weekday as i32; // may be negative (previous month)

        while day <= days_in_month as i32 {
            let mut row = crate::row()
                .key(format!("calendar_week_{}", weeks.len()))
                .gap(2.0)
                .padding(2.0);
            for _ in 0..7 {
                if day >= 1 && day <= days_in_month as i32 {
                    let d = day as u32;
                    let is_selected = self.selected_day == Some(d);
                    let is_today = self.today_day == Some(d);
                    let in_range = range.is_some_and(|(start, end)| (start..=end).contains(&d));
                    let variant = if is_selected {
                        crate::CardVariant::Interactive
                    } else if is_today {
                        crate::CardVariant::Outlined
                    } else {
                        crate::CardVariant::Plain
                    };
                    let mut text = crate::label_builder(d.to_string())
                        .w(Length::px(20.0))
                        .text_align(TextAlign::Center);
                    let mut content = crate::row().padding(4.0);
                    if in_range {
                        content = content.bg(ColorToken::Accent);
                        text = text.text_color(ColorToken::AccentForeground);
                    }
                    let cell = crate::card()
                        .key(format!("calendar_day_{d}"))
                        .padding(0.0)
                        .w(Length::px(28.0))
                        .flex_shrink(0.0)
                        .variant(variant)
                        .child(content.child(text.build()))
                        .build();
                    row = row.child(cell);
                } else {
                    // Empty cell (day from adjacent month)
                    row = row.child(crate::column().width(28.0).height(28.0).build());
                }
                day += 1;
            }
            weeks.push(row.build());
        }
        weeks
    }
}

impl<M: Clone + 'static> From<CalendarBuilder<M>> for WidgetNode<M> {
    fn from(b: CalendarBuilder<M>) -> Self {
        b.build()
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

fn month_name(m: u32) -> &'static str {
    match m {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "Unknown",
    }
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn first_weekday_of_month(year: i32, month: u32) -> u32 {
    // Zeller-like / Tomohiko Sakamoto's algorithm for day of week (0=Sun)
    let y = if month < 3 { year - 1 } else { year };
    let m = if month < 3 { month + 12 } else { month };
    let q = 1i32; // first day of month
    let k = y % 100;
    let j = y / 100;
    let h = (q + (13 * (m as i32 + 1)) / 5 + k + k / 4 + j / 4 + 5 * j) % 7;
    // h is 0=Sat, 1=Sun ... 6=Fri -> convert to 0=Sun
    let dow = (h + 6) % 7;
    dow as u32
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
#[path = "calendar_range_tests.rs"]
mod range_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn calendar_builds_with_header() {
        let node: WidgetNode<TestMsg> = calendar(2026, 7).build();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant");
        };
        // Header row + weekday row + day rows (at least 4 weeks)
        assert!(
            col.children.len() >= 5,
            "calendar should have header + weekday + day rows"
        );
    }

    #[test]
    fn calendar_hide_header() {
        let node: WidgetNode<TestMsg> = calendar(2026, 7).hide_header().build();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant");
        };
        // Without header: weekday row + day rows
        assert!(col.children.len() >= 4);
    }

    #[test]
    fn calendar_selected_day() {
        let node: WidgetNode<TestMsg> = calendar(2026, 7).selected_day(15).build();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant");
        };
        // The day grid starts at index 1 (after header) or 0 (no header)
        assert!(col.children.len() >= 3);
    }

    #[test]
    fn calendar_february_leap_year() {
        // 2024 is a leap year
        let node: WidgetNode<TestMsg> = calendar(2024, 2).build();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant");
        };
        assert!(
            col.children.len() >= 5,
            "February 2024 should have header + weekday + day rows"
        );
    }

    #[test]
    fn calendar_february_non_leap() {
        // 2023 is not a leap year
        let node: WidgetNode<TestMsg> = calendar(2023, 2).build();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant");
        };
        assert!(col.children.len() >= 5);
    }

    #[test]
    fn calendar_from_trait() {
        let node: WidgetNode<TestMsg> = calendar(2026, 7).into();
        let WidgetNode::Column(_) = &node else {
            panic!("expected Column variant");
        };
    }

    #[test]
    fn calendar_month_clamped() {
        let b = calendar::<TestMsg>(2026, 13);
        assert_eq!(b.month, 12);
        let b2 = calendar::<TestMsg>(2026, 0);
        assert_eq!(b2.month, 1);
    }

    #[test]
    fn calendar_first_weekday() {
        // 2026-07-01 is Wednesday (3)
        let wd = first_weekday_of_month(2026, 7);
        assert_eq!(wd, 3, "July 2026 starts on Wednesday");

        // 2026-01-01 is Thursday (4)
        let wd = first_weekday_of_month(2026, 1);
        assert_eq!(wd, 4, "January 2026 starts on Thursday");
    }

    #[test]
    fn calendar_days_in_month() {
        assert_eq!(days_in_month(2026, 1), 31);
        assert_eq!(days_in_month(2026, 2), 28);
        assert_eq!(days_in_month(2024, 2), 29); // leap year
        assert_eq!(days_in_month(2026, 4), 30);
    }

    #[test]
    fn calendar_is_leap_year() {
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(2023));
        assert!(!is_leap_year(1900));
        assert!(is_leap_year(2000));
    }
}
