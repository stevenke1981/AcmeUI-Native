//! Layer 2 — Message activation: state transitions via [`ActivationCtx`].
//!
//! Bundles all mutable Gallery state references so [`handle_message`] can
//! be called without coupling to the `Gallery` struct directly.

use crate::types::*;

/// Context bundling all mutable state fields needed by [`handle_message`].
///
/// This is the event-equivalent of `render::frame::RenderCtx` — explicit
/// parameter passing instead of `&mut self`.
pub struct ActivationCtx<'a> {
    pub dark: &'a mut bool,
    pub density: &'a mut Density,
    pub show_focus_rings: &'a mut bool,
    pub selected_category: &'a mut usize,
    pub selected_page: &'a mut usize,
    pub scroll: &'a mut f32,
    pub nav_rail_selected: &'a mut usize,
    pub tab_bar_selected: &'a mut usize,
    pub tab_bar_zh_selected: &'a mut usize,
    pub tree_expanded: &'a mut u32,
    pub tree_selected: &'a mut Option<&'static str>,
    pub table_sort_col: &'a mut Option<usize>,
    pub table_sort_asc: &'a mut bool,
    pub table_selected_row: &'a mut Option<usize>,
}

/// Dispatch a [`GalleryMessage`] against the activation context.
///
/// Returns `true` if the message was handled (all known messages return `true`).
///
/// # Pure-ish
///
/// While this function mutates state via `ctx`, it is *deterministic* — given
/// the same context + message it always produces the same side-effects. No I/O,
/// no randomness, no external dependencies.
pub fn handle_message(ctx: &mut ActivationCtx, message: GalleryMessage) -> bool {
    match message {
        GalleryMessage::ToggleTheme => {
            *ctx.dark = !*ctx.dark;
            true
        }
        GalleryMessage::ToggleDensity => {
            *ctx.density = ctx.density.toggle();
            true
        }
        GalleryMessage::ToggleFocusRings => {
            *ctx.show_focus_rings = !*ctx.show_focus_rings;
            true
        }
        GalleryMessage::SelectCategory(i) => {
            let changed = *ctx.selected_category != i;
            *ctx.selected_category = i;
            *ctx.selected_page = 0;
            if changed {
                *ctx.scroll = 0.0;
            }
            true
        }
        GalleryMessage::SelectPage(i) => {
            *ctx.selected_page = i;
            *ctx.scroll = 0.0;
            true
        }
        GalleryMessage::NavRailSelect(i) => {
            *ctx.nav_rail_selected = i;
            true
        }
        GalleryMessage::TabBarSelect(i) => {
            *ctx.tab_bar_selected = i;
            true
        }
        GalleryMessage::TabBarZhSelect(i) => {
            *ctx.tab_bar_zh_selected = i;
            true
        }
        GalleryMessage::TreeSelectKey(key) => {
            if *ctx.tree_selected == Some(key) {
                *ctx.tree_expanded = crate::helpers::tree_toggle_expanded(*ctx.tree_expanded, key);
            }
            *ctx.tree_selected = Some(key);
            true
        }
        GalleryMessage::TreeToggleKey(key) => {
            *ctx.tree_expanded = crate::helpers::tree_toggle_expanded(*ctx.tree_expanded, key);
            *ctx.tree_selected = Some(key);
            true
        }
        GalleryMessage::TableSort(col) => {
            if *ctx.table_sort_col == Some(col) {
                *ctx.table_sort_asc = !*ctx.table_sort_asc;
            } else {
                *ctx.table_sort_col = Some(col);
                *ctx.table_sort_asc = true;
            }
            true
        }
        GalleryMessage::TableSelectRow(orig) => {
            *ctx.table_selected_row = Some(orig);
            true
        }
        GalleryMessage::FocusDemo | GalleryMessage::DpiInfo => true,
    }
}
