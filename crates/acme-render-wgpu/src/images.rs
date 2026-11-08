//! RGBA8 image resources: upload validation, texture creation and retention.

use std::collections::{HashMap, HashSet};

use acme_core::{DrawCommand, ImageKey, Rgba8Image, Scene};

pub(crate) struct ImageResource {
    pub(crate) image: Rgba8Image,
    pub(crate) _texture: wgpu::Texture,
    pub(crate) bind_group: wgpu::BindGroup,
}

pub(crate) type ImageResourceKey = (ImageKey, u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ImageUploadLayout {
    pub(crate) bytes_per_row: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ImageUploadError {
    ExceedsMaxDimension { width: u32, height: u32, max: u32 },
    RowPitchOverflow,
}

pub(crate) fn image_resource_key(image: &Rgba8Image) -> ImageResourceKey {
    (image.key(), image.revision())
}

pub(crate) fn collect_scene_images(scene: &Scene) -> HashMap<ImageResourceKey, &Rgba8Image> {
    scene
        .commands()
        .iter()
        .filter_map(|command| match command {
            DrawCommand::Image(primitive) => {
                Some((image_resource_key(&primitive.image), &primitive.image))
            }
            _ => None,
        })
        .collect()
}

pub(crate) fn retain_live_image_resources<T>(
    resources: &mut HashMap<ImageResourceKey, T>,
    live: &HashSet<ImageResourceKey>,
) {
    resources.retain(|key, _| live.contains(key));
}

pub(crate) fn validate_image_upload(
    width: u32,
    height: u32,
    max_dimension: u32,
) -> Result<ImageUploadLayout, ImageUploadError> {
    if width > max_dimension || height > max_dimension {
        return Err(ImageUploadError::ExceedsMaxDimension {
            width,
            height,
            max: max_dimension,
        });
    }
    let bytes_per_row = width
        .checked_mul(4)
        .ok_or(ImageUploadError::RowPitchOverflow)?;
    Ok(ImageUploadLayout { bytes_per_row })
}

/// Texture format for RGBA8 images: non-sRGB so sRGB-encoded image bytes pass
/// through unchanged to the non-sRGB swapchain (matching the CPU renderer).
pub(crate) const IMAGE_TEXTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

pub(crate) fn create_image_resource(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    layout: &wgpu::BindGroupLayout,
    sampler: &wgpu::Sampler,
    image: Rgba8Image,
    upload_layout: ImageUploadLayout,
) -> ImageResource {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("acme rgba8 image"),
        size: wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: IMAGE_TEXTURE_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    write_rgba8_texture(queue, &texture, &image, upload_layout);
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("acme rgba8 image bind group"),
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
    ImageResource {
        image,
        _texture: texture,
        bind_group,
    }
}

pub(crate) fn write_rgba8_texture(
    queue: &wgpu::Queue,
    texture: &wgpu::Texture,
    image: &Rgba8Image,
    upload_layout: ImageUploadLayout,
) {
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        image.pixels(),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(upload_layout.bytes_per_row),
            rows_per_image: Some(image.height()),
        },
        wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        },
    );
}
