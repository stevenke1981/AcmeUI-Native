//! Scene atlas upload validation and bytes-per-texel helpers.

use acme_core::GlyphFormat as SceneGlyphFormat;

use crate::atlas_pages::{UploadKey, UploadReject, check_upload, upload_key};

/// Bytes per texel for a scene glyph format.
pub(crate) fn scene_bytes_per_pixel(format: SceneGlyphFormat) -> usize {
    match format {
        SceneGlyphFormat::Alpha8 => 1,
        SceneGlyphFormat::Rgba8 => 4,
    }
}

/// Validates one scene atlas upload and returns its dedup key. The key includes
/// page and format, so the same rectangle on another page is a distinct region.
pub(crate) fn validate_scene_upload(
    upload: &acme_core::AtlasUpload,
) -> Result<UploadKey, UploadReject> {
    check_upload(
        upload.page,
        upload.origin,
        upload.size,
        scene_bytes_per_pixel(upload.format),
        upload.pixels.len(),
    )?;
    Ok(upload_key(
        upload.page,
        upload.format,
        upload.origin,
        upload.size,
    ))
}
