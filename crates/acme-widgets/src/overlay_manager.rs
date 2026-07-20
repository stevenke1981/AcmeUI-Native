use acme_core::{Logical, NodeId, Rect};

/// Z-order layers for overlay widgets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum OverlayLayer {
    Main,
    Floating,
    Modal,
    Tooltip,
    Drag,
    Debug,
}

/// Manages overlay stacking and positioning.
#[derive(Clone, Debug)]
pub struct OverlayManager {
    layers: Vec<(OverlayLayer, NodeId, Rect<Logical>)>,
}

impl OverlayManager {
    pub fn new() -> Self {
        Self { layers: vec![] }
    }

    pub fn push(&mut self, layer: OverlayLayer, id: NodeId, rect: Rect<Logical>) {
        self.layers.push((layer, id, rect));
    }

    pub fn raise(&mut self, id: NodeId) {
        if let Some(pos) = self.layers.iter().position(|(_, nid, _)| *nid == id) {
            let entry = self.layers.remove(pos);
            self.layers.push(entry);
        }
    }

    pub fn dismiss(&mut self, id: NodeId) {
        self.layers.retain(|(_, nid, _)| *nid != id);
    }

    pub fn top_of(&self, layer: OverlayLayer) -> Option<NodeId> {
        self.layers
            .iter()
            .rev()
            .find(|(l, _, _)| *l == layer)
            .map(|(_, id, _)| *id)
    }
}

impl Default for OverlayManager {
    fn default() -> Self {
        Self::new()
    }
}
