//! One-item Gallery host; the public component/state do not depend on winit.
use super::*;
use acme_platform::PlatformKey;
use acme_ui::mobile::{SwipeActionsState, swipe_actions};
pub(super) struct SwipeDemo {
    pub state: SwipeActionsState,
    pub visible: bool,
    pub status: &'static str,
    pub pointer: Option<u64>,
    pub keyboard: bool,
    pub rect: Option<[f32; 4]>,
    clip: [f32; 4],
}
impl SwipeDemo {
    pub fn new() -> Self {
        Self {
            state: SwipeActionsState::default(),
            visible: true,
            status: "closed",
            pointer: None,
            keyboard: false,
            rect: None,
            clip: [0.0; 4],
        }
    }
    pub fn cancel_pointer(&mut self) {
        self.pointer = None;
        self.state.cancel();
    }
    fn contains(&self, x: f32, y: f32, scroll: f32) -> bool {
        self.rect.is_some_and(|r| {
            let r = intersect_clip([r[0], r[1] - scroll, r[2], r[3]], self.clip);
            r[2] > 0.0 && r[3] > 0.0 && x >= r[0] && x < r[0] + r[2] && y >= r[1] && y < r[1] + r[3]
        })
    }
}
fn find_rect(
    widget: &WidgetNode<GalleryMessage>,
    layout: &LayoutNode,
    snapshot: &acme_layout::LayoutSnapshot,
) -> Option<[f32; 4]> {
    if widget
        .key()
        .is_some_and(|k| k.as_str() == "gallery_swipe_row")
    {
        return snapshot
            .get(layout.id)
            .map(|r| [r.x, r.y, r.width, r.height]);
    }
    widget
        .children()
        .iter()
        .zip(&layout.children)
        .find_map(|(w, l)| find_rect(w, l, snapshot))
}
impl Gallery {
    pub(super) fn swipe_examples(&self) -> WidgetNode<GalleryMessage> {
        let mut demo=column().key("gallery_swipe_examples").gap(8.0)
            .child(acme_widgets::label_builder("Drag left to reveal; right to close. F7 toggles; Enter archives; Delete deletes; Escape closes.").wrap(acme_widgets::LabelWrap::Character).build())
            .child(row().gap(8.0)
                .child(button("gallery_swipe_toggle","Reveal / hide (F7)").on_click(GalleryMessage::SwipeToggle))
                .child(button("gallery_swipe_restore","Restore item").on_click(GalleryMessage::SwipeRestore)));
        if self.swipe.visible {
            demo = demo.child(
                column()
                    .w(Length::Percent(1.0))
                    .max_w(Length::px(480.0))
                    .child(
                        swipe_actions(
                            "gallery_swipe_row",
                            mobile_list_item(
                                "Inbox message · 收件匣",
                                Some("Swipe this item to reveal its actions. 滑動以顯示動作。"),
                            )
                            .key("gallery_swipe_item"),
                        )
                        .revealed(self.swipe.state.revealed())
                        .on_archive(GalleryMessage::SwipeArchive)
                        .on_delete(GalleryMessage::SwipeDelete),
                    ),
            );
        }
        demo.child(label(format!("Swipe: {}", self.swipe.status)))
            .build()
    }
    pub(super) fn sync_swipe_bounds(
        &mut self,
        widget: &WidgetNode<GalleryMessage>,
        layout: &LayoutNode,
        snapshot: &acme_layout::LayoutSnapshot,
        clip: Option<[f32; 4]>,
    ) {
        self.swipe.rect = find_rect(widget, layout, snapshot);
        self.swipe.clip = clip.unwrap_or([0.0; 4]);
        if self.swipe.rect.is_none() {
            self.swipe.cancel_pointer();
            self.swipe.keyboard = false;
        }
    }
    pub(super) fn dispatch_swipe(&mut self, message: GalleryMessage) -> bool {
        match message {
            GalleryMessage::SwipeToggle if self.swipe.visible => {
                self.swipe.state.set_revealed(!self.swipe.state.revealed());
                self.swipe.cancel_pointer();
                self.swipe.status = if self.swipe.state.revealed() {
                    "revealed"
                } else {
                    "closed"
                };
            }
            GalleryMessage::SwipeRestore => {
                self.swipe.visible = true;
                self.swipe.state.set_revealed(false);
                self.swipe.cancel_pointer();
                self.swipe.status = "restored";
            }
            GalleryMessage::SwipeArchive | GalleryMessage::SwipeDelete
                if self.swipe.visible && self.swipe.state.revealed() =>
            {
                self.swipe.visible = false;
                self.swipe.state.set_revealed(false);
                self.swipe.cancel_pointer();
                self.swipe.keyboard = false;
                self.swipe.status = if message == GalleryMessage::SwipeArchive {
                    "archived"
                } else {
                    "deleted"
                };
            }
            _ => return false,
        }
        self.pressed = None;
        self.hovered = None;
        true
    }
    pub(super) fn swipe_event(&mut self, event: &PlatformEvent) -> Option<bool> {
        match event {
            PlatformEvent::Key {
                key: PlatformKey::Function(7),
                pressed: true,
                ..
            } if self.swipe.visible => {
                if !self.search.input.preedit.is_empty() {
                    return Some(false);
                }
                self.search.blur();
                self.list_focus = None;
                self.swipe.keyboard = true;
                if let Some(r) = self.swipe.rect {
                    let top = r[1] - self.scroll;
                    if top < self.swipe.clip[1]
                        || top + r[3] > self.swipe.clip[1] + self.swipe.clip[3]
                    {
                        self.scroll = (r[1] - TOOLBAR_HEIGHT - 8.0).clamp(0.0, self.max_scroll);
                    }
                }
                Some(self.dispatch_swipe(GalleryMessage::SwipeToggle))
            }
            PlatformEvent::Key {
                key: PlatformKey::Escape,
                pressed: true,
                ..
            } if (self.swipe.state.revealed() || self.swipe.pointer.is_some())
                && self.search.input.preedit.is_empty() =>
            {
                self.swipe.state.set_revealed(false);
                self.swipe.cancel_pointer();
                self.swipe.keyboard = false;
                self.swipe.status = "closed";
                self.pressed = None;
                self.hovered = None;
                Some(true)
            }
            PlatformEvent::Key {
                key, pressed: true, ..
            } if self.swipe.keyboard && self.search.input.preedit.is_empty() => match key {
                PlatformKey::Enter => Some(self.dispatch_swipe(GalleryMessage::SwipeArchive)),
                PlatformKey::Delete => Some(self.dispatch_swipe(GalleryMessage::SwipeDelete)),
                PlatformKey::Escape => {
                    self.swipe.state.set_revealed(false);
                    self.swipe.cancel_pointer();
                    self.swipe.keyboard = false;
                    self.swipe.status = "closed";
                    Some(true)
                }
                _ => None,
            },
            PlatformEvent::PointerButton {
                pressed: true,
                x,
                y,
                button: 0,
                pointer,
                ..
            } => {
                if self.swipe.pointer.is_some() {
                    return Some(false);
                }
                self.cursor = (*x, *y);
                self.swipe.keyboard = false;
                if !self.swipe.contains(*x, *y, self.scroll) {
                    return None;
                }
                if self.hit().is_some_and(|i| {
                    matches!(
                        self.button_info[i].message,
                        GalleryMessage::SwipeArchive | GalleryMessage::SwipeDelete
                    )
                }) {
                    return None;
                }
                self.swipe.state.begin(*x, *y);
                self.swipe.pointer = Some(*pointer);
                self.search.blur();
                self.list_focus = None;
                self.pressed = None;
                self.hovered = None;
                Some(true)
            }
            PlatformEvent::PointerMoved { x, y, .. } if self.swipe.pointer.is_some() => {
                self.cursor = (*x, *y);
                self.swipe.state.update(*x, *y);
                self.pressed = None;
                self.hovered = None;
                Some(true)
            }
            PlatformEvent::PointerButton {
                pressed: false,
                x,
                y,
                button,
                pointer,
                ..
            } if self.swipe.pointer.is_some() => {
                if *button != 0 || self.swipe.pointer != Some(*pointer) {
                    return Some(false);
                }
                self.cursor = (*x, *y);
                self.swipe.state.end(*x, *y);
                self.swipe.pointer = None;
                self.pressed = None;
                self.hovered = None;
                self.swipe.status = if self.swipe.state.revealed() {
                    "revealed"
                } else {
                    "closed"
                };
                Some(true)
            }
            PlatformEvent::PointerLeft { .. }
            | PlatformEvent::FocusChanged { gained: false, .. }
            | PlatformEvent::Scroll { .. }
            | PlatformEvent::Resized { .. } => {
                if self.swipe.pointer.is_some() {
                    self.swipe.cancel_pointer();
                    self.pressed = None;
                    self.hovered = None;
                }
                if matches!(event, PlatformEvent::FocusChanged { gained: false, .. }) {
                    self.swipe.keyboard = false;
                }
                None
            }
            _ => None,
        }
    }
}
