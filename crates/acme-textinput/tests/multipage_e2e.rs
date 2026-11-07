//! ROADMAP 011 P6: multi-page glyph atlas, end to end.
//!
//! A `with_config` atlas with 64x64 pages (at most 4) is filled by real
//! text-input frames until page 1 is opened. The final frame goes through the
//! public `render_text_input` path (text -> Scene) and is drawn by
//! `CpuRenderer`. Its pixels must equal the same frame drawn from one
//! 2048x2048 page. The test skips with a message when the host has no fonts.

use acme_core::{ATLAS_MAX_PAGES, ATLAS_PAGE_EDGE, DrawCommand, Scene};
use acme_render_cpu::CpuRenderer;
use acme_text::{FontSystem, GlyphAtlas, GlyphAtlasConfig, TextConstraints, TextStyle};
use acme_textinput::{TextInputState, handle_text, render_text_input};
use acme_theme::Theme;

const PAGE_EDGE: u32 = 64;
const MAX_PAGES: u32 = 4;
const SURFACE: (u32, u32) = (320, 64);
const RECT: [f32; 4] = [0.0, 0.0, 300.0, 48.0];
const FILL_TEXT: &str = "Hello, 世界! Atlas pages 0123456789";
const FRAME_TEXT: &str = "Multi-page 多頁 glyph";
/// Filler frames use body sizes above this offset so their glyphs never match
/// the final frame's glyph cache keys.
const FILL_SIZE_OFFSET: f32 = 10.0;
const MAX_FILL_FRAMES: usize = 64;

fn small_pages() -> GlyphAtlas {
    GlyphAtlas::with_config(GlyphAtlasConfig {
        page_width: PAGE_EDGE,
        page_height: PAGE_EDGE,
        max_pages: MAX_PAGES,
    })
}

/// One focused text-input frame drawn with `theme`'s body size.
fn text_frame(text: &str, theme: &Theme, fonts: &mut FontSystem, atlas: &mut GlyphAtlas) -> Scene {
    let mut state = TextInputState::new();
    state.focused = true;
    handle_text(&mut state, text);
    let mut scene = Scene::new();
    render_text_input(
        &mut scene, &mut state, fonts, atlas, RECT, theme, 1.0, true, None,
    );
    scene
}

/// Atlas page index of every glyph drawn by `scene`.
fn glyph_pages(scene: &Scene) -> Vec<u32> {
    scene
        .commands()
        .iter()
        .flat_map(|cmd| match cmd {
            DrawCommand::Text(text) => text
                .glyphs
                .iter()
                .map(|glyph| glyph.atlas_page)
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        })
        .collect()
}

/// `false` when the host provides no usable font (the test then skips).
fn host_has_glyphs(fonts: &mut FontSystem) -> bool {
    let style = TextStyle::default();
    let layout = fonts.shape("A", &style, TextConstraints::default(), 1.0);
    let mut scratch = GlyphAtlas::new(64, 64);
    !fonts.prepare(&layout, &mut scratch).glyphs.is_empty()
}

/// Runs filler frames, each rendered, until the atlas has opened page 1.
/// Returns the number of frames that ran.
fn fill_until_second_page(
    renderer: &mut CpuRenderer,
    fonts: &mut FontSystem,
    atlas: &mut GlyphAtlas,
) -> usize {
    let body = Theme::light().typography.body;
    for step in 0..MAX_FILL_FRAMES {
        if atlas.page_count() >= 2 {
            return step;
        }
        atlas.begin_frame();
        let mut theme = Theme::light();
        theme.typography.body = body + FILL_SIZE_OFFSET + step as f32;
        let scene = text_frame(FILL_TEXT, &theme, fonts, atlas);
        renderer.render_scene(&scene).expect("filler frame renders");
    }
    MAX_FILL_FRAMES
}

// Compile-time guards: a violated bound breaks the build, not only this test.
const _: () = assert!(GlyphAtlasConfig::DEFAULT_PAGE_EDGE <= ATLAS_PAGE_EDGE);
const _: () = assert!(GlyphAtlasConfig::DEFAULT_MAX_PAGES <= GlyphAtlasConfig::MAX_PAGES_CAP);

#[test]
fn config_constants_match_the_core_atlas_contract() {
    assert_eq!(GlyphAtlasConfig::MAX_PAGES_CAP, ATLAS_MAX_PAGES);
    assert_eq!(GlyphAtlasConfig::DEFAULT_PAGE_EDGE, 2048);
    assert_eq!(
        GlyphAtlasConfig::default().max_pages,
        GlyphAtlasConfig::DEFAULT_MAX_PAGES
    );
}

#[test]
fn multi_page_text_frame_matches_single_page_pixels() {
    let mut fonts = FontSystem::new();
    if !host_has_glyphs(&mut fonts) {
        eprintln!("SKIP multi_page_text_frame_matches_single_page_pixels: host has no glyphs");
        return;
    }
    let mut atlas = small_pages();
    let mut renderer = CpuRenderer::new(SURFACE.0, SURFACE.1, 1.0).expect("renderer");
    let frames = fill_until_second_page(&mut renderer, &mut fonts, &mut atlas);
    assert!(
        atlas.page_count() >= 2,
        "page 1 must open within {frames} filler frames (page_count {})",
        atlas.page_count()
    );

    let theme = Theme::light();
    atlas.begin_frame();
    let scene = text_frame(FRAME_TEXT, &theme, &mut fonts, &mut atlas);
    let pages = glyph_pages(&scene);
    assert!(!pages.is_empty(), "final frame must draw glyphs");
    // Larger host glyphs can open page 2 or 3 within one filler frame, so the
    // final frame may land on any page after 0; it must not stay on page 0.
    assert!(
        pages.iter().any(|&page| page > 0),
        "at least one glyph must be drawn from a page after 0, got pages {pages:?}"
    );
    renderer
        .render_scene(&scene)
        .expect("multi-page frame renders");

    let mut single = GlyphAtlas::new(2048, 2048);
    single.begin_frame();
    let reference_scene = text_frame(FRAME_TEXT, &theme, &mut fonts, &mut single);
    assert!(glyph_pages(&reference_scene).iter().all(|&page| page == 0));
    let mut reference = CpuRenderer::new(SURFACE.0, SURFACE.1, 1.0).expect("renderer");
    reference
        .render_scene(&reference_scene)
        .expect("single-page frame renders");

    let pixels = renderer.pixels();
    assert!(
        pixels.iter().any(|&pixel| pixel != pixels[0]),
        "frame must not be uniform"
    );
    assert!(
        pixels == reference.pixels(),
        "multi-page pixels differ from the single-page reference"
    );
}
