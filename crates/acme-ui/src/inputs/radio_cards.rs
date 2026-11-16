//! RadioCards — card-style radio selection group.
//! Aligns with Radix Themes Radio Cards.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// A single radio card option.
#[derive(Clone, Debug)]
pub struct RadioCardOption {
    pub value: String,
    pub title: String,
    pub description: Option<String>,
}

impl RadioCardOption {
    pub fn new(value: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            title: title.into(),
            description: None,
        }
    }

    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }
}

/// Builder for radio cards.
pub struct RadioCardsBuilder<M> {
    pub id: WidgetKey,
    pub options: Vec<RadioCardOption>,
    pub selected: Option<String>,
    pub on_select: Option<fn(&str) -> M>,
}

/// Create a radio cards builder.
pub fn radio_cards<M: Clone + 'static>() -> RadioCardsBuilder<M> {
    RadioCardsBuilder {
        id: WidgetKey::from("radio_cards"),
        options: Vec::new(),
        selected: None,
        on_select: None,
    }
}

impl<M: Clone + 'static> RadioCardsBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn option(mut self, option: RadioCardOption) -> Self {
        self.options.push(option);
        self
    }

    pub fn selected(mut self, value: impl Into<String>) -> Self {
        self.selected = Some(value.into());
        self
    }

    /// Build each option's typed selection message from its value.
    ///
    /// With a handler, every option becomes a Card keyed `"{id}:card:{value}"`
    /// holding a Button keyed `"{id}:option:{value}"`. Option values must be
    /// unique within one group because they are part of those keys. Without a
    /// handler, options stay passive labels.
    pub fn on_select(mut self, f: fn(&str) -> M) -> Self {
        self.on_select = Some(f);
        self
    }

    /// Message selecting the next (`forward`) or previous option, wrapping.
    ///
    /// With no or an unknown selection, returns the first option (forward) or
    /// the last option (backward). Returns `None` when there are no options or
    /// no `on_select` handler. Option values must be unique.
    pub fn navigate(&self, forward: bool) -> Option<M> {
        let handler = self.on_select?;
        let len = self.options.len();
        if len == 0 {
            return None;
        }
        let current = self
            .selected
            .as_deref()
            .and_then(|s| self.options.iter().position(|o| o.value == s));
        let next = match (current, forward) {
            (Some(i), true) => (i + 1) % len,
            (Some(i), false) => (i + len - 1) % len,
            (None, true) => 0,
            (None, false) => len - 1,
        };
        Some(handler(&self.options[next].value))
    }
}

impl<M: Clone + 'static> From<RadioCardsBuilder<M>> for WidgetNode<M> {
    fn from(b: RadioCardsBuilder<M>) -> Self {
        let prefix = b.id.as_str().to_owned();
        let mut col = crate::column::<M>().key(b.id).gap(8.0).padding(4.0);
        for option in &b.options {
            let is_selected = b.selected.as_deref() == Some(&option.value);
            let marker = if is_selected { "◉" } else { "○" };
            let title = format!("{} {}", marker, option.title);
            let card = crate::card::<M>()
                .variant(acme_widgets::CardVariant::Outlined)
                .padding(12.0)
                .gap(4.0);
            let mut card = if let Some(select) = b.on_select {
                let variant = if is_selected {
                    crate::ButtonVariant::Primary
                } else {
                    crate::ButtonVariant::Ghost
                };
                card.key(format!("{prefix}:card:{}", option.value)).child(
                    crate::button(format!("{prefix}:option:{}", option.value), title)
                        .variant(variant)
                        .full_width(true)
                        .on_click(select(&option.value)),
                )
            } else {
                card.child(crate::label(title))
            };
            if let Some(desc) = &option.description {
                card = card.child(crate::label(desc.clone()));
            }
            col = col.child(card.build());
        }
        col.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Selected(String),
    }

    fn select_msg(v: &str) -> Msg {
        Msg::Selected(v.to_string())
    }

    #[test]
    fn radio_cards_produces_column() {
        let node: WidgetNode<Msg> = radio_cards()
            .option(RadioCardOption::new("a", "Option A"))
            .into();
        assert!(matches!(node, WidgetNode::Column(_)));
    }

    #[test]
    fn radio_cards_child_count() {
        let node: WidgetNode<Msg> = radio_cards()
            .option(RadioCardOption::new("a", "A"))
            .option(RadioCardOption::new("b", "B"))
            .option(RadioCardOption::new("c", "C"))
            .into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column");
        };
        assert_eq!(c.children.len(), 3);
    }

    #[test]
    fn radio_cards_selected_default_none() {
        let b = radio_cards::<Msg>();
        assert!(b.selected.is_none());
    }

    #[test]
    fn radio_cards_on_select_emits_selected_with_value() {
        let b = radio_cards::<Msg>().on_select(select_msg);
        assert_eq!(
            b.on_select.map(|f| f("b")),
            Some(Msg::Selected("b".to_string()))
        );
    }

    fn find_by_key<'a>(node: &'a WidgetNode<Msg>, key: &str) -> Option<&'a WidgetNode<Msg>> {
        if node.key().map(|k| k.as_str()) == Some(key) {
            return Some(node);
        }
        node.children().iter().find_map(|c| find_by_key(c, key))
    }

    fn collect_keys(node: &WidgetNode<Msg>, out: &mut Vec<String>) {
        if let Some(k) = node.key() {
            out.push(k.as_str().to_owned());
        }
        for child in node.children() {
            collect_keys(child, out);
        }
    }

    fn plan(key: &str, values: &[&str], selected: Option<&str>) -> RadioCardsBuilder<Msg> {
        let mut b = radio_cards::<Msg>().key(key).on_select(select_msg);
        for v in values {
            b = b.option(RadioCardOption::new(*v, format!("Plan {v} · 方案")).description("desc"));
        }
        if let Some(s) = selected {
            b = b.selected(s);
        }
        b
    }

    /// Replica of the pre-055 passive conversion, used to pin the legacy tree.
    fn legacy_tree(b: &RadioCardsBuilder<Msg>) -> WidgetNode<Msg> {
        let mut col = crate::column::<Msg>()
            .key(b.id.clone())
            .gap(8.0)
            .padding(4.0);
        for option in &b.options {
            let is_selected = b.selected.as_deref() == Some(&option.value);
            let marker = if is_selected { "◉" } else { "○" };
            let mut card = crate::card::<Msg>()
                .variant(acme_widgets::CardVariant::Outlined)
                .padding(12.0)
                .gap(4.0)
                .child(crate::label(format!("{} {}", marker, option.title)));
            if let Some(desc) = &option.description {
                card = card.child(crate::label(desc.clone()));
            }
            col = col.child(card.build());
        }
        col.build()
    }

    #[test]
    fn radio_cards_options_bind_typed_select_messages_to_instance_keyed_buttons() {
        let node: WidgetNode<Msg> = plan("plans", &["a", "b"], Some("b")).into();
        assert_eq!(node.key().unwrap().as_str(), "plans");
        assert_eq!(node.children().len(), 2);
        for (index, (value, marker)) in [("a", "○"), ("b", "◉")].into_iter().enumerate() {
            let card = &node.children()[index];
            assert!(matches!(card, WidgetNode::Card(_)));
            assert_eq!(card.key().unwrap().as_str(), format!("plans:card:{value}"));
            let Some(WidgetNode::Button(button)) =
                find_by_key(card, &format!("plans:option:{value}"))
            else {
                panic!("option {value} must be a keyed Button inside its card");
            };
            assert_eq!(button.label, format!("{marker} Plan {value} · 方案"));
            assert_eq!(button.activate(), Some(&Msg::Selected(value.to_string())));
            assert!(
                card.children()
                    .iter()
                    .any(|c| matches!(c, WidgetNode::Label(l) if l.text == "desc"))
            );
        }
    }

    #[test]
    fn radio_cards_without_handler_stay_passive_and_keep_legacy_tree() {
        let build = || {
            radio_cards::<Msg>()
                .key("passive")
                .option(RadioCardOption::new("a", "A").description("first"))
                .option(RadioCardOption::new("b", "B"))
                .selected("a")
        };
        let mut b = build();
        let expected = legacy_tree(&b);
        let node: WidgetNode<Msg> = build().into();
        assert_eq!(node, expected);
        let mut keys = Vec::new();
        collect_keys(&node, &mut keys);
        assert_eq!(keys, ["passive"]);
        assert!(b.navigate(true).is_none());
        b.options.clear();
        assert!(b.navigate(false).is_none());
    }

    #[test]
    fn radio_cards_navigate_wraps_and_handles_empty_unknown_and_unselected() {
        let sel = |v: &str| Some(Msg::Selected(v.to_string()));
        let values = ["a", "b", "c"];
        assert_eq!(plan("n", &values, Some("a")).navigate(true), sel("b"));
        assert_eq!(plan("n", &values, Some("c")).navigate(true), sel("a"));
        assert_eq!(plan("n", &values, Some("a")).navigate(false), sel("c"));
        assert_eq!(plan("n", &values, Some("b")).navigate(false), sel("a"));
        assert_eq!(plan("n", &values, None).navigate(true), sel("a"));
        assert_eq!(plan("n", &values, None).navigate(false), sel("c"));
        assert_eq!(plan("n", &values, Some("zz")).navigate(true), sel("a"));
        assert_eq!(plan("n", &values, Some("zz")).navigate(false), sel("c"));
        assert_eq!(
            plan("n", &["only"], Some("only")).navigate(true),
            sel("only")
        );
        assert_eq!(plan("n", &[], None).navigate(true), None);
        assert_eq!(plan("n", &[], Some("a")).navigate(false), None);
    }

    #[test]
    fn radio_cards_retained_ids_survive_selection_change_and_keyed_reorder() {
        use acme_core::RetainedTree;
        use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};
        let mut tree = RetainedTree::new();
        let mut saved = None;
        let steps: [(&[&str], Option<&str>, bool); 4] = [
            (&["a", "b", "c"], None, false),
            (&["c", "a", "b"], Some("a"), true),
            (&["b", "c", "a"], Some("c"), false),
            (&["a", "b", "c"], Some("b"), true),
        ];
        for (order, selected, reverse) in steps {
            let mut roots: Vec<WidgetNode<Msg>> = vec![
                plan("x", order, selected).into(),
                plan("y", order, selected).into(),
            ];
            if reverse {
                roots.reverse();
            }
            tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
            let binding = bind_retained(&roots, &tree);
            assert_eq!(binding.mismatches(), 0);
            let ids = ["x", "y"].map(|root_key| {
                let root = roots
                    .iter()
                    .find(|n| n.key().unwrap().as_str() == root_key)
                    .unwrap();
                ["a", "b", "c"].map(|v| {
                    let card = find_by_key(root, &format!("{root_key}:card:{v}")).unwrap();
                    let button = find_by_key(root, &format!("{root_key}:option:{v}")).unwrap();
                    (
                        binding.id_for(card).unwrap(),
                        binding.id_for(button).unwrap(),
                    )
                })
            });
            if let Some(saved) = saved {
                assert_eq!(ids, saved);
            }
            assert_ne!(ids[0], ids[1]);
            saved = Some(ids);
        }
    }

    #[test]
    fn radio_cards_duplicate_keys_absent_and_selected_variant_uses_theme_tokens() {
        let node: WidgetNode<Msg> = plan("dup", &["a", "b", "c"], Some("b")).into();
        let mut keys = Vec::new();
        collect_keys(&node, &mut keys);
        assert_eq!(keys.len(), 7);
        let mut unique = keys.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), keys.len(), "duplicate keys: {keys:?}");
        let variant = |v: &str| match find_by_key(&node, &format!("dup:option:{v}")) {
            Some(WidgetNode::Button(b)) => b.variant,
            _ => panic!("missing option button {v}"),
        };
        assert_eq!(variant("b"), crate::ButtonVariant::Primary);
        assert_eq!(variant("a"), crate::ButtonVariant::Ghost);
        assert_eq!(variant("c"), crate::ButtonVariant::Ghost);
        assert_ne!(variant("a"), variant("b"));
        // Theme tokens only: production code must not construct colors directly.
        let source = include_str!("radio_cards.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        for forbidden in ["Color", "rgb(", "rgba(", "srgb", "0x"] {
            assert!(!production.contains(forbidden), "found {forbidden}");
        }
    }
}
