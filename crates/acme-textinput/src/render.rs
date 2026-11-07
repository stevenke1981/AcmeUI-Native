//! Rendering of a text input: colours, background, glyphs, caret and preedit.

use super::*;

/// Helper: convert a [`ThemeColor`] to an RGBA `[f32; 4]` array.
pub(crate) fn theme_color_to_array(c: &ThemeColor) -> [f32; 4] {
    [c.red, c.green, c.blue, c.alpha]
}

/// Helper: intersect two `[x, y, w, h]` rectangles.
pub(crate) fn intersect_rect(a: [f32; 4], b: [f32; 4]) -> [f32; 4] {
    let x = a[0].max(b[0]);
    let y = a[1].max(b[1]);
    let right = (a[0] + a[2]).min(b[0] + b[2]);
    let bottom = (a[1] + a[3]).min(b[1] + b[3]);
    if right > x && bottom > y {
        [x, y, right - x, bottom - y]
    } else {
        [0.0, 0.0, 0.0, 0.0]
    }
}

// ---------------------------------------------------------------------------
// TextInput rendering helpers
// ---------------------------------------------------------------------------

/// Theme-derived colours for a text input.
pub(crate) struct TextInputColors {
    text_color: [f32; 4],
    border_color: [f32; 4],
    disabled_text: [f32; 4],
    text_muted: [f32; 4],
    bg_color: [f32; 4],
}

/// Compute all theme-derived colours for a text input in one call.
pub(crate) fn compute_text_input_colors(
    theme: &Theme,
    state: &TextInputState,
    focused: bool,
) -> TextInputColors {
    let text_color = theme_color_to_array(&theme.colors.foreground);
    let disabled_text = theme_color_to_array(&theme.colors.disabled_text);
    let text_muted = theme_color_to_array(&theme.colors.muted_foreground);
    let bg_color = theme_color_to_array(&theme.colors.surface);

    // Border colour: invalid > focus > default
    let border_color = if state.invalid {
        theme_color_to_array(&theme.colors.danger)
    } else if focused {
        theme_color_to_array(&theme.colors.ring)
    } else {
        theme_color_to_array(&theme.colors.border)
    };

    // Slightly dimmed border when readonly
    let border_color = if state.readonly {
        let [r, g, b, a] = border_color;
        [r * 0.6, g * 0.6, b * 0.6, a]
    } else {
        border_color
    };

    TextInputColors {
        text_color,
        border_color,
        disabled_text,
        text_muted,
        bg_color,
    }
}

/// Push background fill and border quads for a text input.
pub(crate) fn render_text_input_background(
    scene: &mut Scene,
    rect: [f32; 4],
    border_color: [f32; 4],
    bg_color: [f32; 4],
    theme: &Theme,
) {
    let border_width = 1.0;

    // 1. Background quad
    scene.push(DrawCommand::Quad(QuadPrimitive {
        rect: Rect::new(rect[0], rect[1], rect[2], rect[3]),
        color: Color::rgba(bg_color[0], bg_color[1], bg_color[2], bg_color[3]),
        radius: 0.0,
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    }));

    // 2. Border quad
    scene.push(DrawCommand::Quad(QuadPrimitive {
        rect: Rect::new(rect[0], rect[1], rect[2], rect[3]),
        color: Color::rgba(0.0, 0.0, 0.0, 0.0),
        radius: theme.radii.sm,
        border_width,
        border_color: Color::rgba(
            border_color[0],
            border_color[1],
            border_color[2],
            border_color[3],
        ),
    }));
}

/// Prepare glyph draw-calls and atlas uploads from the cached text layout.
///
/// Returns `(glyphs, uploads)` suitable for a `DrawCommand::Text` primitive.
pub(crate) fn prepare_text_glyphs(
    state: &TextInputState,
    fonts: &mut FontSystem,
    atlas: &mut GlyphAtlas,
) -> (Vec<GlyphDraw>, Vec<SceneAtlasUpload>) {
    let (_, cached) = state
        .cached_layout
        .as_ref()
        .expect("prepare_text_glyphs called before cached_layout was populated");

    // Prepare glyphs from cached layout
    let prepared = fonts.prepare(cached, atlas);

    // Convert prepared text to scene types
    let mut glyphs = Vec::new();
    let mut uploads = Vec::new();
    for region in &prepared.uploads {
        let pfmt = match region.format {
            acme_text::AtlasFormat::Alpha8 => GlyphFormat::Alpha8,
            acme_text::AtlasFormat::Rgba8 => GlyphFormat::Rgba8,
        };
        uploads.push(SceneAtlasUpload {
            page: region.page,
            origin: [region.x, region.y],
            size: [region.width, region.height],
            format: pfmt,
            pixels: region.pixels.clone(),
        });
    }
    for glyph in &prepared.glyphs {
        let gfmt = match glyph.format {
            acme_text::AtlasFormat::Alpha8 => GlyphFormat::Alpha8,
            acme_text::AtlasFormat::Rgba8 => GlyphFormat::Rgba8,
        };
        glyphs.push(GlyphDraw {
            x: glyph.x as f32,
            y: glyph.y as f32,
            width: glyph.width as f32,
            height: glyph.height as f32,
            atlas_x: glyph.atlas_x,
            atlas_y: glyph.atlas_y,
            atlas_page: glyph.atlas_page,
            format: gfmt,
        });
    }
    (glyphs, uploads)
}

/// Renders a TextInput cursor and selection into a [`Scene`].
///
/// `rect` is `[x, y, width, height]` in logical pixels.
/// `clip` is an optional outer clip rectangle; the content area is further
/// clipped to the inner padding region.
#[allow(clippy::too_many_arguments)]
pub fn render_text_input(
    scene: &mut Scene,
    state: &mut TextInputState,
    fonts: &mut FontSystem,
    atlas: &mut GlyphAtlas,
    rect: [f32; 4], // [x, y, width, height] in logical pixels
    theme: &Theme,
    scale: f32,
    focused: bool,
    clip: Option<[f32; 4]>,
) {
    let [x, y, w, h] = rect;
    if w <= 0.0 || h <= 0.0 {
        return;
    }

    let padding = theme.spacing.px2; // 8px vertical/horizontal inner padding
    let colors = compute_text_input_colors(theme, state, focused);
    render_text_input_background(scene, rect, colors.border_color, colors.bg_color, theme);

    // Build style from theme typography tokens
    let style = TextStyle {
        font_size: theme.typography.body,
        line_height: theme.typography.body * theme.typography.line_height,
        ..TextStyle::default()
    };
    // Compute content area (inside padding)
    let content_x = x + padding;
    let content_y = y + padding;
    let content_w = (w - 2.0 * padding).max(0.0);
    let content_h = (h - 2.0 * padding).max(0.0);

    // Build the clip rectangle for the content area
    let content_clip = [content_x, content_y, content_w, content_h];
    let effective_clip = match clip {
        Some(c) => intersect_rect(c, content_clip),
        None => content_clip,
    };
    // 3. Text content (or placeholder)
    let display = state.display_text();
    let is_empty = display.is_empty();
    let has_placeholder = !state.placeholder.is_empty();

    // If text is empty and a placeholder is set, render that instead.
    let render_text = if is_empty && has_placeholder {
        &state.placeholder
    } else {
        &display
    };

    // Choose colour: placeholder gets muted; readonly gets disabled. A composing
    // field must never borrow the placeholder token: while a preedit is active
    // the placeholder is not drawn, so the composition takes the text colour.
    let composing = !state.preedit.is_empty();
    let render_color = if is_empty && has_placeholder && !composing {
        colors.text_muted
    } else if state.readonly {
        colors.disabled_text
    } else {
        colors.text_color
    };
    // We want the text to be vertically centered in the content area.
    // Shape to get the text height, then compute y offset.
    let constraints = TextConstraints {
        max_width: Some(content_w),
        wrap: TextWrap::None,
    };
    // Use cached layout if the render_text hasn't changed — avoids re-shaping
    // every frame for static or infrequently-changing text.
    if state
        .cached_layout
        .as_ref()
        .is_none_or(|(t, _)| t != render_text)
    {
        let new_layout = fonts.shape(render_text, &style, constraints, scale);
        state.cached_layout = Some((render_text.to_string(), new_layout));
    }
    let (_, cached) = state
        .cached_layout
        .as_ref()
        .expect("cached_layout just populated above");
    let text_height = cached.height;
    // Vertically center
    let text_y = content_y + (content_h - text_height).max(0.0) / 2.0;

    // Apply horizontal scroll offset (text scrolls left, so content_x shifts)
    let text_origin_x = content_x - state.scroll_offset;

    // Push clip, then text, then pop clip. While an IME preedit is active the
    // committed text and the composition are drawn as separate runs so the
    // composition appears inline and the text after the caret is shifted.
    let clip = [
        effective_clip[0],
        effective_clip[1],
        effective_clip[2],
        effective_clip[3],
    ];
    let preedit_runs = preedit::preedit_runs(state, &display, fonts, &style, scale);
    if let Some(runs) = &preedit_runs {
        preedit::draw_composing_text(
            scene,
            state,
            runs,
            fonts,
            atlas,
            &style,
            scale,
            text_origin_x,
            text_y,
            render_color,
            clip,
        );
    } else {
        // Prepare glyphs from cached layout
        let (glyphs, uploads) = prepare_text_glyphs(state, fonts, atlas);
        scene.push(DrawCommand::PushClip(Rect::new(
            effective_clip[0],
            effective_clip[1],
            effective_clip[2],
            effective_clip[3],
        )));
        scene.push(DrawCommand::Text(TextPrimitive {
            origin: Point::new(text_origin_x, text_y),
            color: Color::rgba(
                render_color[0],
                render_color[1],
                render_color[2],
                render_color[3],
            ),
            glyphs,
            uploads,
        }));
        scene.push(DrawCommand::PopClip);
    }
    // 4. Selection highlight. While a preedit is active the text after the caret
    // is drawn shifted, so the highlight is emitted in pieces that follow the
    // drawn runs instead of one span across the unshifted display text.
    let selection_spans =
        preedit::selection_spans(state, &display, preedit_runs.as_ref(), fonts, &style, scale);
    for (start_x, width) in selection_spans {
        if width > 0.0 {
            scene.push(DrawCommand::PushClip(Rect::new(
                effective_clip[0],
                effective_clip[1],
                effective_clip[2],
                effective_clip[3],
            )));
            scene.push(DrawCommand::Quad(QuadPrimitive {
                rect: Rect::new(text_origin_x + start_x, content_y, width, content_h),
                color: Color::rgba(0.3, 0.5, 0.9, 0.25),
                radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            }));
            scene.push(DrawCommand::PopClip);
        }
    }
    // 5. Cursor blink
    if focused {
        let cx = byte_offset_to_x_inner(&display, state.display_cursor(), fonts, &style, scale);
        let cursor_x_pos = text_origin_x + cx;
        // Only draw cursor if it's within the visible content area
        if cursor_x_pos < content_x + content_w {
            scene.push(DrawCommand::PushClip(Rect::new(
                effective_clip[0],
                effective_clip[1],
                effective_clip[2],
                effective_clip[3],
            )));
            scene.push(DrawCommand::Quad(QuadPrimitive {
                rect: Rect::new(cursor_x_pos, content_y, CARET_WIDTH, content_h),
                color: Color::rgba(
                    colors.text_color[0],
                    colors.text_color[1],
                    colors.text_color[2],
                    colors.text_color[3],
                ),
                radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            }));
            scene.push(DrawCommand::PopClip);
        }
    }
    // 6. IME preedit underline and composition cursor
    if let Some(runs) = &preedit_runs {
        preedit::draw_preedit_underline(
            scene,
            state,
            runs,
            fonts,
            &style,
            scale,
            text_origin_x,
            content_y + content_h - 2.5,
            colors.text_color,
            clip,
        );
    }
}
