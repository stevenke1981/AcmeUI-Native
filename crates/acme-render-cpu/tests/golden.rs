//! Deterministic golden pixel regression for the CPU renderer.
//!
//! Every scene here is text-free (font availability differs per OS), rendered at
//! scale 1.0 and compared with a committed binary P6 PPM fixture at
//! `tests/golden/<name>.ppm`. Each scene also asserts a few hand-derived sample
//! pixels on every run, so a fixture can never silently bless a blank or broken frame.
//!
//! Regenerate fixtures (only after an intentional rendering change):
//!
//! ```text
//! ACME_BLESS_GOLDEN=1 cargo test --locked -p acme-render-cpu --test golden
//! ```
//!
//! On mismatch the rendered frame is written to `target/golden-actual/<name>.ppm`.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use acme_core::{
    Color, DrawCommand, ImageFit, ImageKey, ImagePrimitive, LayerParams, QuadPrimitive, Rect,
    Rgba8Image, Scene,
};
use acme_render_cpu::CpuRenderer;

/// Maximum per-channel difference (0..=255) that still counts as equal. Must stay <= 2.
const CHANNEL_TOLERANCE: u8 = 2;
/// Maximum number of pixels allowed to exceed `CHANNEL_TOLERANCE` before a scene fails.
const MAX_DIFFERING_PIXELS: usize = 0;
/// Setting this environment variable to `1` writes the fixtures instead of comparing.
const BLESS_ENV: &str = "ACME_BLESS_GOLDEN";
/// Command shown to whoever needs to create or refresh a fixture.
const BLESS_COMMAND: &str =
    "ACME_BLESS_GOLDEN=1 cargo test --locked -p acme-render-cpu --test golden";
/// Largest edge allowed for a committed fixture.
const MAX_FIXTURE_EDGE: u32 = 128;

const _: () = assert!(CHANNEL_TOLERANCE <= 2, "golden tolerance must stay <= 2");

/// A tightly packed RGB8 frame (row-major, 3 bytes per pixel).
#[derive(Clone, Debug, PartialEq, Eq)]
struct RgbImage {
    width: u32,
    height: u32,
    rgb: Vec<u8>,
}

impl RgbImage {
    fn filled(width: u32, height: u32, rgb: [u8; 3]) -> Self {
        let count = width as usize * height as usize;
        Self {
            width,
            height,
            rgb: rgb.repeat(count),
        }
    }

    /// Converts packed `0x00RRGGBB` pixels, as returned by `CpuRenderer::pixels`.
    fn from_packed(width: u32, height: u32, packed: &[u32]) -> Self {
        let rgb = packed
            .iter()
            .flat_map(|p| [(p >> 16) as u8, (p >> 8) as u8, *p as u8])
            .collect();
        Self { width, height, rgb }
    }

    fn pixel(&self, x: u32, y: u32) -> [u8; 3] {
        let at = (y as usize * self.width as usize + x as usize) * 3;
        [self.rgb[at], self.rgb[at + 1], self.rgb[at + 2]]
    }

    fn encode_ppm(&self) -> Vec<u8> {
        let mut out = format!("P6\n{} {}\n255\n", self.width, self.height).into_bytes();
        out.extend_from_slice(&self.rgb);
        out
    }

    fn decode_ppm(bytes: &[u8]) -> Result<Self, String> {
        let mut pos = 0;
        let magic = next_token(bytes, &mut pos)?;
        if magic != "P6" {
            return Err(format!("expected magic P6, found {magic:?}"));
        }
        let width = parse_header_field(next_token(bytes, &mut pos)?, "width")?;
        let height = parse_header_field(next_token(bytes, &mut pos)?, "height")?;
        let maxval = parse_header_field(next_token(bytes, &mut pos)?, "maxval")?;
        if maxval != 255 {
            return Err(format!("expected maxval 255, found {maxval}"));
        }
        // Exactly one whitespace byte separates the header from the raster.
        match bytes.get(pos) {
            Some(byte) if byte.is_ascii_whitespace() => pos += 1,
            _ => return Err("missing whitespace before raster data".to_owned()),
        }
        let expected = (width as usize)
            .checked_mul(height as usize)
            .and_then(|pixels| pixels.checked_mul(3))
            .ok_or_else(|| "raster size overflows".to_owned())?;
        let raster = &bytes[pos..];
        if raster.len() != expected {
            return Err(format!(
                "raster holds {} bytes, expected {expected} for {width}x{height}",
                raster.len()
            ));
        }
        Ok(Self {
            width,
            height,
            rgb: raster.to_vec(),
        })
    }
}

fn next_token<'a>(bytes: &'a [u8], pos: &mut usize) -> Result<&'a str, String> {
    loop {
        while *pos < bytes.len() && bytes[*pos].is_ascii_whitespace() {
            *pos += 1;
        }
        if bytes.get(*pos) != Some(&b'#') {
            break;
        }
        while *pos < bytes.len() && bytes[*pos] != b'\n' {
            *pos += 1;
        }
    }
    let start = *pos;
    while *pos < bytes.len() && !bytes[*pos].is_ascii_whitespace() {
        *pos += 1;
    }
    if start == *pos {
        return Err("unexpected end of PPM header".to_owned());
    }
    std::str::from_utf8(&bytes[start..*pos]).map_err(|error| format!("non-UTF-8 header: {error}"))
}

fn parse_header_field(token: &str, field: &str) -> Result<u32, String> {
    token
        .parse::<u32>()
        .map_err(|error| format!("invalid {field} {token:?}: {error}"))
}

/// A pixel that the scene must produce, independent of the stored fixture.
struct Sample {
    x: u32,
    y: u32,
    rgb: [u8; 3],
}

struct GoldenCase {
    name: &'static str,
    width: u32,
    height: u32,
    build: fn() -> Scene,
    samples: &'static [Sample],
}

const GOLDEN_CASES: &[GoldenCase] = &[
    GoldenCase {
        name: "solid_quads",
        width: 64,
        height: 64,
        build: scene_solid_quads,
        samples: &[
            Sample {
                x: 2,
                y: 2,
                rgb: [51, 51, 51],
            },
            Sample {
                x: 10,
                y: 10,
                rgb: [255, 0, 0],
            },
            Sample {
                x: 40,
                y: 12,
                rgb: [0, 204, 0],
            },
            Sample {
                x: 10,
                y: 50,
                rgb: [0, 0, 255],
            },
            Sample {
                x: 32,
                y: 32,
                rgb: [153, 0, 153],
            },
            Sample {
                x: 60,
                y: 60,
                rgb: [51, 51, 51],
            },
        ],
    },
    GoldenCase {
        name: "rounded_bordered",
        width: 64,
        height: 64,
        build: scene_rounded_bordered,
        samples: &[
            Sample {
                x: 4,
                y: 4,
                rgb: [0, 0, 0],
            },
            Sample {
                x: 32,
                y: 16,
                rgb: [0, 102, 204],
            },
            Sample {
                x: 4,
                y: 16,
                rgb: [255, 255, 255],
            },
            Sample {
                x: 32,
                y: 48,
                rgb: [102, 0, 102],
            },
            Sample {
                x: 32,
                y: 40,
                rgb: [0, 255, 0],
            },
        ],
    },
    GoldenCase {
        name: "nested_clips",
        width: 64,
        height: 64,
        build: scene_nested_clips,
        samples: &[
            Sample {
                x: 2,
                y: 2,
                rgb: [51, 51, 51],
            },
            Sample {
                x: 12,
                y: 12,
                rgb: [0, 0, 255],
            },
            Sample {
                x: 30,
                y: 30,
                rgb: [255, 0, 0],
            },
            Sample {
                x: 50,
                y: 50,
                rgb: [51, 51, 51],
            },
            Sample {
                x: 54,
                y: 54,
                rgb: [0, 255, 0],
            },
        ],
    },
    GoldenCase {
        name: "nested_layers",
        width: 64,
        height: 64,
        build: scene_nested_layers,
        samples: &[
            Sample {
                x: 2,
                y: 2,
                rgb: [255, 255, 255],
            },
            Sample {
                x: 10,
                y: 10,
                rgb: [255, 128, 128],
            },
            Sample {
                x: 30,
                y: 30,
                rgb: [192, 128, 192],
            },
        ],
    },
    GoldenCase {
        name: "overlapping_alpha",
        width: 64,
        height: 64,
        build: scene_overlapping_alpha,
        samples: &[
            Sample {
                x: 2,
                y: 2,
                rgb: [255, 255, 255],
            },
            Sample {
                x: 6,
                y: 6,
                rgb: [255, 127, 127],
            },
            Sample {
                x: 30,
                y: 30,
                rgb: [127, 63, 191],
            },
            Sample {
                x: 50,
                y: 50,
                rgb: [127, 127, 255],
            },
            Sample {
                x: 62,
                y: 62,
                rgb: [255, 255, 255],
            },
        ],
    },
    GoldenCase {
        name: "rgba_image",
        width: 64,
        height: 64,
        build: scene_rgba_image,
        samples: &[
            Sample {
                x: 2,
                y: 2,
                rgb: [0, 0, 0],
            },
            Sample {
                x: 12,
                y: 12,
                rgb: [255, 0, 0],
            },
            Sample {
                x: 36,
                y: 12,
                rgb: [0, 255, 0],
            },
            Sample {
                x: 12,
                y: 36,
                rgb: [0, 0, 255],
            },
            Sample {
                x: 36,
                y: 36,
                rgb: [128, 128, 128],
            },
        ],
    },
];

fn solid(rect: [f32; 4], color: Color) -> DrawCommand {
    DrawCommand::Quad(QuadPrimitive {
        rect: Rect::new(rect[0], rect[1], rect[2], rect[3]),
        color,
        radius: 0.0,
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    })
}

fn rounded(rect: [f32; 4], radius: f32, fill: Color, border: Option<(f32, Color)>) -> DrawCommand {
    let (border_width, border_color) = border.unwrap_or((0.0, Color::TRANSPARENT));
    DrawCommand::Quad(QuadPrimitive {
        rect: Rect::new(rect[0], rect[1], rect[2], rect[3]),
        color: fill,
        radius,
        border_width,
        border_color,
    })
}

fn scene_solid_quads() -> Scene {
    let mut scene = Scene::with_clear(Color::rgba(0.2, 0.2, 0.2, 1.0));
    scene.push(solid(
        [8.0, 8.0, 24.0, 16.0],
        Color::rgba(1.0, 0.0, 0.0, 1.0),
    ));
    scene.push(solid(
        [32.0, 8.0, 24.0, 16.0],
        Color::rgba(0.0, 0.8, 0.0, 1.0),
    ));
    scene.push(solid(
        [8.0, 32.0, 48.0, 24.0],
        Color::rgba(0.0, 0.0, 1.0, 1.0),
    ));
    // Painter order: this overlaps the three quads above and must win where they meet.
    scene.push(solid(
        [20.0, 20.0, 24.0, 24.0],
        Color::rgba(0.6, 0.0, 0.6, 1.0),
    ));
    scene
}

fn scene_rounded_bordered() -> Scene {
    let mut scene = Scene::with_clear(Color::rgba(0.0, 0.0, 0.0, 1.0));
    scene.push(rounded(
        [4.0, 4.0, 56.0, 24.0],
        8.0,
        Color::rgba(0.0, 0.4, 0.8, 1.0),
        Some((2.0, Color::rgba(1.0, 1.0, 1.0, 1.0))),
    ));
    scene.push(rounded(
        [8.0, 40.0, 48.0, 16.0],
        8.0,
        Color::rgba(0.4, 0.0, 0.4, 1.0),
        Some((1.0, Color::rgba(0.0, 1.0, 0.0, 1.0))),
    ));
    scene
}

fn scene_nested_clips() -> Scene {
    let mut scene = Scene::with_clear(Color::rgba(0.2, 0.2, 0.2, 1.0));
    scene.push(DrawCommand::PushClip(Rect::new(8.0, 8.0, 40.0, 40.0)));
    scene.push(solid(
        [0.0, 0.0, 64.0, 64.0],
        Color::rgba(0.0, 0.0, 1.0, 1.0),
    ));
    scene.push(DrawCommand::PushClip(Rect::new(20.0, 20.0, 40.0, 40.0)));
    scene.push(solid(
        [0.0, 0.0, 64.0, 64.0],
        Color::rgba(1.0, 0.0, 0.0, 1.0),
    ));
    scene.push(DrawCommand::PopClip);
    scene.push(DrawCommand::PopClip);
    // Drawn after the clips are popped, so it is not clipped at all.
    scene.push(solid(
        [52.0, 52.0, 8.0, 8.0],
        Color::rgba(0.0, 1.0, 0.0, 1.0),
    ));
    scene
}

fn scene_nested_layers() -> Scene {
    let mut scene = Scene::with_clear(Color::rgba(1.0, 1.0, 1.0, 1.0));
    scene.push(DrawCommand::BeginLayer(LayerParams { opacity: 0.5 }));
    scene.push(solid(
        [8.0, 8.0, 48.0, 48.0],
        Color::rgba(1.0, 0.0, 0.0, 1.0),
    ));
    scene.push(DrawCommand::BeginLayer(LayerParams { opacity: 0.5 }));
    scene.push(solid(
        [16.0, 16.0, 32.0, 32.0],
        Color::rgba(0.0, 0.0, 1.0, 1.0),
    ));
    scene.push(DrawCommand::EndLayer);
    scene.push(DrawCommand::EndLayer);
    scene
}

fn scene_overlapping_alpha() -> Scene {
    let mut scene = Scene::with_clear(Color::rgba(1.0, 1.0, 1.0, 1.0));
    scene.push(solid(
        [4.0, 4.0, 40.0, 40.0],
        Color::rgba(1.0, 0.0, 0.0, 0.5),
    ));
    scene.push(solid(
        [20.0, 20.0, 40.0, 40.0],
        Color::rgba(0.0, 0.0, 1.0, 0.5),
    ));
    scene
}

fn scene_rgba_image() -> Scene {
    // 2x2 RGBA8: red, green / blue, half-transparent white.
    let pixels = vec![
        255, 0, 0, 255, 0, 255, 0, 255, //
        0, 0, 255, 255, 255, 255, 255, 128,
    ];
    let image = Rgba8Image::new(ImageKey::new(7), 1, 2, 2, pixels)
        .expect("2x2 RGBA8 data has the correct length");
    let mut scene = Scene::with_clear(Color::rgba(0.0, 0.0, 0.0, 1.0));
    scene.push(DrawCommand::Image(ImagePrimitive {
        image,
        rect: Rect::new(8.0, 8.0, 48.0, 48.0),
        fit: ImageFit::Fill,
    }));
    scene
}

fn render_case(case: &GoldenCase) -> RgbImage {
    let scene = (case.build)();
    let mut cpu =
        CpuRenderer::new(case.width, case.height, 1.0).expect("golden frame dimensions are valid");
    cpu.render_scene(&scene).expect("golden scene is valid");
    RgbImage::from_packed(case.width, case.height, cpu.pixels())
}

/// First-differing-pixel summary used by the failure message.
struct PixelDiff {
    differing: usize,
    first: Option<(u32, u32, [u8; 3], [u8; 3])>,
}

fn pixel_diff(expected: &RgbImage, actual: &RgbImage) -> PixelDiff {
    let mut diff = PixelDiff {
        differing: 0,
        first: None,
    };
    let pairs = expected.rgb.chunks_exact(3).zip(actual.rgb.chunks_exact(3));
    for (index, (want, got)) in pairs.enumerate() {
        let within = want
            .iter()
            .zip(got)
            .all(|(w, g)| w.abs_diff(*g) <= CHANNEL_TOLERANCE);
        if within {
            continue;
        }
        diff.differing += 1;
        if diff.first.is_none() {
            let x = index as u32 % expected.width;
            let y = index as u32 / expected.width;
            diff.first = Some((x, y, [want[0], want[1], want[2]], [got[0], got[1], got[2]]));
        }
    }
    diff
}

fn compare_images(name: &str, expected: &RgbImage, actual: &RgbImage) -> Result<(), String> {
    if expected.width != actual.width || expected.height != actual.height {
        return Err(format!(
            "{name}: dimensions differ: fixture {}x{}, rendered {}x{}",
            expected.width, expected.height, actual.width, actual.height
        ));
    }
    let diff = pixel_diff(expected, actual);
    if diff.differing > MAX_DIFFERING_PIXELS {
        let (x, y, want, got) = diff.first.unwrap_or((0, 0, [0; 3], [0; 3]));
        return Err(format!(
            "{name}: {}x{}, {} pixel(s) differ beyond per-channel tolerance {CHANNEL_TOLERANCE} \
             (max allowed {MAX_DIFFERING_PIXELS}); first at ({x},{y}): fixture {want:?}, rendered {got:?}",
            actual.width, actual.height, diff.differing
        ));
    }
    Ok(())
}

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_path(name: &str) -> PathBuf {
    crate_dir()
        .join("tests")
        .join("golden")
        .join(format!("{name}.ppm"))
}

fn actual_path(name: &str) -> PathBuf {
    crate_dir()
        .join("..")
        .join("..")
        .join("target")
        .join("golden-actual")
        .join(format!("{name}.ppm"))
}

fn write_file(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    fs::write(path, bytes).map_err(|error| format!("cannot write {}: {error}", path.display()))
}

fn write_actual(name: &str, image: &RgbImage) -> String {
    let path = actual_path(name);
    match write_file(&path, &image.encode_ppm()) {
        Ok(()) => path.display().to_string(),
        Err(error) => format!("<not written: {error}>"),
    }
}

fn bless_requested() -> bool {
    std::env::var(BLESS_ENV).is_ok_and(|value| value.trim() == "1")
}

fn check_case(case: &GoldenCase, actual: &RgbImage, bless: bool) -> Result<(), String> {
    let fixture = fixture_path(case.name);
    if bless {
        return write_file(&fixture, &actual.encode_ppm())
            .map(|()| println!("blessed {}", fixture.display()));
    }
    let bytes = match fs::read(&fixture) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let written = write_actual(case.name, actual);
            return Err(format!(
                "{}: missing golden fixture {}; create it with `{BLESS_COMMAND}`. \
                 Rendered frame written to {written}",
                case.name,
                fixture.display()
            ));
        }
        Err(error) => {
            return Err(format!(
                "{}: cannot read {}: {error}",
                case.name,
                fixture.display()
            ));
        }
    };
    let expected = RgbImage::decode_ppm(&bytes).map_err(|error| {
        format!(
            "{}: malformed fixture {}: {error}",
            case.name,
            fixture.display()
        )
    })?;
    match compare_images(case.name, &expected, actual) {
        Ok(()) => {
            // Remove a stale artifact from an earlier failing run; a missing file is fine.
            let _ = fs::remove_file(actual_path(case.name));
            Ok(())
        }
        Err(report) => {
            let written = write_actual(case.name, actual);
            Err(format!(
                "{report}\n  fixture: {}\n  rendered frame written to: {written}",
                fixture.display()
            ))
        }
    }
}

fn check_samples(case: &GoldenCase, actual: &RgbImage) -> Vec<String> {
    let mut problems = Vec::new();
    let colours: HashSet<[u8; 3]> = actual
        .rgb
        .chunks_exact(3)
        .map(|c| [c[0], c[1], c[2]])
        .collect();
    if colours.len() < 2 {
        problems.push(format!("{}: rendered frame is blank", case.name));
    }
    for sample in case.samples {
        let got = actual.pixel(sample.x, sample.y);
        let within = got
            .iter()
            .zip(sample.rgb)
            .all(|(g, want)| g.abs_diff(want) <= CHANNEL_TOLERANCE);
        if !within {
            problems.push(format!(
                "{}: pixel ({},{}) is {got:?}, scene should produce {:?}",
                case.name, sample.x, sample.y, sample.rgb
            ));
        }
    }
    problems
}

#[test]
fn golden_scenes_match_committed_fixtures() {
    assert!(
        GOLDEN_CASES.len() >= 5,
        "golden suite must keep at least five canonical scenes"
    );
    let bless = bless_requested();
    let failures: Vec<String> = GOLDEN_CASES
        .iter()
        .filter_map(|case| {
            assert!(
                case.width <= MAX_FIXTURE_EDGE && case.height <= MAX_FIXTURE_EDGE,
                "{} exceeds the {MAX_FIXTURE_EDGE}px fixture limit",
                case.name
            );
            check_case(case, &render_case(case), bless).err()
        })
        .collect();
    assert!(
        failures.is_empty(),
        "{} golden scene(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn rendered_scenes_hit_hand_derived_sample_pixels() {
    let failures: Vec<String> = GOLDEN_CASES
        .iter()
        .flat_map(|case| check_samples(case, &render_case(case)))
        .collect();
    assert!(
        failures.is_empty(),
        "sample pixel check failed:\n{}",
        failures.join("\n")
    );
}

#[test]
fn ppm_round_trips_through_encode_and_decode() {
    let mut image = RgbImage::filled(3, 2, [10, 20, 30]);
    image.rgb[4] = 200;
    let decoded = RgbImage::decode_ppm(&image.encode_ppm()).expect("own output decodes");
    assert_eq!(decoded, image);
}

#[test]
fn ppm_decoder_accepts_comments_and_rejects_malformed_input() {
    let commented = b"P6\n# note\n1 1\n255\n\x01\x02\x03";
    let decoded = RgbImage::decode_ppm(commented).expect("comment is skipped");
    assert_eq!(decoded.pixel(0, 0), [1, 2, 3]);

    let cases: [(&[u8], &str); 4] = [
        (b"P3\n1 1\n255\n\x01\x02\x03", "magic"),
        (b"P6\n1 1\n65535\n\x01\x02\x03", "maxval"),
        (b"P6\n2 2\n255\n\x01\x02\x03", "raster holds"),
        (b"P6\n1 1\n255", "whitespace"),
    ];
    for (bytes, expect) in cases {
        let error = RgbImage::decode_ppm(bytes).expect_err("malformed PPM must be rejected");
        assert!(
            error.contains(expect),
            "{error:?} should mention {expect:?}"
        );
    }
}

#[test]
fn comparator_tolerates_drift_within_channel_tolerance() {
    let expected = RgbImage::filled(2, 1, [10, 10, 10]);
    let actual = RgbImage::filled(2, 1, [12, 8, 10]);
    assert_eq!(compare_images("unit", &expected, &actual), Ok(()));
}

#[test]
fn comparator_flags_one_pixel_beyond_tolerance_with_coordinates() {
    let expected = RgbImage::filled(2, 2, [10, 10, 10]);
    let mut actual = expected.clone();
    // Pixel (1,1) sits at index 1 * 2 + 1 = 3; differ by 3 on red, one above tolerance.
    let at = 3 * 3;
    actual.rgb[at] = 13;
    let message = compare_images("unit", &expected, &actual).expect_err("must fail");
    assert!(message.contains("1 pixel(s) differ"), "{message}");
    assert!(message.contains("first at (1,1)"), "{message}");
    assert!(message.contains("[10, 10, 10]"), "{message}");
    assert!(message.contains("[13, 10, 10]"), "{message}");
}

#[test]
fn comparator_reports_dimension_mismatch() {
    let expected = RgbImage::filled(2, 2, [0, 0, 0]);
    let actual = RgbImage::filled(2, 1, [0, 0, 0]);
    let message = compare_images("unit", &expected, &actual).expect_err("must fail");
    assert!(message.contains("dimensions differ"), "{message}");
}
