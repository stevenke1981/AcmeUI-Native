//! Mobile progress — determinate progress bar with optional label.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Builder for a mobile progress bar.
pub struct MobileProgressBuilder<M> {
    pub id: WidgetKey,
    pub value: f32,
    pub total: f32,
    pub label: Option<String>,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a mobile progress bar builder.
pub fn mobile_progress<M: Clone + 'static>(value: f32, total: f32) -> MobileProgressBuilder<M> {
    MobileProgressBuilder {
        id: WidgetKey::from("mobile_progress"),
        value,
        total: if total <= 0.0 { 1.0 } else { total },
        label: None,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> MobileProgressBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }

    /// Normalized fraction in [0.0, 1.0].
    pub fn fraction(&self) -> f32 {
        (self.value / self.total).clamp(0.0, 1.0)
    }
}

impl<M: Clone + 'static> From<MobileProgressBuilder<M>> for WidgetNode<M> {
    fn from(b: MobileProgressBuilder<M>) -> Self {
        let pct = ((b.value / b.total).clamp(0.0, 1.0) * 100.0) as u32;
        let bar_label = format!("{}%", pct);

        let mut col = crate::column::<M>().key(b.id).gap(4.0).padding(8.0);
        if let Some(text) = b.label {
            col = col.child(crate::label(text));
        }
        // Track (full width) containing fill indicator
        let track = crate::row::<M>()
            .height(6.0)
            .child(crate::label(bar_label))
            .build();
        col = col.child(track);
        col.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn mobile_progress_produces_column() {
        let node: WidgetNode<Msg> = mobile_progress(50.0, 100.0).into();
        assert!(matches!(node, WidgetNode::Column(_)));
    }

    #[test]
    fn mobile_progress_with_label_has_two_children() {
        let node: WidgetNode<Msg> = mobile_progress(1.0, 4.0).label("Loading").into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column");
        };
        assert_eq!(c.children.len(), 2);
    }

    #[test]
    fn mobile_progress_fraction_clamped() {
        let b = mobile_progress::<Msg>(200.0, 100.0);
        assert_eq!(b.fraction(), 1.0);
    }
}
