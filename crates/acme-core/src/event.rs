use crate::{Logical, NodeId, Point, Rect};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PointerId(pub u64);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
    Other(u16),
}
#[derive(Clone, Debug, PartialEq)]
pub enum PointerEvent {
    Move {
        pointer: PointerId,
        position: Point<Logical>,
    },
    Down {
        pointer: PointerId,
        button: PointerButton,
        position: Point<Logical>,
    },
    Up {
        pointer: PointerId,
        button: PointerButton,
        position: Point<Logical>,
    },
    Wheel {
        delta: Point<Logical>,
        position: Point<Logical>,
    },
    Cancel {
        pointer: PointerId,
    },
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub meta: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyboardEvent {
    pub key: String,
    pub pressed: bool,
    pub repeat: bool,
    pub modifiers: Modifiers,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GraphemeIndex(pub usize);
#[derive(Clone, Debug, PartialEq)]
pub enum ImeEvent {
    Enabled,
    Disabled,
    Preedit {
        value: String,
        cursor: Option<(GraphemeIndex, GraphemeIndex)>,
    },
    Commit(String),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImeCaretArea(pub Rect<Logical>);
#[derive(Clone, Debug, PartialEq)]
pub enum UiEvent {
    Pointer(PointerEvent),
    Keyboard(KeyboardEvent),
    Ime(ImeEvent),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventPhase {
    Capture,
    Target,
    Bubble,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EventResult {
    pub stop_propagation: bool,
    pub prevent_default: bool,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HitTestPath(pub Vec<NodeId>);
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HitTestEntry {
    pub id: NodeId,
    pub bounds: Rect<Logical>,
    pub enabled: bool,
}
pub fn hit_test(entries: &[HitTestEntry], point: Point<Logical>) -> Option<NodeId> {
    entries
        .iter()
        .rev()
        .find(|e| e.enabled && e.bounds.contains(point))
        .map(|e| e.id)
}
pub fn dispatch_event(
    mut path: HitTestPath,
    event: &UiEvent,
    mut handler: impl FnMut(NodeId, EventPhase, &UiEvent) -> EventResult,
) -> EventResult {
    if path.0.is_empty() {
        return EventResult::default();
    }
    let target = path.0.pop().unwrap();
    let mut out = EventResult::default();
    for id in &path.0 {
        let r = handler(*id, EventPhase::Capture, event);
        out.prevent_default |= r.prevent_default;
        if r.stop_propagation {
            return EventResult {
                stop_propagation: true,
                ..out
            };
        }
    }
    let r = handler(target, EventPhase::Target, event);
    out.prevent_default |= r.prevent_default;
    if r.stop_propagation {
        return EventResult {
            stop_propagation: true,
            ..out
        };
    }
    for id in path.0.iter().rev() {
        let r = handler(*id, EventPhase::Bubble, event);
        out.prevent_default |= r.prevent_default;
        if r.stop_propagation {
            return EventResult {
                stop_propagation: true,
                ..out
            };
        }
    }
    out
}

#[derive(Default)]
pub struct FocusManager {
    focused: Option<NodeId>,
    order: Vec<NodeId>,
    eligible: HashSet<NodeId>,
    captures: HashMap<PointerId, NodeId>,
}
impl FocusManager {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn focused(&self) -> Option<NodeId> {
        self.focused
    }
    pub fn set_order(&mut self, items: impl IntoIterator<Item = (NodeId, bool)>) {
        self.order.clear();
        self.eligible.clear();
        for (id, ok) in items {
            self.order.push(id);
            if ok {
                self.eligible.insert(id);
            }
        }
        if self.focused.is_some_and(|id| !self.eligible.contains(&id)) {
            self.focused = None
        }
    }
    pub fn request_focus(&mut self, id: NodeId) -> bool {
        if self.eligible.contains(&id) {
            self.focused = Some(id);
            true
        } else {
            false
        }
    }
    pub fn clear_focus(&mut self) {
        self.focused = None
    }
    pub fn focus_next(&mut self, reverse: bool) -> Option<NodeId> {
        let eligible: Vec<_> = self
            .order
            .iter()
            .copied()
            .filter(|id| self.eligible.contains(id))
            .collect();
        if eligible.is_empty() {
            self.focused = None;
            return None;
        }
        let i = self
            .focused
            .and_then(|f| eligible.iter().position(|x| *x == f));
        let next = match (i, reverse) {
            (Some(0), true) => eligible.len() - 1,
            (Some(i), true) => i - 1,
            (Some(i), false) => (i + 1) % eligible.len(),
            (None, true) => eligible.len() - 1,
            (None, false) => 0,
        };
        self.focused = Some(eligible[next]);
        self.focused
    }
    pub fn capture_pointer(&mut self, p: PointerId, id: NodeId) {
        self.captures.insert(p, id);
    }
    pub fn pointer_capture(&self, p: PointerId) -> Option<NodeId> {
        self.captures.get(&p).copied()
    }
    pub fn release_pointer(&mut self, p: PointerId) {
        self.captures.remove(&p);
    }
    pub fn remove_node(&mut self, id: NodeId) {
        if self.focused == Some(id) {
            self.focused = None
        }
        self.eligible.remove(&id);
        self.order.retain(|x| *x != id);
        self.captures.retain(|_, v| *v != id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::RetainedTree;
    fn ids() -> (NodeId, NodeId, NodeId) {
        let mut t = RetainedTree::new();
        t.reconcile_roots(&[
            crate::ViewNode::new("a", "x"),
            crate::ViewNode::new("b", "x"),
            crate::ViewNode::new("c", "x"),
        ])
        .unwrap();
        (t.roots()[0], t.roots()[1], t.roots()[2])
    }
    #[test]
    fn event_order_and_stop() {
        let (a, b, c) = ids();
        let mut calls = vec![];
        let result = dispatch_event(
            HitTestPath(vec![a, b, c]),
            &UiEvent::Ime(ImeEvent::Enabled),
            |id, phase, _| {
                calls.push((id, phase));
                EventResult {
                    stop_propagation: id == c,
                    ..Default::default()
                }
            },
        );
        assert_eq!(
            calls,
            vec![
                (a, EventPhase::Capture),
                (b, EventPhase::Capture),
                (c, EventPhase::Target)
            ]
        );
        assert!(result.stop_propagation);
    }
    #[test]
    fn focus_wraps_skips_and_cleans_capture() {
        let (a, b, c) = ids();
        let mut f = FocusManager::new();
        f.set_order([(a, true), (b, false), (c, true)]);
        assert_eq!(f.focus_next(false), Some(a));
        assert_eq!(f.focus_next(false), Some(c));
        f.capture_pointer(PointerId(1), c);
        f.remove_node(c);
        assert_eq!(f.focused(), None);
        assert_eq!(f.pointer_capture(PointerId(1)), None);
    }
    #[test]
    fn hit_test_prefers_last_painted() {
        let (a, b, _) = ids();
        let p = Point::new(5., 5.);
        assert_eq!(
            hit_test(
                &[
                    HitTestEntry {
                        id: a,
                        bounds: Rect::new(0., 0., 10., 10.),
                        enabled: true
                    },
                    HitTestEntry {
                        id: b,
                        bounds: Rect::new(0., 0., 10., 10.),
                        enabled: true
                    }
                ],
                p
            ),
            Some(b)
        );
    }
}
