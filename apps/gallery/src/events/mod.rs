//! Event handling module — layered architecture.
//!
//! Architecture (bottom → top):
//!
//!   ┌──────────────────────────────────────────────┐
//!   │  main.rs — event() match → PlatformEvent     │  Layer 4
//!   ├──────────────────────────────────────────────┤
//!   │  dispatch.rs — per-event-type handlers        │  Layer 3
//!   │              (each takes explicit refs)       │   Event Routing
//!   ├──────────────────────────────────────────────┤
//!   │  activate.rs — ActivationCtx + handle_message│  Layer 2
//!   │  ime.rs — compute_ime_caret_rect             │   State Transition
//!   ├──────────────────────────────────────────────┤
//!   │  hit.rs — hit_test (pure query)              │  Layer 1
//!   │                                              │   Query
//!   └──────────────────────────────────────────────┘

mod activate;
mod dispatch;
mod hit;
mod ime;

pub use activate::{ActivationCtx, handle_message};
pub use dispatch::{
    handle_enter_space_key, handle_general_key, handle_ime_commit, handle_ime_preedit,
    handle_pointer_moved, handle_pointer_pressed, handle_pointer_released, handle_scroll_event,
    handle_tab_key, handle_tree_arrow_key,
};
pub use hit::hit_test;
pub use ime::compute_ime_caret_rect;
