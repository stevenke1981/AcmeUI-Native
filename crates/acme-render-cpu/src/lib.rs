//! Deterministic software rasterizer for AcmeUI's backend-neutral Scene.
//! Pixels exposed to window presenters use 0x00RRGGBB (softbuffer format).
//! The internal framebuffer uses straight 0xAARRGGBB for correct layer compositing.

use acme_core::{
    AtlasUpload, Color, DrawCommand, GlyphFormat, ImagePrimitive, QuadPrimitive, Rect, Scene,
    TextPrimitive,
};

mod coverage;
use coverage::quad_coverage;

const ATLAS: usize = 2048;
const MAX_PIXELS: usize = 32_000_000;
const MAX_LAYERS: usize = 16;
const MAX_LAYER_PIXELS: usize = 48_000_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CpuRenderError {
    InvalidDimensions,
    InvalidScale,
    InvalidScene(&'static str),
    InvalidGlyphUpload,
}

impl std::fmt::Display for CpuRenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidDimensions => write!(f, "invalid or oversized software framebuffer"),
            Self::InvalidScale => write!(f, "invalid framebuffer scale factor"),
            Self::InvalidScene(reason) => write!(f, "invalid scene: {reason}"),
            Self::InvalidGlyphUpload => write!(
                f,
                "invalid glyph upload (only atlas page zero is supported)"
            ),
        }
    }
}
impl std::error::Error for CpuRenderError {}

#[derive(Clone, Copy)]
struct Bounds {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}
impl Bounds {
    fn from_rect(rect: Rect<acme_core::Logical>, scale: f32) -> Self {
        let x = rect.origin.x.get() * scale;
        let y = rect.origin.y.get() * scale;
        Self {
            left: x,
            top: y,
            right: x + rect.size.width.get() * scale,
            bottom: y + rect.size.height.get() * scale,
        }
    }
    fn intersect(self, other: Self) -> Self {
        Self {
            left: self.left.max(other.left),
            top: self.top.max(other.top),
            right: self.right.min(other.right),
            bottom: self.bottom.min(other.bottom),
        }
    }
    fn contains(self, x: f32, y: f32) -> bool {
        x >= self.left && x < self.right && y >= self.top && y < self.bottom
    }
}

fn channel(v: f32) -> u32 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u32
}
fn pack(c: Color) -> u32 {
    (channel(c.a) << 24) | (channel(c.r) << 16) | (channel(c.g) << 8) | channel(c.b)
}
fn blend(dst: u32, src: u32, opacity: f32) -> u32 {
    let sa = ((src >> 24) & 255) as f32 * opacity.clamp(0.0, 1.0);
    if sa <= 0.0 {
        return dst;
    }
    let da = ((dst >> 24) & 255) as f32;
    let out_a = sa + da * (1.0 - sa / 255.0);
    if out_a <= 0.0 {
        return 0;
    }
    let c = |shift: u32| {
        let s = ((src >> shift) & 255u32) as f32;
        let d = ((dst >> shift) & 255u32) as f32;
        ((s * sa + d * da * (1.0 - sa / 255.0)) / out_a).round() as u32
    };
    ((out_a.round() as u32) << 24) | (c(16) << 16) | (c(8) << 8) | c(0)
}

fn for_pixels(
    width: u32,
    height: u32,
    bounds: Bounds,
    clip: Option<Bounds>,
    mut paint: impl FnMut(usize, f32, f32),
) {
    // Minimized surfaces and malformed geometry must not enter the raster loops.
    if width == 0 || height == 0 {
        return;
    }
    if ![bounds.left, bounds.top, bounds.right, bounds.bottom]
        .iter()
        .all(|value| value.is_finite())
    {
        return;
    }
    let visible = clip.map_or(bounds, |region| bounds.intersect(region));
    let left = visible.left.floor().max(0.0).min(width as f32) as u32;
    let top = visible.top.floor().max(0.0).min(height as f32) as u32;
    let right = visible.right.ceil().max(0.0).min(width as f32) as u32;
    let bottom = visible.bottom.ceil().max(0.0).min(height as f32) as u32;
    if left >= right || top >= bottom {
        return;
    }
    for y in top..bottom {
        for x in left..right {
            let (px, py) = (x as f32 + 0.5, y as f32 + 0.5);
            if !clip.is_none_or(|c| c.contains(px, py)) {
                continue;
            }
            paint(y as usize * width as usize + x as usize, px, py);
        }
    }
}

fn draw_quad(
    pixels: &mut [u32],
    width: u32,
    height: u32,
    scale: f32,
    clip: Option<Bounds>,
    quad: &QuadPrimitive,
) {
    let rect = Bounds::from_rect(quad.rect, scale);
    let radius = quad.radius.max(0.0) * scale;
    let border = quad.border_width.max(0.0) * scale;
    let inner = Bounds {
        left: rect.left + border,
        top: rect.top + border,
        right: rect.right - border,
        bottom: rect.bottom - border,
    };
    let inner_radius = (radius - border).max(0.0);
    let fill = pack(quad.color);
    let stroke = pack(quad.border_color);
    for_pixels(width, height, rect, clip, |idx, x, y| {
        let coverage = quad_coverage(rect, radius, x, y);
        if coverage <= 0.0 {
            return;
        }
        if border > 0.0 {
            // Stroke area is outer minus inner, so it cannot go negative.
            let inside = quad_coverage(inner, inner_radius, x, y);
            pixels[idx] = blend(pixels[idx], fill, inside);
            pixels[idx] = blend(pixels[idx], stroke, (coverage - inside).max(0.0));
        } else {
            pixels[idx] = blend(pixels[idx], fill, coverage);
        }
    });
}

fn draw_image(
    pixels: &mut [u32],
    width: u32,
    height: u32,
    scale: f32,
    clip: Option<Bounds>,
    image: &ImagePrimitive,
) {
    let dest = Bounds::from_rect(image.fitted_rect(), scale);
    let dw = dest.right - dest.left;
    let dh = dest.bottom - dest.top;
    if dw <= 0.0 || dh <= 0.0 {
        return;
    }
    let sw = image.image.width() as usize;
    let sh = image.image.height() as usize;
    let source = image.image.pixels();
    for_pixels(width, height, dest, clip, |idx, x, y| {
        if !dest.contains(x, y) {
            return;
        }
        let sx = (((x - dest.left) / dw * sw as f32) as usize).min(sw - 1);
        let sy = (((y - dest.top) / dh * sh as f32) as usize).min(sh - 1);
        let at = (sy * sw + sx) * 4;
        let src = ((source[at + 3] as u32) << 24)
            | ((source[at] as u32) << 16)
            | ((source[at + 1] as u32) << 8)
            | source[at + 2] as u32;
        pixels[idx] = blend(pixels[idx], src, 1.0);
    });
}

#[derive(Default)]
struct GlyphAtlases {
    alpha: Option<Vec<u8>>,
    rgba: Option<Vec<u8>>,
}
impl GlyphAtlases {
    fn validate_upload(u: &AtlasUpload) -> Result<(), CpuRenderError> {
        let bpp: usize = match u.format {
            GlyphFormat::Alpha8 => 1,
            GlyphFormat::Rgba8 => 4,
        };
        let (x, y, w, h) = (
            u.origin[0] as usize,
            u.origin[1] as usize,
            u.size[0] as usize,
            u.size[1] as usize,
        );
        let len = w.checked_mul(h).and_then(|v| v.checked_mul(bpp));
        if u.page != 0
            || x.checked_add(w).is_none_or(|v| v > ATLAS)
            || y.checked_add(h).is_none_or(|v| v > ATLAS)
            || len != Some(u.pixels.len())
        {
            return Err(CpuRenderError::InvalidGlyphUpload);
        }
        Ok(())
    }
    fn upload(&mut self, u: &AtlasUpload) {
        // The scene preflight checks all uploads before any persistent state changes.
        debug_assert!(Self::validate_upload(u).is_ok());
        let bpp = match u.format {
            GlyphFormat::Alpha8 => 1,
            GlyphFormat::Rgba8 => 4,
        };
        let (x, y, w, h) = (
            u.origin[0] as usize,
            u.origin[1] as usize,
            u.size[0] as usize,
            u.size[1] as usize,
        );
        let atlas = match u.format {
            GlyphFormat::Alpha8 => self.alpha.get_or_insert_with(|| vec![0; ATLAS * ATLAS]),
            GlyphFormat::Rgba8 => self.rgba.get_or_insert_with(|| vec![0; ATLAS * ATLAS * 4]),
        };
        for row in 0..h {
            let start = ((y + row) * ATLAS + x) * bpp;
            let src = row * w * bpp;
            atlas[start..start + w * bpp].copy_from_slice(&u.pixels[src..src + w * bpp]);
        }
    }
    fn sample(&self, format: GlyphFormat, x: usize, y: usize) -> u32 {
        if x >= ATLAS || y >= ATLAS {
            return 0;
        }
        let offset = y * ATLAS + x;
        match format {
            GlyphFormat::Alpha8 => self
                .alpha
                .as_ref()
                .map_or(0, |a| (a[offset] as u32) << 24 | 0x00ff_ffff),
            GlyphFormat::Rgba8 => self.rgba.as_ref().map_or(0, |a| {
                let i = offset * 4;
                ((a[i + 3] as u32) << 24)
                    | ((a[i] as u32) << 16)
                    | ((a[i + 1] as u32) << 8)
                    | (a[i + 2] as u32)
            }),
        }
    }
}

fn draw_text(
    pixels: &mut [u32],
    width: u32,
    height: u32,
    scale: f32,
    clip: Option<Bounds>,
    text: &TextPrimitive,
    atlases: &GlyphAtlases,
) {
    let tint = pack(text.color);
    for glyph in &text.glyphs {
        // Glyph dimensions and glyph.x/y are physical pixels, just as in wgpu.
        let rect = Bounds {
            left: glyph.x + text.origin.x.get() * scale,
            top: glyph.y + text.origin.y.get() * scale,
            right: glyph.x + text.origin.x.get() * scale + glyph.width,
            bottom: glyph.y + text.origin.y.get() * scale + glyph.height,
        };
        for_pixels(width, height, rect, clip, |idx, x, y| {
            if !rect.contains(x, y) {
                return;
            }
            // Float-to-integer casts saturate. A far-offscreen glyph can therefore
            // yield usize::MAX; adding an atlas origin must not panic or wrap.
            let Some(gx) = (glyph.atlas_x as usize).checked_add((x - rect.left) as usize) else {
                return;
            };
            let Some(gy) = (glyph.atlas_y as usize).checked_add((y - rect.top) as usize) else {
                return;
            };
            let mask = atlases.sample(glyph.format, gx, gy);
            let src = match glyph.format {
                GlyphFormat::Alpha8 => {
                    ((mask >> 24) * (tint >> 24 & 255) / 255) << 24 | (tint & 0x00ff_ffff)
                }
                GlyphFormat::Rgba8 => {
                    let alpha = ((mask >> 24) * (tint >> 24 & 255) / 255) << 24;
                    let r = ((mask >> 16 & 255) * (tint >> 16 & 255) / 255) << 16;
                    let g = ((mask >> 8 & 255) * (tint >> 8 & 255) / 255) << 8;
                    let b = (mask & 255) * (tint & 255) / 255;
                    alpha | r | g | b
                }
            };
            pixels[idx] = blend(pixels[idx], src, 1.0);
        });
    }
}

struct Layer {
    opacity: f32,
    pixels: Vec<u32>,
}

/// Headless CPU renderer. This crate never allocates GPU resources or a native window.
pub struct CpuRenderer {
    width: u32,
    height: u32,
    scale_factor: f32,
    framebuffer: Vec<u32>,
    output: Vec<u32>,
    atlases: GlyphAtlases,
}
impl CpuRenderer {
    pub fn new(width: u32, height: u32, scale_factor: f32) -> Result<Self, CpuRenderError> {
        let mut renderer = Self {
            width: 0,
            height: 0,
            scale_factor: 1.0,
            framebuffer: Vec::new(),
            output: Vec::new(),
            atlases: GlyphAtlases::default(),
        };
        renderer.resize(width, height, scale_factor)?;
        Ok(renderer)
    }
    pub fn resize(
        &mut self,
        width: u32,
        height: u32,
        scale_factor: f32,
    ) -> Result<(), CpuRenderError> {
        if !scale_factor.is_finite() || scale_factor <= 0.0 {
            return Err(CpuRenderError::InvalidScale);
        }
        let len = (width as usize)
            .checked_mul(height as usize)
            .filter(|len| *len <= MAX_PIXELS)
            .ok_or(CpuRenderError::InvalidDimensions)?;
        self.width = width;
        self.height = height;
        self.scale_factor = scale_factor;
        self.framebuffer.resize(len, 0);
        self.output.resize(len, 0);
        Ok(())
    }
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    /// Final opaque 0x00RRGGBB pixels (row-major), ready for softbuffer.
    pub fn pixels(&self) -> &[u32] {
        &self.output
    }
    fn validate_scene(&self, scene: &Scene) -> Result<(), CpuRenderError> {
        let mut layers = 0usize;
        let mut clips = 0usize;
        let max_layers = MAX_LAYERS.min(MAX_LAYER_PIXELS / self.framebuffer.len().max(1));
        // Preserve the original error order without allocating layers or copying atlases.
        for cmd in scene.commands() {
            match cmd {
                DrawCommand::PushClip(_) => clips += 1,
                DrawCommand::PopClip => {
                    clips = clips
                        .checked_sub(1)
                        .ok_or(CpuRenderError::InvalidScene("unmatched PopClip"))?;
                }
                DrawCommand::BeginLayer(_) => {
                    layers += 1;
                    if layers > max_layers {
                        return Err(CpuRenderError::InvalidScene(
                            "layer allocation budget exceeded",
                        ));
                    }
                }
                DrawCommand::EndLayer => {
                    layers = layers
                        .checked_sub(1)
                        .ok_or(CpuRenderError::InvalidScene("unmatched EndLayer"))?;
                }
                DrawCommand::Text(text) => {
                    for upload in &text.uploads {
                        GlyphAtlases::validate_upload(upload)?;
                    }
                }
                DrawCommand::Quad(_) | DrawCommand::Image(_) => {}
            }
        }
        if layers != 0 || clips != 0 {
            return Err(CpuRenderError::InvalidScene("unclosed clip or layer"));
        }
        Ok(())
    }
    /// Render a complete scene, preserving the last frame and glyph atlases on rejection.
    pub fn render_scene(&mut self, scene: &Scene) -> Result<(), CpuRenderError> {
        self.validate_scene(scene)?;
        self.framebuffer.fill(pack(scene.clear_color()));
        let mut layers: Vec<Layer> = Vec::new();
        let mut clips: Vec<Bounds> = Vec::new();
        for cmd in scene.commands() {
            let clip = clips.last().copied();
            match cmd {
                DrawCommand::PushClip(r) => {
                    let next = Bounds::from_rect(*r, self.scale_factor);
                    clips.push(clip.map_or(next, |existing| existing.intersect(next)));
                }
                DrawCommand::PopClip => {
                    clips.pop().expect("scene preflight validated clip balance");
                }
                DrawCommand::BeginLayer(params) => {
                    layers.push(Layer {
                        opacity: params.opacity.clamp(0.0, 1.0),
                        pixels: vec![0; self.framebuffer.len()],
                    });
                }
                DrawCommand::EndLayer => {
                    let layer = layers
                        .pop()
                        .expect("scene preflight validated layer balance");
                    let target = layers
                        .last_mut()
                        .map_or(&mut self.framebuffer, |l| &mut l.pixels);
                    for (dst, src) in target.iter_mut().zip(layer.pixels.iter()) {
                        *dst = blend(*dst, *src, layer.opacity);
                    }
                }
                DrawCommand::Quad(q) => {
                    let target = layers
                        .last_mut()
                        .map_or(&mut self.framebuffer, |l| &mut l.pixels);
                    draw_quad(target, self.width, self.height, self.scale_factor, clip, q);
                }
                DrawCommand::Image(image) => {
                    let target = layers
                        .last_mut()
                        .map_or(&mut self.framebuffer, |l| &mut l.pixels);
                    draw_image(
                        target,
                        self.width,
                        self.height,
                        self.scale_factor,
                        clip,
                        image,
                    );
                }
                DrawCommand::Text(text) => {
                    for upload in &text.uploads {
                        self.atlases.upload(upload);
                    }
                    let target = layers
                        .last_mut()
                        .map_or(&mut self.framebuffer, |l| &mut l.pixels);
                    draw_text(
                        target,
                        self.width,
                        self.height,
                        self.scale_factor,
                        clip,
                        text,
                        &self.atlases,
                    );
                }
            }
        }
        // Software display surfaces are opaque; composite any remaining scene alpha over black.
        for (dst, pixel) in self.output.iter_mut().zip(self.framebuffer.iter()) {
            let a = (pixel >> 24) & 255;
            let r = (pixel >> 16) & 255;
            let g = (pixel >> 8) & 255;
            let b = pixel & 255;
            *dst = ((r * a / 255) << 16) | ((g * a / 255) << 8) | (b * a / 255);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::{AtlasUpload, GlyphDraw, ImageFit, ImageKey, LayerParams, Point, Rgba8Image};

    fn quad(rect: Rect<acme_core::Logical>, color: Color) -> DrawCommand {
        DrawCommand::Quad(QuadPrimitive {
            rect,
            color,
            radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        })
    }
    fn render(scene: &Scene, w: u32, h: u32) -> CpuRenderer {
        let mut cpu = CpuRenderer::new(w, h, 1.0).unwrap();
        cpu.render_scene(scene).unwrap();
        cpu
    }
    #[test]
    fn clear_and_ordered_painter() {
        let mut s = Scene::with_clear(Color::rgba(0.0, 0.0, 0.0, 1.0));
        s.push(quad(
            Rect::new(0.0, 0.0, 3.0, 3.0),
            Color::rgba(1.0, 0.0, 0.0, 1.0),
        ));
        s.push(quad(
            Rect::new(1.0, 1.0, 1.0, 1.0),
            Color::rgba(0.0, 1.0, 0.0, 1.0),
        ));
        let c = render(&s, 3, 3);
        assert_eq!(c.pixels()[0], 0x00ff0000);
        assert_eq!(c.pixels()[4], 0x0000ff00);
    }
    #[test]
    fn alpha_blends_over_clear() {
        let mut s = Scene::with_clear(Color::rgba(1.0, 1.0, 1.0, 1.0));
        s.push(quad(
            Rect::new(0.0, 0.0, 1.0, 1.0),
            Color::rgba(0.0, 0.0, 0.0, 0.5),
        ));
        let pixel = render(&s, 1, 1).pixels()[0] & 255;
        assert!((127..=128).contains(&pixel));
    }
    #[test]
    fn nested_clipping_intersects() {
        let mut s = Scene::new();
        s.push(DrawCommand::PushClip(Rect::new(0.0, 0.0, 3.0, 3.0)));
        s.push(DrawCommand::PushClip(Rect::new(2.0, 2.0, 3.0, 3.0)));
        s.push(quad(
            Rect::new(0.0, 0.0, 5.0, 5.0),
            Color::rgba(1.0, 0.0, 0.0, 1.0),
        ));
        s.push(DrawCommand::PopClip);
        s.push(DrawCommand::PopClip);
        let c = render(&s, 5, 5);
        assert_eq!(c.pixels()[2 * 5 + 2], 0x00ff0000);
        assert_eq!(c.pixels()[6], 0);
        assert_eq!(c.pixels()[3 * 5 + 3], 0);
    }
    #[test]
    fn round_rect_and_border() {
        let mut s = Scene::new();
        s.push(DrawCommand::Quad(QuadPrimitive {
            rect: Rect::new(0.0, 0.0, 8.0, 8.0),
            color: Color::rgba(0.0, 1.0, 0.0, 1.0),
            radius: 3.0,
            border_width: 1.0,
            border_color: Color::rgba(1.0, 0.0, 0.0, 1.0),
        }));
        let c = render(&s, 8, 8);
        // Corner pixel (0, 0): the r = 3 arc clips a sliver of its far corner. The exact
        // area is 0.029 (red 7). The 8x8 estimate resolves one sample of 64 (red 4).
        // The old signed-distance rule dropped it to 0 because its centre sits more
        // than half a pixel outside the arc.
        let corner_red = (c.pixels()[0] >> 16) & 255;
        assert!(
            (1..=8).contains(&corner_red),
            "corner sliver red {corner_red}"
        );
        assert_eq!(c.pixels()[3 * 8 + 3], 0x0000ff00);
        assert_eq!(c.pixels()[4], 0x00ff0000);
    }
    #[test]
    fn image_contain_keeps_letterbox() {
        let img = Rgba8Image::new(
            ImageKey::new(1),
            1,
            2,
            1,
            vec![255, 0, 0, 255, 0, 0, 255, 255],
        )
        .unwrap();
        let mut s = Scene::new();
        s.push(DrawCommand::Image(ImagePrimitive {
            image: img,
            rect: Rect::new(0.0, 0.0, 4.0, 4.0),
            fit: ImageFit::Contain,
        }));
        let c = render(&s, 4, 4);
        assert_eq!(c.pixels()[0], 0);
        assert_eq!(c.pixels()[4], 0x00ff0000);
        assert_eq!(c.pixels()[7], 0x000000ff);
    }
    #[test]
    fn alpha_glyph_and_persistent_atlas() {
        let mut t = TextPrimitive {
            origin: Point::new(0.0, 0.0),
            color: Color::rgba(1.0, 1.0, 1.0, 1.0),
            glyphs: vec![GlyphDraw {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
                atlas_x: 5,
                atlas_y: 6,
                atlas_page: 0,
                format: GlyphFormat::Alpha8,
            }],
            uploads: vec![AtlasUpload {
                page: 0,
                origin: [5, 6],
                size: [1, 1],
                format: GlyphFormat::Alpha8,
                pixels: vec![255],
            }],
        };
        let mut s = Scene::new();
        s.push(DrawCommand::Text(t.clone()));
        let mut c = CpuRenderer::new(1, 1, 1.0).unwrap();
        c.render_scene(&s).unwrap();
        assert_eq!(c.pixels()[0], 0x00ffffff);
        t.uploads.clear();
        let mut s2 = Scene::new();
        s2.push(DrawCommand::Text(t));
        c.render_scene(&s2).unwrap();
        assert_eq!(c.pixels()[0], 0x00ffffff);
    }
    #[test]
    fn layer_opacity_composes_once() {
        let mut s = Scene::new();
        s.push(DrawCommand::BeginLayer(LayerParams { opacity: 0.5 }));
        s.push(quad(
            Rect::new(0.0, 0.0, 1.0, 1.0),
            Color::rgba(1.0, 0.0, 0.0, 1.0),
        ));
        s.push(quad(
            Rect::new(0.0, 0.0, 1.0, 1.0),
            Color::rgba(1.0, 0.0, 0.0, 1.0),
        ));
        s.push(DrawCommand::EndLayer);
        let c = render(&s, 1, 1);
        assert_eq!((c.pixels()[0] >> 16) & 255, 128);
    }
    #[test]
    fn rejects_bad_commands_and_data() {
        let mut s = Scene::new();
        s.push(DrawCommand::PopClip);
        let mut cpu = CpuRenderer::new(1, 1, 1.0).unwrap();
        assert!(matches!(
            cpu.render_scene(&s),
            Err(CpuRenderError::InvalidScene(_))
        ));
        assert!(matches!(
            GlyphAtlases::validate_upload(&AtlasUpload {
                page: 1,
                origin: [0, 0],
                size: [1, 1],
                format: GlyphFormat::Alpha8,
                pixels: vec![1]
            }),
            Err(CpuRenderError::InvalidGlyphUpload)
        ));
        assert!(matches!(
            CpuRenderer::new(100_000, 100_000, 1.0),
            Err(CpuRenderError::InvalidDimensions)
        ));
    }
    #[test]
    fn physical_scale_and_resize() {
        let mut s = Scene::new();
        s.push(quad(
            Rect::new(0.0, 0.0, 1.0, 1.0),
            Color::rgba(1.0, 0.0, 0.0, 1.0),
        ));
        let mut c = CpuRenderer::new(2, 2, 2.0).unwrap();
        c.render_scene(&s).unwrap();
        assert!(c.pixels().iter().all(|p| *p == 0x00ff0000));
        c.resize(1, 1, 1.0).unwrap();
        c.render_scene(&s).unwrap();
        assert_eq!(c.dimensions(), (1, 1));
    }
}
