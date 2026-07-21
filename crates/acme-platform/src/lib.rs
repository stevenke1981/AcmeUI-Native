//! Windows-first winit runtime. Public events contain no winit platform types.
#![forbid(unsafe_op_in_unsafe_fn)]

mod clipboard;
pub use clipboard::Clipboard;

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
    keyboard::{Key, NamedKey},
    window::{Window, WindowId as WinitWindowId},
};

/// Global pointer/touch ID counter for tracking unique pointer identities.
static NEXT_POINTER_ID: AtomicU64 = AtomicU64::new(1);

/// A unique window identifier within the framework.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WindowId(pub u64);

#[derive(Clone, Debug)]
pub struct WindowConfig {
    pub title: String,
    pub width: f64,
    pub height: f64,
}
impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "AcmeUI Native".into(),
            width: 1100.0,
            height: 760.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PlatformEvent {
    // ?? Existing variants (backward-compatible, unchanged) ??
    Resized {
        window: WindowId,
        logical_width: f32,
        logical_height: f32,
        scale_factor: f32,
    },
    PointerMoved {
        window: WindowId,
        x: f32,
        y: f32,
    },
    PointerButton {
        window: WindowId,
        pressed: bool,
        x: f32,
        y: f32,
        button: u16,
        pointer: u64,
    },
    Scroll {
        window: WindowId,
        delta_y: f32,
    },
    Key {
        window: WindowId,
        key: PlatformKey,
        pressed: bool,
        shift: bool,
        ctrl: bool,
        text: Option<String>,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformKey {
    Tab,
    Enter,
    Space,
    Escape,
    ArrowLeft,
    ArrowRight,
    Backspace,
    Delete,
    Home,
    End,
    Other,
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
    /// Returns the configurations for all windows that should be created on startup.
    ///
    /// By default, returns a single window from [`window_config()`].
    /// Override this method to create multiple windows with different titles and sizes.
    ///
    /// # Example
    ///
    /// ```ignore
    /// fn windows(&self) -> Vec<WindowConfig> {
    ///     vec![
    ///         WindowConfig { title: "Editor".into(), width: 1200.0, height: 800.0 },
    ///         WindowConfig { title: "Inspector".into(), width: 400.0, height: 600.0 },
    ///     ]
    /// }
    /// ```
    fn windows(&self) -> Vec<WindowConfig> {
        vec![self.window_config()]
    }
    fn event(&mut self, _event: PlatformEvent) -> bool {
        false
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
    fn frame(&mut self, context: FrameContext) -> Scene;
}

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("event loop error: {0}")]
    EventLoop(String),
}

/// Runs an application until its final window closes.
pub fn run<A: Application>(app: A) -> Result<(), PlatformError> {
    let event_loop =
        EventLoop::new().map_err(|error| PlatformError::EventLoop(error.to_string()))?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut runtime = Runtime::new(app);
    event_loop
        .run_app(&mut runtime)
        .map_err(|error| PlatformError::EventLoop(error.to_string()))?;
    Ok(())
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

struct WindowState {
    id: WindowId,
    window: Arc<Window>,
    renderer: Renderer,
    cursor: (f32, f32),
    shift: bool,
    ctrl: bool,
    alt: bool,
    meta: bool,
    dirty: bool,
}

struct Runtime<A> {
    app: A,
    windows: HashMap<WinitWindowId, WindowState>,
    next_window_id: u64,
}

impl<A> Runtime<A> {
    fn new(app: A) -> Self {
        Self {
            app,
            windows: HashMap::new(),
            next_window_id: 0,
        }
    }
}

impl<A: Application> ApplicationHandler for Runtime<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !self.windows.is_empty() {
            return;
        }
        let configs = self.app.windows();
        for config in configs {
            let attrs = Window::default_attributes()
                .with_title(&config.title)
                .with_inner_size(LogicalSize::new(config.width, config.height))
                .with_resizable(true);
            let Ok(window) = event_loop.create_window(attrs) else {
                tracing::error!("failed to create window: {}", config.title);
                continue;
            };
            let window = Arc::new(window);
            window.set_ime_allowed(true);
            let size = window.inner_size();
            let scale = window.scale_factor() as f32;
            match pollster::block_on(Renderer::new(
                window.clone(),
                size.width,
                size.height,
                scale,
            )) {
                Ok(renderer) => {
                    let winit_id = window.id();
                    let win_id = WindowId(self.next_window_id);
                    self.next_window_id += 1;
                    self.windows.insert(
                        winit_id,
                        WindowState {
                            id: win_id,
                            window: window.clone(),
                            renderer,
                            cursor: (0.0, 0.0),
                            shift: false,
                            ctrl: false,
                            alt: false,
                            meta: false,
                            dirty: true,
                        },
                    );
                    window.request_redraw();
                }
                Err(error) => {
                    tracing::error!(
                        %error,
                        "renderer initialization failed for window: {}",
                        config.title,
                    );
                }
            }
        }
        if self.windows.is_empty() {
            tracing::error!("no windows could be created");
            event_loop.exit();
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        id: WinitWindowId,
        event: WindowEvent,
    ) {
        let Runtime { app, windows, .. } = self;


        match event {
            WindowEvent::CloseRequested => {
                handle_window_lifecycle(app, windows, id, event_loop, false);
            }
            WindowEvent::Destroyed => {
                handle_window_lifecycle(app, windows, id, event_loop, true);
            }
            WindowEvent::Resized(size) => {
                handle_resize(app, windows, id, Some(size));
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                handle_resize(app, windows, id, None);
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
            }
            WindowEvent::RedrawRequested => {
                handle_redraw(app, windows, id, event_loop);
            }
            _ => {}
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
                    },
                    WindowConfig {
                        title: "Window 2".into(),
                        width: 400.0,
                        height: 300.0,
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
        assert_eq!(configs[1].title, "Window 2");
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
}

// ---------------------------------------------------------------------------
// Helper functions for window_event dispatch (free functions)
// ---------------------------------------------------------------------------

fn handle_window_lifecycle(
    app: &mut dyn Application,
    windows: &mut HashMap<WinitWindowId, WindowState>,
    id: WinitWindowId,
    event_loop: &ActiveEventLoop,
    destroy: bool,
) {
    if !destroy {
        let win_id = windows.get(&id).map(|s| s.id);
        if let Some(win_id) = win_id {
            let _ = app.event(PlatformEvent::WindowCloseRequested(win_id));
        }
    }
    windows.remove(&id);
    if windows.is_empty() {
        event_loop.exit();
    }
}

fn handle_resize(
    app: &mut dyn Application,
    windows: &mut HashMap<WinitWindowId, WindowState>,
    id: WinitWindowId,
    size: Option<winit::dpi::PhysicalSize<u32>>,
) {
    let Some(state) = windows.get_mut(&id) else {
        return;
    };
    let (w, h) = match size {
        Some(s) => (s.width, s.height),
        None => {
            let s = state.window.inner_size();
            (s.width, s.height)
        }
    };
    let scale = state.window.scale_factor() as f32;
    let win_id = state.id;
    state.renderer.resize(w, h, scale);
    let dirty = match size {
        Some(_) => app.event(PlatformEvent::Resized {
            window: win_id,
            logical_width: w as f32 / scale,
            logical_height: h as f32 / scale,
            scale_factor: scale,
        }),
        None => true,
    };
    state.dirty |= dirty;
    state.window.request_redraw();
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
    state.cursor = (x as f32 / scale, y as f32 / scale);
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
        let dirty = app.event(PlatformEvent::CursorLeft { window: win_id });
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

    let dirty = app.event(PlatformEvent::PointerButton {
        window: win_id,
        pressed,
        x,
        y,
        button: button_code,
        pointer,
    });
    state.dirty |= dirty;
    if state.dirty {
        state.window.request_redraw();
    }
}

fn handle_scroll(
    app: &mut dyn Application,
    windows: &mut HashMap<WinitWindowId, WindowState>,
    id: WinitWindowId,
    delta: MouseScrollDelta,
) {
    let (scale, win_id) = match windows.get(&id) {
        Some(state) => (state.window.scale_factor() as f32, state.id),
        None => return,
    };
    let delta_y = match delta {
        MouseScrollDelta::LineDelta(_, y) => y * 32.0,
        MouseScrollDelta::PixelDelta(p) => p.y as f32 / scale,
    };
    let dirty = app.event(PlatformEvent::Scroll {
        window: win_id,
        delta_y,
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
    let text = match &key_event.logical_key {
        Key::Character(s) => Some(s.to_string()),
        _ => None,
    };
    let key = match key_event.logical_key.clone() {
        Key::Named(NamedKey::Tab) => PlatformKey::Tab,
        Key::Named(NamedKey::Enter) => PlatformKey::Enter,
        Key::Named(NamedKey::Space) => PlatformKey::Space,
        Key::Named(NamedKey::Escape) => PlatformKey::Escape,
        Key::Named(NamedKey::ArrowLeft) => PlatformKey::ArrowLeft,
        Key::Named(NamedKey::ArrowRight) => PlatformKey::ArrowRight,
        Key::Named(NamedKey::Backspace) => PlatformKey::Backspace,
        Key::Named(NamedKey::Delete) => PlatformKey::Delete,
        Key::Named(NamedKey::Home) => PlatformKey::Home,
        Key::Named(NamedKey::End) => PlatformKey::End,
        _ => PlatformKey::Other,
    };
    let dirty = app.event(PlatformEvent::Key {
        window: win_id,
        key,
        pressed: key_event.state == ElementState::Pressed,
        shift,
        ctrl,
        text,
    });
    if let Some(state) = windows.get_mut(&id) {
        state.dirty |= dirty;
        if state.dirty {
            state.window.request_redraw();
        }
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
        state.window.set_ime_cursor_area(
            LogicalPosition::new(0.0, 0.0),
            LogicalSize::new(0.0, 0.0),
        );
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
    windows: &HashMap<WinitWindowId, WindowState>,
    id: WinitWindowId,
    path: std::path::PathBuf,
) {
    if let Some(state) = windows.get(&id) {
        let win_id = state.id;
        let paths = vec![lossy_path_string(&path)];
        let _ = app.event(PlatformEvent::FileDropped {
            window: win_id,
            paths,
        });
    }
}

fn handle_redraw(
    app: &mut dyn Application,
    windows: &mut HashMap<WinitWindowId, WindowState>,
    id: WinitWindowId,
    event_loop: &ActiveEventLoop,
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
    let scene = app.frame(FrameContext {
        window: win_id,
        logical_width: size.width as f32 / scale,
        logical_height: size.height as f32 / scale,
        scale_factor: scale,
    });
    match state.renderer.render_scene(&scene) {
        SurfaceAction::Reconfigure => state.window.request_redraw(),
        SurfaceAction::Exit => event_loop.exit(),
        SurfaceAction::DeviceLost => {
            tracing::warn!("GPU device lost, attempting recovery...");
            match state.renderer.on_device_lost() {
                Ok(()) => {
                    // The renderer rebuilt EMPTY GPU atlases; the app owns the
                    // CPU atlas and must invalidate it so glyphs re-upload.
                    notify_gpu_recovered(app, win_id);
                    state.dirty = true;
                    state.window.request_redraw();
                }
                Err(e) => {
                    tracing::error!("device recovery failed: {e}");
                    event_loop.exit();
                }
            }
        }
        _ => {}
    }
    state.dirty = false;
}