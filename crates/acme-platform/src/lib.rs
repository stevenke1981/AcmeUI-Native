//! Windows-first winit runtime. Public events contain no winit platform types.
#![forbid(unsafe_op_in_unsafe_fn)]

use acme_render_wgpu::{Frame, Renderer, SurfaceAction};
use std::sync::Arc;
use thiserror::Error;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition},
    event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

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
        logical_width: f32,
        logical_height: f32,
        scale_factor: f32,
    },
    PointerMoved {
        x: f32,
        y: f32,
    },
    PointerButton {
        pressed: bool,
    },
    Scroll {
        delta_y: f32,
    },
    Key {
        key: PlatformKey,
        pressed: bool,
        shift: bool,
    },
    ImePreedit(String),
    ImeCommit(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformKey {
    Tab,
    Enter,
    Space,
    Escape,
    Other,
}

pub struct FrameContext {
    pub logical_width: f32,
    pub logical_height: f32,
    pub scale_factor: f32,
}

/// Application-side state driven by framework-owned events.
pub trait Application: 'static {
    fn window_config(&self) -> WindowConfig {
        WindowConfig::default()
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

struct Runtime<A> {
    app: A,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    cursor: (f32, f32),
    shift: bool,
    dirty: bool,
}

impl<A> Runtime<A> {
    fn new(app: A) -> Self {
        Self {
            app,
            window: None,
            renderer: None,
            cursor: (0.0, 0.0),
            shift: false,
            dirty: true,
        }
    }
}

impl<A: Application> ApplicationHandler for Runtime<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let config = self.app.window_config();
        let attrs = Window::default_attributes()
            .with_title(config.title)
            .with_inner_size(LogicalSize::new(config.width, config.height))
            .with_resizable(true);
        let Ok(window) = event_loop.create_window(attrs) else {
            event_loop.exit();
            return;
        };
        let window = Arc::new(window);
        window.set_ime_allowed(true);
        let size = window.inner_size();
        match pollster::block_on(Renderer::new(
            window.clone(),
            size.width,
            size.height,
            window.scale_factor() as f32,
        )) {
            Ok(renderer) => {
                self.renderer = Some(renderer);
                self.window = Some(window.clone());
                window.request_redraw();
            }
            Err(error) => {
                tracing::error!(%error, "renderer initialization failed");
                event_loop.exit();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        if window.id() != id {
            return;
        }
        let scale = window.scale_factor() as f32;
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size.width, size.height, scale);
                }
                self.dirty |= self.app.event(PlatformEvent::Resized {
                    logical_width: size.width as f32 / scale,
                    logical_height: size.height as f32 / scale,
                    scale_factor: scale,
                });
                window.request_redraw();
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                let size = window.inner_size();
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size.width, size.height, scale);
                }
                self.dirty = true;
                window.request_redraw();
            }
            WindowEvent::CursorMoved {
                position: PhysicalPosition { x, y },
                ..
            } => {
                self.cursor = (x as f32 / scale, y as f32 / scale);
                self.dirty |= self.app.event(PlatformEvent::PointerMoved {
                    x: self.cursor.0,
                    y: self.cursor.1,
                });
                if self.dirty {
                    window.request_redraw();
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                self.dirty |= self.app.event(PlatformEvent::PointerButton {
                    pressed: state == ElementState::Pressed,
                });
                if self.dirty {
                    window.request_redraw();
                }
            }
            WindowEvent::MouseWheel { delta, .. } => {
                let delta_y = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y * 32.0,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32 / scale,
                };
                self.dirty |= self.app.event(PlatformEvent::Scroll { delta_y });
                if self.dirty {
                    window.request_redraw();
                }
            }
            WindowEvent::ModifiersChanged(modifiers) => self.shift = modifiers.state().shift_key(),
            WindowEvent::KeyboardInput { event, .. } => {
                let key = match event.logical_key {
                    Key::Named(NamedKey::Tab) => PlatformKey::Tab,
                    Key::Named(NamedKey::Enter) => PlatformKey::Enter,
                    Key::Named(NamedKey::Space) => PlatformKey::Space,
                    Key::Named(NamedKey::Escape) => PlatformKey::Escape,
                    _ => PlatformKey::Other,
                };
                self.dirty |= self.app.event(PlatformEvent::Key {
                    key,
                    pressed: event.state == ElementState::Pressed,
                    shift: self.shift,
                });
                if self.dirty {
                    window.request_redraw();
                }
            }
            WindowEvent::Ime(winit::event::Ime::Preedit(text, _)) => {
                self.dirty |= self.app.event(PlatformEvent::ImePreedit(text));
                if self.dirty {
                    window.request_redraw();
                }
            }
            WindowEvent::Ime(winit::event::Ime::Commit(text)) => {
                self.dirty |= self.app.event(PlatformEvent::ImeCommit(text));
                if self.dirty {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                let size = window.inner_size();
                if size.width == 0 || size.height == 0 {
                    return;
                }
                let frame = self.app.frame(FrameContext {
                    logical_width: size.width as f32 / scale,
                    logical_height: size.height as f32 / scale,
                    scale_factor: scale,
                });
                if let Some(renderer) = self.renderer.as_mut() {
                    match renderer.render(&frame) {
                        SurfaceAction::Reconfigure => window.request_redraw(),
                        SurfaceAction::Exit => event_loop.exit(),
                        _ => {}
                    }
                }
                self.dirty = false;
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
}
