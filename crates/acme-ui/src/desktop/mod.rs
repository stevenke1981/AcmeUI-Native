//! Desktop-specific components — window chrome, navigation, property inspection,
//! dock panels, markdown rendering, image placeholders, and resize handles.
//!
//! Each module provides a builder struct that converts into [`WidgetNode<M>`]
//! using existing AcmeUI-Native primitives.

pub mod command_bar;
pub mod dock;
pub mod image_view;
pub mod markdown_view;
pub mod menubar;
pub mod navigation_view;
pub mod property_grid;
pub mod resize_handle;
pub mod sidenav;
pub mod status_tray;
pub mod taskbar;
pub mod title_bar;
pub mod window_controls;

pub use command_bar::*;
pub use dock::*;
pub use image_view::*;
pub use markdown_view::*;
pub use menubar::*;
pub use navigation_view::*;
pub use property_grid::*;
pub use resize_handle::*;
pub use sidenav::*;
pub use status_tray::*;
pub use taskbar::*;
pub use title_bar::*;
pub use window_controls::*;
