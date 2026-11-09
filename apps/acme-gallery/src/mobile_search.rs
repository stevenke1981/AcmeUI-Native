//! Private Gallery host for the single Mobile search example.
//! Editing and grapheme boundaries remain owned by acme-textinput.
use super::*;
use acme_core::{Color, DrawCommand, QuadPrimitive, Rect, Scene};
use acme_platform::{Clipboard, PlatformKey};
use acme_textinput::{
    TextInputState, handle_key, handle_keyboard_shortcut, handle_text, render_text_input,
    scroll_caret_into_view,
};

#[derive(Clone, Copy, Debug, PartialEq)]
enum Focus {
    Input,
    Submit,
    Clear,
}

pub(super) struct SearchDemo {
    pub input: TextInputState,
    pub submitted: String,
    /// Layout coordinates before the Gallery scroll offset.
    pub rect: Option<[f32; 4]>,
    pub caret: Option<[f32; 4]>,
    focus: Option<Focus>,
    submit_rect: Option<[f32; 4]>,
    clear_rect: Option<[f32; 4]>,
    clip: [f32; 4],
    clipboard: Option<Clipboard>,
}

impl SearchDemo {
    pub fn new() -> Self {
        Self {
            input: TextInputState::new(),
            submitted: "—".into(),
            rect: None,
            caret: None,
            focus: None,
            submit_rect: None,
            clear_rect: None,
            clip: [0.0; 4],
            clipboard: None,
        }
    }

    fn focus(&mut self, focus: Option<Focus>) {
        if self.focus != focus {
            self.input.cancel_preedit();
        }
        self.focus = focus;
        self.input.focused = focus == Some(Focus::Input);
        self.caret = None;
    }

    pub fn blur(&mut self) {
        self.focus(None);
    }

    pub fn reveal_scroll(&self, scroll: f32) -> f32 {
        let rect = match self.focus {
            Some(Focus::Input) => self.rect,
            Some(Focus::Submit) => self.submit_rect,
            Some(Focus::Clear) => self.clear_rect,
            None => None,
        };
        let Some(r) = rect else {
            return scroll;
        };
        if r[1] - scroll < self.clip[1] {
            r[1] - self.clip[1]
        } else if r[1] + r[3] - scroll > self.clip[1] + self.clip[3] {
            r[1] + r[3] - self.clip[1] - self.clip[3]
        } else {
            scroll
        }
    }

    pub fn submit(&mut self) -> bool {
        if !self.input.preedit.is_empty() {
            return false;
        }
        self.submitted.clone_from(&self.input.text);
        true
    }

    pub fn clear(&mut self) -> bool {
        // Use the edit engine so clearing is undoable and cancels composition.
        self.input.cancel_preedit();
        self.input.select_all();
        let focused = self.input.focused;
        self.input.focused = true;
        handle_key(
            &mut self.input,
            &PlatformKey::Backspace,
            true,
            None,
            false,
            false,
        );
        self.input.focused = focused;
        self.input.scroll_offset = 0.0;
        self.caret = None;
        true
    }

    /// Return Some only when this host consumes the event.
    pub fn event(&mut self, event: &PlatformEvent, scroll: f32) -> Option<bool> {
        match event {
            PlatformEvent::PointerButton {
                pressed: true,
                x,
                y,
                ..
            } => {
                let target = [
                    (self.rect, Focus::Input),
                    (self.submit_rect, Focus::Submit),
                    (self.clear_rect, Focus::Clear),
                ]
                .into_iter()
                .find_map(|(rect, focus)| {
                    rect.filter(|r| {
                        contains([r[0], r[1] - scroll, r[2], r[3]], *x, *y)
                            && contains(self.clip, *x, *y)
                    })
                    .map(|_| focus)
                });
                self.focus(target);
                // Gallery still owns pointer press/release activation.
                None
            }
            PlatformEvent::Key {
                key: PlatformKey::Tab,
                pressed: true,
                shift,
                ..
            } if self.rect.is_some() => {
                let next = match (self.focus, shift) {
                    (None, false) | (Some(Focus::Submit), true) => Some(Focus::Input),
                    (Some(Focus::Input), false) | (Some(Focus::Clear), true) => Some(Focus::Submit),
                    (Some(Focus::Submit), false) | (None, true) => Some(Focus::Clear),
                    (Some(Focus::Input), true) | (Some(Focus::Clear), false) => None,
                };
                self.focus(next);
                Some(true)
            }
            PlatformEvent::Key {
                key,
                pressed,
                ctrl,
                shift,
                alt,
                meta,
                text,
                ..
            } if self.focus.is_some() => {
                if !pressed {
                    return Some(false);
                }
                if *key == PlatformKey::Enter || (*key == PlatformKey::Space && !self.input.focused)
                {
                    return Some(match self.focus {
                        Some(Focus::Input | Focus::Submit) => self.submit(),
                        Some(Focus::Clear) => self.clear(),
                        None => false,
                    });
                }
                if !self.input.focused {
                    return Some(false);
                }
                // During composition the IME owns editing/navigation keys.
                if !self.input.preedit.is_empty() && *key != PlatformKey::Escape {
                    return Some(false);
                }
                let shortcut = *ctrl || *meta;
                if shortcut {
                    if let Some(text) = text.as_deref() {
                        let command = if *shift && text.eq_ignore_ascii_case("z") {
                            "y".into()
                        } else {
                            text.to_lowercase()
                        };
                        if self.clipboard.is_none() && matches!(command.as_str(), "c" | "v" | "x") {
                            self.clipboard = Clipboard::new().ok();
                        }
                        if handle_keyboard_shortcut(
                            &mut self.input,
                            &command,
                            self.clipboard.as_ref(),
                        ) {
                            self.caret = None;
                            return Some(true);
                        }
                    }
                }
                let changed = handle_key(&mut self.input, key, true, None, shortcut, *shift);
                if !self.input.focused {
                    self.focus = None;
                }
                self.caret = None;
                if changed {
                    return Some(true);
                }
                if !shortcut && !alt && self.input.preedit.is_empty() {
                    if let Some(text) = text.as_deref().filter(|s| !s.chars().any(char::is_control))
                    {
                        return Some(handle_text(&mut self.input, text));
                    }
                }
                Some(false)
            }
            PlatformEvent::ImePreedit { text, cursor, .. } if self.input.focused => {
                self.input.set_preedit(text, *cursor);
                self.caret = None;
                Some(true)
            }
            PlatformEvent::ImeCommit { text, .. } if self.input.focused => {
                self.caret = None;
                Some(handle_text(&mut self.input, text))
            }
            PlatformEvent::ImeDisabled(_) => {
                self.input.cancel_preedit();
                self.caret = None;
                Some(true)
            }
            PlatformEvent::FocusChanged { gained: false, .. } => {
                self.blur();
                Some(true)
            }
            _ => None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn paint(
        &mut self,
        scene: &mut Scene,
        widget: &WidgetNode<GalleryMessage>,
        layout: &LayoutNode,
        snapshot: &acme_layout::LayoutSnapshot,
        theme: &Theme,
        scale: f32,
        scroll: f32,
        fonts: &mut FontSystem,
        atlas: &mut GlyphAtlas,
    ) {
        self.rect = find_rect(widget, layout, snapshot, "gallery_search__input");
        self.submit_rect = find_rect(widget, layout, snapshot, "gallery_search__submit");
        self.clear_rect = find_rect(widget, layout, snapshot, "gallery_search__clear");
        let Some(rect) = self.rect else {
            self.blur();
            return;
        };
        let Some(clip) = find_rect(widget, layout, snapshot, "content_scroll") else {
            return;
        };
        self.clip = clip;
        let rect = [rect[0], rect[1] - scroll, rect[2], rect[3]];
        let style = TextStyle {
            font_size: theme.typography.body,
            line_height: theme.typography.body * theme.typography.line_height,
            ..TextStyle::default()
        };
        scroll_caret_into_view(
            &mut self.input,
            fonts,
            &style,
            scale,
            (rect[2] - 2.0 * theme.spacing.px2).max(0.0),
        );
        self.input.placeholder = "Search · 搜尋".into();
        scene.push(DrawCommand::PushClip(Rect::new(
            clip[0], clip[1], clip[2], clip[3],
        )));
        render_text_input(
            scene,
            &mut self.input,
            fonts,
            atlas,
            rect,
            theme,
            scale,
            self.focus == Some(Focus::Input),
            Some(clip),
        );
        if let Some(action) = match self.focus {
            Some(Focus::Submit) => self.submit_rect,
            Some(Focus::Clear) => self.clear_rect,
            _ => None,
        } {
            let c = theme.colors.ring;
            scene.push(DrawCommand::Quad(QuadPrimitive {
                rect: Rect::new(action[0], action[1] - scroll, action[2], action[3]),
                color: Color::TRANSPARENT,
                radius: theme.radii.md,
                border_width: 2.0,
                border_color: Color::rgba(c.red, c.green, c.blue, c.alpha),
            }));
        }
        scene.push(DrawCommand::PopClip);
        self.caret = if self.input.focused && intersects(rect, clip) {
            let [x, y, w, h] = self.input.ime_caret_area(fonts, &style, scale);
            let padding = theme.spacing.px2;
            Some([
                rect[0] + padding + x.clamp(0.0, (rect[2] - 2.0 * padding).max(0.0)),
                (rect[1] + padding + y).clamp(clip[1], (clip[1] + clip[3] - h).max(clip[1])),
                w,
                h,
            ])
        } else {
            None
        };
    }
}

fn contains(r: [f32; 4], x: f32, y: f32) -> bool {
    x >= r[0] && x < r[0] + r[2] && y >= r[1] && y < r[1] + r[3]
}

fn intersects(a: [f32; 4], b: [f32; 4]) -> bool {
    a[0] < b[0] + b[2] && b[0] < a[0] + a[2] && a[1] < b[1] + b[3] && b[1] < a[1] + a[3]
}

fn find_rect(
    widget: &WidgetNode<GalleryMessage>,
    layout: &LayoutNode,
    snapshot: &acme_layout::LayoutSnapshot,
    key: &str,
) -> Option<[f32; 4]> {
    if widget.key().is_some_and(|k| k.as_str() == key) {
        return snapshot
            .get(layout.id)
            .map(|r| [r.x, r.y, r.width, r.height]);
    }
    widget
        .children()
        .iter()
        .zip(&layout.children)
        .find_map(|(w, l)| find_rect(w, l, snapshot, key))
}
