//! Indicator — status indicator with dot, optional ping, and label.
//! Absorbs gpui-component's indicator strength (richer than status_dot).

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Indicator status variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IndicatorStatus {
    Online,
    #[default]
    Offline,
    Busy,
    Away,
}

impl IndicatorStatus {
    pub fn glyph(&self) -> &'static str {
        match self {
            Self::Online => "🟢",
            Self::Offline => "⚪",
            Self::Busy => "🔴",
            Self::Away => "🟡",
        }
    }
}

/// Builder for a status indicator.
pub struct IndicatorBuilder<M> {
    pub id: WidgetKey,
    pub status: IndicatorStatus,
    pub label: Option<String>,
    pub ping: bool,
    _phantom: std::marker::PhantomData<M>,
}

/// Create an indicator builder.
pub fn indicator<M: Clone + 'static>() -> IndicatorBuilder<M> {
    IndicatorBuilder {
        id: WidgetKey::from("indicator"),
        status: IndicatorStatus::default(),
        label: None,
        ping: false,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> IndicatorBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn status(mut self, value: IndicatorStatus) -> Self {
        self.status = value;
        self
    }

    pub fn label(mut self, text: impl Into<String>) -> Self {
        self.label = Some(text.into());
        self
    }

    pub fn ping(mut self, value: bool) -> Self {
        self.ping = value;
        self
    }
}

impl<M: Clone + 'static> From<IndicatorBuilder<M>> for WidgetNode<M> {
    fn from(b: IndicatorBuilder<M>) -> Self {
        let mut row = crate::row::<M>().key(b.id).gap(6.0).padding(2.0);
        let dot = if b.ping {
            format!("{}◌", b.status.glyph())
        } else {
            b.status.glyph().to_string()
        };
        row = row.child(crate::label(dot));
        if let Some(label) = b.label {
            row = row.child(crate::label(label));
        }
        row.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn indicator_produces_row() {
        let node: WidgetNode<Msg> = indicator().into();
        assert!(matches!(node, WidgetNode::Row(_)));
    }

    #[test]
    fn indicator_with_label() {
        let node: WidgetNode<Msg> = indicator()
            .status(IndicatorStatus::Online)
            .label("Active")
            .into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        assert_eq!(r.children.len(), 2);
    }

    #[test]
    fn indicator_status_glyphs() {
        assert_eq!(IndicatorStatus::Online.glyph(), "🟢");
        assert_eq!(IndicatorStatus::Busy.glyph(), "🔴");
    }

    #[test]
    fn indicator_default_offline() {
        let b = indicator::<Msg>();
        assert_eq!(b.status, IndicatorStatus::Offline);
    }
}
