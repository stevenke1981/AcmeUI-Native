//! Surface acquisition outcomes, device-lost recovery state and wgpu error hooks.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::{SurfaceAction, SurfaceStatus};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AcquireOutcome {
    Success,
    Suboptimal,
    OutdatedOrLost,
    TimeoutOrOccluded,
    Validation,
}

pub(crate) fn resolve_surface_action(
    suspended: bool,
    device_lost: bool,
    acquire: AcquireOutcome,
) -> SurfaceAction {
    if suspended {
        return SurfaceAction::Skip;
    }
    if device_lost {
        return SurfaceAction::DeviceLost;
    }
    match acquire {
        AcquireOutcome::Success | AcquireOutcome::Suboptimal => SurfaceAction::Rendered,
        AcquireOutcome::OutdatedOrLost => SurfaceAction::Reconfigure,
        AcquireOutcome::TimeoutOrOccluded | AcquireOutcome::Validation => SurfaceAction::Skip,
    }
}

pub(crate) fn is_device_lost(flag: &AtomicBool) -> bool {
    flag.load(Ordering::Acquire)
}

pub(crate) fn complete_recovery_state(
    device_lost: &AtomicBool,
    status: &mut SurfaceStatus,
    gpu_epoch: &mut u64,
) {
    device_lost.store(false, Ordering::Release);
    *status = SurfaceStatus::Ready;
    *gpu_epoch = gpu_epoch.wrapping_add(1);
}

/// Register wgpu callbacks that mark the shared device-lost flag.
///
/// - `set_device_lost_callback`: real device loss
/// - `on_uncaptured_error`: Internal / OutOfMemory → treat as lost; Validation is logged only
pub(crate) fn register_device_error_handlers(device: &wgpu::Device, flag: &Arc<AtomicBool>) {
    let lost_flag = Arc::clone(flag);
    device.set_device_lost_callback(move |reason, message| {
        tracing::error!(?reason, %message, "GPU device lost callback fired");
        lost_flag.store(true, Ordering::Release);
    });

    let err_flag = Arc::clone(flag);
    device.on_uncaptured_error(Arc::new(move |error: wgpu::Error| match &error {
        wgpu::Error::Validation { description, .. } => {
            tracing::error!(%description, "wgpu validation error (not marking device lost)");
        }
        wgpu::Error::OutOfMemory { .. } => {
            tracing::error!(%error, "wgpu out-of-memory; marking device lost");
            err_flag.store(true, Ordering::Release);
        }
        wgpu::Error::Internal { description, .. } => {
            tracing::error!(%description, "wgpu internal error; marking device lost");
            err_flag.store(true, Ordering::Release);
        }
    }));
}
