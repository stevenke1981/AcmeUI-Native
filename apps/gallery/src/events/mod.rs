//! Event handling module — layered architecture.
//!
//! Architecture (bottom → top):
//!
//!   ┌──────────────────────────────────────────────┐
//!   │  main.rs — PlatformEvent dispatch (event())  │  Layer 3
//!   ├──────────────────────────────────────────────┤
//!   │  activate.rs — ActivationCtx + handle_message│  Layer 2
//!   │  ime.rs — compute_ime_caret_rect             │   State Transition
//!   ├──────────────────────────────────────────────┤
//!   │  hit.rs — hit_test (pure query)              │  Layer 1
//!   │                                              │   Query
//!   └──────────────────────────────────────────────┘

mod activate;
mod hit;
mod ime;

pub use activate::{ActivationCtx, handle_message};
pub use hit::hit_test;
pub use ime::compute_ime_caret_rect;
