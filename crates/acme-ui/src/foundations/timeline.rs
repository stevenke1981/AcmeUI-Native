//! Timeline component — vertical timeline of events with markers.
//!
//! Each item renders as a Row with a colored marker Card on the left and a
//! Column of title, description, and timestamp on the right.

use acme_core::WidgetKey;
use acme_widgets::*;
use std::marker::PhantomData;

/// Kind of timeline item marker.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimelineItemKind {
    Default,
    Success,
    Warning,
    Danger,
    Info,
}

/// A single timeline event.
#[derive(Clone, Debug)]
pub struct TimelineItem {
    pub kind: TimelineItemKind,
    pub title: String,
    pub description: Option<String>,
    pub timestamp: Option<String>,
}

/// Builder for a Timeline component.
pub struct TimelineBuilder<M> {
    pub id: WidgetKey,
    pub items: Vec<TimelineItem>,
    pub reverse: bool,
    pub phantom: PhantomData<M>,
}

/// Create a new Timeline builder.
pub fn timeline<M: Clone + 'static>(id: impl Into<WidgetKey>) -> TimelineBuilder<M> {
    TimelineBuilder {
        id: id.into(),
        items: vec![],
        reverse: false,
        phantom: PhantomData,
    }
}

/// Create a timeline event item.
pub fn timeline_item(title: impl Into<String>, kind: TimelineItemKind) -> TimelineItem {
    TimelineItem {
        kind,
        title: title.into(),
        description: None,
        timestamp: None,
    }
}

impl TimelineItem {
    /// Set the optional description text.
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }

    /// Set the optional timestamp text.
    pub fn timestamp(mut self, value: impl Into<String>) -> Self {
        self.timestamp = Some(value.into());
        self
    }
}

impl<M: Clone + 'static> TimelineBuilder<M> {
    /// Add a single timeline item.
    pub fn item(mut self, item: TimelineItem) -> Self {
        self.items.push(item);
        self
    }

    /// Set all items at once.
    pub fn items(mut self, items: Vec<TimelineItem>) -> Self {
        self.items = items;
        self
    }

    /// Reverse the display order of items.
    pub fn reverse(mut self, value: bool) -> Self {
        self.reverse = value;
        self
    }
}

/// Map a TimelineItemKind to the appropriate CardVariant.
fn kind_to_variant(kind: TimelineItemKind) -> CardVariant {
    match kind {
        TimelineItemKind::Default => CardVariant::Plain,
        TimelineItemKind::Success => CardVariant::Interactive,
        TimelineItemKind::Warning => CardVariant::Outlined,
        TimelineItemKind::Danger => CardVariant::Interactive,
        TimelineItemKind::Info => CardVariant::Muted,
    }
}

/// Render a single timeline item as a Row.
fn render_item<M: Clone + 'static>(_index: usize, item: &TimelineItem) -> WidgetNode<M> {
    let mut right = column::<M>().gap(2.0).child(label::<M>(item.title.clone()));

    if let Some(desc) = &item.description {
        right = right.child(label::<M>(desc.clone()));
    }
    if let Some(ts) = &item.timestamp {
        right = right.child(label::<M>(ts.clone()));
    }

    row::<M>()
        .gap(8.0)
        .child(
            card::<M>()
                .padding(6.0)
                .variant(kind_to_variant(item.kind))
                .child(label::<M>("●")),
        )
        .child(right)
        .build()
}

impl<M: Clone + 'static> From<TimelineBuilder<M>> for WidgetNode<M> {
    fn from(b: TimelineBuilder<M>) -> Self {
        let mut col = column::<M>().gap(8.0);

        let items: Vec<&TimelineItem> = if b.reverse {
            let mut rev: Vec<&TimelineItem> = b.items.iter().collect();
            rev.reverse();
            rev
        } else {
            b.items.iter().collect()
        };

        for (i, item) in items.into_iter().enumerate() {
            col = col.child(render_item::<M>(i, item));
        }

        col.key(b.id).build()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;
    use acme_layout::LayoutKind;

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn timeline_builder_defaults() {
        let tl = timeline::<TestMsg>("tl");
        assert!(tl.items.is_empty());
        assert!(!tl.reverse);
    }

    #[test]
    fn timeline_renders_column() {
        let node: WidgetNode<TestMsg> = timeline("tl")
            .item(timeline_item("Event 1", TimelineItemKind::Default))
            .item(timeline_item("Event 2", TimelineItemKind::Success))
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        assert_eq!(col.children.len(), 2);
    }

    #[test]
    fn timeline_with_description_and_timestamp() {
        let node: WidgetNode<TestMsg> = timeline("tl")
            .item(
                timeline_item("Release", TimelineItemKind::Info)
                    .description("v2.0 deployed")
                    .timestamp("2025-06-01"),
            )
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        // Each item is a Row with Card + inner Column
        let WidgetNode::Row(row) = &col.children[0] else {
            panic!("expected Row");
        };
        assert_eq!(row.children.len(), 2); // marker + content column
    }

    #[test]
    fn timeline_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = timeline("tl")
            .item(timeline_item("A", TimelineItemKind::Default))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert!(!layout.children.is_empty());
    }

    #[test]
    fn timeline_reverse_order() {
        let tl = timeline::<TestMsg>("tl")
            .item(timeline_item("First", TimelineItemKind::Default))
            .item(timeline_item("Second", TimelineItemKind::Success))
            .reverse(true);
        assert!(tl.reverse);
        assert_eq!(tl.items.len(), 2);
    }
}
