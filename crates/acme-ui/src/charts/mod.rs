//! Chart components — static representations using existing primitives.
//!
//! Each chart is non-interactive (v1) and composes Row, Column, Label,
//! and Card widgets into readable data visualizations.

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
