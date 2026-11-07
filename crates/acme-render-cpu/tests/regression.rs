use acme_core::{
    AtlasUpload, Color, DrawCommand, GlyphDraw, GlyphFormat, LayerParams, Point, Scene,
    TextPrimitive,
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
