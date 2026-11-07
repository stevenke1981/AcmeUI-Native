//! Shared helpers for the mobile toggle, stepper and segmented tests.
//!
//! The helpers lay a widget out with the real layout engine and press the
//! laid-out hit areas, like the Gallery does: a press is delivered to the
//! `Button` whose rectangle holds the point, and a disabled button delivers
//! nothing.

use acme_core::NodeId;
use acme_layout::{LayoutEngine, LayoutNode, LayoutSnapshot, WidgetLayoutContext};

use crate::WidgetNode;

pub(crate) fn context() -> WidgetLayoutContext {
    WidgetLayoutContext {
        body_font_size: 14.0,
        body_line_height: 20.0,
        label_font_size: 13.0,
        control_height: 36.0,
        scale_factor: 1.0,
    }
}

/// A widget laid out in a 400 x 300 viewport.
pub(crate) struct Laid<M> {
    pub(crate) widget: WidgetNode<M>,
    pub(crate) layout: LayoutNode,
    pub(crate) snapshot: LayoutSnapshot,
}

pub(crate) fn lay_out<M: Clone + 'static>(widget: WidgetNode<M>) -> Laid<M> {
    let layout = widget.to_layout_with_context(NodeId::new(1), &context());
    let snapshot = LayoutEngine::new()
        .compute(&layout, (400.0, 300.0))
        .expect("layout should succeed");
    Laid {
        widget,
        layout,
        snapshot,
    }
}

/// A rectangle as `[x, y, width, height]`.
pub(crate) type Rect = [f32; 4];

/// One button hit area: its rectangle and what a press delivers.
pub(crate) struct Hit<M> {
    pub(crate) rect: Rect,
    pub(crate) message: Option<M>,
}

fn collect<M: Clone>(
    widget: &WidgetNode<M>,
    layout: &LayoutNode,
    snapshot: &LayoutSnapshot,
    out: &mut Vec<Hit<M>>,
) {
    if let WidgetNode::Button(button) = widget {
        if let Some(r) = snapshot.get(layout.id) {
            out.push(Hit {
                rect: [r.x, r.y, r.width, r.height],
                message: button.activate().cloned(),
            });
        }
        return;
    }
    for (w, l) in widget.children().iter().zip(layout.children.iter()) {
        collect(w, l, snapshot, out);
    }
}

impl<M: Clone + 'static> Laid<M> {
    /// Every button in tree order.
    pub(crate) fn hits(&self) -> Vec<Hit<M>> {
        let mut out = Vec::new();
        collect(&self.widget, &self.layout, &self.snapshot, &mut out);
        out
    }

    /// The message delivered by a press at (`x`, `y`): `None` when nothing is
    /// there or the button there is disabled.
    pub(crate) fn press_at(&self, x: f32, y: f32) -> Option<M> {
        self.hits()
            .into_iter()
            .find(|hit| {
                let [hx, hy, w, h] = hit.rect;
                x >= hx && x < hx + w && y >= hy && y < hy + h
            })
            .and_then(|hit| hit.message)
    }

    /// The message delivered by pressing the centre of the `index`th button.
    pub(crate) fn press_button(&self, index: usize) -> Option<M> {
        let [x, y, w, h] = self.hits()[index].rect;
        self.press_at(x + w / 2.0, y + h / 2.0)
    }

    /// The rectangle of the widget whose key is `key`.
    pub(crate) fn rect_of_key(&self, key: &str) -> Rect {
        fn find<M>(widget: &WidgetNode<M>, layout: &LayoutNode, key: &str) -> Option<NodeId> {
            if widget.key().is_some_and(|k| k.as_str() == key) {
                return Some(layout.id);
            }
            widget
                .children()
                .iter()
                .zip(layout.children.iter())
                .find_map(|(w, l)| find(w, l, key))
        }
        let id = find(&self.widget, &self.layout, key).expect("key present");
        let r = self.snapshot.get(id).expect("rect present");
        [r.x, r.y, r.width, r.height]
    }
}
