//! Unit tests for the renderer's pure helpers and the surface state machine.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use acme_core::{
    ATLAS_MAX_PAGES, ATLAS_PAGE_EDGE, DrawCommand, GlyphFormat as SceneGlyphFormat, ImageKey,
    ImagePrimitive, Rgba8Image, Scene,
};

use crate::atlas_pages::UploadReject;
use crate::draws::{DrawBatchDesc, push_text_batch, scissor};
use crate::images::{
    ImageUploadError, ImageUploadLayout, collect_scene_images, retain_live_image_resources,
    validate_image_upload,
};
use crate::instances::{
    image_primitive_to_instance, normalize_color, normalize_scale, text_primitive_glyphs,
};
use crate::surface::{
    AcquireOutcome, complete_recovery_state, is_device_lost, register_device_error_handlers,
    resolve_surface_action,
};
use crate::uploads::{scene_bytes_per_pixel, validate_scene_upload};
use crate::{Frame, Quad, SurfaceAction, SurfaceStatus};

fn test_image(revision: u64, width: u32, height: u32) -> Rgba8Image {
    Rgba8Image::new(
        ImageKey::new(1),
        revision,
        width,
        height,
        vec![255; width as usize * height as usize * 4],
    )
    .unwrap()
}

#[test]
fn same_key_revisions_are_distinct_resources_within_one_scene() {
    let mut scene = Scene::new();
    for revision in [10, 11] {
        scene.push(DrawCommand::Image(ImagePrimitive::contain(
            test_image(revision, 2, 2),
            acme_core::Rect::new(0.0, 0.0, 10.0, 10.0),
        )));
    }

    let images = collect_scene_images(&scene);
    assert_eq!(images.len(), 2);
    assert!(images.contains_key(&(ImageKey::new(1), 10)));
    assert!(images.contains_key(&(ImageKey::new(1), 11)));
}

#[test]
fn image_resource_retention_is_bounded_to_live_scene_keys() {
    let mut resources = HashMap::from([
        ((ImageKey::new(1), 1), "old"),
        ((ImageKey::new(1), 2), "current"),
        ((ImageKey::new(2), 1), "removed"),
    ]);
    let live = HashSet::from([(ImageKey::new(1), 2)]);
    retain_live_image_resources(&mut resources, &live);

    assert_eq!(resources.len(), 1);
    assert_eq!(resources[&(ImageKey::new(1), 2)], "current");
}

#[test]
fn image_upload_layout_rejects_device_limit_and_row_pitch_overflow() {
    assert_eq!(
        validate_image_upload(4097, 10, 4096),
        Err(ImageUploadError::ExceedsMaxDimension {
            width: 4097,
            height: 10,
            max: 4096,
        })
    );
    assert_eq!(
        validate_image_upload(u32::MAX, 1, u32::MAX),
        Err(ImageUploadError::RowPitchOverflow)
    );
    assert_eq!(
        validate_image_upload(1920, 1080, 8192),
        Ok(ImageUploadLayout {
            bytes_per_row: 7680,
        })
    );
}

#[test]
fn image_instance_uses_contained_rect_and_rgba_mode() {
    let primitive = ImagePrimitive::contain(
        test_image(0, 2, 1),
        acme_core::Rect::new(0.0, 0.0, 100.0, 100.0),
    );
    let instance = image_primitive_to_instance(&primitive, 2.0, 800, 600);
    assert_eq!(instance.rect, [0.0, 50.0, 200.0, 100.0]);
    assert_eq!(instance.uv, [0.0, 0.0, 1.0, 1.0]);
    assert_eq!(instance.viewport_mode, [800.0, 600.0, 1.0, 0.0]);
}

#[test]
fn solid_quad_defaults_are_valid() {
    let q = Quad::solid([1.0, 2.0, 3.0, 4.0], [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(q.border_width, 0.0);
}
#[test]
fn empty_frame_has_opaque_clear() {
    assert_eq!(Frame::default().clear[3], 1.0);
}

#[test]
fn scissor_intersects_the_physical_viewport() {
    assert_eq!(
        scissor([-5.0, 4.0, 20.0, 10.0], 2.0, 100, 100),
        Some([0, 8, 30, 20])
    );
    assert_eq!(
        scissor([0.25, 1.5, 10.75, 20.125], 2.0, 100, 100),
        Some([0, 3, 22, 41])
    );
    assert_eq!(scissor([200.0, 0.0, 10.0, 10.0], 1.0, 100, 100), None);
    assert_eq!(scissor([10.0, 10.0, 0.0, 10.0], 1.0, 100, 100), None);
    assert_eq!(scissor([f32::NAN, 0.0, 10.0, 10.0], 1.0, 100, 100), None);
}

#[test]
fn device_lost_action_is_distinct() {
    assert_ne!(SurfaceAction::DeviceLost, SurfaceAction::Exit);
    assert_ne!(SurfaceAction::DeviceLost, SurfaceAction::Reconfigure);
}

#[test]
fn resolve_surface_action_covers_all_transitions() {
    use AcquireOutcome::*;
    let outcomes = [
        Success,
        Suboptimal,
        OutdatedOrLost,
        TimeoutOrOccluded,
        Validation,
    ];

    // Suspended always wins → Skip, regardless of device_lost or acquire.
    for &acquire in &outcomes {
        for &device_lost in &[false, true] {
            assert_eq!(
                resolve_surface_action(true, device_lost, acquire),
                SurfaceAction::Skip,
                "suspended must Skip (device_lost={device_lost}, acquire={acquire:?})"
            );
        }
    }

    // Not suspended, device_lost → DeviceLost, regardless of acquire.
    for &acquire in &outcomes {
        assert_eq!(
            resolve_surface_action(false, true, acquire),
            SurfaceAction::DeviceLost,
            "device_lost must map to DeviceLost (acquire={acquire:?})"
        );
    }

    // Not suspended, not device_lost → depends on acquire outcome.
    assert_eq!(
        resolve_surface_action(false, false, Success),
        SurfaceAction::Rendered
    );
    assert_eq!(
        resolve_surface_action(false, false, Suboptimal),
        SurfaceAction::Rendered
    );
    assert_eq!(
        resolve_surface_action(false, false, OutdatedOrLost),
        SurfaceAction::Reconfigure
    );
    assert_eq!(
        resolve_surface_action(false, false, TimeoutOrOccluded),
        SurfaceAction::Skip
    );
    assert_eq!(
        resolve_surface_action(false, false, Validation),
        SurfaceAction::Skip
    );
}

#[test]
fn recovery_resets_pure_state_and_bumps_epoch() {
    // Mimic post-device-loss state: device_lost=true, status=Recovering.
    let device_lost = AtomicBool::new(true);
    let mut status = SurfaceStatus::Recovering;
    let mut gpu_epoch = 7u64;
    complete_recovery_state(&device_lost, &mut status, &mut gpu_epoch);
    assert!(
        !is_device_lost(&device_lost),
        "device_lost must clear after recovery"
    );
    assert_eq!(status, SurfaceStatus::Ready, "status must return to Ready");
    assert_eq!(gpu_epoch, 8, "gpu_epoch must increment by one");
}

#[test]
fn device_lost_flag_maps_to_device_lost_action() {
    let flag = AtomicBool::new(false);
    assert_eq!(
        resolve_surface_action(false, is_device_lost(&flag), AcquireOutcome::Success),
        SurfaceAction::Rendered
    );
    flag.store(true, Ordering::Release);
    assert_eq!(
        resolve_surface_action(false, is_device_lost(&flag), AcquireOutcome::Success),
        SurfaceAction::DeviceLost
    );
}

#[test]
#[ignore = "requires real GPU adapter; run with cargo test -p acme-render-wgpu -- --ignored"]
fn device_recovery_smoke_ignored() {
    // Adapter + device + handler registration only (no window / surface).
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        force_fallback_adapter: true,
        compatible_surface: None,
    }));
    let Ok(adapter) = adapter else {
        return;
    };
    let Ok((device, _queue)) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
    else {
        return;
    };
    let flag = Arc::new(AtomicBool::new(false));
    register_device_error_handlers(&device, &flag);
    assert!(!is_device_lost(&flag));
}

#[test]
fn invalid_scale_and_color_are_normalized() {
    assert_eq!(normalize_scale(f32::NAN), 1.0);
    assert_eq!(
        normalize_color([f32::NAN, -1.0, 2.0, 0.5]),
        [0.0, 0.0, 1.0, 0.5]
    );
}

fn scene_glyph(page: u32, x: f32) -> acme_core::GlyphDraw {
    acme_core::GlyphDraw {
        x,
        y: 0.0,
        width: 4.0,
        height: 4.0,
        atlas_x: 8,
        atlas_y: 16,
        atlas_page: page,
        format: SceneGlyphFormat::Alpha8,
    }
}

/// Alpha text primitive whose glyph `i` sits on `pages[i]` at x = `i`.
fn text_with_pages(pages: &[u32]) -> acme_core::TextPrimitive {
    acme_core::TextPrimitive {
        origin: acme_core::Point::new(0.0, 0.0),
        color: acme_core::Color::rgba(1.0, 1.0, 1.0, 1.0),
        glyphs: pages
            .iter()
            .enumerate()
            .map(|(index, &page)| scene_glyph(page, index as f32))
            .collect(),
        uploads: Vec::new(),
    }
}

fn scene_upload(page: u32, format: SceneGlyphFormat, origin: [u32; 2]) -> acme_core::AtlasUpload {
    let bpp = scene_bytes_per_pixel(format);
    acme_core::AtlasUpload {
        page,
        origin,
        size: [2, 2],
        format,
        pixels: vec![0; 4 * bpp],
    }
}

#[test]
fn text_glyphs_carry_their_atlas_page() {
    let prim = text_with_pages(&[0, 0, 1, 1, 0]);
    let glyphs = text_primitive_glyphs(&prim, SceneGlyphFormat::Alpha8, 1.0, 800, 600);
    let pages: Vec<u32> = glyphs.iter().map(|(page, _)| *page).collect();
    assert_eq!(pages, vec![0, 0, 1, 1, 0]);
}

#[test]
fn text_glyph_uv_is_normalized_by_page_edge() {
    let mut prim = text_with_pages(&[0]);
    prim.glyphs[0].atlas_x = 1024;
    prim.glyphs[0].atlas_y = 512;
    let glyphs = text_primitive_glyphs(&prim, SceneGlyphFormat::Alpha8, 1.0, 800, 600);
    let uv = glyphs[0].1.uv;
    let edge = ATLAS_PAGE_EDGE as f32;
    assert_eq!(uv, [1024.0 / edge, 512.0 / edge, 4.0 / edge, 4.0 / edge]);
}

#[test]
fn text_glyph_with_out_of_range_page_is_skipped() {
    let prim = text_with_pages(&[0, ATLAS_MAX_PAGES, 1, u32::MAX]);
    let glyphs = text_primitive_glyphs(&prim, SceneGlyphFormat::Alpha8, 1.0, 800, 600);
    let pages: Vec<u32> = glyphs.iter().map(|(page, _)| *page).collect();
    assert_eq!(pages, vec![0, 1]);
    let xs: Vec<f32> = glyphs.iter().map(|(_, glyph)| glyph.rect[0]).collect();
    assert_eq!(xs, vec![0.0, 2.0]);
}

#[test]
fn text_batch_splits_into_per_page_runs_in_painter_order() {
    let prim = text_with_pages(&[0, 0, 1, 1, 0]);
    let glyphs = text_primitive_glyphs(&prim, SceneGlyphFormat::Alpha8, 1.0, 800, 600);
    let mut instances = Vec::new();
    let mut draws = Vec::new();
    push_text_batch(
        glyphs,
        SceneGlyphFormat::Alpha8,
        None,
        &mut instances,
        &mut draws,
    );

    let runs: Vec<(u32, u32, u32)> = draws
        .iter()
        .filter_map(|draw| match draw {
            DrawBatchDesc::Glyph(run) => Some((run.page, run.start, run.count)),
            _ => None,
        })
        .collect();
    assert_eq!(runs, vec![(0, 0, 2), (1, 2, 2), (0, 4, 1)]);
    assert_eq!(draws.len(), 3, "one draw per page run");
    let xs: Vec<f32> = instances.iter().map(|glyph| glyph.rect[0]).collect();
    assert_eq!(xs, vec![0.0, 1.0, 2.0, 3.0, 4.0], "painter order kept");
}

#[test]
fn text_batch_with_no_glyphs_emits_no_draws() {
    let mut instances = Vec::new();
    let mut draws = Vec::new();
    push_text_batch(
        Vec::new(),
        SceneGlyphFormat::Rgba8,
        None,
        &mut instances,
        &mut draws,
    );
    assert!(instances.is_empty());
    assert!(draws.is_empty());
}

#[test]
fn scene_upload_key_distinguishes_page_and_format_for_the_same_rect() {
    let uploads = [
        scene_upload(0, SceneGlyphFormat::Alpha8, [2, 3]),
        scene_upload(1, SceneGlyphFormat::Alpha8, [2, 3]),
        scene_upload(0, SceneGlyphFormat::Rgba8, [2, 3]),
    ];
    let keys: HashSet<_> = uploads
        .iter()
        .map(|upload| validate_scene_upload(upload).expect("valid upload"))
        .collect();
    assert_eq!(
        keys.len(),
        3,
        "same x,y,w,h on other page/format is distinct"
    );
}

#[test]
fn scene_upload_validation_rejects_out_of_range_page() {
    let upload = scene_upload(ATLAS_MAX_PAGES, SceneGlyphFormat::Alpha8, [0, 0]);
    assert_eq!(
        validate_scene_upload(&upload),
        Err(UploadReject::PageOutOfRange)
    );
    let last = scene_upload(ATLAS_MAX_PAGES - 1, SceneGlyphFormat::Alpha8, [0, 0]);
    assert!(validate_scene_upload(&last).is_ok());
}
