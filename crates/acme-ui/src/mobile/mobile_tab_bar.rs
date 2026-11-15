//! Mobile tab bar component — horizontal row of keyed selectable tab buttons.

use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_widgets::{ButtonVariant, button, row};

/// Builder for a mobile tab bar.
pub struct MobileTabBarBuilder<M> {
    pub id: WidgetKey,
    pub tabs: Vec<String>,
    pub selected_index: usize,
    pub disabled: bool,
    pub on_select: Option<fn(usize) -> M>,
}

/// Create a mobile tab bar builder with initial tab labels.
pub fn mobile_tab_bar<M: Clone + 'static>(tabs: Vec<impl Into<String>>) -> MobileTabBarBuilder<M> {
    MobileTabBarBuilder {
        id: WidgetKey::from("mobile_tab_bar"),
        tabs: tabs.into_iter().map(Into::into).collect(),
        selected_index: 0,
        disabled: false,
        on_select: None,
    }
}

impl<M: Clone + 'static> MobileTabBarBuilder<M> {
    /// Override the root widget key.
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    /// Append a tab label.
    pub fn tab(mut self, title: impl Into<String>) -> Self {
        self.tabs.push(title.into());
        self
    }

    /// Set the active tab index.
    pub fn selected_index(mut self, index: usize) -> Self {
        self.selected_index = index;
        self
    }

    /// Disable all tabs in the bar.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    /// Set the callback invoked with the clicked tab index.
    pub fn on_select(mut self, f: fn(usize) -> M) -> Self {
        self.on_select = Some(f);
        self
    }
}

impl<M: Clone + 'static> From<MobileTabBarBuilder<M>> for WidgetNode<M> {
    fn from(b: MobileTabBarBuilder<M>) -> Self {
        let id_str = b.id.as_str().to_string();
        let mut container = row::<M>().key(b.id).gap(4.0).padding(4.0);
        for (idx, title) in b.tabs.iter().enumerate() {
            let active = idx == b.selected_index;
            let btn = button::<M>(format!("{id_str}_tab_{idx}").as_str(), title)
                .variant(if active {
                    ButtonVariant::Primary
                } else {
                    ButtonVariant::Ghost
                })
                .disabled(b.disabled);
            let node: WidgetNode<M> = match b.on_select {
                Some(cb) => btn.on_click(cb(idx)),
                None => btn.into(),
            };
            container = container.child(node);
        }
        container.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;
    use acme_layout::LayoutKind;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Select(usize),
    }

    #[test]
    fn mobile_tab_bar_defaults_and_layout() {
        let node: WidgetNode<Msg> = mobile_tab_bar(vec!["Home", "Search", "Profile"])
            .selected_index(1)
            .on_select(Msg::Select)
            .into();

        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        assert_eq!(r.children.len(), 3);

        let WidgetNode::Button(b0) = &r.children[0] else {
            panic!("expected Button");
        };
        assert_eq!(b0.variant, ButtonVariant::Ghost);
        assert_eq!(b0.activate(), Some(&Msg::Select(0)));

        let WidgetNode::Button(b1) = &r.children[1] else {
            panic!("expected Button");
        };
        assert_eq!(b1.variant, ButtonVariant::Primary);
        assert_eq!(b1.activate(), Some(&Msg::Select(1)));

        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Row);
        assert_eq!(layout.children.len(), 3);
    }

    #[test]
    fn mobile_tab_bar_disabled_prevents_activation() {
        let node: WidgetNode<Msg> = mobile_tab_bar(vec!["A", "B"])
            .disabled(true)
            .on_select(Msg::Select)
            .into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row");
        };
        let WidgetNode::Button(b) = &r.children[0] else {
            panic!("expected Button");
        };
        assert!(b.disabled);
        assert_eq!(b.activate(), None);
    }
}
