//! Per-page glyph atlas textures (ROADMAP 011).

use acme_core::{ATLAS_MAX_PAGES, ATLAS_PAGE_EDGE, GlyphFormat};

/// One atlas page: an `ATLAS_PAGE_EDGE` square texture plus the bind group
/// that exposes it to the text pipeline.
pub(crate) struct AtlasPage {
    pub(crate) texture: wgpu::Texture,
    pub(crate) bind_group: wgpu::BindGroup,
}

/// Lazily created per-page textures, one slot list per glyph format.
#[derive(Default)]
pub(crate) struct AtlasPages {
    alpha: Vec<Option<AtlasPage>>,
    rgba: Vec<Option<AtlasPage>>,
}

impl AtlasPages {
    /// Returns the page, creating its texture on first use.
    ///
    /// # Panics
    /// Panics if `page >= ATLAS_MAX_PAGES`; callers validate with `check_upload`.
    pub(crate) fn ensure(
        &mut self,
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        format: GlyphFormat,
        page: u32,
    ) -> &AtlasPage {
        assert!(page < ATLAS_MAX_PAGES, "atlas page {page} out of range");
        let slot = &mut self.slots_mut(format)[page as usize];
        slot.get_or_insert_with(|| AtlasPage::new(device, layout, sampler, format, page))
    }

    /// Returns the page if it has already been created.
    pub(crate) fn get(&self, format: GlyphFormat, page: u32) -> Option<&AtlasPage> {
        let slots = match format {
            GlyphFormat::Alpha8 => &self.alpha,
            GlyphFormat::Rgba8 => &self.rgba,
        };
        slots.get(page as usize)?.as_ref()
    }

    /// Slot list for `format`, grown to `ATLAS_MAX_PAGES` on first use.
    fn slots_mut(&mut self, format: GlyphFormat) -> &mut Vec<Option<AtlasPage>> {
        let slots = match format {
            GlyphFormat::Alpha8 => &mut self.alpha,
            GlyphFormat::Rgba8 => &mut self.rgba,
        };
        if slots.len() < ATLAS_MAX_PAGES as usize {
            slots.resize_with(ATLAS_MAX_PAGES as usize, || None);
        }
        slots
    }
}

pub(crate) fn atlas_texture_format(format: GlyphFormat) -> (wgpu::TextureFormat, &'static str) {
    match format {
        GlyphFormat::Alpha8 => (wgpu::TextureFormat::R8Unorm, "alpha"),
        // Scene colours are sRGB-encoded; keep RGBA8 glyph texels in non-sRGB format
        // so sampling passes sRGB bytes through unchanged to the non-sRGB surface.
        GlyphFormat::Rgba8 => (wgpu::TextureFormat::Rgba8Unorm, "rgba"),
    }
}

impl AtlasPage {
    fn new(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        format: GlyphFormat,
        page: u32,
    ) -> Self {
        let (texture_format, name) = atlas_texture_format(format);
        let label = format!("{name} atlas page {page}");
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&label),
            size: wgpu::Extent3d {
                width: ATLAS_PAGE_EDGE,
                height: ATLAS_PAGE_EDGE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: texture_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | if cfg!(test) {
                    wgpu::TextureUsages::COPY_SRC
                } else {
                    wgpu::TextureUsages::empty()
                },
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(&label),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });
        Self {
            texture,
            bind_group,
        }
    }
}

/// Reason a glyph atlas upload was rejected before touching the GPU.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UploadReject {
    /// Page index is `>= ATLAS_MAX_PAGES`.
    PageOutOfRange,
    /// Width or height is zero.
    EmptyRegion,
    /// `origin + size` exceeds `ATLAS_PAGE_EDGE` (or overflows `u32`).
    RegionOutsidePage,
    /// Payload length is not `width * height * bytes_per_pixel`.
    PayloadSizeMismatch,
}

/// Identity of one atlas upload: `(page, format, x, y, width, height)`.
pub(crate) type UploadKey = (u32, GlyphFormat, u32, u32, u32, u32);

/// A maximal run of consecutive draws that share one atlas page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PageRun {
    pub page: u32,
    /// Index of the first draw in the input sequence.
    pub start: usize,
    /// Number of consecutive draws in the run.
    pub len: usize,
}

/// Validates an upload region against page bounds and payload size.
///
/// Checks run in order: page index, non-empty size, region inside
/// `ATLAS_PAGE_EDGE` (overflow-safe), then payload length. Never panics.
pub(crate) fn check_upload(
    page: u32,
    origin: [u32; 2],
    size: [u32; 2],
    bpp: usize,
    pixels_len: usize,
) -> Result<(), UploadReject> {
    if page >= ATLAS_MAX_PAGES {
        return Err(UploadReject::PageOutOfRange);
    }
    let [width, height] = size;
    if width == 0 || height == 0 {
        return Err(UploadReject::EmptyRegion);
    }
    let fits_x = origin[0]
        .checked_add(width)
        .is_some_and(|end| end <= ATLAS_PAGE_EDGE);
    let fits_y = origin[1]
        .checked_add(height)
        .is_some_and(|end| end <= ATLAS_PAGE_EDGE);
    if !(fits_x && fits_y) {
        return Err(UploadReject::RegionOutsidePage);
    }
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|texels| texels.checked_mul(bpp));
    if expected != Some(pixels_len) {
        return Err(UploadReject::PayloadSizeMismatch);
    }
    Ok(())
}

/// Builds the dedup key for an upload.
pub(crate) fn upload_key(
    page: u32,
    format: GlyphFormat,
    origin: [u32; 2],
    size: [u32; 2],
) -> UploadKey {
    (page, format, origin[0], origin[1], size[0], size[1])
}

/// Groups page indices into maximal consecutive runs, preserving order.
pub(crate) fn page_runs(pages: impl IntoIterator<Item = u32>) -> Vec<PageRun> {
    let mut runs: Vec<PageRun> = Vec::new();
    for (index, page) in pages.into_iter().enumerate() {
        match runs.last_mut() {
            Some(run) if run.page == page => run.len += 1,
            _ => runs.push(PageRun {
                page,
                start: index,
                len: 1,
            }),
        }
    }
    runs
}

#[cfg(test)]
mod tests {
    use super::*;

    const EDGE: u32 = ATLAS_PAGE_EDGE;

    /// Same entries as the glyph atlas layout in `lib.rs::build_render_resources`.
    fn texture_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("test atlas layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        })
    }

    #[test]
    fn check_upload_accepts_a_valid_region() {
        assert_eq!(check_upload(0, [0, 0], [4, 4], 1, 16), Ok(()),);
    }

    #[test]
    fn check_upload_rejects_page_at_or_above_max_pages() {
        assert_eq!(
            check_upload(ATLAS_MAX_PAGES, [0, 0], [1, 1], 1, 1),
            Err(UploadReject::PageOutOfRange)
        );
        assert_eq!(
            check_upload(u32::MAX, [0, 0], [1, 1], 1, 1),
            Err(UploadReject::PageOutOfRange)
        );
    }

    #[test]
    fn check_upload_accepts_last_valid_page() {
        assert_eq!(
            check_upload(ATLAS_MAX_PAGES - 1, [0, 0], [1, 1], 1, 1),
            Ok(())
        );
    }

    #[test]
    fn check_upload_rejects_zero_width_or_height() {
        assert_eq!(
            check_upload(0, [0, 0], [0, 4], 1, 0),
            Err(UploadReject::EmptyRegion)
        );
        assert_eq!(
            check_upload(0, [0, 0], [4, 0], 4, 0),
            Err(UploadReject::EmptyRegion)
        );
    }

    #[test]
    fn check_upload_accepts_region_touching_the_page_edge() {
        assert_eq!(check_upload(0, [EDGE - 1, EDGE - 1], [1, 1], 1, 1), Ok(()));
        assert_eq!(
            check_upload(0, [0, 0], [EDGE, EDGE], 1, (EDGE * EDGE) as usize),
            Ok(())
        );
    }

    #[test]
    fn check_upload_rejects_region_past_the_page_edge() {
        assert_eq!(
            check_upload(0, [EDGE, 0], [1, 1], 1, 1),
            Err(UploadReject::RegionOutsidePage)
        );
        assert_eq!(
            check_upload(0, [0, EDGE], [1, 1], 1, 1),
            Err(UploadReject::RegionOutsidePage)
        );
        assert_eq!(
            check_upload(0, [EDGE - 1, 0], [2, 1], 1, 2),
            Err(UploadReject::RegionOutsidePage)
        );
    }

    #[test]
    fn check_upload_region_math_is_overflow_safe() {
        assert_eq!(
            check_upload(0, [u32::MAX, 0], [1, 1], 1, 1),
            Err(UploadReject::RegionOutsidePage)
        );
        assert_eq!(
            check_upload(0, [1, u32::MAX], [u32::MAX, 1], 1, 1),
            Err(UploadReject::RegionOutsidePage)
        );
        assert_eq!(
            check_upload(0, [u32::MAX, u32::MAX], [u32::MAX, u32::MAX], 4, 0),
            Err(UploadReject::RegionOutsidePage)
        );
    }

    #[test]
    fn check_upload_rejects_payload_length_mismatch() {
        assert_eq!(
            check_upload(0, [0, 0], [4, 4], 1, 15),
            Err(UploadReject::PayloadSizeMismatch)
        );
        assert_eq!(
            check_upload(0, [0, 0], [4, 4], 4, 16),
            Err(UploadReject::PayloadSizeMismatch)
        );
        assert_eq!(check_upload(0, [0, 0], [4, 4], 4, 64), Ok(()));
    }

    #[test]
    fn check_upload_payload_math_is_overflow_safe() {
        assert_eq!(
            check_upload(0, [0, 0], [EDGE, EDGE], usize::MAX, 0),
            Err(UploadReject::PayloadSizeMismatch)
        );
        assert_eq!(
            check_upload(0, [0, 0], [EDGE, EDGE], 4, usize::MAX),
            Err(UploadReject::PayloadSizeMismatch)
        );
    }

    #[test]
    fn check_upload_reports_the_first_failing_rule() {
        assert_eq!(
            check_upload(ATLAS_MAX_PAGES, [EDGE, 0], [0, 0], 1, 0),
            Err(UploadReject::PageOutOfRange)
        );
        assert_eq!(
            check_upload(0, [EDGE, 0], [0, 0], 1, 0),
            Err(UploadReject::EmptyRegion)
        );
    }

    #[test]
    fn page_runs_empty_input_has_no_runs() {
        assert_eq!(page_runs(std::iter::empty()), Vec::<PageRun>::new());
    }

    #[test]
    fn page_runs_single_draw_is_one_run() {
        assert_eq!(
            page_runs([3]),
            vec![PageRun {
                page: 3,
                start: 0,
                len: 1
            }]
        );
    }

    #[test]
    fn page_runs_consecutive_equal_pages_merge() {
        assert_eq!(
            page_runs([2, 2, 2]),
            vec![PageRun {
                page: 2,
                start: 0,
                len: 3
            }]
        );
    }

    #[test]
    fn page_runs_alternating_pages_split_every_draw() {
        assert_eq!(
            page_runs([0, 1, 0]),
            vec![
                PageRun {
                    page: 0,
                    start: 0,
                    len: 1
                },
                PageRun {
                    page: 1,
                    start: 1,
                    len: 1
                },
                PageRun {
                    page: 0,
                    start: 2,
                    len: 1
                },
            ]
        );
    }

    #[test]
    fn page_runs_mixed_input_preserves_order_and_indices() {
        assert_eq!(
            page_runs([0, 0, 1, 1, 1, 0]),
            vec![
                PageRun {
                    page: 0,
                    start: 0,
                    len: 2
                },
                PageRun {
                    page: 1,
                    start: 2,
                    len: 3
                },
                PageRun {
                    page: 0,
                    start: 5,
                    len: 1
                },
            ]
        );
    }

    #[test]
    fn upload_key_is_stable_for_identical_uploads() {
        let a = upload_key(1, GlyphFormat::Alpha8, [2, 3], [4, 5]);
        let b = upload_key(1, GlyphFormat::Alpha8, [2, 3], [4, 5]);
        assert_eq!(a, b);
    }

    #[test]
    fn upload_key_distinguishes_page_for_the_same_region() {
        let page0 = upload_key(0, GlyphFormat::Alpha8, [2, 3], [4, 5]);
        let page1 = upload_key(1, GlyphFormat::Alpha8, [2, 3], [4, 5]);
        assert_ne!(page0, page1);
    }

    #[test]
    fn upload_key_distinguishes_format_for_the_same_region() {
        let alpha = upload_key(0, GlyphFormat::Alpha8, [2, 3], [4, 5]);
        let rgba = upload_key(0, GlyphFormat::Rgba8, [2, 3], [4, 5]);
        assert_ne!(alpha, rgba);
    }

    #[test]
    fn upload_key_distinguishes_region_geometry() {
        let base = upload_key(0, GlyphFormat::Rgba8, [2, 3], [4, 5]);
        assert_ne!(base, upload_key(0, GlyphFormat::Rgba8, [3, 3], [4, 5]));
        assert_ne!(base, upload_key(0, GlyphFormat::Rgba8, [2, 3], [5, 4]));
    }

    /// Needs a real or fallback wgpu adapter. Run with
    /// `cargo test --locked -p acme-render-wgpu -- --ignored atlas_pages_ensure_gpu`.
    #[test]
    #[ignore = "requires a wgpu adapter; run with -- --ignored"]
    fn atlas_pages_ensure_gpu() {
        let instance = wgpu::Instance::default();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            force_fallback_adapter: true,
            compatible_surface: None,
        }));
        let Ok(adapter) = adapter else {
            eprintln!("atlas_pages_ensure_gpu: no adapter, test skipped");
            return;
        };
        let Ok((device, _queue)) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
        else {
            eprintln!("atlas_pages_ensure_gpu: no device, test skipped");
            return;
        };
        let layout = texture_layout(&device);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());
        let mut pages = AtlasPages::default();
        assert!(pages.get(GlyphFormat::Alpha8, 0).is_none());
        let page = pages.ensure(&device, &layout, &sampler, GlyphFormat::Alpha8, 0);
        assert_eq!(page.texture.format(), wgpu::TextureFormat::R8Unorm);
        assert_eq!(page.texture.size().width, EDGE);
        assert_eq!(page.texture.size().height, EDGE);
        let rgba = pages.ensure(&device, &layout, &sampler, GlyphFormat::Rgba8, 7);
        assert_eq!(rgba.texture.format(), wgpu::TextureFormat::Rgba8Unorm);
        assert!(pages.get(GlyphFormat::Rgba8, 7).is_some());
        assert!(pages.get(GlyphFormat::Rgba8, 6).is_none());
    }

    #[test]
    fn atlas_texture_format_uses_non_srgb_rgba8() {
        assert_eq!(
            atlas_texture_format(GlyphFormat::Alpha8),
            (wgpu::TextureFormat::R8Unorm, "alpha")
        );
        assert_eq!(
            atlas_texture_format(GlyphFormat::Rgba8),
            (wgpu::TextureFormat::Rgba8Unorm, "rgba")
        );
    }
}
