//! Controlled trailing actions for list content.
//!
//! The host routes a primary pointer sequence beginning inside the item to
//! `SwipeActionsState`, then passes `revealed()` to the builder. Revealing never
//! invokes an action. The host owns pointer identity/capture and cancellation.
//! This is threshold-based reveal on release, without translation or animation.
use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_layout::Length;
use acme_widgets::style::Styled;

/// Backend-neutral gesture state using logical coordinates.
#[derive(Clone, Debug, Default)]
pub struct SwipeActionsState {
    revealed: bool,
    start: Option<(f32, f32)>,
    horizontal: bool,
    cancelled: bool,
}
impl SwipeActionsState {
    pub fn revealed(&self) -> bool {
        self.revealed
    }
    /// Set controlled visibility and cancel any in-progress gesture.
    pub fn set_revealed(&mut self, value: bool) {
        self.cancel();
        self.revealed = value;
    }
    /// Start only after the host has hit-tested this item's visible bounds.
    pub fn begin(&mut self, x: f32, y: f32) {
        self.cancel();
        self.start = (x.is_finite() && y.is_finite()).then_some((x, y));
    }
    /// Return whether this sequence has claimed horizontal movement (12 px).
    /// A vertical-first sequence stays rejected until the next `begin`.
    pub fn update(&mut self, x: f32, y: f32) -> bool {
        let Some((sx, sy)) = self.start else {
            return false;
        };
        let (dx, dy) = (x - sx, y - sy);
        if !dx.is_finite() || !dy.is_finite() {
            self.cancelled = true;
        }
        if self.cancelled {
            return false;
        }
        if !self.horizontal {
            if dy.abs() >= 12.0 && dy.abs() >= dx.abs() {
                self.cancelled = true;
                return false;
            }
            self.horizontal = dx.abs() >= 12.0 && dx.abs() > dy.abs();
        }
        self.horizontal
    }
    /// Finish a matching pointer release. Left reveals / right closes at 48 px.
    /// Return whether visibility changed; no action message is emitted here.
    pub fn end(&mut self, x: f32, y: f32) -> bool {
        let previous = self.revealed;
        if self.update(x, y) {
            let dx = x - self.start.unwrap().0;
            if dx <= -48.0 {
                self.revealed = true;
            }
            if dx >= 48.0 {
                self.revealed = false;
            }
        }
        self.cancel();
        self.revealed != previous
    }
    /// Cancel a gesture while preserving the already-revealed state.
    pub fn cancel(&mut self) {
        self.start = None;
        self.horizontal = false;
        self.cancelled = false;
    }
}

/// Stateless list content plus optional trailing Archive/Delete actions.
pub struct SwipeActionsBuilder<M> {
    pub id: WidgetKey,
    pub child: WidgetNode<M>,
    pub revealed: bool,
    pub on_archive: Option<M>,
    pub on_delete: Option<M>,
}
/// Create a controlled swipe row. Provide an instance key and wrapping content.
pub fn swipe_actions<M: Clone + 'static>(
    id: impl Into<WidgetKey>,
    child: impl Into<WidgetNode<M>>,
) -> SwipeActionsBuilder<M> {
    SwipeActionsBuilder {
        id: id.into(),
        child: child.into(),
        revealed: false,
        on_archive: None,
        on_delete: None,
    }
}
impl<M: Clone + 'static> SwipeActionsBuilder<M> {
    pub fn revealed(mut self, value: bool) -> Self {
        self.revealed = value;
        self
    }
    pub fn on_archive(mut self, message: M) -> Self {
        self.on_archive = Some(message);
        self
    }
    pub fn on_delete(mut self, message: M) -> Self {
        self.on_delete = Some(message);
        self
    }
}
impl<M: Clone + 'static> From<SwipeActionsBuilder<M>> for WidgetNode<M> {
    fn from(b: SwipeActionsBuilder<M>) -> Self {
        let id = b.id.as_str();
        let mut root = crate::row()
            .key(b.id.clone())
            .w(Length::Percent(1.0))
            .min_h(Length::px(44.0))
            .child(
                crate::column()
                    .key(format!("{id}__content"))
                    .min_w(Length::px(0.0))
                    .flex_1()
                    .flex_shrink(1.0)
                    .child(b.child),
            );
        if b.revealed && (b.on_archive.is_some() || b.on_delete.is_some()) {
            let mut actions = crate::column()
                .key(format!("{id}__actions"))
                .w(Length::px(72.0))
                .flex_shrink(0.0)
                .gap(4.0);
            for (suffix, label, variant, message) in [
                (
                    "archive",
                    "Archive",
                    crate::ButtonVariant::Secondary,
                    b.on_archive,
                ),
                (
                    "delete",
                    "Delete",
                    crate::ButtonVariant::Danger,
                    b.on_delete,
                ),
            ] {
                if let Some(message) = message {
                    actions = actions.child(
                        crate::button(format!("{id}__{suffix}"), label)
                            .variant(variant)
                            .w(Length::px(72.0))
                            .h(Length::px(44.0))
                            .p(0.0)
                            .on_click(message),
                    );
                }
            }
            root = root.gap(8.0).child(actions);
        }
        root.build()
    }
}
#[cfg(test)]
#[path = "swipe_actions_tests.rs"]
mod tests;
