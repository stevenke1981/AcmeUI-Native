//! CopyButton — copy text to clipboard with visual feedback state.
//! Absorbs gpui-component's clipboard/copy-button strength.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Copy feedback state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CopyState {
    #[default]
    Idle,
    Copied,
}

/// Builder for a copy button.
pub struct CopyButtonBuilder<M> {
    pub id: WidgetKey,
    pub text: String,
    pub state: CopyState,
    pub idle_label: String,
    pub copied_label: String,
    pub on_copy: Option<M>,
}

/// Create a copy button builder.
pub fn copy_button<M: Clone + 'static>(text: impl Into<String>) -> CopyButtonBuilder<M> {
    CopyButtonBuilder {
        id: WidgetKey::from("copy_button"),
        text: text.into(),
        state: CopyState::default(),
        idle_label: "Copy".to_string(),
        copied_label: "Copied ✓".to_string(),
        on_copy: None,
    }
}

impl<M: Clone + 'static> CopyButtonBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn state(mut self, value: CopyState) -> Self {
        self.state = value;
        self
    }

    pub fn idle_label(mut self, text: impl Into<String>) -> Self {
        self.idle_label = text.into();
        self
    }

    pub fn copied_label(mut self, text: impl Into<String>) -> Self {
        self.copied_label = text.into();
        self
    }

    pub fn on_copy(mut self, msg: M) -> Self {
        self.on_copy = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<CopyButtonBuilder<M>> for WidgetNode<M> {
    fn from(b: CopyButtonBuilder<M>) -> Self {
        let label = match b.state {
            CopyState::Idle => b.idle_label,
            CopyState::Copied => b.copied_label,
        };
        let btn = crate::button(b.id, label);
        if let Some(msg) = b.on_copy {
            btn.on_click(msg)
        } else {
            btn.into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Copy,
    }

    #[test]
    fn copy_button_produces_button() {
        let node: WidgetNode<Msg> = copy_button("secret").on_copy(Msg::Copy).into();
        assert!(matches!(node, WidgetNode::Button(_)));
    }

    #[test]
    fn copy_button_idle_label() {
        let node: WidgetNode<Msg> = copy_button("x").into();
        let WidgetNode::Button(b) = &node else {
            panic!("expected Button");
        };
        assert_eq!(b.label, "Copy");
    }

    #[test]
    fn copy_button_copied_label() {
        let node: WidgetNode<Msg> = copy_button("x").state(CopyState::Copied).into();
        let WidgetNode::Button(b) = &node else {
            panic!("expected Button");
        };
        assert_eq!(b.label, "Copied ✓");
    }
}
