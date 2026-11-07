//! Glyph atlas page creation and scene atlas upload.

use std::collections::HashSet;

use acme_core::{DrawCommand, Scene};

use crate::atlas_pages::UploadKey;
use crate::draws::DrawBatchDesc;
use crate::uploads::{scene_bytes_per_pixel, validate_scene_upload};

use super::Renderer;

impl Renderer {
    /// Ensures every (format, page) pair used by `draws` has its texture and bind group.
    pub(super) fn ensure_glyph_pages(&mut self, draws: &[DrawBatchDesc]) {
        for draw in draws {
            if let DrawBatchDesc::Glyph(run) = draw {
                self.atlas_pages.ensure(
                    &self.device,
                    &self.texture_layout,
                    &self.image_sampler,
                    run.format,
                    run.page,
                );
            }
        }
    }

    /// Upload atlas regions from a [`Scene`]'s text primitives into their pages,
    /// deduplicating by [`UploadKey`] within the frame.
    /// Returns (total_count, skipped_count, bytes_uploaded).
    pub(super) fn upload_scene_atlas(
        &mut self,
        scene: &Scene,
        uploaded: &mut HashSet<UploadKey>,
    ) -> (u64, u64, u64) {
        let mut total = 0u64;
        let mut skipped = 0u64;
        let mut atlas_bytes = 0u64;
        for cmd in scene.commands() {
            let DrawCommand::Text(prim) = cmd else {
                continue;
            };
            for upload in &prim.uploads {
                let key = match validate_scene_upload(upload) {
                    Ok(key) => key,
                    Err(reject) => {
                        tracing::warn!(page = upload.page, ?reject, "rejected glyph atlas upload");
                        continue;
                    }
                };
                total += 1;
                if !uploaded.insert(key) {
                    skipped += 1;
                    continue;
                }
                let bytes_per_pixel = scene_bytes_per_pixel(upload.format) as u32;
                let [x, y] = upload.origin;
                let [width, height] = upload.size;
                let page = self.atlas_pages.ensure(
                    &self.device,
                    &self.texture_layout,
                    &self.image_sampler,
                    upload.format,
                    upload.page,
                );
                self.queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &page.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d { x, y, z: 0 },
                        aspect: wgpu::TextureAspect::All,
                    },
                    &upload.pixels,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(width * bytes_per_pixel),
                        rows_per_image: Some(height),
                    },
                    wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                );
                atlas_bytes += width as u64 * height as u64 * bytes_per_pixel as u64;
            }
        }
        (total, skipped, atlas_bytes)
    }
}
