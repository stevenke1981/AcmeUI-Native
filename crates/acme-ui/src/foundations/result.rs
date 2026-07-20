//! Result component — status result page with icon, title, subtitle, and extra.
//!
//! Renders a centered Column with a large status icon, a title, an optional
//! subtitle, and optional extra action content.

use acme_core::WidgetKey;
use acme_widgets::*;

/// Status variant for the Result component.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResultStatus {
    Success,
    Error,
    Info,
    Warning,
}

/// Builder for a Result component.
pub struct ResultBuilder<M> {
    pub id: WidgetKey,
    pub status: ResultStatus,
    pub title: String,
    pub subtitle: Option<String>,
    pub extra: Option<WidgetNode<M>>,
}

/// Create a new Result builder.
pub fn result<M: Clone + 'static>(
    id: impl Into<WidgetKey>,
    status: ResultStatus,
    title: impl Into<String>,
) -> ResultBuilder<M> {
    ResultBuilder {
        id: id.into(),
        status,
        title: title.into(),
        subtitle: None,
        extra: None,
    }
}

impl<M: Clone + 'static> ResultBuilder<M> {
    /// Set the optional subtitle text.
    pub fn subtitle(mut self, value: impl Into<String>) -> Self {
        self.subtitle = Some(value.into());
        self
    }

    /// Set the optional extra content (e.g. action buttons).
    pub fn extra(mut self, node: impl Into<WidgetNode<M>>) -> Self {
        self.extra = Some(node.into());
        self
    }
}

/// Get the status icon character for a ResultStatus.
fn status_icon_char(status: ResultStatus) -> &'static str {
    match status {
        ResultStatus::Success => "✓",
        ResultStatus::Error => "✕",
        ResultStatus::Info => "i",
        ResultStatus::Warning => "!",
    }
}

impl<M: Clone + 'static> From<ResultBuilder<M>> for WidgetNode<M> {
    fn from(b: ResultBuilder<M>) -> Self {
        let icon_char = status_icon_char(b.status);

        let icon_label = label::<M>(icon_char);
        let icon_card = card::<M>()
            .variant(CardVariant::Interactive)
            .padding(16.0)
            .child(icon_label);

        let mut col = column::<M>()
            .gap(8.0)
            .child(icon_card)
            .child(label::<M>(b.title.clone()));

        if let Some(sub) = &b.subtitle {
            col = col.child(label::<M>(sub.clone()));
        }

        if let Some(extra) = b.extra {
            col = col.child(extra);
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
    fn result_builder_defaults() {
        let r = result::<TestMsg>("r", ResultStatus::Success, "Done!");
        assert_eq!(r.title, "Done!");
        assert_eq!(r.status, ResultStatus::Success);
        assert!(r.subtitle.is_none());
        assert!(r.extra.is_none());
    }

    #[test]
    fn result_renders_column() {
        let node: WidgetNode<TestMsg> =
            result("r", ResultStatus::Error, "Failed").into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        // icon card + title = 2
        assert_eq!(col.children.len(), 2);
    }

    #[test]
    fn result_with_subtitle() {
        let node: WidgetNode<TestMsg> = result("r", ResultStatus::Info, "Info")
            .subtitle("Something happened")
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        // icon card + title + subtitle = 3
        assert_eq!(col.children.len(), 3);
    }

    #[test]
    fn result_status_icon_characters() {
        assert_eq!(status_icon_char(ResultStatus::Success), "✓");
        assert_eq!(status_icon_char(ResultStatus::Error), "✕");
        assert_eq!(status_icon_char(ResultStatus::Info), "i");
        assert_eq!(status_icon_char(ResultStatus::Warning), "!");
    }

    #[test]
    fn result_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> =
            result("r", ResultStatus::Success, "OK").into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert!(!layout.children.is_empty());
    }
}
