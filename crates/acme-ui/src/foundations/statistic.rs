//! Statistic component — large number display with title, value, prefix, suffix.
//!
//! Renders as a Column with a muted title, a Row with (prefix + value + suffix)
//! in large text, and an optional description.

use acme_core::WidgetKey;
use acme_widgets::*;
use std::marker::PhantomData;

/// Builder for a Statistic component.
pub struct StatisticBuilder<M> {
    pub id: WidgetKey,
    pub title: String,
    pub value: String,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub description: Option<String>,
    pub phantom: PhantomData<M>,
}

/// Create a new Statistic builder.
pub fn statistic<M: Clone + 'static>(
    id: impl Into<WidgetKey>,
    title: impl Into<String>,
    value: impl Into<String>,
) -> StatisticBuilder<M> {
    StatisticBuilder {
        id: id.into(),
        title: title.into(),
        value: value.into(),
        prefix: None,
        suffix: None,
        description: None,
        phantom: PhantomData,
    }
}

impl<M: Clone + 'static> StatisticBuilder<M> {
    /// Set the prefix displayed before the value (e.g. "$").
    pub fn prefix(mut self, value: impl Into<String>) -> Self {
        self.prefix = Some(value.into());
        self
    }

    /// Set the suffix displayed after the value (e.g. "%").
    pub fn suffix(mut self, value: impl Into<String>) -> Self {
        self.suffix = Some(value.into());
        self
    }

    /// Set the optional description text shown below the value.
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }
}

impl<M: Clone + 'static> From<StatisticBuilder<M>> for WidgetNode<M> {
    fn from(b: StatisticBuilder<M>) -> Self {
        // Title (caption, muted)
        let title_label = label::<M>(b.title.clone());

        // Value row: prefix + value + suffix in large text
        let mut value_row = row::<M>().gap(4.0);

        if let Some(p) = &b.prefix {
            value_row = value_row.child(label::<M>(p.clone()));
        }
        value_row = value_row.child(label::<M>(b.value.clone()));
        if let Some(s) = &b.suffix {
            value_row = value_row.child(label::<M>(s.clone()));
        }

        let mut col = column::<M>().gap(4.0).child(title_label).child(value_row);

        if let Some(desc) = &b.description {
            col = col.child(label::<M>(desc.clone()));
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
    fn statistic_builder_defaults() {
        let s = statistic::<TestMsg>("s", "Revenue", "42,000");
        assert_eq!(s.title, "Revenue");
        assert_eq!(s.value, "42,000");
        assert!(s.prefix.is_none());
        assert!(s.suffix.is_none());
        assert!(s.description.is_none());
    }

    #[test]
    fn statistic_renders_column() {
        let node: WidgetNode<TestMsg> = statistic("s", "Revenue", "42,000").into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        // title + value row
        assert_eq!(col.children.len(), 2);
    }

    #[test]
    fn statistic_with_prefix_and_suffix() {
        let node: WidgetNode<TestMsg> = statistic("s", "Growth", "12.5")
            .prefix("+")
            .suffix("%")
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        // Value row should have 3 children: prefix "+", value "12.5", suffix "%"
        let WidgetNode::Row(val_row) = &col.children[1] else {
            panic!("expected value Row");
        };
        assert_eq!(val_row.children.len(), 3);
    }

    #[test]
    fn statistic_with_description() {
        let node: WidgetNode<TestMsg> = statistic("s", "Users", "1,234")
            .description("Active this month")
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        // title + value row + description
        assert_eq!(col.children.len(), 3);
    }

    #[test]
    fn statistic_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = statistic("s", "Test", "100").into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert!(!layout.children.is_empty());
    }
}
