//! Chart components — **structural placeholders** using existing primitives.
//!
//! Each chart is non-interactive (v1) and composes Row, Column, Label,
//! and Card widgets into readable data visualizations.
//!
//! > **Note:** These are layout-level placeholders, not true rendered charts.
//! > They display data as text markers and labels. A future version should
//! > integrate a vector rendering pipeline (e.g., custom wgpu passes or vello)
//! > for actual chart drawing with axes, curves, and fills.

pub mod area_chart;
pub mod bar_chart;
pub mod box_plot;
pub mod bubble_chart;
pub mod candlestick_chart;
pub mod donut_chart;
pub mod funnel_chart;
pub mod gauge;
pub mod heatmap;
pub mod histogram;
pub mod line_chart;
pub mod parallel_coordinates;
pub mod pie_chart;
pub mod radar_chart;
pub mod radial_bar;
pub mod scatter_chart;
pub mod sparkline;
pub mod timeline_chart;
pub mod treemap;
pub mod waterfall;

pub use area_chart::*;
pub use bar_chart::*;
pub use box_plot::*;
pub use bubble_chart::*;
pub use candlestick_chart::*;
pub use donut_chart::*;
pub use funnel_chart::*;
pub use gauge::*;
pub use heatmap::*;
pub use histogram::*;
pub use line_chart::*;
pub use parallel_coordinates::*;
pub use pie_chart::*;
pub use radar_chart::*;
pub use radial_bar::*;
pub use scatter_chart::*;
pub use sparkline::*;
pub use timeline_chart::*;
pub use treemap::*;
pub use waterfall::*;
