//! High-level input components.
//!
//! Each module provides a builder struct that converts into [`WidgetNode<M>`]
//! using existing AcmeUI-Native primitives.

pub mod autocomplete;
pub mod cascader;
pub mod checkbox;
pub mod color_picker;
pub mod combobox;
pub mod date_picker;
pub mod multi_select;
pub mod pin_input;
pub mod form_message;
pub mod icon_button;
pub mod number_input;
pub mod password_input;
pub mod radio;
pub mod range_slider;
pub mod rating;
pub mod search_input;
pub mod segmented_control;
pub mod select;
pub mod slider;
pub mod switch;
pub mod textarea;
pub mod time_picker;
pub mod toggle_button;
pub mod transfer;
pub mod tree_select;

pub use autocomplete::*;
pub use cascader::*;
pub use checkbox::*;
pub use color_picker::*;
pub use combobox::*;
pub use date_picker::*;
pub use multi_select::*;
pub use pin_input::*;
pub use form_message::*;
pub use icon_button::*;
pub use number_input::*;
pub use password_input::*;
pub use radio::*;
pub use range_slider::*;
pub use rating::*;
pub use search_input::*;
pub use segmented_control::*;
pub use select::*;
pub use slider::*;
pub use switch::*;
pub use textarea::*;
pub use time_picker::*;
pub use toggle_button::*;
pub use transfer::*;
pub use tree_select::*;
