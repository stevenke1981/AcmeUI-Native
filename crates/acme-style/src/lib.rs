//! GPUI-inspired Styled trait + Tailwind CSS-like utility styling system for AcmeUI Native.
//!
//! # Overview
//!
//! `acme-style` provides a composable, chainable styling API inspired by both
//! [GPUI](https://github.com/zed-industries/zed/tree/main/crates/gpui) (Zed's
//! GPU-accelerated UI framework) and [Tailwind CSS](https://tailwindcss.com/).
//!
//! ## Core concepts
//!
//! - **[`Style`]** — Accumulates all visual and layout properties (width, padding,
//!   background color, border radius, shadow, etc.).
//! - **[`Styled`]** — Trait that any widget type can implement to gain
//!   tailwind‑like chainable methods (`w_4`, `h_4`, `bg_primary`, `rounded_md`, …).
//! - **[`ColorToken`]** — Theme‑aware color references. A `ColorToken` can be
//!   either a direct [`ThemeColor`] or a semantic token (e.g. `Primary`,
//!   `Foreground`) that resolves against the active [`Theme`] at render time.
//!
//! # Example
//!
//! ```ignore
//! use acme_style::prelude::*;
//!
//! // Tailwind-style utility chaining
//! col()
//!     .w_full()
//!     .p_4()
//!     .gap_4()
//!     .bg_surface()
//!     .rounded_lg()
//!     .shadow_md()
//!     .child(
//!         text("Hello AcmeUI!")
//!             .text_foreground()
//!             .font_size(18)
//!     )
//!     .build()
//! ```

#![forbid(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

mod color;
mod style;
mod styled;
mod tailwind;

pub use color::*;
pub use style::*;
pub use styled::*;
pub use tailwind::*;

/// Prelude: import everything you need for day‑to‑day styling.
pub mod prelude {
    pub use crate::color::ColorToken;
    pub use crate::style::Style;
    pub use crate::styled::Styled;
    pub use crate::tailwind::*;

    // Re-export layout types commonly used with style methods
    pub use acme_layout::{Edges, Length, Overflow};
}
