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
pub mod gauge;
pub mod line_chart;
pub mod pie_chart;
pub mod sparkline;

pub use area_chart::*;
pub use bar_chart::*;
pub use gauge::*;
pub use line_chart::*;
pub use pie_chart::*;
pub use sparkline::*;
