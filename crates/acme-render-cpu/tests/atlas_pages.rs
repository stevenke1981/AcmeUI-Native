//! Multi-page glyph atlas behavior of the CPU renderer (ROADMAP 011, SPEC section 3).

use acme_core::{
    ATLAS_MAX_PAGES, ATLAS_PAGE_EDGE, AtlasUpload, Color, DrawCommand, GlyphDraw, GlyphFormat,
    Point, Scene, TextPrimitive,
};
use acme_render_cpu::{CpuRenderError, CpuRenderer};

fn red() -> Color {
    Color::rgba(1.0, 0.0, 0.0, 1.0)
}

fn glyph(x: f32, page: u32, format: GlyphFormat, atlas: [u32; 2]) -> GlyphDraw {
    GlyphDraw {
        x,
        y: 0.0,
        width: 1.0,
        height: 1.0,
        atlas_x: atlas[0],
        atlas_y: atlas[1],
        atlas_page: page,
        format,
    }
}

fn upload(page: u32, format: GlyphFormat, origin: [u32; 2], pixels: Vec<u8>) -> AtlasUpload {
    AtlasUpload {
        page,
        origin,
        size: [1, 1],
        format,
        pixels,
    }
}

fn opaque_texel(format: GlyphFormat) -> Vec<u8> {
    match format {
        GlyphFormat::Alpha8 => vec![255],
        GlyphFormat::Rgba8 => vec![0, 0, 0, 255],
    }
}

fn text(glyphs: Vec<GlyphDraw>, uploads: Vec<AtlasUpload>) -> DrawCommand {
    DrawCommand::Text(TextPrimitive {
        origin: Point::new(0.0, 0.0),
        color: Color::rgba(1.0, 1.0, 1.0, 1.0),
        glyphs,
        uploads,
    })
}

fn frame(clear: Color, commands: Vec<DrawCommand>) -> Scene {
    let mut scene = Scene::with_clear(clear);
    for command in commands {
        scene.push(command);
    }
    scene
}

fn run(clear: Color, width: u32, commands: Vec<DrawCommand>) -> Vec<u32> {
    let mut cpu = CpuRenderer::new(width, 1, 1.0).unwrap();
    cpu.render_scene(&frame(clear, commands)).unwrap();
    cpu.pixels().to_vec()
}

#[test]
fn alpha8_page_two_is_visible_while_page_zero_at_same_origin_is_unaffected() {
    let out = run(
        Color::TRANSPARENT,
        4,
        vec![text(
            vec![
                glyph(0.0, 0, GlyphFormat::Alpha8, [0, 0]),
                glyph(1.0, 2, GlyphFormat::Alpha8, [0, 0]),
                glyph(2.0, 0, GlyphFormat::Alpha8, [0, 0]),
                glyph(3.0, 2, GlyphFormat::Alpha8, [0, 0]),
            ],
            vec![
                upload(0, GlyphFormat::Alpha8, [0, 0], vec![255]),
                upload(2, GlyphFormat::Alpha8, [0, 0], vec![128]),
            ],
        )],
    );
    assert_eq!(out, [0x00ffffff, 0x00808080, 0x00ffffff, 0x00808080]);
}

#[test]
fn rgba8_page_two_is_visible_while_page_zero_at_same_origin_is_unaffected() {
    let out = run(
        Color::TRANSPARENT,
        2,
        vec![text(
            vec![
                glyph(0.0, 2, GlyphFormat::Rgba8, [0, 0]),
                glyph(1.0, 0, GlyphFormat::Rgba8, [0, 0]),
            ],
            vec![
                upload(0, GlyphFormat::Rgba8, [0, 0], vec![0, 0, 255, 255]),
                upload(2, GlyphFormat::Rgba8, [0, 0], vec![255, 0, 0, 255]),
            ],
        )],
    );
    assert_eq!(out, [0x00ff0000, 0x000000ff]);
}

#[test]
fn never_uploaded_pages_draw_transparent_for_both_formats() {
    // Page 2 is allocated in both formats, so the probes also rule out a shared buffer.
    let out = run(
        red(),
        4,
        vec![text(
            vec![
                glyph(0.0, 5, GlyphFormat::Alpha8, [0, 0]),
                glyph(1.0, 6, GlyphFormat::Rgba8, [0, 0]),
                glyph(2.0, 7, GlyphFormat::Alpha8, [0, 0]),
                glyph(3.0, 0, GlyphFormat::Rgba8, [0, 0]),
            ],
            vec![
                upload(2, GlyphFormat::Alpha8, [0, 0], vec![255]),
                upload(2, GlyphFormat::Rgba8, [0, 0], vec![0, 255, 0, 255]),
            ],
        )],
    );
    assert_eq!(out, [0x00ff0000; 4]);
}

#[test]
fn out_of_range_page_is_rejected_and_preserves_cache_and_framebuffer() {
    let mut cpu = CpuRenderer::new(2, 1, 1.0).unwrap();
    let seed = frame(
        Color::TRANSPARENT,
        vec![text(
            vec![
                glyph(0.0, 2, GlyphFormat::Alpha8, [0, 0]),
                glyph(1.0, 2, GlyphFormat::Rgba8, [0, 0]),
            ],
            vec![
                upload(2, GlyphFormat::Alpha8, [0, 0], vec![255]),
                upload(2, GlyphFormat::Rgba8, [0, 0], vec![17, 34, 51, 255]),
            ],
        )],
    );
    cpu.render_scene(&seed).unwrap();
    let previous = [0x00ffffff, 0x00112233];
    assert_eq!(cpu.pixels(), &previous);
    let probe = frame(
        Color::TRANSPARENT,
        vec![text(
            vec![
                glyph(0.0, 2, GlyphFormat::Alpha8, [0, 0]),
                glyph(1.0, 2, GlyphFormat::Rgba8, [0, 0]),
            ],
            vec![],
        )],
    );

    for bad_page in [ATLAS_MAX_PAGES, ATLAS_MAX_PAGES + 1, u32::MAX] {
        for format in [GlyphFormat::Alpha8, GlyphFormat::Rgba8] {
            // The valid overwrite must not leak in: the whole scene is rejected.
            let rejected = frame(
                red(),
                vec![
                    text(
                        vec![],
                        vec![
                            upload(2, GlyphFormat::Alpha8, [0, 0], vec![0]),
                            upload(2, GlyphFormat::Rgba8, [0, 0], vec![0, 0, 0, 0]),
                        ],
                    ),
                    text(
                        vec![glyph(0.0, 2, GlyphFormat::Alpha8, [0, 0])],
                        vec![upload(bad_page, format, [0, 0], opaque_texel(format))],
                    ),
                ],
            );
            assert_eq!(
                cpu.render_scene(&rejected),
                Err(CpuRenderError::InvalidGlyphUpload),
                "page {bad_page} {format:?}"
            );
            assert_eq!(cpu.pixels(), &previous, "rejected scene changed the frame");
            cpu.render_scene(&probe).unwrap();
            assert_eq!(
                cpu.pixels(),
                &previous,
                "rejected scene changed cached pages"
            );
        }
    }
}

#[test]
fn same_slot_replacement_on_page_one_keeps_earlier_draws_in_painter_order() {
    let out = run(
        Color::TRANSPARENT,
        2,
        vec![
            text(
                vec![glyph(0.0, 1, GlyphFormat::Rgba8, [0, 0])],
                vec![upload(1, GlyphFormat::Rgba8, [0, 0], vec![255, 0, 0, 255])],
            ),
            text(
                vec![glyph(1.0, 1, GlyphFormat::Rgba8, [0, 0])],
                vec![upload(1, GlyphFormat::Rgba8, [0, 0], vec![0, 255, 0, 255])],
            ),
        ],
    );
    assert_eq!(out, [0x00ff0000, 0x0000ff00]);
}

#[test]
fn rejected_later_bad_upload_does_not_seed_any_page() {
    for format in [GlyphFormat::Alpha8, GlyphFormat::Rgba8] {
        let mut cpu = CpuRenderer::new(3, 1, 1.0).unwrap();
        let rejected = frame(
            red(),
            vec![
                text(
                    vec![],
                    vec![
                        upload(0, GlyphFormat::Alpha8, [0, 0], vec![255]),
                        upload(1, GlyphFormat::Rgba8, [0, 0], vec![0, 255, 0, 255]),
                        upload(2, GlyphFormat::Alpha8, [0, 0], vec![255]),
                    ],
                ),
                text(
                    vec![],
                    vec![upload(
                        ATLAS_MAX_PAGES,
                        format,
                        [0, 0],
                        opaque_texel(format),
                    )],
                ),
            ],
        );
        assert_eq!(
            cpu.render_scene(&rejected),
            Err(CpuRenderError::InvalidGlyphUpload)
        );
        let probe = frame(
            red(),
            vec![text(
                vec![
                    glyph(0.0, 0, GlyphFormat::Alpha8, [0, 0]),
                    glyph(1.0, 1, GlyphFormat::Rgba8, [0, 0]),
                    glyph(2.0, 2, GlyphFormat::Alpha8, [0, 0]),
                ],
                vec![],
            )],
        );
        cpu.render_scene(&probe).unwrap();
        assert_eq!(cpu.pixels(), &[0x00ff0000; 3], "{format:?}");
    }
}

#[test]
fn pages_one_through_seven_each_keep_their_own_texels() {
    let mut uploads = Vec::new();
    let mut glyphs = Vec::new();
    let mut expected = Vec::new();
    for page in 1..ATLAS_MAX_PAGES {
        let level = u8::try_from(page * 30).unwrap();
        uploads.push(upload(
            page,
            GlyphFormat::Rgba8,
            [0, 0],
            vec![level, 0, 0, 255],
        ));
        glyphs.push(glyph((page - 1) as f32, page, GlyphFormat::Rgba8, [0, 0]));
        expected.push(u32::from(level) << 16);
    }
    let out = run(
        Color::TRANSPARENT,
        ATLAS_MAX_PAGES - 1,
        vec![text(glyphs, uploads)],
    );
    assert_eq!(out, expected);
}

#[test]
fn page_edge_bounds_apply_to_every_accepted_page() {
    let last_page = ATLAS_MAX_PAGES - 1;
    let edge = ATLAS_PAGE_EDGE - 1;
    let out = run(
        Color::TRANSPARENT,
        1,
        vec![text(
            vec![glyph(0.0, last_page, GlyphFormat::Rgba8, [edge, edge])],
            vec![upload(
                last_page,
                GlyphFormat::Rgba8,
                [edge, edge],
                vec![0, 255, 0, 255],
            )],
        )],
    );
    assert_eq!(out, [0x0000ff00]);

    for origin in [[ATLAS_PAGE_EDGE, 0], [0, ATLAS_PAGE_EDGE]] {
        let mut cpu = CpuRenderer::new(1, 1, 1.0).unwrap();
        let scene = frame(
            Color::TRANSPARENT,
            vec![text(
                vec![],
                vec![upload(last_page, GlyphFormat::Alpha8, origin, vec![255])],
            )],
        );
        assert_eq!(
            cpu.render_scene(&scene),
            Err(CpuRenderError::InvalidGlyphUpload),
            "origin {origin:?}"
        );
    }
}
