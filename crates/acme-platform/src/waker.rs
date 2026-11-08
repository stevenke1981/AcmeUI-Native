use crate::{Application, WindowId};

/// Cross-thread handle for waking the platform event loop.
///
/// Cloning and sending a [`Waker`] across threads is cheap. Calling [`Waker::wake`]
/// after the event loop has exited is a no-op.
#[derive(Clone, Debug)]
pub struct Waker {
    inner: WakerInner,
}

#[derive(Clone, Debug)]
enum WakerInner {
    Proxy(winit::event_loop::EventLoopProxy<()>),
    #[cfg(test)]
    Disconnected,
}

impl Waker {
    pub(crate) fn from_proxy(proxy: winit::event_loop::EventLoopProxy<()>) -> Self {
        Self {
            inner: WakerInner::Proxy(proxy),
        }
    }

    #[cfg(test)]
    pub(crate) fn disconnected_for_test() -> Self {
        Self {
            inner: WakerInner::Disconnected,
        }
    }

    /// Wake the UI thread and trigger [`Application::wake`].
    ///
    /// If the event loop has already exited or been dropped, this is a no-op.
    pub fn wake(&self) {
        match &self.inner {
            WakerInner::Proxy(proxy) => {
                let _ = proxy.send_event(());
            }
            #[cfg(test)]
            WakerInner::Disconnected => {}
        }
    }
}

/// Backend-agnostic pointer cursor icon.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CursorIcon {
    #[default]
    Default,
    Text,
    Pointer,
    ResizeHorizontal,
    ResizeVertical,
    Grab,
    Grabbing,
    Move,
    NotAllowed,
}

/// Map a platform-neutral [`CursorIcon`] to a winit [`winit::window::CursorIcon`].
pub(crate) fn map_cursor_icon(icon: CursorIcon) -> winit::window::CursorIcon {
    match icon {
        CursorIcon::Default => winit::window::CursorIcon::Default,
        CursorIcon::Text => winit::window::CursorIcon::Text,
        CursorIcon::Pointer => winit::window::CursorIcon::Pointer,
        CursorIcon::ResizeHorizontal => winit::window::CursorIcon::EwResize,
        CursorIcon::ResizeVertical => winit::window::CursorIcon::NsResize,
        CursorIcon::Grab => winit::window::CursorIcon::Grab,
        CursorIcon::Grabbing => winit::window::CursorIcon::Grabbing,
        CursorIcon::Move => winit::window::CursorIcon::Move,
        CursorIcon::NotAllowed => winit::window::CursorIcon::NotAllowed,
    }
}

/// Host window control requests queued by an [`Application`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WindowRequest {
    SetTitle(String),
    /// Unminimize, show, and focus the window.
    Focus,
    Minimize,
    Close,
    SetImeAllowed(bool),
    SetCursor(CursorIcon),
}

/// Pure helper used by the runtime to drain pending [`WindowRequest`] batches
/// for a set of active windows in order.
pub(crate) fn drain_window_requests<A, I>(
    app: &mut A,
    window_ids: I,
) -> Vec<(WindowId, Vec<WindowRequest>)>
where
    A: Application + ?Sized,
    I: IntoIterator<Item = WindowId>,
{
    let mut out = Vec::new();
    for window_id in window_ids {
        let requests = app.take_window_requests(window_id);
        if !requests.is_empty() {
            out.push((window_id, requests));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FrameContext;
    use acme_core::Scene;

    const _: () = {
        fn assert_send_sync_clone<T: Send + Sync + Clone>() {}
        let _ = assert_send_sync_clone::<Waker>;
    };

    #[test]
    fn waker_wake_after_loop_drop_is_noop() {
        #[cfg(target_os = "windows")]
        {
            use winit::event_loop::EventLoop;
            use winit::platform::windows::EventLoopBuilderExtWindows;

            if let Ok(event_loop) = EventLoop::<()>::with_user_event()
                .with_any_thread(true)
                .build()
            {
                let waker = Waker::from_proxy(event_loop.create_proxy());
                let cloned = waker.clone();
                drop(event_loop);
                // Waking after the EventLoop is dropped must be a no-op.
                waker.wake();
                std::thread::spawn(move || {
                    cloned.wake();
                })
                .join()
                .unwrap();
                return;
            }
        }

        let waker = Waker::disconnected_for_test();
        let cloned = waker.clone();
        waker.wake();
        cloned.wake();
    }

    #[test]
    fn drain_window_requests_drains_all_windows_and_leaves_empty_on_second_pass() {
        struct RequestApp {
            w1: Vec<WindowRequest>,
            w2: Vec<WindowRequest>,
        }

        impl Application for RequestApp {
            fn take_window_requests(&mut self, window: WindowId) -> Vec<WindowRequest> {
                match window.0 {
                    1 => std::mem::take(&mut self.w1),
                    2 => std::mem::take(&mut self.w2),
                    _ => Vec::new(),
                }
            }

            fn frame(&mut self, _ctx: FrameContext) -> Scene {
                Scene::new()
            }
        }

        let mut app = RequestApp {
            w1: vec![
                WindowRequest::SetTitle("CrewDesk Terminal".into()),
                WindowRequest::Focus,
                WindowRequest::SetImeAllowed(true),
            ],
            w2: vec![WindowRequest::Minimize, WindowRequest::Close],
        };

        let first = drain_window_requests(&mut app, [WindowId(1), WindowId(99), WindowId(2)]);
        assert_eq!(
            first,
            vec![
                (
                    WindowId(1),
                    vec![
                        WindowRequest::SetTitle("CrewDesk Terminal".into()),
                        WindowRequest::Focus,
                        WindowRequest::SetImeAllowed(true),
                    ]
                ),
                (
                    WindowId(2),
                    vec![WindowRequest::Minimize, WindowRequest::Close]
                ),
            ]
        );

        let second = drain_window_requests(&mut app, [WindowId(1), WindowId(2)]);
        assert!(second.is_empty());
    }

    #[test]
    fn map_cursor_icon_maps_every_variant() {
        assert_eq!(CursorIcon::default(), CursorIcon::Default);
        assert_eq!(
            map_cursor_icon(CursorIcon::Default),
            winit::window::CursorIcon::Default
        );
        assert_eq!(
            map_cursor_icon(CursorIcon::Text),
            winit::window::CursorIcon::Text
        );
        assert_eq!(
            map_cursor_icon(CursorIcon::Pointer),
            winit::window::CursorIcon::Pointer
        );
        assert_eq!(
            map_cursor_icon(CursorIcon::ResizeHorizontal),
            winit::window::CursorIcon::EwResize
        );
        assert_eq!(
            map_cursor_icon(CursorIcon::ResizeVertical),
            winit::window::CursorIcon::NsResize
        );
        assert_eq!(
            map_cursor_icon(CursorIcon::Grab),
            winit::window::CursorIcon::Grab
        );
        assert_eq!(
            map_cursor_icon(CursorIcon::Grabbing),
            winit::window::CursorIcon::Grabbing
        );
        assert_eq!(
            map_cursor_icon(CursorIcon::Move),
            winit::window::CursorIcon::Move
        );
        assert_eq!(
            map_cursor_icon(CursorIcon::NotAllowed),
            winit::window::CursorIcon::NotAllowed
        );
    }
}
