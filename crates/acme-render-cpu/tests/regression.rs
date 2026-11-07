use acme_core::{
    ATLAS_MAX_PAGES, AtlasUpload, Color, DrawCommand, GlyphDraw, GlyphFormat, LayerParams, Point,
    QuadPrimitive, Rect, Scene, TextPrimitive,
};
use acme_render_cpu::{CpuRenderError, CpuRenderer};

fn text_scene(glyph: GlyphDraw, uploads: Vec<AtlasUpload>) -> Scene {
    let mut scene = Scene::new();
    scene.push(DrawCommand::Text(TextPrimitive {
        origin: Point::new(0.0, 0.0),
        color: Color::rgba(1.0, 1.0, 1.0, 1.0),
        glyphs: vec![glyph],
        uploads,
    }));
    scene
}

fn glyph() -> GlyphDraw {
    GlyphDraw {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 1.0,
        atlas_x: 0,
        atlas_y: 0,
        atlas_page: 0,
        format: GlyphFormat::Alpha8,
    }
}

#[test]
fn extreme_offscreen_glyph_does_not_overflow_atlas_coordinates() {
    let mut draw = glyph();
    draw.x = -1.0e30;
    draw.width = 1.0e31;
    draw.atlas_x = 1;
    let mut cpu = CpuRenderer::new(2, 1, 1.0).unwrap();
    cpu.render_scene(&text_scene(draw, vec![])).unwrap();
    assert_eq!(cpu.pixels(), &[0, 0]);
}

#[test]
fn failed_resize_preserves_dimensions_and_last_output() {
    let mut cpu = CpuRenderer::new(2, 1, 1.0).unwrap();
    let scene = Scene::with_clear(Color::rgba(1.0, 0.0, 0.0, 1.0));
    cpu.render_scene(&scene).unwrap();
    for scale in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        assert_eq!(cpu.resize(3, 4, scale), Err(CpuRenderError::InvalidScale));
        assert_eq!(cpu.dimensions(), (2, 1));
        assert_eq!(cpu.pixels(), &[0x00ff0000, 0x00ff0000]);
    }
}

#[test]
fn rejected_scene_does_not_publish_a_partial_frame() {
    let mut cpu = CpuRenderer::new(1, 1, 1.0).unwrap();
    cpu.render_scene(&Scene::with_clear(Color::rgba(0.0, 1.0, 0.0, 1.0)))
        .unwrap();
    let mut invalid = Scene::new();
    invalid.push(DrawCommand::EndLayer);
    assert!(matches!(
        cpu.render_scene(&invalid),
        Err(CpuRenderError::InvalidScene(_))
    ));
    assert_eq!(cpu.pixels(), &[0x0000ff00]);
}

#[test]
fn malformed_atlas_upload_is_rejected() {
    let upload = AtlasUpload {
        page: 0,
        origin: [0, 0],
        size: [2, 1],
        format: GlyphFormat::Alpha8,
        pixels: vec![255],
    };
    let mut cpu = CpuRenderer::new(1, 1, 1.0).unwrap();
    assert_eq!(
        cpu.render_scene(&text_scene(glyph(), vec![upload])),
        Err(CpuRenderError::InvalidGlyphUpload)
    );
}

#[test]
fn nested_layer_limit_applies_even_to_empty_frames() {
    let mut cpu = CpuRenderer::new(0, 0, 1.0).unwrap();
    let mut scene = Scene::new();
    for _ in 0..17 {
        scene.push(DrawCommand::BeginLayer(LayerParams { opacity: 1.0 }));
    }
    assert!(matches!(
        cpu.render_scene(&scene),
        Err(CpuRenderError::InvalidScene(_))
    ));
}

fn red_quad() -> DrawCommand {
    DrawCommand::Quad(QuadPrimitive {
        rect: Rect::new(0.0, 0.0, 4.0, 4.0),
        color: Color::rgba(1.0, 0.0, 0.0, 1.0),
        radius: 0.0,
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    })
}

#[test]
fn minimized_frame_accepts_draws_without_pixels() {
    let mut cpu = CpuRenderer::new(0, u32::MAX, 1.0).unwrap();
    let mut scene = Scene::new();
    scene.push(DrawCommand::Quad(QuadPrimitive {
        rect: Rect::new(0.0, 0.0, 4.0, f32::MAX),
        color: Color::rgba(1.0, 0.0, 0.0, 1.0),
        radius: 0.0,
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    }));
    cpu.render_scene(&scene).unwrap();
    assert!(cpu.pixels().is_empty());
}

#[test]
fn fractional_clip_preserves_pixel_area() {
    let mut cpu = CpuRenderer::new(4, 4, 1.0).unwrap();
    let mut scene = Scene::new();
    scene.push(DrawCommand::PushClip(Rect::new(1.6, 1.6, 1.0, 1.0)));
    scene.push(red_quad());
    scene.push(DrawCommand::PopClip);
    cpu.render_scene(&scene).unwrap();
    let mut expected = vec![0; 16];
    expected[5] = 0x0029_0000;
    expected[6] = 0x003d_0000;
    expected[9] = 0x003d_0000;
    expected[10] = 0x005c_0000;
    assert_eq!(cpu.pixels(), expected);
}

#[test]
fn disjoint_nested_clip_is_empty_and_pop_restores_parent() {
    let mut cpu = CpuRenderer::new(4, 4, 1.0).unwrap();
    let mut scene = Scene::new();
    scene.push(DrawCommand::PushClip(Rect::new(0.0, 0.0, 1.0, 1.0)));
    scene.push(DrawCommand::PushClip(Rect::new(2.0, 2.0, 1.0, 1.0)));
    scene.push(red_quad());
    scene.push(DrawCommand::PopClip);
    scene.push(red_quad());
    scene.push(DrawCommand::PopClip);
    cpu.render_scene(&scene).unwrap();
    let mut expected = vec![0; 16];
    expected[0] = 0x00ff0000;
    assert_eq!(cpu.pixels(), expected);
}

#[test]
fn geometry_overflow_after_scaling_is_ignored() {
    let mut cpu = CpuRenderer::new(2, 2, f32::MAX).unwrap();
    let mut scene = Scene::new();
    scene.push(red_quad());
    cpu.render_scene(&scene).unwrap();
    assert_eq!(cpu.pixels(), &[0; 4]);
}

#[test]
fn rgba_atlas_updates_replace_previous_pixels() {
    let mut cpu = CpuRenderer::new(1, 1, 1.0).unwrap();
    let mut draw = glyph();
    draw.format = GlyphFormat::Rgba8;
    for (pixels, expected) in [
        (vec![255, 0, 0, 255], 0x00ff0000),
        (vec![0, 0, 255, 255], 0x000000ff),
    ] {
        let upload = AtlasUpload {
            page: 0,
            origin: [0, 0],
            size: [1, 1],
            format: GlyphFormat::Rgba8,
            pixels,
        };
        cpu.render_scene(&text_scene(draw, vec![upload])).unwrap();
        assert_eq!(cpu.pixels(), &[expected]);
    }
}

fn atlas_pair_uploads(alpha: u8, rgba: [u8; 4]) -> Vec<AtlasUpload> {
    [
        (GlyphFormat::Alpha8, vec![alpha]),
        (GlyphFormat::Rgba8, rgba.to_vec()),
    ]
    .into_iter()
    .map(|(format, pixels)| AtlasUpload {
        page: 0,
        origin: [0, 0],
        size: [1, 1],
        format,
        pixels,
    })
    .collect()
}

fn atlas_pair_scene(uploads: Vec<AtlasUpload>) -> Scene {
    let mut scene = Scene::new();
    scene.push(DrawCommand::Text(TextPrimitive {
        origin: Point::new(0.0, 0.0),
        color: Color::rgba(1.0, 1.0, 1.0, 1.0),
        glyphs: vec![
            glyph(),
            GlyphDraw {
                x: 1.0,
                format: GlyphFormat::Rgba8,
                ..glyph()
            },
        ],
        uploads,
    }));
    scene
}

fn assert_rejection_preserves_atlas_pair(scene: &Scene, error: CpuRenderError) {
    let mut cpu = CpuRenderer::new(2, 1, 1.0).unwrap();
    cpu.render_scene(&atlas_pair_scene(atlas_pair_uploads(
        255,
        [17, 34, 51, 255],
    )))
    .unwrap();
    let previous = [0x00ffffff, 0x00112233];
    assert_eq!(cpu.pixels(), &previous);

    assert_eq!(cpu.render_scene(scene), Err(error));
    assert_eq!(cpu.pixels(), &previous, "rejected frame changed output");
    cpu.render_scene(&atlas_pair_scene(vec![])).unwrap();
    assert_eq!(
        cpu.pixels(),
        &previous,
        "rejected frame changed cached glyphs"
    );

    // A rejected update must not prevent a later successful replacement.
    cpu.render_scene(&atlas_pair_scene(atlas_pair_uploads(
        128,
        [68, 85, 102, 255],
    )))
    .unwrap();
    let replacement = [0x00808080, 0x00445566];
    assert_eq!(cpu.pixels(), &replacement);
    cpu.render_scene(&atlas_pair_scene(vec![])).unwrap();
    assert_eq!(cpu.pixels(), &replacement);
}

#[test]
fn rejected_scene_structure_preserves_both_glyph_atlases() {
    let cases = [
        (vec![DrawCommand::PopClip], "unmatched PopClip"),
        (vec![DrawCommand::EndLayer], "unmatched EndLayer"),
        (
            vec![DrawCommand::PushClip(Rect::new(0.0, 0.0, 2.0, 1.0))],
            "unclosed clip or layer",
        ),
        (
            vec![DrawCommand::BeginLayer(LayerParams { opacity: 1.0 })],
            "unclosed clip or layer",
        ),
        (
            vec![DrawCommand::BeginLayer(LayerParams { opacity: 1.0 }); 17],
            "layer allocation budget exceeded",
        ),
    ];
    for (commands, reason) in cases {
        let mut rejected = atlas_pair_scene(atlas_pair_uploads(0, [255, 0, 0, 255]));
        for command in commands {
            rejected.push(command);
        }
        assert_rejection_preserves_atlas_pair(&rejected, CpuRenderError::InvalidScene(reason));
    }
}

fn bad_atlas_uploads() -> Vec<AtlasUpload> {
    let mut bad = Vec::new();
    for valid in atlas_pair_uploads(255, [255, 255, 255, 255]) {
        bad.push(AtlasUpload {
            page: ATLAS_MAX_PAGES,
            ..valid.clone()
        });
        bad.push(AtlasUpload {
            origin: [2048, 0],
            ..valid.clone()
        });
        bad.push(AtlasUpload {
            origin: [0, 2048],
            ..valid.clone()
        });
        bad.push(AtlasUpload {
            size: [2, 1],
            ..valid.clone()
        });
        bad.push(AtlasUpload {
            size: [u32::MAX, u32::MAX],
            ..valid
        });
    }
    bad
}

#[test]
fn later_bad_upload_in_same_text_preserves_both_glyph_atlases() {
    for bad in bad_atlas_uploads() {
        let mut uploads = atlas_pair_uploads(0, [255, 0, 0, 255]);
        uploads.push(bad);
        assert_rejection_preserves_atlas_pair(
            &atlas_pair_scene(uploads),
            CpuRenderError::InvalidGlyphUpload,
        );
    }
}

#[test]
fn later_bad_text_primitive_preserves_both_glyph_atlases() {
    for bad in bad_atlas_uploads() {
        let mut rejected = atlas_pair_scene(atlas_pair_uploads(0, [255, 0, 0, 255]));
        rejected.push(DrawCommand::Text(TextPrimitive {
            origin: Point::new(0.0, 0.0),
            color: Color::TRANSPARENT,
            glyphs: vec![],
            uploads: vec![bad],
        }));
        assert_rejection_preserves_atlas_pair(&rejected, CpuRenderError::InvalidGlyphUpload);
    }
}

#[test]
fn rejected_scene_does_not_seed_either_glyph_atlas() {
    let mut cpu = CpuRenderer::new(2, 1, 1.0).unwrap();
    cpu.render_scene(&Scene::with_clear(Color::rgba(0.0, 1.0, 0.0, 1.0)))
        .unwrap();
    let mut rejected = atlas_pair_scene(atlas_pair_uploads(255, [255, 0, 0, 255]));
    rejected.push(DrawCommand::EndLayer);
    assert_eq!(
        cpu.render_scene(&rejected),
        Err(CpuRenderError::InvalidScene("unmatched EndLayer"))
    );
    assert_eq!(cpu.pixels(), &[0x0000ff00; 2]);
    cpu.render_scene(&atlas_pair_scene(vec![])).unwrap();
    assert_eq!(cpu.pixels(), &[0; 2]);
}

#[test]
fn accepted_uploads_keep_text_command_order_and_cached_replacements() {
    for (format, first, second, expected) in [
        (
            GlyphFormat::Alpha8,
            vec![255],
            vec![128],
            [0x00ffffff, 0x00808080, 0x00808080],
        ),
        (
            GlyphFormat::Rgba8,
            vec![255, 0, 0, 255],
            vec![0, 0, 255, 255],
            [0x00ff0000, 0x000000ff, 0x000000ff],
        ),
    ] {
        let mut scene = Scene::new();
        scene.push(DrawCommand::PushClip(Rect::new(0.0, 0.0, 3.0, 1.0)));
        scene.push(DrawCommand::BeginLayer(LayerParams { opacity: 1.0 }));
        for (x, pixels) in [Some(first), Some(second), None].into_iter().enumerate() {
            scene.push(DrawCommand::Text(TextPrimitive {
                origin: Point::new(0.0, 0.0),
                color: Color::rgba(1.0, 1.0, 1.0, 1.0),
                glyphs: vec![GlyphDraw {
                    x: x as f32,
                    format,
                    ..glyph()
                }],
                uploads: pixels
                    .into_iter()
                    .map(|pixels| AtlasUpload {
                        page: 0,
                        origin: [0, 0],
                        size: [1, 1],
                        format,
                        pixels,
                    })
                    .collect(),
            }));
        }
        scene.push(DrawCommand::EndLayer);
        scene.push(DrawCommand::PopClip);
        let mut cpu = CpuRenderer::new(3, 1, 1.0).unwrap();
        cpu.render_scene(&scene).unwrap();
        assert_eq!(cpu.pixels(), &expected);
        cpu.render_scene(&text_scene(GlyphDraw { format, ..glyph() }, vec![]))
            .unwrap();
        assert_eq!(cpu.pixels(), &[expected[2], 0, 0]);
    }
}
