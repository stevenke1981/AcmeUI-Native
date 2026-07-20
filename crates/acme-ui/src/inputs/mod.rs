//! High-level input components.
//!
//! Each module provides a builder struct that converts into [`WidgetNode<M>`]
//! using existing AcmeUI-Native primitives.

pub mod checkbox;
pub mod combobox;
pub mod radio;
pub mod search_input;
pub mod segmented_control;
pub mod select;
pub mod slider;
pub mod switch;
pub mod textarea;

pub use checkbox::*;
pub use combobox::*;
pub use radio::*;
pub use search_input::*;
pub use segmented_control::*;
pub use select::*;
pub use slider::*;
pub use switch::*;
pub use textarea::*;
