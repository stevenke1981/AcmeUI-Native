//! Frame rendering context and standalone render functions (Layer 4).
//!
//! All functions are pure with respect to `Gallery` — state is passed explicitly
//! via [`RenderCtx`] so that rendering logic is decoupled from app state.

use acme_layout::{LayoutNode, LayoutSnapshot};
use acme_render_wgpu::Frame;
use acme_text::{FontSystem, GlyphAtlas};
use acme_textinput::{render_text_input, TextInputState};
use acme_theme::Theme;
use acme_widgets::{ButtonState, WidgetNode, button};

use crate::render::layout::GalleryNodeIds;
use crate::types::CATEGORIES;
use crate::render::{
    add_text, find_text_input_marker, quad_rect, render_content,
};
use crate::types::GalleryMessage;

// ── Render Context ──────────────────────────────────────────────────────────

/// Per-frame rendering context — bundles all parameters needed by render
/// functions so they never access `Gallery` directly (pure).
pub struct RenderCtx<'a> {
    // Output frame
    pub frame: &'a mut Frame,

    // Font / glyph systems (mutated by add_text / render_text_input)
    pub fonts: &'a mut FontSystem,
    pub atlas: &'a mut GlyphAtlas,

    // Per-frame computed pipeline data
    pub snapshot: &'a LayoutSnapshot,
    pub ids: &'a GalleryNodeIds,
    pub description: &'a WidgetNode<GalleryMessage>,
    pub root: &'a LayoutNode,
    pub theme: &'a Theme,
    pub scale: f32,

    // Gallery interaction state (immutable for this frame)
    pub hovered: Option<usize>,
    pub pressed: Option<usize>,
    pub focused: usize,
    pub show_focus_rings: bool,
    pub selected_category: usize,
    pub selected_page: usize,
    pub scroll: f32,

    // Toolbar labels (computed from Gallery state before rendering)
    pub toolbar_labels: &'a [&'a str; 3],

    // Mutable text‑input / IME state
    pub text_input_rect: &'a mut [f32; 4],
    pub text_input: &'a mut TextInputState,
    pub ime_caret_window_rect: &'a mut Option<[f32; 4]>,
    pub ime_text: &'a mut String,
}

// ── Layer 4 Render Functions ────────────────────────────────────────────────

/// Render sidebar: background → title → 8 category buttons.
pub fn render_sidebar(ctx: &mut RenderCtx) {
    let colors = ctx.theme.colors;

    // Background
    if let Some(r) = ctx.snapshot.get(ctx.ids.sidebar) {
        ctx.frame.quads.push(quad_rect(
            [r.x, r.y, r.width, r.height],
            colors.surface,
            0.0,
            1.0,
            colors.border,
        ));
    }

    // Title
    if let Some(r) = ctx.snapshot.get(ctx.ids.sidebar_label) {
        add_text(
            ctx.fonts,
            ctx.atlas,
            ctx.frame,
            "AcmeUI",
            ([r.x + 4.0, r.y + 2.0], 18.0),
            colors.foreground,
            ctx.scale,
            None,
            ctx.theme.typography.line_height,
        );
    }

    // Category buttons
    for (i, &btn_id) in ctx.ids.sidebar_buttons.iter().enumerate() {
        let btn_idx = i;
        let Some(r) = ctx.snapshot.get(btn_id) else {
            continue;
        };
        let is_selected = i == ctx.selected_category;
        let st = ButtonState {
            hovered: ctx.hovered == Some(btn_idx),
            pressed: ctx.pressed == Some(btn_idx),
            focused: ctx.focused == btn_idx,
        };
        let bg = if is_selected {
            colors.accent
        } else if st.hovered {
            colors.ghost_hover
        } else {
            colors.surface
        };
        let fg = if is_selected {
            colors.primary_foreground
        } else {
            colors.foreground
        };
        ctx.frame.quads.push(quad_rect(
            [r.x, r.y, r.width, r.height],
            bg,
            ctx.theme.radii.md,
            if is_selected || (st.focused && ctx.show_focus_rings) {
                2.0
            } else {
                1.0
            },
            if st.focused {
                colors.ring
            } else {
                colors.border
            },
        ));
        add_text(
            ctx.fonts,
            ctx.atlas,
            ctx.frame,
            CATEGORIES[i].name,
            ([r.x + 12.0, r.y + 9.0], ctx.theme.typography.label),
            fg,
            ctx.scale,
            None,
            ctx.theme.typography.line_height,
        );
    }
}

/// Render toolbar: background → 3 toggle buttons.
pub fn render_toolbar(ctx: &mut RenderCtx) {
    let colors = ctx.theme.colors;

    // Background
    if let Some(r) = ctx.snapshot.get(ctx.ids.toolbar) {
        ctx.frame.quads.push(quad_rect(
            [r.x, r.y, r.width, r.height],
            colors.surface,
            0.0,
            1.0,
            colors.border,
        ));
    }

    // Buttons
    for (i, (&btn_id, &label_text)) in
        ctx.ids.toolbar_buttons.iter().zip(ctx.toolbar_labels.iter()).enumerate()
    {
        let btn_idx = 8 + i;
        let Some(r) = ctx.snapshot.get(btn_id) else {
            continue;
        };
        let st = ButtonState {
            hovered: ctx.hovered == Some(btn_idx),
            pressed: ctx.pressed == Some(btn_idx),
            focused: ctx.focused == btn_idx,
        };
        let btn = button::<GalleryMessage>("", "");
        let resolved = btn.resolve_style(ctx.theme, st);
        ctx.frame.quads.push(quad_rect(
            [r.x, r.y, r.width, r.height],
            resolved.background,
            ctx.theme.radii.md,
            if st.focused && ctx.show_focus_rings {
                2.0
            } else {
                1.0
            },
            if st.focused {
                resolved.focus
            } else {
                resolved.border
            },
        ));
        add_text(
            ctx.fonts,
            ctx.atlas,
            ctx.frame,
            label_text,
            ([r.x + 12.0, r.y + 7.0], 13.0),
            resolved.foreground,
            ctx.scale,
            None,
            ctx.theme.typography.line_height,
        );
    }
}

/// Render scroll‑view page content via DFS widget dispatch.
pub fn render_page_content(ctx: &mut RenderCtx) {
    if let Some(sv_rect) = ctx.snapshot.get(ctx.ids.scroll_view) {
        let clip = [sv_rect.x, sv_rect.y, sv_rect.width, sv_rect.height];
        let mut btn_idx = 11;
        render_content(
            ctx.frame,
            ctx.description,
            ctx.root,
            ctx.snapshot,
            ctx.theme,
            ctx.scale,
            ctx.scroll,
            clip,
            &mut btn_idx,
            ctx.hovered,
            ctx.pressed,
            ctx.focused,
            ctx.fonts,
            ctx.atlas,
            ctx.show_focus_rings,
        );
    }
}

/// Render the TextInput overlay on the Inputs / TextInput page.
pub fn render_text_input_overlay(ctx: &mut RenderCtx) {
    #[allow(clippy::collapsible_if)]
    if ctx.selected_category == 1 && ctx.selected_page == 1 {
        if let Some(ph_id) = find_text_input_marker(ctx.description, ctx.root) {
            if let Some(ph) = ctx.snapshot.get(ph_id) {
                let y = ph.y - ctx.scroll;
                let rect = [ph.x, y, ph.width, ph.height];
                *ctx.text_input_rect = rect;
                let focused = ctx.text_input.focused;
                render_text_input(
                    ctx.frame,
                    ctx.text_input,
                    ctx.fonts,
                    ctx.atlas,
                    rect,
                    ctx.theme,
                    ctx.scale,
                    focused,
                    None,
                );
                // IME caret cache refresh is handled by the caller after this returns.

                if !ctx.ime_text.is_empty() {
                    add_text(
                        ctx.fonts,
                        ctx.atlas,
                        ctx.frame,
                        &format!("Committed: {}", ctx.ime_text),
                        ([ph.x + 2.0, y + ph.height + 6.0], 14.0),
                        ctx.theme.colors.muted_foreground,
                        ctx.scale,
                        None,
                        ctx.theme.typography.line_height,
                    );
                }
            }
        }
    } else {
        *ctx.text_input_rect = [0.0; 4];
        if ctx.ime_caret_window_rect.is_some() {
            *ctx.ime_caret_window_rect = None;
        }
    }
}
