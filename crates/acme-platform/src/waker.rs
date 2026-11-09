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
    /// North-west / south-east diagonal resize (`\` arrow).
    ResizeNwSe,
    /// North-east / south-west diagonal resize (`/` arrow).
    ResizeNeSw,
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
        CursorIcon::ResizeNwSe => winit::window::CursorIcon::NwseResize,
        CursorIcon::ResizeNeSw => winit::window::CursorIcon::NeswResize,
        CursorIcon::Grab => winit::window::CursorIcon::Grab,
        CursorIcon::Grabbing => winit::window::CursorIcon::Grabbing,
        CursorIcon::Move => winit::window::CursorIcon::Move,
        CursorIcon::NotAllowed => winit::window::CursorIcon::NotAllowed,
    }
}

/// Platform-neutral window edge or corner used for custom-frame resizing.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ResizeEdge {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

/// Map a platform-neutral [`ResizeEdge`] to a winit [`winit::window::ResizeDirection`].
pub(crate) fn map_resize_edge(edge: ResizeEdge) -> winit::window::ResizeDirection {
    use winit::window::ResizeDirection;
    match edge {
        ResizeEdge::North => ResizeDirection::North,
        ResizeEdge::South => ResizeDirection::South,
        ResizeEdge::East => ResizeDirection::East,
        ResizeEdge::West => ResizeDirection::West,
        ResizeEdge::NorthEast => ResizeDirection::NorthEast,
        ResizeEdge::NorthWest => ResizeDirection::NorthWest,
        ResizeEdge::SouthEast => ResizeDirection::SouthEast,
        ResizeEdge::SouthWest => ResizeDirection::SouthWest,
    }
}

/// Host window control requests queued by an [`Application`].
///
/// `PartialEq` only: `ShowSystemMenu` carries `f32` coordinates.
#[derive(Clone, Debug, PartialEq)]
pub enum WindowRequest {
    SetTitle(String),
    /// Unminimize, show, and focus the window.
    Focus,
    Minimize,
    Close,
    SetImeAllowed(bool),
    SetCursor(CursorIcon),
    /// Toggle between maximized and restored.
    ToggleMaximize,
    /// Explicitly maximize (`true`) or restore (`false`).
    SetMaximized(bool),
    /// Begin an OS-driven window move from the current pointer press
    /// (Windows: enters the system move loop; release is consumed by the OS).
    DragMove,
    /// Begin an OS-driven window resize from the given edge or corner.
    DragResize(ResizeEdge),
    /// Show the native system menu at logical client coordinates `(x, y)`
    /// (Windows only; no-op elsewhere).
    ShowSystemMenu {
        x: f32,
        y: f32,
    },
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
        assert_eq!(
            map_cursor_icon(CursorIcon::ResizeNwSe),
            winit::window::CursorIcon::NwseResize
        );
        assert_eq!(
            map_cursor_icon(CursorIcon::ResizeNeSw),
            winit::window::CursorIcon::NeswResize
        );
    }

    #[test]
    fn map_resize_edge_maps_all_eight_directions() {
        use winit::window::ResizeDirection as Dir;
        let cases = [
            (ResizeEdge::North, Dir::North),
            (ResizeEdge::South, Dir::South),
            (ResizeEdge::East, Dir::East),
            (ResizeEdge::West, Dir::West),
            (ResizeEdge::NorthEast, Dir::NorthEast),
            (ResizeEdge::NorthWest, Dir::NorthWest),
            (ResizeEdge::SouthEast, Dir::SouthEast),
            (ResizeEdge::SouthWest, Dir::SouthWest),
        ];
        for (edge, expected) in cases {
            assert_eq!(map_resize_edge(edge), expected, "edge {edge:?}");
        }
    }

    #[test]
    fn drain_window_requests_preserves_order_for_frameless_variants() {
        struct FramelessApp {
            pending: Vec<WindowRequest>,
        }

        impl Application for FramelessApp {
            fn take_window_requests(&mut self, window: WindowId) -> Vec<WindowRequest> {
                if window.0 == 7 {
                    std::mem::take(&mut self.pending)
                } else {
                    Vec::new()
                }
            }

            fn frame(&mut self, _ctx: FrameContext) -> Scene {
                Scene::new()
            }
        }

        let queued = vec![
            WindowRequest::DragMove,
            WindowRequest::ToggleMaximize,
            WindowRequest::DragResize(ResizeEdge::SouthEast),
            WindowRequest::SetMaximized(false),
            WindowRequest::ShowSystemMenu { x: 12.5, y: 3.0 },
            WindowRequest::SetCursor(CursorIcon::ResizeNwSe),
        ];
        let mut app = FramelessApp {
            pending: queued.clone(),
        };

        let drained = drain_window_requests(&mut app, [WindowId(7)]);
        assert_eq!(drained, vec![(WindowId(7), queued)]);
        assert!(drain_window_requests(&mut app, [WindowId(7)]).is_empty());
    }
}
