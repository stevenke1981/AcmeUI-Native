//! Sharp quads use exact clip intersections; other paths use an area-mask product.

use acme_core::{
    AtlasUpload, Color, DrawCommand, GlyphDraw, GlyphFormat, ImageFit, ImageKey, ImagePrimitive,
    LayerParams, Logical, Point, QuadPrimitive, Rect, Rgba8Image, Scene, TextPrimitive,
};
use acme_render_cpu::CpuRenderer;

#[derive(Clone, Copy, Debug)]
enum Content {
    Quad,
    Rounded,
    Image,
    Alpha8,
    Rgba8,
}
const CONTENTS: [Content; 5] = [
    Content::Quad,
    Content::Rounded,
    Content::Image,
    Content::Alpha8,
    Content::Rgba8,
];
const SCALES: [f32; 5] = [1.0, 1.25, 1.5, 1.75, 2.0];

fn assert_sharp_self_clip(border_width: f32) {
    let rect = Rect::new(0.375, 0.375, 5.25, 5.25);
    for scale in SCALES {
        assert_ne!((rect.origin.x.get() * scale).fract(), 0.0);
        for alpha in [128_u8, 255] {
            let quad = QuadPrimitive {
                rect,
                color: Color::rgba(1.0, 0.0, 0.0, f32::from(alpha) / 255.0),
                radius: 0.0,
                border_width,
                border_color: Color::rgba(0.0, 1.0, 0.0, 0.6),
            };
            let uncut = render([DrawCommand::Quad(quad)], scale);
            assert!(
                uncut
                    .iter()
                    .any(|pixel| *pixel != 0 && *pixel != 0x00ff_0000)
            );
            let clipped = render(
                [
                    DrawCommand::PushClip(rect),
                    DrawCommand::Quad(quad),
                    DrawCommand::PopClip,
                ],
                scale,
            );
            assert_eq!(
                clipped, uncut,
                "{scale}x alpha {alpha} border {border_width}"
            );
        }
    }
}

#[test]
fn sharp_coincident_outer_clip_preserves_edges_at_every_dpi() {
    assert_sharp_self_clip(0.0);
}

#[test]
fn sharp_border_coincident_outer_clip_preserves_edges_at_every_dpi() {
    assert_sharp_self_clip(0.5);
}

#[test]
fn sharp_border_clip_at_or_inside_inner_has_only_exact_fill_at_every_dpi() {
    let quad = QuadPrimitive {
        rect: Rect::new(0.375, 0.375, 5.25, 5.25),
        color: Color::rgba(1.0, 0.0, 0.0, 0.6),
        radius: 0.0,
        border_width: 1.0,
        border_color: Color::rgba(0.0, 1.0, 0.0, 1.0),
    };
    for scale in SCALES {
        for clip in [
            Rect::new(1.375, 1.375, 3.25, 3.25),
            Rect::new(2.375, 2.375, 1.25, 1.25),
        ] {
            let clipped = render(
                [
                    DrawCommand::PushClip(clip),
                    DrawCommand::Quad(quad),
                    DrawCommand::PopClip,
                ],
                scale,
            );
            let expected = render(
                [DrawCommand::Quad(QuadPrimitive {
                    rect: clip,
                    border_width: 0.0,
                    ..quad
                })],
                scale,
            );
            assert_eq!(clipped, expected, "{scale}x clip {clip:?}");
        }
    }
}

#[test]
fn sharp_orthogonal_edges_keep_axis_overlap_product_at_every_dpi() {
    let rect = Rect::new(0.375, -2.0, 5.25, 20.0);
    let clip = Rect::new(-2.0, 0.625, 20.0, 4.75);
    for scale in SCALES {
        let pixels = render(
            [
                DrawCommand::PushClip(clip),
                DrawCommand::Quad(QuadPrimitive {
                    rect,
                    color: Color::rgba(1.0, 0.0, 0.0, 1.0),
                    radius: 0.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                }),
                DrawCommand::PopClip,
            ],
            scale,
        );
        for y in 0..16 {
            for x in 0..16 {
                let expected = ((255.0 * area(rect, scale, x, y) * area(clip, scale, x, y)).round()
                    as u32)
                    << 16;
                assert_eq!(
                    pixels[(y * 16 + x) as usize],
                    expected,
                    "{scale}x ({x},{y})"
                );
            }
        }
    }
}

fn content(kind: Content, alpha: u8) -> DrawCommand {
    match kind {
        Content::Quad | Content::Rounded => DrawCommand::Quad(QuadPrimitive {
            rect: Rect::new(0.0, 0.0, 16.0, 16.0),
            color: Color::rgba(1.0, 0.0, 0.0, f32::from(alpha) / 255.0),
            radius: if matches!(kind, Content::Rounded) {
                5.0
            } else {
                0.0
            },
            border_width: if matches!(kind, Content::Rounded) {
                1.5
            } else {
                0.0
            },
            border_color: Color::rgba(0.0, 1.0, 0.0, 0.6),
        }),
        Content::Image => DrawCommand::Image(ImagePrimitive {
            image: Rgba8Image::new(ImageKey::new(1), 0, 1, 1, vec![255, 0, 0, alpha]).unwrap(),
            rect: Rect::new(0.0, 0.0, 16.0, 16.0),
            fit: ImageFit::Fill,
        }),
        Content::Alpha8 | Content::Rgba8 => {
            let format = if matches!(kind, Content::Alpha8) {
                GlyphFormat::Alpha8
            } else {
                GlyphFormat::Rgba8
            };
            let pixels = if format == GlyphFormat::Alpha8 {
                vec![alpha; 16 * 16]
            } else {
                [255, 0, 0, alpha].repeat(16 * 16)
            };
            DrawCommand::Text(TextPrimitive {
                origin: Point::new(0.0, 0.0),
                color: if format == GlyphFormat::Alpha8 {
                    Color::rgba(1.0, 0.0, 0.0, 1.0)
                } else {
                    Color::rgba(1.0, 1.0, 1.0, 1.0)
                },
                glyphs: vec![GlyphDraw {
                    x: 0.0,
                    y: 0.0,
                    width: 16.0,
                    height: 16.0,
                    atlas_x: 0,
                    atlas_y: 0,
                    atlas_page: 0,
                    format,
                }],
                uploads: vec![AtlasUpload {
                    page: 0,
                    origin: [0, 0],
                    size: [16, 16],
                    format,
                    pixels,
                }],
            })
        }
    }
}

fn render(commands: impl IntoIterator<Item = DrawCommand>, scale: f32) -> Vec<u32> {
    let mut scene = Scene::with_clear(Color::TRANSPARENT);
    for command in commands {
        scene.push(command);
    }
    let mut cpu = CpuRenderer::new(16, 16, scale).unwrap();
    cpu.render_scene(&scene).unwrap();
    cpu.pixels().to_vec()
}

fn area(rect: Rect<Logical>, scale: f32, x: u32, y: u32) -> f32 {
    let left = rect.origin.x.get() * scale;
    let top = rect.origin.y.get() * scale;
    let right = left + rect.size.width.get() * scale;
    let bottom = top + rect.size.height.get() * scale;
    let x = x as f32;
    let y = y as f32;
    (right.min(x + 1.0) - left.max(x)).clamp(0.0, 1.0)
        * (bottom.min(y + 1.0) - top.max(y)).clamp(0.0, 1.0)
}

fn assert_fractional_clip(kind: Content) {
    let clip = Rect::new(1.375, 1.375, 2.125, 2.125);
    for scale in SCALES {
        for alpha in [128, 255] {
            let pixels = render(
                [
                    DrawCommand::PushClip(clip),
                    content(kind, alpha),
                    DrawCommand::PopClip,
                ],
                scale,
            );
            for y in 0..16 {
                for x in 0..16 {
                    let expected =
                        ((f32::from(alpha) * area(clip, scale, x, y)).round() as u32) << 16;
                    assert_eq!(
                        pixels[(y * 16 + x) as usize],
                        expected,
                        "{kind:?} {scale}x alpha {alpha} ({x},{y})"
                    );
                }
            }
        }
    }
}

#[test]
fn fractional_quad_clip_at_every_dpi() {
    assert_fractional_clip(Content::Quad);
}
#[test]
fn fractional_image_clip_at_every_dpi() {
    assert_fractional_clip(Content::Image);
}
#[test]
fn fractional_alpha8_glyph_clip_at_every_dpi() {
    assert_fractional_clip(Content::Alpha8);
}
#[test]
fn fractional_rgba8_glyph_clip_at_every_dpi() {
    assert_fractional_clip(Content::Rgba8);
}

#[test]
fn subpixel_clip_without_any_inside_centre_still_paints_its_area() {
    let clip = Rect::new(0.125, 0.125, 0.25, 0.25);
    let pixels = render(
        [
            DrawCommand::PushClip(clip),
            content(Content::Quad, 255),
            DrawCommand::PopClip,
        ],
        1.0,
    );
    assert_eq!(pixels[0], 0x0010_0000); // round(255 / 16)
    assert!(pixels[1..].iter().all(|pixel| *pixel == 0));
}

#[test]
fn scrolling_sharp_shape_uses_exact_viewport_intersection() {
    let clip = Rect::new(1.25, 1.5, 3.5, 3.25);
    for scale in [1.25, 1.5] {
        for scroll in [0.0, 0.125, 0.5, 0.875] {
            let rect = Rect::new(0.75, 1.25 - scroll, 4.5, 3.5);
            let pixels = render(
                [
                    DrawCommand::PushClip(clip),
                    DrawCommand::Quad(QuadPrimitive {
                        rect,
                        color: Color::rgba(1.0, 0.0, 0.0, 1.0),
                        radius: 0.0,
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    }),
                    DrawCommand::PopClip,
                ],
                scale,
            );
            for y in 0..16 {
                for x in 0..16 {
                    // x=[1.25,4.75), y=[1.5,4.75-scroll): hand-derived
                    // continuous intersection of this moving rect and viewport.
                    let intersection = Rect::new(1.25, 1.5, 3.5, 3.25 - scroll);
                    let expected = ((255.0 * area(intersection, scale, x, y)).round() as u32) << 16;
                    assert_eq!(
                        pixels[(y * 16 + x) as usize],
                        expected,
                        "{scale}x scroll {scroll} ({x},{y})"
                    );
                }
            }
        }
    }
}

#[test]
fn sharp_border_inner_coincident_clip_does_not_add_a_stroke_seam() {
    let pixels = render(
        [
            DrawCommand::PushClip(Rect::new(0.5, 0.5, 3.0, 3.0)),
            DrawCommand::Quad(QuadPrimitive {
                rect: Rect::new(0.0, 0.0, 4.0, 4.0),
                color: Color::rgba(1.0, 0.0, 0.0, 1.0),
                radius: 0.0,
                border_width: 0.5,
                border_color: Color::rgba(0.0, 1.0, 0.0, 1.0),
            }),
            DrawCommand::PopClip,
        ],
        1.0,
    );
    // Clip equals the inner rect: exact clipped outer and inner areas agree,
    // leaving half-red fill at (0,1), quarter-red at (0,0), and no green stroke.
    assert_eq!(pixels[16], 0x0080_0000);
    assert_eq!(pixels[0], 0x0040_0000);
    assert_eq!(pixels[17], 0x00ff_0000);
}

#[test]
fn rounded_shape_area_and_clip_area_are_multiplied() {
    let pixels = render(
        [
            DrawCommand::PushClip(Rect::new(0.5, 0.0, 1.5, 2.0)),
            DrawCommand::Quad(QuadPrimitive {
                rect: Rect::new(0.0, 0.0, 2.0, 2.0),
                color: Color::rgba(1.0, 0.0, 0.0, 1.0),
                radius: 1.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            }),
            DrawCommand::PopClip,
        ],
        1.0,
    );
    assert_eq!(
        pixels[0],
        ((255.0 * std::f32::consts::PI / 8.0).round() as u32) << 16
    );
}

#[test]
fn repeated_nested_clip_is_not_applied_twice_and_pop_restores_parent() {
    let parent = Rect::new(0.5, 0.5, 4.0, 4.0);
    let child = Rect::new(1.25, 1.25, 2.5, 2.5);
    for kind in [
        Content::Quad,
        Content::Image,
        Content::Alpha8,
        Content::Rgba8,
    ] {
        let once = render(
            [
                DrawCommand::PushClip(child),
                content(kind, 128),
                DrawCommand::PopClip,
            ],
            1.0,
        );
        let twice = render(
            [
                DrawCommand::PushClip(parent),
                DrawCommand::PushClip(child),
                DrawCommand::PushClip(child),
                content(kind, 128),
                DrawCommand::PopClip,
                DrawCommand::PopClip,
                DrawCommand::PopClip,
            ],
            1.0,
        );
        assert_eq!(once, twice, "{kind:?}");
        let popped = render(
            [
                DrawCommand::PushClip(parent),
                DrawCommand::PushClip(child),
                DrawCommand::PopClip,
                content(kind, 128),
                DrawCommand::PopClip,
            ],
            1.0,
        );
        let expected = render(
            [
                DrawCommand::PushClip(parent),
                content(kind, 128),
                DrawCommand::PopClip,
            ],
            1.0,
        );
        assert_eq!(popped, expected, "{kind:?}");
    }
}

#[test]
fn disjoint_subpixel_and_zero_area_clips_remain_empty() {
    for clip in [
        Rect::new(0.2, 0.2, 0.0, 0.5),
        Rect::new(0.3, 0.3, -0.1, 0.1),
        Rect::new(1.5, 1.5, 0.25, 0.25),
    ] {
        let pixels = render(
            [
                DrawCommand::PushClip(Rect::new(0.1, 0.1, 0.25, 0.25)),
                DrawCommand::PushClip(clip),
                content(Content::Quad, 255),
                DrawCommand::PopClip,
                DrawCommand::PopClip,
            ],
            1.0,
        );
        assert!(pixels.iter().all(|pixel| *pixel == 0), "{clip:?}");
    }
}

#[test]
fn sharp_disjoint_parts_of_one_pixel_have_zero_joint_area() {
    let pixels = render(
        [
            DrawCommand::PushClip(Rect::new(0.75, 0.0, 0.25, 1.0)),
            DrawCommand::Quad(QuadPrimitive {
                rect: Rect::new(0.0, 0.0, 0.25, 1.0),
                color: Color::rgba(1.0, 0.0, 0.0, 1.0),
                radius: 0.0,
                border_width: 0.0,
                border_color: Color::TRANSPARENT,
            }),
            DrawCommand::PopClip,
        ],
        1.0,
    );
    // Sharp coverage uses the empty geometric intersection, not 1/4 * 1/4.
    assert!(pixels.iter().all(|pixel| *pixel == 0));
}

#[test]
fn integer_clips_are_byte_identical_to_legacy_centre_mask() {
    for scale in SCALES {
        for kind in CONTENTS {
            let draw = content(kind, 153);
            let uncut = render(
                [
                    DrawCommand::BeginLayer(LayerParams { opacity: 0.75 }),
                    draw.clone(),
                    DrawCommand::EndLayer,
                ],
                scale,
            );
            let clipped = render(
                [
                    DrawCommand::PushClip(Rect::new(4.0, 4.0, 4.0, 4.0)),
                    DrawCommand::BeginLayer(LayerParams { opacity: 0.75 }),
                    draw,
                    DrawCommand::EndLayer,
                    DrawCommand::PopClip,
                ],
                scale,
            );
            let (lo, hi) = ((4.0 * scale) as u32, (8.0 * scale) as u32);
            let mut expected = uncut;
            for y in 0..16 {
                for x in 0..16 {
                    if x < lo || x >= hi || y < lo || y >= hi {
                        expected[(y * 16 + x) as usize] = 0;
                    }
                }
            }
            assert_eq!(clipped, expected, "{kind:?} {scale}x");
            if let Some(dir) = std::env::var_os("ACME_CLIP_CAPTURE") {
                let dir = std::path::PathBuf::from(dir);
                std::fs::create_dir_all(&dir).unwrap();
                let bytes: Vec<u8> = clipped.iter().flat_map(|p| p.to_le_bytes()).collect();
                std::fs::write(dir.join(format!("integer-{kind:?}-{scale}.bin")), bytes).unwrap();
            }
        }
    }
}
