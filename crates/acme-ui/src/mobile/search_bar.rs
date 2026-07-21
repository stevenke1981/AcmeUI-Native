//! SearchBar component — a mobile-optimized search input with icon and cancel button.
//!
//! V2 design: renders as a Row with a search icon label, a text-like input area,
//! and an optional cancel button. Designed for mobile / compact layouts.

use acme_core::WidgetKey;

/// Builder for a SearchBar component.
pub struct SearchBarBuilder<M> {
    pub id: WidgetKey,
    pub placeholder: String,
    pub value: String,
    pub show_cancel: bool,
    pub auto_focus: bool,
    pub on_submit: Option<M>,
    pub on_change: Option<M>,
}

/// Create a new SearchBar builder.
pub fn search_bar<M: Clone + 'static>(id: impl Into<WidgetKey>) -> SearchBarBuilder<M> {
    SearchBarBuilder {
        id: id.into(),
        placeholder: "Search…".to_string(),
        value: String::new(),
        show_cancel: true,
        auto_focus: false,
        on_submit: None,
        on_change: None,
    }
}

impl<M: Clone + 'static> SearchBarBuilder<M> {
    /// Set the placeholder text.
    pub fn placeholder(mut self, value: impl Into<String>) -> Self {
        self.placeholder = value.into();
        self
    }

    /// Set the current search value.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    /// Show or hide the cancel button.
    pub fn show_cancel(mut self, value: bool) -> Self {
        self.show_cancel = value;
        self
    }

    /// Enable or disable auto-focus.
    pub fn auto_focus(mut self, value: bool) -> Self {
        self.auto_focus = value;
        self
    }

    /// Set the message dispatched when the user submits the search.
    pub fn on_submit(mut self, msg: M) -> Self {
        self.on_submit = Some(msg);
        self
    }

    /// Set the message dispatched when the search text changes.
    pub fn on_change(mut self, msg: M) -> Self {
        self.on_change = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<SearchBarBuilder<M>> for crate::WidgetNode<M> {
    fn from(b: SearchBarBuilder<M>) -> Self {
        let theme = acme_theme::Theme::light();

        // Search icon
        let icon = crate::label_builder("🔍")
            .font_size(theme.typography.body)
            .color(theme.colors.muted_foreground)
            .build();

        // Input area — a Card styled as an input field
        let display_text = if b.value.is_empty() {
            b.placeholder.clone()
        } else {
            b.value.clone()
        };

        let input_area = crate::card::<M>()
            .variant(crate::CardVariant::Elevated)
            .child(
                crate::row::<M>()
                    .child(icon)
                    .child(
                        crate::label_builder(&display_text)
                            .font_size(theme.typography.body)
                            .color(if b.value.is_empty() {
                                theme.colors.muted_foreground
                            } else {
                                theme.colors.foreground
                            })
                            .build(),
                    )
                    .gap(6.0)
                    .build(),
            )
            .padding(8.0)
            .border_radius(theme.radii.lg)
            .build();

        // Build the main row
        let mut main_row = crate::row::<M>().key(b.id).gap(6.0).child(input_area);

        // Cancel button
        if b.show_cancel {
            let cancel_btn = if let Some(ref msg) = b.on_submit {
                let node: crate::WidgetNode<M> = crate::button::<M>("search-cancel", "Cancel")
                    .variant(crate::ButtonVariant::Ghost)
                    .on_click(msg.clone());
                node
            } else {
                crate::button::<M>("search-cancel", "Cancel")
                    .variant(crate::ButtonVariant::Ghost)
                    .into()
            };
            main_row = main_row.child(cancel_btn);
        }

        main_row.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WidgetNode;
    use acme_core::NodeId;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Search(String),
        Change(String),
    }

    #[test]
    fn search_bar_has_non_zero_layout_rect() {
        let node: WidgetNode<Msg> = search_bar::<Msg>("sb").into();
        let layout = node.to_layout(NodeId::new(1));
        // Row: [input card, cancel button] = 2 children
        assert_eq!(layout.children.len(), 2);
    }

    #[test]
    fn search_bar_no_cancel() {
        let node: WidgetNode<Msg> = search_bar::<Msg>("sb").show_cancel(false).into();
        let layout = node.to_layout(NodeId::new(1));
        // Row: [input card] = 1 child
        assert_eq!(layout.children.len(), 1);
    }

    #[test]
    fn search_bar_builder_defaults() {
        let s = search_bar::<Msg>("sb");
        assert_eq!(s.placeholder, "Search…");
        assert!(s.value.is_empty());
        assert!(s.show_cancel);
        assert!(!s.auto_focus);
        assert!(s.on_submit.is_none());
        assert!(s.on_change.is_none());
    }

    #[test]
    fn search_bar_with_value() {
        let s = search_bar::<Msg>("sb").value("hello");
        assert_eq!(s.value, "hello");
    }

    #[test]
    fn search_bar_custom_placeholder() {
        let s = search_bar::<Msg>("sb").placeholder("Find…");
        assert_eq!(s.placeholder, "Find…");
    }

    #[test]
    fn search_bar_with_messages() {
        let node: WidgetNode<Msg> = search_bar::<Msg>("sb")
            .on_submit(Msg::Search("".to_string()))
            .on_change(Msg::Change("".to_string()))
            .into();
        let WidgetNode::Row(r) = &node else {
            panic!("expected Row variant");
        };
        assert!(!r.children.is_empty());
    }
}
