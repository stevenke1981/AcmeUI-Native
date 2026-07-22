//! Operational dashboard starter. Replace placeholder metrics with validated data.

use acme_ui::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum DashboardMessage {
    Refresh,
    ChangeRange,
    OpenDetails,
    OpenSettings,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DashboardState {
    pub range_label: String,
    pub primary_metric: String,
    pub secondary_metric: String,
    pub status: String,
    pub activity_summary: String,
}

pub fn dashboard_view(state: &DashboardState) -> WidgetNode<DashboardMessage> {
    windows11_template("Operations Dashboard")
        .subtitle("Validated metrics, trends and recent activity")
        .child(
            row::<DashboardMessage>()
                .key("dashboard-toolbar")
                .gap(8.0)
                .child(button("range", state.range_label.as_str()).on_click(DashboardMessage::ChangeRange))
                .child(button("refresh", "Refresh").primary().on_click(DashboardMessage::Refresh))
                .child(button("settings", "Settings").on_click(DashboardMessage::OpenSettings))
                .build(),
        )
        .child(
            row::<DashboardMessage>()
                .key("metric-strip")
                .gap(12.0)
                .child(
                    card::<DashboardMessage>()
                        .gap(4.0)
                        .padding(16.0)
                        .child(label("Primary metric"))
                        .child(label(state.primary_metric.as_str()))
                        .build(),
                )
                .child(
                    card::<DashboardMessage>()
                        .gap(4.0)
                        .padding(16.0)
                        .child(label("Secondary metric"))
                        .child(label(state.secondary_metric.as_str()))
                        .build(),
                )
                .child(
                    card::<DashboardMessage>()
                        .gap(4.0)
                        .padding(16.0)
                        .child(label("System status"))
                        .child(label(state.status.as_str()))
                        .build(),
                )
                .build(),
        )
        .child(
            card::<DashboardMessage>()
                .key("primary-analysis")
                .gap(12.0)
                .padding(20.0)
                .child(label("Primary trend / chart region"))
                .child(label("Replace this region with a real chart backed by product data."))
                .child(button("details", "Open details").on_click(DashboardMessage::OpenDetails))
                .build(),
        )
        .child(
            column::<DashboardMessage>()
                .key("recent-activity")
                .gap(8.0)
                .child(label("Recent activity"))
                .child(separator())
                .child(label(state.activity_summary.as_str()))
                .build(),
        )
        .build()
}
