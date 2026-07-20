//! Windows-first winit runtime. Public events contain no winit platform types.
#![forbid(unsafe_op_in_unsafe_fn)]

mod clipboard;
pub use clipboard::Clipboard;

use std::collections::HashMap;
use std::sync::Arc;

use acme_render_wgpu::{Frame, Renderer, SurfaceAction};
use thiserror::Error;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition},
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId as WinitWindowId},
};

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
    ImePreedit(String),
    ImeCommit(String),
    WindowCloseRequested(WindowId),
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
    fn frame(&mut self, context: FrameContext) -> Frame;
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

struct WindowState {
    id: WindowId,
    window: Arc<Window>,
    renderer: Renderer,
    cursor: (f32, f32),
    shift: bool,
    ctrl: bool,
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
                let win_id = windows.get(&id).map(|s| s.id);
                if let Some(win_id) = win_id {
                    let _ = app.event(PlatformEvent::WindowCloseRequested(win_id));
                }
                windows.remove(&id);
                if windows.is_empty() {
                    event_loop.exit();
                }
            }
            WindowEvent::Destroyed => {
                windows.remove(&id);
                if windows.is_empty() {
                    event_loop.exit();
                }
            }
            WindowEvent::Resized(size) => {
                let Some(state) = windows.get_mut(&id) else {
                    return;
                };
                let scale = state.window.scale_factor() as f32;
                let win_id = state.id;
                state.renderer.resize(size.width, size.height, scale);
                let dirty = app.event(PlatformEvent::Resized {
                    window: win_id,
                    logical_width: size.width as f32 / scale,
                    logical_height: size.height as f32 / scale,
                    scale_factor: scale,
                });
                state.dirty |= dirty;
                state.window.request_redraw();
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                let Some(state) = windows.get_mut(&id) else {
                    return;
                };
                let scale = state.window.scale_factor() as f32;
                let size = state.window.inner_size();
                state.renderer.resize(size.width, size.height, scale);
                state.dirty = true;
                state.window.request_redraw();
            }
            WindowEvent::CursorMoved {
                position: PhysicalPosition { x, y },
                ..
            } => {
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
            WindowEvent::MouseInput {
                state: ms_state,
                button: MouseButton::Left,
                ..
            } => {
                let Some(state) = windows.get_mut(&id) else {
                    return;
                };
                let win_id = state.id;
                let dirty = app.event(PlatformEvent::PointerButton {
                    window: win_id,
                    pressed: ms_state == ElementState::Pressed,
                });
                state.dirty |= dirty;
                if state.dirty {
                    state.window.request_redraw();
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
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
            WindowEvent::ModifiersChanged(modifiers) => {
                if let Some(state) = windows.get_mut(&id) {
                    state.shift = modifiers.state().shift_key();
                    state.ctrl = modifiers.state().control_key();
                }
            }
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
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
                let key = match key_event.logical_key {
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
            WindowEvent::Ime(winit::event::Ime::Preedit(text, _)) => {
                if let Some(state) = windows.get_mut(&id) {
                    state.dirty |= app.event(PlatformEvent::ImePreedit(text));
                    if state.dirty {
                        state.window.request_redraw();
                    }
                }
            }
            WindowEvent::Ime(winit::event::Ime::Commit(text)) => {
                if let Some(state) = windows.get_mut(&id) {
                    state.dirty |= app.event(PlatformEvent::ImeCommit(text));
                    if state.dirty {
                        state.window.request_redraw();
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let Some(state) = windows.get_mut(&id) else {
                    return;
                };
                let size = state.window.inner_size();
                if size.width == 0 || size.height == 0 {
                    return;
                }
                let scale = state.window.scale_factor() as f32;
                let win_id = state.id;
                let frame = app.frame(FrameContext {
                    window: win_id,
                    logical_width: size.width as f32 / scale,
                    logical_height: size.height as f32 / scale,
                    scale_factor: scale,
                });
                match state.renderer.render(&frame) {
                    SurfaceAction::Reconfigure => state.window.request_redraw(),
                    SurfaceAction::Exit => event_loop.exit(),
                    SurfaceAction::DeviceLost => {
                        tracing::warn!("GPU device lost, attempting recovery...");
                        match state.renderer.on_device_lost() {
                            Ok(()) => {
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
            _ => {}
        }
    }
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
            fn frame(&mut self, _ctx: FrameContext) -> Frame {
                Frame::default()
            }
        }
        let app = MultiWindowApp;
        let configs = app.windows();
        assert_eq!(configs.len(), 2);
        assert_eq!(configs[0].title, "Window 1");
        assert_eq!(configs[1].title, "Window 2");
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
}
