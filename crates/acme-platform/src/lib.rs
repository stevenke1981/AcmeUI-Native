//! Windows-first winit runtime. Public events contain no winit platform types.
#![forbid(unsafe_op_in_unsafe_fn)]
#![allow(clippy::items_after_test_module)]

mod backend;
mod clipboard;
mod keys;
mod scheduling;
use scheduling::{BackgroundPoll, FrameRetry};
mod waker;
pub use backend::RenderBackendPreference;
use backend::{CPU_FEATURE_REQUIRED, RendererPlan, plan_renderer, resolve_render_backend_from_env};
pub use clipboard::Clipboard;
pub use keys::PlatformKey;
use keys::{map_logical_key, resolve_key_character, resolve_key_text};
pub use waker::{CursorIcon, ResizeEdge, Waker, WindowRequest};
use waker::{
    OpenWindowRequest, drain_window_requests, map_cursor_icon, map_resize_edge,
    plan_window_requests,
};

#[cfg(feature = "cpu-renderer")]
mod cpu_window;
#[cfg(feature = "cpu-renderer")]
use cpu_window::CpuWindowRenderer;

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use acme_core::Scene;
use acme_render_wgpu::{Renderer, SurfaceAction};
use thiserror::Error;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalPosition, LogicalSize, PhysicalPosition},
    event::{ElementState, Ime, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId as WinitWindowId},
};

/// Global pointer/touch ID counter for tracking unique pointer identities.
static NEXT_POINTER_ID: AtomicU64 = AtomicU64::new(1);

/// A unique window identifier within the framework.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WindowId(pub u64);

#[derive(Clone, Debug, PartialEq)]
pub struct WindowConfig {
    pub title: String,
    /// Initial logical client (inner) width in pixels.
    pub width: f64,
    /// Initial logical client (inner) height in pixels.
    pub height: f64,
    /// Optional minimum logical client (inner) width.
    pub min_width: Option<f64>,
    /// Optional minimum logical client (inner) height.
    pub min_height: Option<f64>,
    /// Optional initial logical outer top-left position `(x, y)`.
    pub position: Option<(f64, f64)>,
    /// `false` removes the native title bar and border (custom frame). On Windows the
    /// OS still provides Aero Snap, Alt+F4 and the Alt+Space system menu.
    pub decorations: bool,
    /// Windows only, effective when `decorations` is `false`: DWM drop shadow.
    /// The shadow draws a 1px line along the top edge.
    pub undecorated_shadow: bool,
}
impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "AcmeUI Native".into(),
            width: 1100.0,
            height: 760.0,
            min_width: None,
            min_height: None,
            position: None,
            decorations: true,
            undecorated_shadow: true,
        }
    }
}

/// Resolve the minimum logical window size for a window: per-window [`WindowConfig`]
/// bounds win over [`Application::minimum_window_size`].
pub(crate) fn resolve_min_window_size(
    config: &WindowConfig,
    app_min: Option<(f64, f64)>,
) -> Option<(f64, f64)> {
    let width = config.min_width.or_else(|| app_min.map(|(w, _)| w));
    let height = config.min_height.or_else(|| app_min.map(|(_, h)| h));
    match (width, height) {
        (None, None) => None,
        (w, h) => Some((w.unwrap_or(0.0), h.unwrap_or(0.0))),
    }
}

/// Convert a physical client (inner) size into logical `(width, height)` pixels.
pub(crate) fn physical_size_to_logical(
    size: winit::dpi::PhysicalSize<u32>,
    scale_factor: f32,
) -> (f32, f32) {
    let scale = if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    };
    (size.width as f32 / scale, size.height as f32 / scale)
}

/// Convert a physical outer window top-left position into logical `(x, y)` coordinates.
pub(crate) fn physical_position_to_logical(
    position: PhysicalPosition<i32>,
    scale_factor: f32,
) -> (f32, f32) {
    let scale = if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    };
    (position.x as f32 / scale, position.y as f32 / scale)
}

/// Convert physical cursor coordinates (which may be negative or outside the
/// client area while mouse capture is active during a button-held drag) into
/// logical `(x, y)` coordinates without clamping.
pub(crate) fn pointer_position_to_logical(x: f64, y: f64, scale_factor: f32) -> (f32, f32) {
    let scale = if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    };
    (x as f32 / scale, y as f32 / scale)
}

/// Events emitted when the pointer leaves the window client area.
/// Emits [`PlatformEvent::PointerLeft`] alongside [`PlatformEvent::CursorLeft`]
/// for backward compatibility.
pub(crate) fn cursor_left_events(window: WindowId) -> [PlatformEvent; 2] {
    [
        PlatformEvent::PointerLeft { window },
        PlatformEvent::CursorLeft { window },
    ]
}

#[derive(Clone, Debug, PartialEq)]
pub enum PlatformEvent {
    // ?? Existing variants (backward-compatible, unchanged) ??
    /// Window client (inner) area resized. `logical_width` and `logical_height`
    /// are logical client dimensions matching [`WindowConfig::width`] and
    /// [`WindowConfig::height`].
    Resized {
        window: WindowId,
        logical_width: f32,
        logical_height: f32,
        scale_factor: f32,
    },
    /// Window outer top-left position moved. `x` and `y` are logical outer
    /// coordinates matching [`WindowConfig::position`].
    Moved {
        window: WindowId,
        x: f32,
        y: f32,
    },
    /// Window maximized state changed (or was first observed). Emitted before the
    /// matching [`PlatformEvent::Resized`] for the same frame or resize.
    MaximizedChanged {
        window: WindowId,
        maximized: bool,
    },
    PointerMoved {
        window: WindowId,
        x: f32,
        y: f32,
    },
    /// Pointer left the window client area (emitted alongside [`PlatformEvent::CursorLeft`]).
    PointerLeft {
        window: WindowId,
    },
    PointerButton {
        window: WindowId,
        pressed: bool,
        x: f32,
        y: f32,
        button: u16,
        pointer: u64,
        shift: bool,
        ctrl: bool,
        alt: bool,
    },
    Scroll {
        window: WindowId,
        delta_x: f32,
        delta_y: f32,
        lines: bool,
        x: f32,
        y: f32,
        /// Keyboard modifiers held when the wheel event was delivered
        /// (tracked from winit `ModifiersChanged`).
        shift: bool,
        ctrl: bool,
        alt: bool,
        meta: bool,
    },
    Key {
        window: WindowId,
        key: PlatformKey,
        pressed: bool,
        shift: bool,
        ctrl: bool,
        alt: bool,
        meta: bool,
        repeat: bool,
        text: Option<String>,
        character: Option<String>,
    },
    /// IME preedit with WindowId and cursor position.
    ImePreedit {
        window: WindowId,
        text: String,
        cursor: Option<(usize, usize)>,
    },
    /// IME commit with WindowId.
    ImeCommit {
        window: WindowId,
        text: String,
    },
    WindowCloseRequested(WindowId),

    /// IME input method was enabled for the given window.
    ImeEnabled(WindowId),
    /// IME input method was disabled for the given window.
    ImeDisabled(WindowId),

    /// Window keyboard focus changed. `node_id` is set by the framework layer.
    FocusChanged {
        window: WindowId,
        gained: bool,
        node_id: u64,
    },

    /// Cursor entered the window client area.
    CursorEntered {
        window: WindowId,
        x: f32,
        y: f32,
    },
    /// Cursor left the window client area.
    CursorLeft {
        window: WindowId,
    },

    /// One or more files were dropped onto the window.
    FileDropped {
        window: WindowId,
        paths: Vec<String>,
    },

    /// Request from an assistive technology (screen reader, switch device)
    /// to scroll the identified node into the visible viewport.
    ///
    /// `node_id` is the AcmeUI layout node ID, NOT an AccessKit node ID.
    /// The framework layer should resolve the node's bounding box from the
    /// current layout snapshot and scroll the nearest scroll container so
    /// that the node becomes visible.
    AccessibilityScrollIntoView {
        window: WindowId,
        node_id: u64,
    },
}

/// Platform-agnostic keyboard modifier flags.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KeyModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub meta: bool,
}

impl KeyModifiers {
    /// Returns `true` when no modifier keys are held.
    pub fn is_empty(&self) -> bool {
        !(self.shift || self.control || self.alt || self.meta)
    }

    /// Returns `true` when at least one modifier key is held.
    pub fn any(&self) -> bool {
        !self.is_empty()
    }

    /// Returns `true` when the platform's primary shortcut modifier is held
    /// (`Command` / `meta` on macOS/iOS, `Ctrl` / `control` elsewhere).
    pub fn primary_shortcut(&self) -> bool {
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            self.meta
        }
        #[cfg(not(any(target_os = "macos", target_os = "ios")))]
        {
            self.control
        }
    }

    /// Returns `true` when the primary shortcut modifier is held for the given
    /// target convention (`meta` when `macos` is `true`, `control` otherwise).
    pub fn primary_shortcut_for_macos(&self, macos: bool) -> bool {
        if macos { self.meta } else { self.control }
    }
}

/// Structured view of a [`PlatformEvent::Key`] event with grouped modifiers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyInput {
    pub window: WindowId,
    pub key: PlatformKey,
    pub pressed: bool,
    pub modifiers: KeyModifiers,
    pub repeat: bool,
    pub text: Option<String>,
    pub character: Option<String>,
}

impl KeyInput {
    /// Returns `true` if this is a pressed key event matching `key` with the
    /// platform's [`KeyModifiers::primary_shortcut`] modifier active.
    pub fn is_primary_shortcut(&self, key: PlatformKey) -> bool {
        self.pressed && self.key == key && self.modifiers.primary_shortcut()
    }
}

impl PlatformEvent {
    /// Extract keyboard modifiers from [`PlatformEvent::Key`],
    /// [`PlatformEvent::PointerButton`] or [`PlatformEvent::Scroll`],
    /// returning `None` for other variants.
    pub fn key_modifiers(&self) -> Option<KeyModifiers> {
        match self {
            Self::Scroll {
                shift,
                ctrl,
                alt,
                meta,
                ..
            } => Some(KeyModifiers {
                shift: *shift,
                control: *ctrl,
                alt: *alt,
                meta: *meta,
            }),
            Self::Key {
                shift,
                ctrl,
                alt,
                meta,
                ..
            } => Some(KeyModifiers {
                shift: *shift,
                control: *ctrl,
                alt: *alt,
                meta: *meta,
            }),
            Self::PointerButton {
                shift, ctrl, alt, ..
            } => Some(KeyModifiers {
                shift: *shift,
                control: *ctrl,
                alt: *alt,
                meta: false,
            }),
            _ => None,
        }
    }

    /// Extract a [`KeyInput`] view if this event is [`PlatformEvent::Key`].
    pub fn key_input(&self) -> Option<KeyInput> {
        match self {
            Self::Key {
                window,
                key,
                pressed,
                shift,
                ctrl,
                alt,
                meta,
                repeat,
                text,
                character,
            } => Some(KeyInput {
                window: *window,
                key: *key,
                pressed: *pressed,
                modifiers: KeyModifiers {
                    shift: *shift,
                    control: *ctrl,
                    alt: *alt,
                    meta: *meta,
                },
                repeat: *repeat,
                text: text.clone(),
                character: character.clone(),
            }),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FrameContext {
    pub window: WindowId,
    pub logical_width: f32,
    pub logical_height: f32,
    pub scale_factor: f32,
}

/// Application-side state driven by framework-owned events.
pub trait Application: 'static {
    fn window_config(&self) -> WindowConfig {
        WindowConfig::default()
    }
    /// Preferred rendering backend when `ACME_RENDER_BACKEND` is not explicitly set.
    fn render_backend(&self) -> RenderBackendPreference {
        RenderBackendPreference::Auto
    }
    /// Minimum logical window size applied to every application window.
    fn minimum_window_size(&self) -> Option<(f64, f64)> {
        None
    }
    /// Returns the configurations for all windows that should be created on startup.
    ///
    /// By default, returns a single window from [`Application::window_config`].
    /// Override this method to create multiple windows with different titles and sizes.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn windows(&self) -> Vec<WindowConfig> {
    ///     vec![
    ///         WindowConfig { title: "Editor".into(), width: 1200.0, height: 800.0, ..Default::default() },
    ///         WindowConfig { title: "Inspector".into(), width: 400.0, height: 600.0, ..Default::default() },
    ///     ]
    /// }
    /// ```
    fn windows(&self) -> Vec<WindowConfig> {
        vec![self.window_config()]
    }
    fn event(&mut self, _event: PlatformEvent) -> bool {
        false
    }
    /// Called once with a cross-thread [`Waker`] before the first window is created.
    fn on_waker(&mut self, _waker: Waker) {}

    /// Called on the UI thread for each delivered [`Waker::wake`].
    /// Returning `true` requests a redraw of every window.
    fn wake(&mut self) -> bool {
        false
    }

    /// Drain pending host window requests for `window`.
    ///
    /// The runtime drains this after every event dispatch, after [`Application::wake`],
    /// and after [`Application::frame`], applying the requests to that window.
    fn take_window_requests(&mut self, _window: WindowId) -> Vec<WindowRequest> {
        Vec::new()
    }

    /// Called when the application should update the IME cursor area for a window.
    ///
    /// `rect` is `[x, y, width, height]` in logical pixels relative to the window's
    /// client area. The framework calls this during IME composition after resolving
    /// the area via [`Application::ime_cursor_area`] (app-authoritative) with a
    /// mouse-position fallback. Existing apps may ignore this notification.
    fn set_ime_cursor_area(&mut self, _window: WindowId, _rect: [f32; 4]) {}

    /// Return the preferred IME candidate area for `window` in logical pixels
    /// relative to the window client area (`[x, y, width, height]`).
    ///
    /// `None` means the platform should use its fallback (mouse cursor position).
    /// Apps with a focused text field should return the caret rectangle so the
    /// system IME candidate window tracks the text caret rather than the pointer.
    fn ime_cursor_area(&self, _window: WindowId) -> Option<[f32; 4]> {
        None
    }

    /// Called after the renderer successfully recreates GPU resources.
    ///
    /// Applications owning CPU-side caches that mirror GPU resources should
    /// invalidate them here so the next frame repopulates the new GPU resources.
    fn on_gpu_recovered(&mut self, _window: WindowId) {}

    /// Decide whether a user close request (title-bar close button, Alt+F4, taskbar
    /// close) actually closes `window`.
    ///
    /// Called after [`PlatformEvent::WindowCloseRequested`] has been delivered through
    /// [`Application::event`]. Returning `false` keeps the window open so the
    /// application can confirm unsaved changes and close later with
    /// [`WindowRequest::Close`]. The default `true` preserves the previous behaviour.
    /// Requests queued by either callback are drained while `window` is still live.
    /// An independent replacement should use `owner: None`.
    fn close_requested(&mut self, _window: WindowId) -> bool {
        true
    }

    /// Called after a window requested with [`WindowRequest::Open`] was created and
    /// its renderer initialised. `token` is the value given in the request; the
    /// new window already has a redraw pending.
    fn window_opened(&mut self, _token: u64, _window: WindowId) {}

    /// Called when a [`WindowRequest::Open`] could not be fulfilled (unknown owner,
    /// OS window creation failure or renderer initialisation failure).
    fn window_open_failed(&mut self, _token: u64, _error: String) {}

    /// Called after `window` has been removed from the runtime, whether it was
    /// closed by [`WindowRequest::Close`], by an unvetoed user close request (see
    /// [`Application::close_requested`]) or destroyed by the OS (for example
    /// together with its owner). Not called for windows that never opened.
    /// Closing a window other than the last one keeps the event loop running.
    fn window_closed(&mut self, _window: WindowId) {}

    /// Optional polling interval for background-thread work such as global hotkeys.
    /// `None` keeps the runtime fully event-driven while `Some` schedules redraws
    /// without forcing a busy `ControlFlow::Poll` loop. Intervals are clamped
    /// to 1 ms through 24 hours; changing an interval reschedules from now.
    fn background_poll_interval(&self) -> Option<std::time::Duration> {
        None
    }

    /// Poll application-owned background queues. Return `true` when a redraw
    /// is needed. The runtime calls this only when `background_poll_interval`
    /// is enabled, so idle windows do not repaint continuously.
    fn background_tick(&mut self) -> bool {
        false
    }
    fn frame(&mut self, context: FrameContext) -> Scene;
}

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("event loop error: {0}")]
    EventLoop(String),
    /// No window could be created or initialised; `reasons` holds one entry per failure.
    #[error("no window could be created: {}", .reasons.join("; "))]
    NoWindows { reasons: Vec<String> },
    /// The event loop stopped because a renderer failed and could not recover.
    #[error("unrecoverable renderer failure: {0}")]
    Renderer(String),
}

/// Runs an application until its final window closes.
///
/// Returns `Ok(())` when the last window closes normally. Returns
/// [`PlatformError::NoWindows`] when no window could be initialised, and
/// [`PlatformError::Renderer`] when the loop stopped on an unrecoverable
/// renderer failure.
pub fn run<A: Application>(mut app: A) -> Result<(), PlatformError> {
    let event_loop = EventLoop::<()>::with_user_event()
        .build()
        .map_err(|error| PlatformError::EventLoop(error.to_string()))?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let waker = Waker::from_proxy(event_loop.create_proxy());
    app.on_waker(waker);
    let mut runtime = Runtime::new(app);
    let loop_result = event_loop.run_app(&mut runtime);
    finish_run(
        runtime.fatal.take(),
        loop_result.err().map(|error| error.to_string()),
    )
}

/// Store `error` as the loop's fatal error unless an earlier one is already recorded.
/// The first failure is kept because later ones are usually consequences of it.
fn record_fatal(slot: &mut Option<PlatformError>, error: PlatformError) {
    if slot.is_none() {
        *slot = Some(error);
    }
}

/// Map the recorded fatal error and the event-loop outcome onto the result of
/// [`run`]. A recorded fatal error is the root cause, so it wins over a loop error.
fn finish_run(
    fatal: Option<PlatformError>,
    loop_error: Option<String>,
) -> Result<(), PlatformError> {
    match (fatal, loop_error) {
        (Some(error), _) => Err(error),
        (None, Some(error)) => Err(PlatformError::EventLoop(error)),
        (None, None) => Ok(()),
    }
}

fn notify_gpu_recovered(app: &mut dyn Application, window: WindowId) {
    app.on_gpu_recovered(window);
}

/// Resolve the IME candidate rectangle: prefer the app-provided caret area,
/// otherwise fall back to a 1?24 logical rect at the mouse cursor.
///
/// Pure helper so unit tests can verify wiring without a GPU or event loop.
pub fn resolve_ime_cursor_area(app_rect: Option<[f32; 4]>, mouse: (f32, f32)) -> [f32; 4] {
    app_rect.unwrap_or([mouse.0, mouse.1, 1.0, 24.0])
}

fn apply_ime_cursor_area(
    app: &mut dyn Application,
    window: &Window,
    win_id: WindowId,
    mouse: (f32, f32),
) {
    let rect = resolve_ime_cursor_area(app.ime_cursor_area(win_id), mouse);
    app.set_ime_cursor_area(win_id, rect);
    window.set_ime_cursor_area(
        LogicalPosition::new(rect[0] as f64, rect[1] as f64),
        LogicalSize::new(rect[2] as f64, rect[3] as f64),
    );
}

// Renderer dispatch is private: Application always emits the same backend-neutral Scene.
enum WindowRenderer {
    Gpu(Box<Renderer>),
    #[cfg(feature = "cpu-renderer")]
    Failed,
    #[cfg(feature = "cpu-renderer")]
    Cpu(CpuWindowRenderer),
}

impl WindowRenderer {
    fn resize(&mut self, width: u32, height: u32, scale: f32) -> bool {
        match self {
            Self::Gpu(renderer) => {
                renderer.resize(width, height, scale);
                true
            }
            #[cfg(feature = "cpu-renderer")]
            Self::Failed => false,
            #[cfg(feature = "cpu-renderer")]
            Self::Cpu(renderer) => {
                if let Err(error) = renderer.resize(width, height, scale) {
                    tracing::error!(%error, "CPU renderer resize failed");
                    false
                } else {
                    true
                }
            }
        }
    }

    fn dimensions(&self, fallback: (u32, u32)) -> (u32, u32) {
        match self {
            Self::Gpu(_) => fallback,
            #[cfg(feature = "cpu-renderer")]
            Self::Failed => (0, 0),
            #[cfg(feature = "cpu-renderer")]
            Self::Cpu(renderer) => renderer.dimensions(),
        }
    }

    fn scale_factor(&self, fallback: f32) -> f32 {
        match self {
            Self::Gpu(_) => fallback,
            #[cfg(feature = "cpu-renderer")]
            Self::Failed => fallback,
            #[cfg(feature = "cpu-renderer")]
            Self::Cpu(renderer) => renderer.scale_factor(),
        }
    }

    /// Render one frame. The second value carries the root cause when the
    /// action is [`SurfaceAction::Exit`], so `run()` can report it.
    fn render_scene(&mut self, scene: &Scene) -> (SurfaceAction, Option<String>) {
        match self {
            Self::Gpu(renderer) => (renderer.render_scene(scene), None),
            #[cfg(feature = "cpu-renderer")]
            Self::Failed => (
                SurfaceAction::Exit,
                Some("renderer is in the failed state after an earlier error".into()),
            ),
            #[cfg(feature = "cpu-renderer")]
            Self::Cpu(renderer) => match renderer.present(scene) {
                Ok(()) => (SurfaceAction::Rendered, None),
                Err(error) => {
                    tracing::error!(%error, "CPU renderer present failed");
                    (
                        SurfaceAction::Exit,
                        Some(format!("CPU renderer present failed: {error}")),
                    )
                }
            },
        }
    }
}

fn initialize_renderer(
    window: Arc<Window>,
    width: u32,
    height: u32,
    scale: f32,
    preference: RenderBackendPreference,
) -> Result<WindowRenderer, String> {
    match plan_renderer(preference, cfg!(feature = "cpu-renderer")) {
        RendererPlan::CpuOnly => {
            tracing::info!("Using forced CPU software renderer");
            create_cpu_renderer(window, width, height, scale)
        }
        RendererPlan::CpuUnavailable => Err(CPU_FEATURE_REQUIRED.into()),
        RendererPlan::GpuOnly => create_gpu_renderer(window, width, height, scale),
        RendererPlan::GpuThenCpu => match create_gpu_renderer(window.clone(), width, height, scale)
        {
            Ok(renderer) => Ok(renderer),
            Err(error) => {
                tracing::warn!(%error, "GPU initialization failed; falling back to CPU");
                create_cpu_renderer(window, width, height, scale)
            }
        },
    }
}

fn create_gpu_renderer(
    window: Arc<Window>,
    width: u32,
    height: u32,
    scale: f32,
) -> Result<WindowRenderer, String> {
    pollster::block_on(Renderer::new(window, width, height, scale))
        .map(|renderer| WindowRenderer::Gpu(Box::new(renderer)))
        .map_err(|error| error.to_string())
}

#[cfg(feature = "cpu-renderer")]
fn create_cpu_renderer(
    window: Arc<Window>,
    width: u32,
    height: u32,
    scale: f32,
) -> Result<WindowRenderer, String> {
    CpuWindowRenderer::new(window, width, height, scale).map(WindowRenderer::Cpu)
}

#[cfg(not(feature = "cpu-renderer"))]
fn create_cpu_renderer(
    _window: Arc<Window>,
    _width: u32,
    _height: u32,
    _scale: f32,
) -> Result<WindowRenderer, String> {
    Err(CPU_FEATURE_REQUIRED.into())
}

struct WindowState {
    id: WindowId,
    window: Arc<Window>,
    renderer: WindowRenderer,
    renderer_dims: (u32, u32),
    renderer_scale: f32,
    cursor: (f32, f32),
    shift: bool,
    ctrl: bool,
    alt: bool,
    meta: bool,
    dirty: bool,
    retry: FrameRetry,
    /// Last maximized state reported via `MaximizedChanged`; `None` until the first sync.
    maximized: Option<bool>,
}

struct Runtime<A> {
    app: A,
    windows: HashMap<WinitWindowId, WindowState>,
    next_window_id: u64,
    background_poll: BackgroundPoll,
    /// First unrecoverable error that stopped the loop; read back by [`run`].
    fatal: Option<PlatformError>,
}

impl<A> Runtime<A> {
    fn new(app: A) -> Self {
        Self {
            app,
            windows: HashMap::new(),
            next_window_id: 0,
            background_poll: BackgroundPoll::default(),
            fatal: None,
        }
    }
}

impl<A: Application> ApplicationHandler for Runtime<A> {
    fn user_event(&mut self, event_loop: &ActiveEventLoop, (): ()) {
        if self.app.wake() {
            for state in self.windows.values_mut() {
                state.dirty = true;
                state.window.request_redraw();
            }
        }
        process_window_requests(
            &mut self.app,
            &mut self.windows,
            &mut self.next_window_id,
            event_loop,
        );
    }

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        let now = std::time::Instant::now();
        if self
            .background_poll
            .advance(self.app.background_poll_interval(), now)
            && self.app.background_tick()
        {
            for state in self.windows.values() {
                state.window.request_redraw();
            }
        }
        let mut next = self.background_poll.deadline;
        for state in self.windows.values_mut() {
            if state.retry.take_due(now) {
                state.window.request_redraw();
            }
            next = next.into_iter().chain(state.retry.deadline).min();
        }
        event_loop.set_control_flow(next.map_or(ControlFlow::Wait, ControlFlow::WaitUntil));
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !self.windows.is_empty() {
            return;
        }
        let configs = self.app.windows();
        let minimum_size = self.app.minimum_window_size();
        let backend_preference = resolve_render_backend_from_env(self.app.render_backend());
        let mut failures = Vec::new();
        for config in configs {
            match create_window_state(
                event_loop,
                &config,
                minimum_size,
                backend_preference,
                None,
                &mut self.next_window_id,
            ) {
                Ok((winit_id, state)) => {
                    self.windows.insert(winit_id, state);
                }
                Err(error) => failures.push(error),
            }
        }
        if self.windows.is_empty() {
            if failures.is_empty() {
                failures.push("Application::windows() returned no window configurations".into());
            }
            tracing::error!(reasons = ?failures, "no windows could be created");
            record_fatal(
                &mut self.fatal,
                PlatformError::NoWindows { reasons: failures },
            );
            event_loop.exit();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        id: WinitWindowId,
        event: WindowEvent,
    ) {
        let Runtime {
            app,
            windows,
            next_window_id,
            fatal,
            ..
        } = self;

        let mut should_drain_requests = true;
        match event {
            WindowEvent::CloseRequested => {
                let Some(state) = windows.get(&id) else {
                    return;
                };
                if notify_close_requested(app, state.id) {
                    // Let requests from the closing source use the normal open-before-close
                    // order. Keep the post-event drain for window_closed on surviving sources.
                    if fatal.is_none() {
                        process_window_requests(app, windows, next_window_id, event_loop);
                    }
                    if windows.contains_key(&id) {
                        remove_window(app, windows, id, event_loop);
                    }
                } else {
                    state.window.request_redraw();
                }
            }
            WindowEvent::Destroyed => {
                if !windows.contains_key(&id) {
                    return;
                }
                remove_window(app, windows, id, event_loop);
            }
            WindowEvent::Resized(size) => {
                handle_resize(app, windows, id, Some(size), None);
            }
            WindowEvent::Moved(position) => {
                handle_moved(app, windows, id, position);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                handle_resize(app, windows, id, None, Some(scale_factor as f32));
            }
            WindowEvent::CursorMoved {
                position: PhysicalPosition { x, y },
                ..
            } => {
                handle_pointer_moved(app, windows, id, x, y);
            }
            WindowEvent::CursorEntered { .. } => {
                handle_cursor_enter(app, windows, id);
            }
            WindowEvent::CursorLeft { .. } => {
                handle_cursor_left(app, windows, id);
            }
            WindowEvent::Focused(gained) => {
                handle_focus(app, windows, id, gained);
            }
            WindowEvent::MouseInput {
                state: ms_state,
                button,
                ..
            } => {
                handle_mouse_input(app, windows, id, ms_state, button);
            }
            WindowEvent::MouseWheel { delta, .. } => {
                handle_scroll(app, windows, id, delta);
            }
            WindowEvent::ModifiersChanged(modifiers) => {
                handle_modifiers(windows, id, &modifiers.state());
                should_drain_requests = false;
            }
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                handle_keyboard(app, windows, id, key_event);
            }
            WindowEvent::Ime(Ime::Enabled) => {
                handle_ime_enabled(app, windows, id);
            }
            WindowEvent::Ime(Ime::Disabled) => {
                handle_ime_disabled(app, windows, id);
            }
            WindowEvent::Ime(Ime::Preedit(text, cursor)) => {
                handle_ime_preedit(app, windows, id, text, cursor);
            }
            WindowEvent::Ime(Ime::Commit(text)) => {
                handle_ime_commit(app, windows, id, text);
            }
            WindowEvent::DroppedFile(path) => {
                handle_file_drop(app, windows, id, path);
            }
            WindowEvent::HoveredFileCancelled | WindowEvent::HoveredFile(_) => {
                // Not currently mapped; available for future use.
                should_drain_requests = false;
            }
            WindowEvent::RedrawRequested => {
                handle_redraw(app, windows, id, event_loop, fatal);
            }
            _ => {
                should_drain_requests = false;
            }
        }
        if should_drain_requests && fatal.is_none() && !windows.is_empty() {
            process_window_requests(app, windows, next_window_id, event_loop);
        }
    }
}

/// Convert a [`Path`] to a [`String`], replacing invalid UTF-8 sequences with
/// U+FFFD replacement characters.
fn lossy_path_string(path: &std::path::Path) -> String {
    path.to_string_lossy().into_owned()
}

/// Allocate a new unique pointer ID for tracking pointer/touch identities.
///
/// Returns a monotonically increasing [`u64`]. The first call returns `1`;
/// `0` is reserved for the mouse pointer.
pub fn allocate_pointer_id() -> u64 {
    NEXT_POINTER_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_window_is_visible_sized() {
        let c = WindowConfig::default();
        assert!(c.width > 0.0 && c.height > 0.0);
    }

    #[test]
    fn default_window_config_keeps_native_decorations_with_shadow() {
        let c = WindowConfig::default();
        assert!(c.decorations);
        assert!(c.undecorated_shadow);
    }

    #[test]
    fn maximized_transition_reports_first_sample_and_only_flips() {
        assert_eq!(maximized_transition(None, false), Some(false));
        assert_eq!(maximized_transition(None, true), Some(true));
        assert_eq!(maximized_transition(Some(false), false), None);
        assert_eq!(maximized_transition(Some(true), true), None);
        assert_eq!(maximized_transition(Some(false), true), Some(true));
        assert_eq!(maximized_transition(Some(true), false), Some(false));
    }
    #[test]
    fn platform_events_are_backend_agnostic() {
        assert_eq!(PlatformKey::Tab, PlatformKey::Tab);
    }
    #[test]
    fn window_close_requested_roundtrip() {
        let id = WindowId(42);
        let event = PlatformEvent::WindowCloseRequested(id);
        match event {
            PlatformEvent::WindowCloseRequested(wid) => assert_eq!(wid, id),
            _ => panic!("expected WindowCloseRequested"),
        }
    }
    #[test]
    fn window_id_equality() {
        assert_eq!(WindowId(1), WindowId(1));
        assert_ne!(WindowId(1), WindowId(2));
        let set: std::collections::HashSet<WindowId> = [WindowId(1), WindowId(1), WindowId(2)]
            .into_iter()
            .collect();
        assert_eq!(set.len(), 2);
    }
    #[test]
    fn multiple_window_configs() {
        struct MultiWindowApp;
        impl Application for MultiWindowApp {
            fn windows(&self) -> Vec<WindowConfig> {
                vec![
                    WindowConfig {
                        title: "Window 1".into(),
                        width: 800.0,
                        height: 600.0,
                        min_width: None,
                        ..Default::default()
                    },
                    WindowConfig {
                        title: "Window 2".into(),
                        width: 400.0,
                        height: 300.0,
                        min_width: Some(320.0),
                        min_height: Some(240.0),
                        position: Some((100.0, 120.0)),
                        ..Default::default()
                    },
                ]
            }
            fn frame(&mut self, _ctx: FrameContext) -> Scene {
                Scene::new()
            }
        }
        let app = MultiWindowApp;
        let configs = app.windows();
        assert_eq!(configs.len(), 2);
        assert_eq!(configs[0].title, "Window 1");
        assert_eq!(configs[0].min_width, None);
        assert_eq!(configs[0].min_height, None);
        assert_eq!(configs[0].position, None);
        assert_eq!(configs[1].title, "Window 2");
        assert_eq!(configs[1].min_width, Some(320.0));
        assert_eq!(configs[1].min_height, Some(240.0));
        assert_eq!(configs[1].position, Some((100.0, 120.0)));
    }

    #[test]
    fn resolve_min_window_size_prefers_per_window_over_app_default() {
        let default_cfg = WindowConfig::default();
        assert_eq!(resolve_min_window_size(&default_cfg, None), None);
        assert_eq!(
            resolve_min_window_size(&default_cfg, Some((400.0, 300.0))),
            Some((400.0, 300.0))
        );

        let override_cfg = WindowConfig {
            title: "Custom".into(),
            width: 800.0,
            height: 600.0,
            min_width: Some(640.0),
            min_height: Some(480.0),
            ..Default::default()
        };
        assert_eq!(
            resolve_min_window_size(&override_cfg, Some((400.0, 300.0))),
            Some((640.0, 480.0))
        );
        assert_eq!(
            resolve_min_window_size(&override_cfg, None),
            Some((640.0, 480.0))
        );
    }

    #[test]
    fn scroll_event_exposes_modifiers() {
        let scroll = PlatformEvent::Scroll {
            window: WindowId(1),
            delta_x: 0.0,
            delta_y: -1.0,
            lines: true,
            x: 0.0,
            y: 0.0,
            shift: true,
            ctrl: true,
            alt: false,
            meta: false,
        };
        assert_eq!(
            scroll.key_modifiers(),
            Some(KeyModifiers {
                shift: true,
                control: true,
                alt: false,
                meta: false,
            })
        );
    }

    #[test]
    fn resolve_scroll_delta_preserves_sign_and_marks_line_vs_pixel() {
        let (dx, dy, lines) = resolve_scroll_delta(MouseScrollDelta::LineDelta(-1.5, 2.0), 2.0);
        assert!(lines);
        assert!((dx - (-48.0)).abs() < f32::EPSILON);
        assert!((dy - 64.0).abs() < f32::EPSILON);

        let (px, py, pixel_lines) = resolve_scroll_delta(
            MouseScrollDelta::PixelDelta(PhysicalPosition::new(20.0, -40.0)),
            2.0,
        );
        assert!(!pixel_lines);
        assert!((px - 10.0).abs() < f32::EPSILON);
        assert!((py - (-20.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn physical_bounds_conversions_scale_and_handle_invalid_scale_factors() {
        let size = winit::dpi::PhysicalSize::new(2560, 1440);
        assert_eq!(physical_size_to_logical(size, 2.0), (1280.0, 720.0));
        assert_eq!(physical_size_to_logical(size, 0.0), (2560.0, 1440.0));
        assert_eq!(physical_size_to_logical(size, f32::NAN), (2560.0, 1440.0));

        let pos = PhysicalPosition::new(-300, 450);
        assert_eq!(physical_position_to_logical(pos, 1.5), (-200.0, 300.0));
        assert_eq!(physical_position_to_logical(pos, -1.0), (-300.0, 450.0));

        let moved = PlatformEvent::Moved {
            window: WindowId(3),
            x: -200.0,
            y: 300.0,
        };
        assert_eq!(
            moved,
            PlatformEvent::Moved {
                window: WindowId(3),
                x: -200.0,
                y: 300.0,
            }
        );
    }

    #[test]
    fn needs_resize_detects_dimension_or_scale_mismatch_and_ignores_zero_size() {
        assert!(!needs_resize((1280, 770), 1.0, (1280, 770), 1.0));
        assert!(!needs_resize((1600, 962), 1.25, (1600, 962), 1.25));

        // Stale width/height after cross-DPI monitor transitions must trigger resize.
        assert!(needs_resize((1043, 747), 1.0, (1280, 770), 1.0));
        assert!(needs_resize((1600, 927), 1.25, (1600, 962), 1.25));

        // Scale-factor change alone must also trigger resize.
        assert!(needs_resize((1280, 770), 1.0, (1280, 770), 1.25));
        assert!(needs_resize((1280, 770), 1.0, (1600, 962), 1.25));

        // Minimized / zero-sized windows should not trigger a resize.
        assert!(!needs_resize((1280, 770), 1.0, (0, 770), 1.0));
        assert!(!needs_resize((1280, 770), 1.0, (1280, 0), 1.25));
        assert!(!needs_resize((1280, 770), 1.0, (0, 0), 1.25));
    }

    #[test]
    fn pointer_capture_coordinates_and_pointer_left_events_are_preserved() {
        // Negative and out-of-window coordinates during a captured drag must not clamp.
        assert_eq!(
            pointer_position_to_logical(-150.0, 2400.0, 1.5),
            (-100.0, 1600.0)
        );
        assert_eq!(
            pointer_position_to_logical(-40.0, -20.0, 0.0),
            (-40.0, -20.0)
        );
        assert_eq!(
            pointer_position_to_logical(30.0, -60.0, f32::NAN),
            (30.0, -60.0)
        );

        let events = cursor_left_events(WindowId(9));
        assert_eq!(
            events,
            [
                PlatformEvent::PointerLeft {
                    window: WindowId(9)
                },
                PlatformEvent::CursorLeft {
                    window: WindowId(9)
                },
            ]
        );
    }

    #[test]
    fn application_frame_can_stream_backend_neutral_rgba8_images() {
        struct ImageApp {
            revision: u64,
            width: u32,
            height: u32,
        }

        impl Application for ImageApp {
            fn frame(&mut self, _ctx: FrameContext) -> Scene {
                use acme_core::{DrawCommand, ImageKey, ImagePrimitive, Rect, Rgba8Image};

                let pixels = vec![0x80; self.width as usize * self.height as usize * 4];
                let image = Rgba8Image::new(
                    ImageKey::new(0xCAFE),
                    self.revision,
                    self.width,
                    self.height,
                    pixels,
                )
                .unwrap();
                let mut scene = Scene::new();
                scene.push(DrawCommand::Image(ImagePrimitive::contain(
                    image,
                    Rect::new(10.0, 20.0, 640.0, 360.0),
                )));
                scene
            }
        }

        let mut app = ImageApp {
            revision: 0,
            width: 2,
            height: 1,
        };
        let ctx = FrameContext {
            window: WindowId(1),
            logical_width: 800.0,
            logical_height: 600.0,
            scale_factor: 1.0,
        };
        let first = app.frame(ctx.clone());
        let acme_core::DrawCommand::Image(first_image) = &first.commands()[0] else {
            panic!("expected image command");
        };
        assert_eq!(first_image.image.revision(), 0);
        assert_eq!(
            (first_image.image.width(), first_image.image.height()),
            (2, 1)
        );

        app.revision += 1;
        app.width = 1;
        app.height = 2;
        let second = app.frame(ctx);
        let acme_core::DrawCommand::Image(second_image) = &second.commands()[0] else {
            panic!("expected image command");
        };
        assert_eq!(second_image.image.key(), first_image.image.key());
        assert_eq!(second_image.image.revision(), 1);
        assert_eq!(
            (second_image.image.width(), second_image.image.height()),
            (1, 2)
        );
    }

    #[test]
    fn close_requested_defaults_to_closing_after_the_event_is_delivered() {
        struct DefaultApp {
            events: usize,
        }
        impl Application for DefaultApp {
            fn event(&mut self, event: PlatformEvent) -> bool {
                if matches!(event, PlatformEvent::WindowCloseRequested(_)) {
                    self.events += 1;
                }
                false
            }
            fn frame(&mut self, _ctx: FrameContext) -> Scene {
                Scene::new()
            }
        }
        let mut app = DefaultApp { events: 0 };
        assert!(notify_close_requested(&mut app, WindowId(3)));
        assert_eq!(app.events, 1);
    }

    #[test]
    fn close_requested_can_veto_and_sees_the_event_first() {
        struct VetoApp {
            log: Vec<&'static str>,
        }
        impl Application for VetoApp {
            fn event(&mut self, event: PlatformEvent) -> bool {
                if matches!(event, PlatformEvent::WindowCloseRequested(_)) {
                    self.log.push("event");
                }
                false
            }
            fn close_requested(&mut self, window: WindowId) -> bool {
                assert_eq!(window, WindowId(7));
                self.log.push("decide");
                false
            }
            fn frame(&mut self, _ctx: FrameContext) -> Scene {
                Scene::new()
            }
        }
        let mut app = VetoApp { log: Vec::new() };
        assert!(!notify_close_requested(&mut app, WindowId(7)));
        assert_eq!(app.log, vec!["event", "decide"]);
    }

    #[test]
    fn on_gpu_recovered_default_is_noop_and_overridable() {
        struct RecoveryApp {
            recovered: Vec<WindowId>,
        }
        impl Application for RecoveryApp {
            fn on_gpu_recovered(&mut self, window: WindowId) {
                self.recovered.push(window);
            }
            fn frame(&mut self, _ctx: FrameContext) -> Scene {
                Scene::new()
            }
        }

        // Default impl is a no-op (compiles + does nothing).
        struct DefaultApp;
        impl Application for DefaultApp {
            fn frame(&mut self, _ctx: FrameContext) -> Scene {
                Scene::new()
            }
        }
        let mut default_app = DefaultApp;
        notify_gpu_recovered(&mut default_app, WindowId(1)); // must not panic

        // Overridden impl records recovery notifications through the runtime helper.
        let mut app = RecoveryApp {
            recovered: Vec::new(),
        };
        notify_gpu_recovered(&mut app, WindowId(3));
        notify_gpu_recovered(&mut app, WindowId(5));
        assert_eq!(app.recovered, vec![WindowId(3), WindowId(5)]);
    }

    #[test]
    fn frame_context_carries_window_id() {
        let id = WindowId(7);
        let ctx = FrameContext {
            window: id,
            logical_width: 800.0,
            logical_height: 600.0,
            scale_factor: 2.0,
        };
        assert_eq!(ctx.window, id);
        assert!((ctx.logical_width - 800.0).abs() < f32::EPSILON);
        assert!((ctx.logical_height - 600.0).abs() < f32::EPSILON);
        assert!((ctx.scale_factor - 2.0).abs() < f32::EPSILON);
    }

    #[test]
    fn resolve_ime_cursor_area_prefers_app_rect_over_mouse() {
        let app = Some([10.0, 20.0, 3.0, 24.0]);
        let resolved = resolve_ime_cursor_area(app, (999.0, 888.0));
        assert_eq!(resolved, [10.0, 20.0, 3.0, 24.0]);
    }

    #[test]
    fn resolve_ime_cursor_area_falls_back_to_mouse_when_none() {
        let resolved = resolve_ime_cursor_area(None, (42.0, 84.0));
        assert_eq!(resolved, [42.0, 84.0, 1.0, 24.0]);
    }

    #[test]
    fn ime_cursor_area_default_is_none() {
        struct DefaultApp;
        impl Application for DefaultApp {
            fn frame(&mut self, _ctx: FrameContext) -> Scene {
                Scene::new()
            }
        }
        let app = DefaultApp;
        assert_eq!(app.ime_cursor_area(WindowId(1)), None);
        let resolved = resolve_ime_cursor_area(app.ime_cursor_area(WindowId(1)), (5.0, 6.0));
        assert_eq!(resolved, [5.0, 6.0, 1.0, 24.0]);
    }

    #[test]
    fn background_polling_is_opt_in() {
        struct DefaultApp;
        impl Application for DefaultApp {
            fn frame(&mut self, _ctx: FrameContext) -> Scene {
                Scene::new()
            }
        }
        let mut app = DefaultApp;
        assert_eq!(app.background_poll_interval(), None);
        assert!(!app.background_tick());
    }

    #[test]
    fn ime_cursor_area_override_is_used_by_resolver() {
        struct CaretApp;
        impl Application for CaretApp {
            fn ime_cursor_area(&self, _window: WindowId) -> Option<[f32; 4]> {
                Some([10.0, 20.0, 3.0, 24.0])
            }
            fn frame(&mut self, _ctx: FrameContext) -> Scene {
                Scene::new()
            }
        }
        let app = CaretApp;
        let resolved = resolve_ime_cursor_area(app.ime_cursor_area(WindowId(0)), (1.0, 2.0));
        assert_eq!(resolved, [10.0, 20.0, 3.0, 24.0]);
    }

    #[test]
    fn clean_shutdown_without_fatal_error_is_ok() {
        assert!(finish_run(None, None).is_ok());
    }

    #[test]
    fn recorded_fatal_error_is_returned_by_run_outcome() {
        let mut slot = None;
        record_fatal(&mut slot, PlatformError::Renderer("device gone".into()));
        match finish_run(slot, None) {
            Err(PlatformError::Renderer(message)) => assert_eq!(message, "device gone"),
            other => panic!("expected Renderer error, got {other:?}"),
        }
    }

    #[test]
    fn fatal_error_wins_over_event_loop_error() {
        let fatal = Some(PlatformError::Renderer("root cause".into()));
        match finish_run(fatal, Some("loop broke".into())) {
            Err(PlatformError::Renderer(message)) => assert_eq!(message, "root cause"),
            other => panic!("expected Renderer error, got {other:?}"),
        }
    }

    #[test]
    fn event_loop_error_is_reported_without_fatal_error() {
        match finish_run(None, Some("loop broke".into())) {
            Err(PlatformError::EventLoop(message)) => assert_eq!(message, "loop broke"),
            other => panic!("expected EventLoop error, got {other:?}"),
        }
    }

    #[test]
    fn exit_error_message_includes_root_cause_when_known() {
        assert_eq!(
            exit_error_message(WindowId(2), Some("CPU renderer present failed: boom")),
            "renderer requested exit for window 2: CPU renderer present failed: boom"
        );
        assert_eq!(
            exit_error_message(WindowId(2), None),
            "renderer requested exit for window 2"
        );
    }

    #[test]
    fn first_fatal_error_wins() {
        let mut slot = None;
        record_fatal(&mut slot, PlatformError::Renderer("first".into()));
        record_fatal(&mut slot, PlatformError::Renderer("second".into()));
        assert!(matches!(slot, Some(PlatformError::Renderer(ref m)) if m == "first"));
    }

    #[test]
    fn no_windows_error_lists_every_failure_reason() {
        let error = PlatformError::NoWindows {
            reasons: vec![
                "window 'A': os refused".into(),
                "renderer for window 'B': no adapter".into(),
            ],
        };
        let text = error.to_string();
        assert!(text.contains("no window could be created"), "{text}");
        assert!(text.contains("window 'A': os refused"), "{text}");
        assert!(
            text.contains("renderer for window 'B': no adapter"),
            "{text}"
        );
    }

    #[test]
    fn no_windows_error_is_returned_as_run_outcome() {
        let mut slot = None;
        record_fatal(
            &mut slot,
            PlatformError::NoWindows {
                reasons: vec!["no configs".into()],
            },
        );
        match finish_run(slot, None) {
            Err(PlatformError::NoWindows { reasons }) => {
                assert_eq!(reasons, vec!["no configs".to_string()]);
            }
            other => panic!("expected NoWindows error, got {other:?}"),
        }
    }

    #[test]
    fn renderer_error_display_names_the_failure() {
        let text = PlatformError::Renderer("GPU device recovery failed (x)".into()).to_string();
        assert_eq!(
            text,
            "unrecoverable renderer failure: GPU device recovery failed (x)"
        );
    }

    #[test]
    fn key_modifiers_flags_and_primary_shortcut_work() {
        let empty = KeyModifiers::default();
        assert!(empty.is_empty());
        assert!(!empty.any());
        assert!(!empty.primary_shortcut());
        assert!(!empty.primary_shortcut_for_macos(true));
        assert!(!empty.primary_shortcut_for_macos(false));

        let ctrl_only = KeyModifiers {
            control: true,
            ..Default::default()
        };
        assert!(!ctrl_only.is_empty());
        assert!(ctrl_only.any());
        assert!(ctrl_only.primary_shortcut_for_macos(false));
        assert!(!ctrl_only.primary_shortcut_for_macos(true));

        let meta_only = KeyModifiers {
            meta: true,
            ..Default::default()
        };
        assert!(meta_only.any());
        assert!(meta_only.primary_shortcut_for_macos(true));
        assert!(!meta_only.primary_shortcut_for_macos(false));

        let expected_platform_primary = if cfg!(any(target_os = "macos", target_os = "ios")) {
            meta_only.primary_shortcut() && !ctrl_only.primary_shortcut()
        } else {
            ctrl_only.primary_shortcut() && !meta_only.primary_shortcut()
        };
        assert!(expected_platform_primary);
    }

    #[test]
    fn platform_event_key_modifiers_and_key_input_extract_fields() {
        let key_event = PlatformEvent::Key {
            window: WindowId(7),
            key: PlatformKey::Enter,
            pressed: true,
            shift: true,
            ctrl: true,
            alt: false,
            meta: false,
            repeat: true,
            text: Some("\r".into()),
            character: None,
        };
        let expected_mods = KeyModifiers {
            shift: true,
            control: true,
            alt: false,
            meta: false,
        };
        assert_eq!(key_event.key_modifiers(), Some(expected_mods));
        let input = key_event.key_input().expect("Key event yields KeyInput");
        assert_eq!(
            input,
            KeyInput {
                window: WindowId(7),
                key: PlatformKey::Enter,
                pressed: true,
                modifiers: expected_mods,
                repeat: true,
                text: Some("\r".into()),
                character: None,
            }
        );

        let pointer_event = PlatformEvent::PointerButton {
            window: WindowId(7),
            pressed: true,
            x: 10.0,
            y: 20.0,
            button: 0,
            pointer: 0,
            shift: false,
            ctrl: true,
            alt: true,
        };
        assert_eq!(
            pointer_event.key_modifiers(),
            Some(KeyModifiers {
                shift: false,
                control: true,
                alt: true,
                meta: false,
            })
        );
        assert_eq!(pointer_event.key_input(), None);

        let resize_event = PlatformEvent::Resized {
            window: WindowId(7),
            logical_width: 800.0,
            logical_height: 600.0,
            scale_factor: 1.0,
        };
        assert_eq!(resize_event.key_modifiers(), None);
        assert_eq!(resize_event.key_input(), None);
    }

    #[test]
    fn key_input_is_primary_shortcut_checks_pressed_key_and_platform_modifier() {
        let primary_mods = if cfg!(any(target_os = "macos", target_os = "ios")) {
            KeyModifiers {
                meta: true,
                ..Default::default()
            }
        } else {
            KeyModifiers {
                control: true,
                ..Default::default()
            }
        };

        let pressed_match = KeyInput {
            window: WindowId(1),
            key: PlatformKey::Enter,
            pressed: true,
            modifiers: primary_mods,
            repeat: false,
            text: None,
            character: None,
        };
        assert!(pressed_match.is_primary_shortcut(PlatformKey::Enter));
        assert!(!pressed_match.is_primary_shortcut(PlatformKey::Tab));

        let released = KeyInput {
            pressed: false,
            ..pressed_match.clone()
        };
        assert!(!released.is_primary_shortcut(PlatformKey::Enter));

        let no_mods = KeyInput {
            modifiers: KeyModifiers::default(),
            ..pressed_match
        };
        assert!(!no_mods.is_primary_shortcut(PlatformKey::Enter));
    }

    #[derive(Default)]
    struct LifecycleApp {
        log: Vec<String>,
    }

    impl Application for LifecycleApp {
        fn window_opened(&mut self, token: u64, window: WindowId) {
            self.log.push(format!("opened {token} -> {}", window.0));
        }
        fn window_open_failed(&mut self, token: u64, error: String) {
            self.log.push(format!("failed {token}: {error}"));
        }
        fn window_closed(&mut self, window: WindowId) {
            self.log.push(format!("closed {}", window.0));
        }
        fn frame(&mut self, _ctx: FrameContext) -> Scene {
            Scene::new()
        }
    }

    #[test]
    fn window_config_is_clone_and_partial_eq() {
        let config = WindowConfig {
            title: "Module".into(),
            position: Some((10.0, 20.0)),
            ..Default::default()
        };
        assert_eq!(config.clone(), config);
        assert_ne!(config, WindowConfig::default());
    }

    #[test]
    fn window_lifecycle_callbacks_default_to_noops() {
        struct DefaultApp;
        impl Application for DefaultApp {
            fn frame(&mut self, _ctx: FrameContext) -> Scene {
                Scene::new()
            }
        }
        let mut app = DefaultApp;
        app.window_opened(1, WindowId(3));
        app.window_open_failed(2, "boom".into());
        app.window_closed(WindowId(3));
        notify_window_open_results(&mut app, vec![(1, Ok(WindowId(4)))]);
        assert!(finish_window_removal(&mut app, Some(WindowId(4)), 0));
    }

    #[cfg(all(windows, feature = "cpu-renderer"))]
    #[test]
    #[ignore = "requires a Windows desktop session"]
    fn native_close_opens_replacement_and_destroyed_drains_survivor() {
        use winit::platform::windows::EventLoopBuilderExtWindows;

        #[derive(Default)]
        struct App {
            pending: HashMap<WindowId, Vec<WindowRequest>>,
            lifecycle: LifecycleApp,
        }

        impl Application for App {
            fn render_backend(&self) -> RenderBackendPreference {
                RenderBackendPreference::Cpu
            }

            fn event(&mut self, event: PlatformEvent) -> bool {
                if let PlatformEvent::WindowCloseRequested(window) = event {
                    self.pending
                        .entry(window)
                        .or_default()
                        .push(WindowRequest::Open {
                            token: 1,
                            config: WindowConfig::default(),
                            owner: None,
                        });
                }
                false
            }

            fn close_requested(&mut self, window: WindowId) -> bool {
                self.pending
                    .entry(window)
                    .or_default()
                    .push(WindowRequest::Open {
                        token: 2,
                        config: WindowConfig::default(),
                        owner: None,
                    });
                true
            }

            fn take_window_requests(&mut self, window: WindowId) -> Vec<WindowRequest> {
                self.pending.remove(&window).unwrap_or_default()
            }

            fn window_opened(&mut self, token: u64, window: WindowId) {
                self.lifecycle.window_opened(token, window);
            }

            fn window_open_failed(&mut self, token: u64, error: String) {
                self.lifecycle.window_open_failed(token, error);
            }

            fn window_closed(&mut self, window: WindowId) {
                self.lifecycle.window_closed(window);
                if window == WindowId(2) {
                    self.pending.entry(WindowId(1)).or_default().extend([
                        WindowRequest::Open {
                            token: 3,
                            config: WindowConfig::default(),
                            owner: Some(window),
                        },
                        WindowRequest::Open {
                            token: 4,
                            config: WindowConfig::default(),
                            owner: Some(WindowId(1)),
                        },
                    ]);
                }
            }

            fn frame(&mut self, _ctx: FrameContext) -> Scene {
                Scene::new()
            }
        }

        struct Driver {
            runtime: Runtime<App>,
            exit_after_close: Option<bool>,
            exit_after_destroyed: Option<bool>,
        }
        impl ApplicationHandler for Driver {
            fn resumed(&mut self, event_loop: &ActiveEventLoop) {
                let runtime = &mut self.runtime;
                runtime.resumed(event_loop);
                if let Some((&initial, _)) = runtime.windows.iter().next() {
                    runtime.window_event(event_loop, initial, WindowEvent::CloseRequested);
                    self.exit_after_close = Some(event_loop.exiting());
                }
                if let Some((&destroyed, _)) = runtime
                    .windows
                    .iter()
                    .find(|(_, state)| state.id == WindowId(2))
                {
                    runtime.window_event(event_loop, destroyed, WindowEvent::Destroyed);
                    runtime.window_event(event_loop, destroyed, WindowEvent::Destroyed);
                    self.exit_after_destroyed = Some(event_loop.exiting());
                }
                event_loop.exit();
            }

            fn window_event(&mut self, _: &ActiveEventLoop, _: WinitWindowId, _: WindowEvent) {}
        }

        let event_loop = EventLoop::builder().with_any_thread(true).build().unwrap();
        let mut driver = Driver {
            runtime: Runtime::new(App::default()),
            exit_after_close: None,
            exit_after_destroyed: None,
        };
        event_loop.run_app(&mut driver).unwrap();
        // Check outside the winit callback so an assertion propagates to the test harness.
        assert_eq!(driver.exit_after_close, Some(false));
        assert_eq!(driver.exit_after_destroyed, Some(false));
        let mut live: Vec<_> = driver
            .runtime
            .windows
            .values()
            .map(|state| state.id.0)
            .collect();
        live.sort_unstable();
        assert_eq!(live, [1, 3]);
        let log = &driver.runtime.app.lifecycle.log;
        assert_eq!(log.len(), 6);
        assert_eq!(
            &log[..4],
            ["opened 1 -> 1", "opened 2 -> 2", "closed 0", "closed 2"]
        );
        assert!(log[4].starts_with("failed 3: owner window 2 is not open"));
        assert_eq!(log[5], "opened 4 -> 3");
    }

    #[test]
    fn window_open_results_are_reported_in_request_order() {
        let mut app = LifecycleApp::default();
        notify_window_open_results(
            &mut app,
            vec![
                (30, Ok(WindowId(2))),
                (10, Err("owner window 9 is not open".into())),
                (20, Ok(WindowId(3))),
            ],
        );
        assert_eq!(
            app.log,
            vec![
                "opened 30 -> 2".to_string(),
                "failed 10: owner window 9 is not open".to_string(),
                "opened 20 -> 3".to_string(),
            ]
        );
    }

    #[test]
    fn closing_a_non_last_window_keeps_the_loop_running() {
        let mut app = LifecycleApp::default();
        assert!(!finish_window_removal(&mut app, Some(WindowId(1)), 1));
        assert!(finish_window_removal(&mut app, Some(WindowId(0)), 0));
        // A window that was already removed is not reported twice.
        assert!(finish_window_removal(&mut app, None, 0));
        assert!(!finish_window_removal(&mut app, None, 2));
        assert_eq!(
            app.log,
            vec!["closed 1".to_string(), "closed 0".to_string()]
        );
    }

    #[test]
    fn center_over_owner_centres_smaller_child() {
        assert_eq!(
            center_over_owner((100, 50), (1200, 800), (400, 300)),
            (500, 300)
        );
        // Odd remainders round toward the owner's top-left.
        assert_eq!(center_over_owner((0, 0), (101, 101), (50, 50)), (25, 25));
        // Negative (multi-monitor) owner coordinates are preserved.
        assert_eq!(
            center_over_owner((-1920, -200), (1000, 600), (200, 100)),
            (-1520, 50)
        );
    }

    #[test]
    fn center_over_owner_clamps_larger_child_to_owner_origin() {
        assert_eq!(
            center_over_owner((100, 50), (400, 300), (800, 200)),
            (100, 100)
        );
        assert_eq!(
            center_over_owner((100, 50), (400, 300), (800, 600)),
            (100, 50)
        );
    }

    #[test]
    fn logical_to_physical_size_scales_and_guards_scale() {
        assert_eq!(logical_to_physical_size(400.0, 300.0, 1.5), (600, 450));
        assert_eq!(logical_to_physical_size(400.0, 300.0, 0.0), (400, 300));
        assert_eq!(logical_to_physical_size(400.0, 300.0, f64::NAN), (400, 300));
        assert_eq!(logical_to_physical_size(-5.0, 10.4, 1.0), (0, 10));
    }
}

// ---------------------------------------------------------------------------
// Helper functions for window_event dispatch (free functions)
// ---------------------------------------------------------------------------

/// Remove `id` from the runtime, notify [`Application::window_closed`] once the
/// OS window is gone, and exit the loop only when no window remains.
fn remove_window(
    app: &mut dyn Application,
    windows: &mut HashMap<WinitWindowId, WindowState>,
    id: WinitWindowId,
    event_loop: &ActiveEventLoop,
) {
    // Dropping the state drops the last `Arc<Window>`, destroying the OS window.
    let removed = windows.remove(&id).map(|state| state.id);
    if finish_window_removal(app, removed, windows.len()) {
        event_loop.exit();
    }
}

/// Notify the app about a removed window and return whether the event loop
/// should exit (`true` only when no window remains).
pub(crate) fn finish_window_removal(
    app: &mut dyn Application,
    removed: Option<WindowId>,
    remaining: usize,
) -> bool {
    if let Some(window) = removed {
        app.window_closed(window);
    }
    remaining == 0
}

/// Report runtime window-open outcomes to the app in request order.
pub(crate) fn notify_window_open_results(
    app: &mut dyn Application,
    results: Vec<(u64, Result<WindowId, String>)>,
) {
    for (token, result) in results {
        match result {
            Ok(window) => app.window_opened(token, window),
            Err(error) => app.window_open_failed(token, error),
        }
    }
}

/// Centre a child window over its owner, in physical pixels.
///
/// `owner_pos`/`owner_size` are the owner's outer top-left and outer size;
/// `child_size` is the child's size. The result never lies above or left of the
/// owner's top-left corner, so a child larger than its owner keeps its title bar
/// reachable.
pub(crate) fn center_over_owner(
    owner_pos: (i32, i32),
    owner_size: (u32, u32),
    child_size: (u32, u32),
) -> (i32, i32) {
    let axis = |pos: i32, owner: u32, child: u32| -> i32 {
        let offset = (i64::from(owner) - i64::from(child)) / 2;
        let value =
            (i64::from(pos) + offset.max(0)).clamp(i64::from(i32::MIN), i64::from(i32::MAX));
        value as i32
    };
    (
        axis(owner_pos.0, owner_size.0, child_size.0),
        axis(owner_pos.1, owner_size.1, child_size.1),
    )
}

/// Physical child size for a logical [`WindowConfig`] size at `scale_factor`.
pub(crate) fn logical_to_physical_size(width: f64, height: f64, scale_factor: f64) -> (u32, u32) {
    let scale = if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    };
    let convert = |v: f64| (v.max(0.0) * scale).round().min(f64::from(u32::MAX)) as u32;
    (convert(width), convert(height))
}

/// Position of a window opened over `owner` without an explicit position.
fn owner_centered_position(owner: &Window, config: &WindowConfig) -> Option<PhysicalPosition<i32>> {
    let owner_pos = owner.outer_position().ok()?;
    let owner_size = owner.outer_size();
    let child = logical_to_physical_size(config.width, config.height, owner.scale_factor());
    let (x, y) = center_over_owner(
        (owner_pos.x, owner_pos.y),
        (owner_size.width, owner_size.height),
        child,
    );
    Some(PhysicalPosition::new(x, y))
}

#[cfg(windows)]
fn owner_hwnd(window: &Window) -> Option<isize> {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
    match window.window_handle().ok()?.as_raw() {
        RawWindowHandle::Win32(handle) => Some(handle.hwnd.get()),
        _ => None,
    }
}

/// Create one OS window plus its renderer. Shared by start-up
/// ([`ApplicationHandler::resumed`]) and [`WindowRequest::Open`].
///
/// A new [`WindowId`] is allocated from `next_window_id` only on success, and
/// the window gets a redraw request before it is returned. Errors are
/// human-readable reasons naming the window title.
fn create_window_state(
    event_loop: &ActiveEventLoop,
    config: &WindowConfig,
    minimum_size: Option<(f64, f64)>,
    backend_preference: RenderBackendPreference,
    owner: Option<&Window>,
    next_window_id: &mut u64,
) -> Result<(WinitWindowId, WindowState), String> {
    let mut attrs = Window::default_attributes()
        .with_title(&config.title)
        .with_inner_size(LogicalSize::new(config.width, config.height))
        .with_resizable(true)
        .with_decorations(config.decorations);
    #[cfg(windows)]
    {
        use winit::platform::windows::WindowAttributesExtWindows;
        attrs = attrs.with_undecorated_shadow(config.undecorated_shadow);
        if let Some(hwnd) = owner.and_then(owner_hwnd) {
            attrs = attrs.with_owner_window(hwnd);
        }
    }
    if let Some((width, height)) = resolve_min_window_size(config, minimum_size) {
        attrs = attrs.with_min_inner_size(LogicalSize::new(width, height));
    }
    if let Some((x, y)) = config.position {
        attrs = attrs.with_position(LogicalPosition::new(x, y));
    } else if let Some(position) = owner.and_then(|owner| owner_centered_position(owner, config)) {
        attrs = attrs.with_position(position);
    }
    let window = match event_loop.create_window(attrs) {
        Ok(window) => Arc::new(window),
        Err(error) => {
            tracing::error!(%error, "failed to create window: {}", config.title);
            return Err(format!("window '{}': {error}", config.title));
        }
    };
    window.set_ime_allowed(true);
    let size = window.inner_size();
    let scale = window.scale_factor() as f32;
    let renderer = match initialize_renderer(
        window.clone(),
        size.width,
        size.height,
        scale,
        backend_preference,
    ) {
        Ok(renderer) => renderer,
        Err(error) => {
            tracing::error!(
                %error,
                "renderer initialization failed for window: {}",
                config.title,
            );
            return Err(format!("renderer for window '{}': {error}", config.title));
        }
    };
    let winit_id = window.id();
    let win_id = WindowId(*next_window_id);
    *next_window_id += 1;
    window.request_redraw();
    Ok((
        winit_id,
        WindowState {
            id: win_id,
            window,
            renderer,
            renderer_dims: (size.width, size.height),
            renderer_scale: scale,
            cursor: (0.0, 0.0),
            shift: false,
            ctrl: false,
            alt: false,
            meta: false,
            dirty: true,
            retry: FrameRetry::default(),
            maximized: None,
        },
    ))
}

/// Deliver a close request to `app` and return whether the window should close.
pub(crate) fn notify_close_requested(app: &mut dyn Application, window: WindowId) -> bool {
    let _ = app.event(PlatformEvent::WindowCloseRequested(window));
    app.close_requested(window)
}

/// Return `true` when the renderer's configured physical dimensions or scale
/// factor differ from a non-zero target `inner_size` and `scale_factor`.
pub(crate) fn needs_resize(
    renderer_dims: (u32, u32),
    renderer_scale: f32,
    inner_size: (u32, u32),
    scale_factor: f32,
) -> bool {
    if inner_size.0 == 0 || inner_size.1 == 0 {
        return false;
    }
    renderer_dims != inner_size || (renderer_scale - scale_factor).abs() > f32::EPSILON
}

/// Decide whether a maximized-state report is due. `prev` is the last reported state
/// (`None` before the first report). Returns the new value to report, or `None` if unchanged.
pub(crate) fn maximized_transition(prev: Option<bool>, now: bool) -> Option<bool> {
    (prev != Some(now)).then_some(now)
}

/// Emit [`PlatformEvent::MaximizedChanged`] when the window's maximized state differs
/// from the last report. Returns `true` when the application requested a redraw.
fn sync_maximized(app: &mut dyn Application, state: &mut WindowState) -> bool {
    let now = state.window.is_maximized();
    let Some(maximized) = maximized_transition(state.maximized, now) else {
        return false;
    };
    state.maximized = Some(maximized);
    app.event(PlatformEvent::MaximizedChanged {
        window: state.id,
        maximized,
    })
}

fn handle_resize(
    app: &mut dyn Application,
    windows: &mut HashMap<WinitWindowId, WindowState>,
    id: WinitWindowId,
    size: Option<winit::dpi::PhysicalSize<u32>>,
    scale_override: Option<f32>,
) {
    let Some(state) = windows.get_mut(&id) else {
        return;
    };
    let physical = size.unwrap_or_else(|| state.window.inner_size());
    let (w, h) = (physical.width, physical.height);
    let scale = scale_override
        .filter(|&s| s > 0.0 && s.is_finite())
        .unwrap_or_else(|| state.window.scale_factor() as f32);
    let win_id = state.id;
    state.dirty |= sync_maximized(app, state);
    if state.renderer.resize(w, h, scale) {
        state.renderer_dims = (w, h);
        state.renderer_scale = scale;
    }
    let (logical_width, logical_height) = physical_size_to_logical(physical, scale);
    let dirty = app.event(PlatformEvent::Resized {
        window: win_id,
        logical_width,
        logical_height,
        scale_factor: scale,
    });
    state.dirty |= dirty;
    state.window.request_redraw();
}

fn handle_moved(
    app: &mut dyn Application,
    windows: &mut HashMap<WinitWindowId, WindowState>,
    id: WinitWindowId,
    position: PhysicalPosition<i32>,
) {
    let Some(state) = windows.get_mut(&id) else {
        return;
    };
    let scale = state.window.scale_factor() as f32;
    let win_id = state.id;
    let (x, y) = physical_position_to_logical(position, scale);
    let dirty = app.event(PlatformEvent::Moved {
        window: win_id,
        x,
        y,
    });
    state.dirty |= dirty;
    if state.dirty {
        state.window.request_redraw();
    }
}

fn handle_pointer_moved(
    app: &mut dyn Application,
    windows: &mut HashMap<WinitWindowId, WindowState>,
    id: WinitWindowId,
    x: f64,
    y: f64,
) {
    let Some(state) = windows.get_mut(&id) else {
        return;
    };
    let scale = state.window.scale_factor() as f32;
    let win_id = state.id;
    state.cursor = pointer_position_to_logical(x, y, scale);
    let dirty = app.event(PlatformEvent::PointerMoved {
        window: win_id,
        x: state.cursor.0,
        y: state.cursor.1,
    });
    state.dirty |= dirty;
    if state.dirty {
        state.window.request_redraw();
    }
}

fn handle_cursor_enter(
    app: &mut dyn Application,
    windows: &mut HashMap<WinitWindowId, WindowState>,
    id: WinitWindowId,
) {
    if let Some(state) = windows.get_mut(&id) {
        let win_id = state.id;
        let (x, y) = state.cursor;
        let dirty = app.event(PlatformEvent::CursorEntered {
            window: win_id,
            x,
            y,
        });
        state.dirty |= dirty;
        if state.dirty {
            state.window.request_redraw();
        }
    }
}

fn handle_cursor_left(
    app: &mut dyn Application,
    windows: &mut HashMap<WinitWindowId, WindowState>,
    id: WinitWindowId,
) {
    if let Some(state) = windows.get_mut(&id) {
        let win_id = state.id;
        let mut dirty = false;
        for event in cursor_left_events(win_id) {
            dirty |= app.event(event);
        }
        state.dirty |= dirty;
        if state.dirty {
            state.window.request_redraw();
        }
    }
}

fn handle_focus(
    app: &mut dyn Application,
    windows: &mut HashMap<WinitWindowId, WindowState>,
    id: WinitWindowId,
    gained: bool,
) {
    if let Some(state) = windows.get_mut(&id) {
        let win_id = state.id;
        let dirty = app.event(PlatformEvent::FocusChanged {
            window: win_id,
            gained,
            node_id: 0,
        });
        state.dirty |= dirty;
        if state.dirty {
            state.window.request_redraw();
        }
    }
}

fn handle_mouse_input(
    app: &mut dyn Application,
    windows: &mut HashMap<WinitWindowId, WindowState>,
    id: WinitWindowId,
    ms_state: ElementState,
    button: MouseButton,
) {
    let Some(state) = windows.get_mut(&id) else {
        return;
    };
    let win_id = state.id;
    let pressed = ms_state == ElementState::Pressed;
    let (x, y) = state.cursor;
    let button_code: u16 = match button {
        MouseButton::Left => 0,
        MouseButton::Right => 1,
        MouseButton::Middle => 2,
        MouseButton::Back => 3,
        MouseButton::Forward => 4,
        _ => 0,
    };
    let pointer = 0; // mouse pointer
    let shift = state.shift;
    let ctrl = state.ctrl;
    let alt = state.alt;

    let dirty = app.event(PlatformEvent::PointerButton {
        window: win_id,
        pressed,
        x,
        y,
        button: button_code,
        pointer,
        shift,
        ctrl,
        alt,
    });
    state.dirty |= dirty;
    if state.dirty {
        state.window.request_redraw();
    }
}

fn resolve_scroll_delta(delta: MouseScrollDelta, scale: f32) -> (f32, f32, bool) {
    match delta {
        MouseScrollDelta::LineDelta(x, y) => (x * 32.0, y * 32.0, true),
        MouseScrollDelta::PixelDelta(p) => (p.x as f32 / scale, p.y as f32 / scale, false),
    }
}

fn handle_scroll(
    app: &mut dyn Application,
    windows: &mut HashMap<WinitWindowId, WindowState>,
    id: WinitWindowId,
    delta: MouseScrollDelta,
) {
    let (scale, win_id, (x, y), (shift, ctrl, alt, meta)) = match windows.get(&id) {
        Some(state) => (
            state.window.scale_factor() as f32,
            state.id,
            state.cursor,
            (state.shift, state.ctrl, state.alt, state.meta),
        ),
        None => return,
    };
    let (delta_x, delta_y, lines) = resolve_scroll_delta(delta, scale);
    let dirty = app.event(PlatformEvent::Scroll {
        window: win_id,
        delta_x,
        delta_y,
        lines,
        x,
        y,
        shift,
        ctrl,
        alt,
        meta,
    });
    if let Some(state) = windows.get_mut(&id) {
        state.dirty |= dirty;
        if state.dirty {
            state.window.request_redraw();
        }
    }
}

fn handle_modifiers(
    windows: &mut HashMap<WinitWindowId, WindowState>,
    id: WinitWindowId,
    modifiers: &winit::keyboard::ModifiersState,
) {
    if let Some(state) = windows.get_mut(&id) {
        state.shift = modifiers.shift_key();
        state.ctrl = modifiers.control_key();
        state.alt = modifiers.alt_key();
        state.meta = modifiers.super_key();
    }
}

fn handle_keyboard(
    app: &mut dyn Application,
    windows: &mut HashMap<WinitWindowId, WindowState>,
    id: WinitWindowId,
    key_event: winit::event::KeyEvent,
) {
    let state = match windows.get(&id) {
        Some(s) => s,
        None => return,
    };
    let win_id = state.id;
    let shift = state.shift;
    let ctrl = state.ctrl;
    let alt = state.alt;
    let meta = state.meta;
    let repeat = key_event.repeat;
    let text = resolve_key_text(key_event.text.as_deref(), &key_event.logical_key);
    let character = resolve_key_character(&key_event.logical_key);
    let key = map_logical_key(&key_event.logical_key);
    let dirty = app.event(PlatformEvent::Key {
        window: win_id,
        key,
        pressed: key_event.state == ElementState::Pressed,
        shift,
        ctrl,
        alt,
        meta,
        repeat,
        text,
        character,
    });
    if let Some(state) = windows.get_mut(&id) {
        state.dirty |= dirty;
        if state.dirty {
            state.window.request_redraw();
        }
    }
}

fn process_window_requests<A: Application>(
    app: &mut A,
    windows: &mut HashMap<WinitWindowId, WindowState>,
    next_window_id: &mut u64,
    event_loop: &ActiveEventLoop,
) {
    let window_order: Vec<(WinitWindowId, WindowId)> = windows
        .iter()
        .map(|(&winit_id, state)| (winit_id, state.id))
        .collect();
    let batches = drain_window_requests(app, window_order.iter().map(|&(_, id)| id));
    let mut opens: Vec<OpenWindowRequest> = Vec::new();
    let mut closes: Vec<WinitWindowId> = Vec::new();
    for (win_id, requests) in batches {
        let Some(&(winit_id, _)) = window_order.iter().find(|&&(_, id)| id == win_id) else {
            continue;
        };
        let plan = plan_window_requests(requests);
        if let Some(state) = windows.get_mut(&winit_id) {
            for request in plan.immediate {
                apply_window_request(&state.window, request);
            }
            if plan.redraw {
                state.dirty = true;
                state.window.request_redraw();
            }
        }
        opens.extend(plan.opens);
        if plan.close {
            closes.push(winit_id);
        }
    }
    // Opens run before closes so an owner closed in the same pass still
    // positions and owns its child.
    if !opens.is_empty() {
        let results = open_requested_windows(app, windows, next_window_id, event_loop, opens);
        notify_window_open_results(app, results);
    }
    for winit_id in closes {
        remove_window(app, windows, winit_id, event_loop);
    }
}

/// Create the windows requested with [`WindowRequest::Open`], in order.
fn open_requested_windows<A: Application>(
    app: &mut A,
    windows: &mut HashMap<WinitWindowId, WindowState>,
    next_window_id: &mut u64,
    event_loop: &ActiveEventLoop,
    opens: Vec<OpenWindowRequest>,
) -> Vec<(u64, Result<WindowId, String>)> {
    let minimum_size = app.minimum_window_size();
    let backend_preference = resolve_render_backend_from_env(app.render_backend());
    let mut results = Vec::with_capacity(opens.len());
    for open in opens {
        let owner = match open.owner {
            Some(owner_id) => match windows.values().find(|state| state.id == owner_id) {
                Some(state) => Some(state.window.clone()),
                None => {
                    results.push((
                        open.token,
                        Err(format!(
                            "owner window {} is not open for window '{}'",
                            owner_id.0, open.config.title
                        )),
                    ));
                    continue;
                }
            },
            None => None,
        };
        let created = create_window_state(
            event_loop,
            &open.config,
            minimum_size,
            backend_preference,
            owner.as_deref(),
            next_window_id,
        );
        let result = created.map(|(winit_id, state)| {
            let id = state.id;
            windows.insert(winit_id, state);
            id
        });
        results.push((open.token, result));
    }
    results
}

/// Apply a request that targets an existing window directly.
fn apply_window_request(window: &Window, request: WindowRequest) {
    match request {
        WindowRequest::SetTitle(title) => {
            window.set_title(&title);
        }
        WindowRequest::Focus => {
            window.set_minimized(false);
            window.set_visible(true);
            window.focus_window();
        }
        WindowRequest::Minimize => {
            window.set_minimized(true);
        }
        WindowRequest::SetImeAllowed(allowed) => {
            window.set_ime_allowed(allowed);
        }
        WindowRequest::SetCursor(icon) => {
            window.set_cursor(map_cursor_icon(icon));
        }
        WindowRequest::ToggleMaximize => {
            window.set_maximized(!window.is_maximized());
        }
        WindowRequest::SetMaximized(maximized) => {
            window.set_maximized(maximized);
        }
        WindowRequest::DragMove => {
            if let Err(error) = window.drag_window() {
                tracing::warn!(%error, "window drag-move unavailable");
            }
        }
        WindowRequest::DragResize(edge) => {
            if let Err(error) = window.drag_resize_window(map_resize_edge(edge)) {
                tracing::warn!(%error, ?edge, "window drag-resize unavailable");
            }
        }
        WindowRequest::ShowSystemMenu { x, y } => {
            window.show_window_menu(LogicalPosition::new(f64::from(x), f64::from(y)));
        }
        // Split out by `plan_window_requests` and handled by the caller.
        WindowRequest::Close | WindowRequest::Open { .. } | WindowRequest::RequestRedraw => {}
    }
}

fn handle_ime_enabled(
    app: &mut dyn Application,
    windows: &mut HashMap<WinitWindowId, WindowState>,
    id: WinitWindowId,
) {
    if let Some(state) = windows.get_mut(&id) {
        let win_id = state.id;
        let dirty = app.event(PlatformEvent::ImeEnabled(win_id));
        state.dirty |= dirty;
        // App-authoritative caret rect; mouse only as fallback.
        let mouse = state.cursor;
        apply_ime_cursor_area(app, &state.window, win_id, mouse);
        if state.dirty {
            state.window.request_redraw();
        }
    }
}

fn handle_ime_disabled(
    app: &mut dyn Application,
    windows: &mut HashMap<WinitWindowId, WindowState>,
    id: WinitWindowId,
) {
    if let Some(state) = windows.get_mut(&id) {
        let win_id = state.id;
        let dirty = app.event(PlatformEvent::ImeDisabled(win_id));
        state.dirty |= dirty;
        // Reset IME cursor area to origin when IME is disabled
        app.set_ime_cursor_area(win_id, [0.0, 0.0, 0.0, 0.0]);
        state
            .window
            .set_ime_cursor_area(LogicalPosition::new(0.0, 0.0), LogicalSize::new(0.0, 0.0));
        if state.dirty {
            state.window.request_redraw();
        }
    }
}

fn handle_ime_preedit(
    app: &mut dyn Application,
    windows: &mut HashMap<WinitWindowId, WindowState>,
    id: WinitWindowId,
    text: String,
    cursor: Option<(usize, usize)>,
) {
    if let Some(state) = windows.get_mut(&id) {
        let win_id = state.id;
        // Prefer app caret geometry over mouse during composition.
        let mouse = state.cursor;
        apply_ime_cursor_area(app, &state.window, win_id, mouse);
        let dirty = app.event(PlatformEvent::ImePreedit {
            window: win_id,
            text,
            cursor,
        });
        state.dirty |= dirty;
        if state.dirty {
            state.window.request_redraw();
        }
    }
}

fn handle_ime_commit(
    app: &mut dyn Application,
    windows: &mut HashMap<WinitWindowId, WindowState>,
    id: WinitWindowId,
    text: String,
) {
    if let Some(state) = windows.get_mut(&id) {
        let win_id = state.id;
        let dirty = app.event(PlatformEvent::ImeCommit {
            window: win_id,
            text,
        });
        state.dirty |= dirty;
        if state.dirty {
            state.window.request_redraw();
        }
    }
}

fn handle_file_drop(
    app: &mut dyn Application,
    windows: &mut HashMap<WinitWindowId, WindowState>,
    id: WinitWindowId,
    path: std::path::PathBuf,
) {
    if let Some(state) = windows.get_mut(&id) {
        let win_id = state.id;
        let paths = vec![lossy_path_string(&path)];
        let dirty = app.event(PlatformEvent::FileDropped {
            window: win_id,
            paths,
        });
        state.dirty |= dirty;
        if state.dirty {
            state.window.request_redraw();
        }
    }
}

fn handle_redraw(
    app: &mut dyn Application,
    windows: &mut HashMap<WinitWindowId, WindowState>,
    id: WinitWindowId,
    event_loop: &ActiveEventLoop,
    fatal: &mut Option<PlatformError>,
) {
    let Some(state) = windows.get_mut(&id) else {
        return;
    };
    let size = state.window.inner_size();
    if size.width == 0 || size.height == 0 {
        return;
    }
    let scale = state.window.scale_factor() as f32;
    let win_id = state.id;
    let current_dims = state.renderer.dimensions(state.renderer_dims);
    let current_scale = state.renderer.scale_factor(state.renderer_scale);
    state.dirty |= sync_maximized(app, state);
    if needs_resize(
        current_dims,
        current_scale,
        (size.width, size.height),
        scale,
    ) {
        if state.renderer.resize(size.width, size.height, scale) {
            state.renderer_dims = (size.width, size.height);
            state.renderer_scale = scale;
        }
        let (logical_width, logical_height) = physical_size_to_logical(size, scale);
        state.dirty |= app.event(PlatformEvent::Resized {
            window: win_id,
            logical_width,
            logical_height,
            scale_factor: scale,
        });
    }
    let scene = app.frame(FrameContext {
        window: win_id,
        logical_width: size.width as f32 / scale,
        logical_height: size.height as f32 / scale,
        scale_factor: scale,
    });
    let (action, cause) = state.renderer.render_scene(&scene);
    match action {
        SurfaceAction::Exit => {
            let error = PlatformError::Renderer(exit_error_message(win_id, cause.as_deref()));
            stop_on_fatal(fatal, event_loop, error);
        }
        SurfaceAction::DeviceLost => {
            tracing::warn!("GPU device lost, attempting recovery...");
            let recovered = handle_device_lost(
                app,
                state,
                event_loop,
                fatal,
                size.width,
                size.height,
                scale,
            );
            if recovered {
                // Keep the window dirty so the redraw already requested repaints it.
                return;
            }
        }
        _ => {}
    }
    state.dirty = state.retry.after_frame(action, std::time::Instant::now());
}

/// Message for [`PlatformError::Renderer`] when a window's renderer requests exit.
fn exit_error_message(window: WindowId, cause: Option<&str>) -> String {
    match cause {
        Some(cause) => format!("renderer requested exit for window {}: {cause}", window.0),
        None => format!("renderer requested exit for window {}", window.0),
    }
}

/// Record `error` as the loop's fatal error and ask winit to stop.
fn stop_on_fatal(
    fatal: &mut Option<PlatformError>,
    event_loop: &ActiveEventLoop,
    error: PlatformError,
) {
    record_fatal(fatal, error);
    event_loop.exit();
}

/// Recover from a lost GPU device: rebuild GPU resources, or fall back to the CPU
/// renderer. Stops the loop with [`PlatformError::Renderer`] when both fail.
/// Returns `true` when the window has a working renderer again.
fn handle_device_lost(
    app: &mut dyn Application,
    state: &mut WindowState,
    event_loop: &ActiveEventLoop,
    fatal: &mut Option<PlatformError>,
    width: u32,
    height: u32,
    scale: f32,
) -> bool {
    let win_id = state.id;
    let recovery = match &mut state.renderer {
        WindowRenderer::Gpu(renderer) => renderer.on_device_lost().map_err(|e| e.to_string()),
        #[cfg(feature = "cpu-renderer")]
        WindowRenderer::Failed => Err("renderer already failed".into()),
        #[cfg(feature = "cpu-renderer")]
        WindowRenderer::Cpu(_) => Ok(()),
    };
    let gpu_error = match recovery {
        Ok(()) => {
            // GPU resources were rebuilt, so application-owned glyph caches
            // must be invalidated before the next frame.
            notify_gpu_recovered(app, win_id);
            state.dirty = true;
            state.window.request_redraw();
            return true;
        }
        Err(error) => error,
    };
    tracing::error!(error = %gpu_error, "GPU device recovery failed");
    match switch_to_cpu_after_device_loss(state, width, height, scale) {
        Ok(()) => {
            tracing::warn!("Switched from failed GPU recovery to CPU renderer");
            notify_gpu_recovered(app, win_id);
            state.dirty = true;
            state.window.request_redraw();
            true
        }
        Err(cpu_error) => {
            tracing::error!(error = %cpu_error, "CPU fallback also failed");
            let error = PlatformError::Renderer(format!(
                "GPU device recovery failed ({gpu_error}); CPU fallback failed ({cpu_error})"
            ));
            stop_on_fatal(fatal, event_loop, error);
            false
        }
    }
}

/// Replace the current renderer with a CPU software renderer for the same window.
/// The old GPU surface is released first so the native software surface can be acquired.
#[cfg(feature = "cpu-renderer")]
fn switch_to_cpu_after_device_loss(
    state: &mut WindowState,
    width: u32,
    height: u32,
    scale: f32,
) -> Result<(), String> {
    let old = std::mem::replace(&mut state.renderer, WindowRenderer::Failed);
    drop(old);
    let cpu = CpuWindowRenderer::new(state.window.clone(), width, height, scale)?;
    state.renderer = WindowRenderer::Cpu(cpu);
    state.renderer_dims = (width, height);
    state.renderer_scale = scale;
    Ok(())
}

#[cfg(not(feature = "cpu-renderer"))]
fn switch_to_cpu_after_device_loss(
    _state: &mut WindowState,
    _width: u32,
    _height: u32,
    _scale: f32,
) -> Result<(), String> {
    Err(CPU_FEATURE_REQUIRED.into())
}
