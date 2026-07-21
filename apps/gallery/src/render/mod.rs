//! Render module — layered rendering pipeline for the Gallery.
//!
//! Architecture (bottom → top):
//!
//!   ┌─────────────────────────────────────────────────────┐
//!   │  content.rs    — render_content() DFS dispatch      │  Layer 3
//!   │  hit_test.rs   — hit region collection              │   Widget Rendering
//!   ├─────────────────────────────────────────────────────┤
//!   │  style.rs      — push_widget_style, push_shadow     │  Layer 2
//!   │  text.rs       — add_text (shape + prepare + push)  │   Style & Text
//!   ├─────────────────────────────────────────────────────┤
//!   │  geometry.rs   — rgba, quad_rect, point_in_rect     │  Layer 1
//!   │  layout.rs     — apply_gallery_styles, ids          │   Geometry & Layout
//!   └─────────────────────────────────────────────────────┘

mod content;
mod frame;
mod geometry;
mod hit_test;
mod layout;
mod style;
mod text;

// ── Layer 4: Frame rendering (pure RenderCtx functions) ──
pub use frame::{
    render_page_content, render_sidebar, render_text_input_overlay, render_toolbar, RenderCtx,
};

// ── Layer 3: Widget rendering ──
pub use content::{
    find_text_input_marker, render_content, tree_key_static,
};

// ── Layer 2: Hit testing ──
pub use hit_test::{collect_data_widget_hits, collect_hit_regions};

// ── Layer 2: Style helpers — push_widget_style used internally by content only

// ── Layer 1: Text helpers ──
pub use text::add_text;

// ── Layer 1: Geometry and layout ──
pub use geometry::{point_in_rect, quad_rect, rgba, scrolled_hit_rect};
pub use layout::{apply_gallery_styles, extract_gallery_ids};
