//! CheckboxCards — card-style checkbox selection group.
//! Aligns with Radix Themes Checkbox Cards.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// A single checkbox card option.
#[derive(Clone, Debug)]
pub struct CheckboxCardOption {
    pub value: String,
    pub title: String,
    pub description: Option<String>,
}

impl CheckboxCardOption {
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

/// Builder for checkbox cards.
///
/// Option values must be unique within one group and must not contain `':'`,
/// because handler mode embeds them in widget keys. Neither rule is enforced.
pub struct CheckboxCardsBuilder<M> {
    pub id: WidgetKey,
    pub options: Vec<CheckboxCardOption>,
    pub selected: Vec<String>,
    pub columns: u32,
    pub on_toggle: Option<fn(&str) -> M>,
    /// Value of the keyboard-focused option (handler mode only).
    pub focused: Option<String>,
}

/// Create a checkbox cards builder.
pub fn checkbox_cards<M: Clone + 'static>() -> CheckboxCardsBuilder<M> {
    CheckboxCardsBuilder {
        id: WidgetKey::from("checkbox_cards"),
        options: Vec::new(),
        selected: Vec::new(),
        columns: 2,
        on_toggle: None,
        focused: None,
    }
}

impl<M: Clone + 'static> CheckboxCardsBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn option(mut self, option: CheckboxCardOption) -> Self {
        self.options.push(option);
        self
    }

    /// Values of the checked options. The host owns this state.
    pub fn selected(mut self, values: Vec<impl Into<String>>) -> Self {
        self.selected = values.into_iter().map(Into::into).collect();
        self
    }

    pub fn columns(mut self, value: u32) -> Self {
        self.columns = value.max(1);
        self
    }

    /// Build each option's typed toggle message from its value.
    ///
    /// With a handler, every option becomes a Card keyed `"{id}:card:{value}"`
    /// holding a full-width Button keyed `"{id}:option:{value}"` that emits
    /// `f(value)` on click. Checked options use the Primary variant with `☑`;
    /// unchecked options use Ghost with `☐`. The host flips its checked set
    /// and passes it back through [`selected`](Self::selected). Without a
    /// handler, options stay passive labels.
    pub fn on_toggle(mut self, f: fn(&str) -> M) -> Self {
        self.on_toggle = Some(f);
        self
    }

    /// Mark the keyboard-focused option.
    ///
    /// In handler mode the focused card uses the Interactive variant (others stay
    /// Outlined) and its title is prefixed with `▸ `. Keys do not depend on
    /// focus. Ignored without an `on_toggle` handler; an unknown value marks
    /// nothing.
    pub fn focused(mut self, value: impl Into<String>) -> Self {
        self.focused = Some(value.into());
        self
    }

    /// Value of the next (`forward`) or previous option after `current`,
    /// wrapping.
    ///
    /// With no or an unknown `current`, returns the first option (forward) or
    /// the last option (backward). Returns `None` when there are no options.
    /// This only moves focus: it never emits a toggle message and does not
    /// require a handler. Option values must be unique.
    pub fn navigate_focus(&self, current: Option<&str>, forward: bool) -> Option<&str> {
        let len = self.options.len();
        if len == 0 {
            return None;
        }
        let index = current.and_then(|c| self.options.iter().position(|o| o.value == c));
        let next = match (index, forward) {
            (Some(i), true) => (i + 1) % len,
            (Some(i), false) => (i + len - 1) % len,
            (None, true) => 0,
            (None, false) => len - 1,
        };
        Some(&self.options[next].value)
    }
}

impl<M: Clone + 'static> From<CheckboxCardsBuilder<M>> for WidgetNode<M> {
    fn from(b: CheckboxCardsBuilder<M>) -> Self {
        let prefix = b.id.as_str().to_owned();
        let mut grid = crate::column::<M>().key(b.id).gap(8.0).padding(4.0);
        for option in &b.options {
            let is_checked = b.selected.contains(&option.value);
            let marker = if is_checked { "☑" } else { "☐" };
            let mut card = if let Some(toggle) = b.on_toggle {
                let is_focused = b.focused.as_deref() == Some(option.value.as_str());
                let (card_variant, title) = if is_focused {
                    (
                        acme_widgets::CardVariant::Interactive,
                        format!("▸ {} {}", marker, option.title),
                    )
                } else {
                    (
                        acme_widgets::CardVariant::Outlined,
                        format!("{} {}", marker, option.title),
                    )
                };
                let variant = if is_checked {
                    crate::ButtonVariant::Primary
                } else {
                    crate::ButtonVariant::Ghost
                };
                crate::card::<M>()
                    .key(format!("{prefix}:card:{}", option.value))
                    .variant(card_variant)
                    .padding(12.0)
                    .gap(4.0)
                    .child(
                        crate::button(format!("{prefix}:option:{}", option.value), title)
                            .variant(variant)
                            .full_width(true)
                            .on_click(toggle(&option.value)),
                    )
            } else {
                crate::card::<M>()
                    .variant(acme_widgets::CardVariant::Outlined)
                    .padding(12.0)
                    .gap(4.0)
                    .child(crate::label(format!("{} {}", marker, option.title)))
            };
            if let Some(desc) = &option.description {
                card = card.child(crate::label(desc.clone()));
            }
            grid = grid.child(card.build());
        }
        grid.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Toggled(String),
    }

    fn toggle_msg(v: &str) -> Msg {
        Msg::Toggled(v.to_string())
    }

    #[test]
    fn checkbox_cards_produces_column() {
        let node: WidgetNode<Msg> = checkbox_cards()
            .option(CheckboxCardOption::new("a", "Option A"))
            .into();
        assert!(matches!(node, WidgetNode::Column(_)));
    }

    #[test]
    fn checkbox_cards_child_count() {
        let node: WidgetNode<Msg> = checkbox_cards()
            .option(CheckboxCardOption::new("a", "A"))
            .option(CheckboxCardOption::new("b", "B"))
            .into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column");
        };
        assert_eq!(c.children.len(), 2);
    }

    #[test]
    fn checkbox_cards_selected_default_empty() {
        let b = checkbox_cards::<Msg>();
        assert!(b.selected.is_empty());
    }

    #[test]
    fn checkbox_cards_on_toggle_emits_toggled_with_value() {
        let b = checkbox_cards::<Msg>().on_toggle(toggle_msg);
        assert_eq!(
            b.on_toggle.map(|f| f("a")),
            Some(Msg::Toggled("a".to_string()))
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

    fn group(
        key: &str,
        values: &[&str],
        checked: &[&str],
        focused: Option<&str>,
    ) -> CheckboxCardsBuilder<Msg> {
        let mut b = checkbox_cards::<Msg>().key(key).on_toggle(toggle_msg);
        for v in values {
            b = b.option(
                CheckboxCardOption::new(*v, format!("Item {v} · 項目")).description("desc"),
            );
        }
        b = b.selected(checked.to_vec());
        if let Some(f) = focused {
            b = b.focused(f);
        }
        b
    }

    fn button<'a>(node: &'a WidgetNode<Msg>, key: &str) -> &'a acme_widgets::Button<Msg> {
        match find_by_key(node, key) {
            Some(WidgetNode::Button(b)) => b,
            _ => panic!("missing Button {key}"),
        }
    }

    fn card_variant(node: &WidgetNode<Msg>, key: &str) -> acme_widgets::CardVariant {
        match find_by_key(node, key) {
            Some(WidgetNode::Card(c)) => c.variant,
            _ => panic!("missing Card {key}"),
        }
    }

    /// Replica of the pre-058 passive conversion, used to pin the legacy tree.
    fn legacy_tree(b: &CheckboxCardsBuilder<Msg>) -> WidgetNode<Msg> {
        let mut grid = crate::column::<Msg>()
            .key(b.id.clone())
            .gap(8.0)
            .padding(4.0);
        for option in &b.options {
            let is_checked = b.selected.contains(&option.value);
            let marker = if is_checked { "☑" } else { "☐" };
            let mut card = crate::card::<Msg>()
                .variant(acme_widgets::CardVariant::Outlined)
                .padding(12.0)
                .gap(4.0)
                .child(crate::label(format!("{} {}", marker, option.title)));
            if let Some(desc) = &option.description {
                card = card.child(crate::label(desc.clone()));
            }
            grid = grid.child(card.build());
        }
        grid.build()
    }

    #[test]
    fn checkbox_cards_without_handler_matches_legacy_tree() {
        let build = || {
            checkbox_cards::<Msg>()
                .key("passive")
                .option(CheckboxCardOption::new("a", "A").description("first"))
                .option(CheckboxCardOption::new("b", "B"))
                .option(CheckboxCardOption::new("c", "C"))
                .selected(vec!["a", "c"])
        };
        let expected = legacy_tree(&build());
        let node: WidgetNode<Msg> = build().into();
        assert_eq!(node, expected);
        // `focused` is a handler-mode affordance; passive trees ignore it.
        let focused: WidgetNode<Msg> = build().focused("b").into();
        assert_eq!(focused, expected);
        let mut keys = Vec::new();
        collect_keys(&node, &mut keys);
        assert_eq!(keys, ["passive"]);
    }

    #[test]
    fn checkbox_cards_on_toggle_binds_instance_keyed_buttons() {
        let node: WidgetNode<Msg> = group("notify", &["a", "b"], &["b"], None).into();
        assert!(matches!(node, WidgetNode::Column(_)));
        assert_eq!(node.key().unwrap().as_str(), "notify");
        assert_eq!(node.children().len(), 2);
        for (index, (value, marker)) in [("a", "☐"), ("b", "☑")].into_iter().enumerate() {
            let card = &node.children()[index];
            assert!(matches!(card, WidgetNode::Card(_)));
            assert_eq!(card.key().unwrap().as_str(), format!("notify:card:{value}"));
            let b = button(card, &format!("notify:option:{value}"));
            assert_eq!(b.label, format!("{marker} Item {value} · 項目"));
            assert!(b.full_width);
            assert_eq!(b.activate(), Some(&Msg::Toggled(value.to_string())));
            assert!(
                card.children()
                    .iter()
                    .any(|c| matches!(c, WidgetNode::Label(l) if l.text == "desc"))
            );
        }
        let mut keys = Vec::new();
        collect_keys(&node, &mut keys);
        let mut unique = keys.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(keys.len(), 5);
        assert_eq!(unique.len(), keys.len(), "duplicate keys: {keys:?}");
    }

    #[test]
    fn checkbox_cards_checked_is_primary_unchecked_ghost() {
        let node: WidgetNode<Msg> = group("c", &["a", "b", "c"], &["a", "c"], None).into();
        for (value, variant, marker) in [
            ("a", crate::ButtonVariant::Primary, "☑"),
            ("b", crate::ButtonVariant::Ghost, "☐"),
            ("c", crate::ButtonVariant::Primary, "☑"),
        ] {
            let b = button(&node, &format!("c:option:{value}"));
            assert_eq!(b.variant, variant, "option {value}");
            assert!(b.label.starts_with(marker), "option {value}: {}", b.label);
            assert_eq!(
                card_variant(&node, &format!("c:card:{value}")),
                acme_widgets::CardVariant::Outlined
            );
        }
        // Theme tokens only: production code must not construct colors directly.
        let source = include_str!("checkbox_cards.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        for forbidden in ["Color", "rgb(", "rgba(", "srgb", "0x"] {
            assert!(!production.contains(forbidden), "found {forbidden}");
        }
    }

    #[test]
    fn checkbox_cards_focused_card_uses_interactive_variant_and_marker() {
        let node: WidgetNode<Msg> = group("f", &["a", "b", "c"], &["c"], Some("b")).into();
        assert_eq!(
            card_variant(&node, "f:card:b"),
            acme_widgets::CardVariant::Interactive
        );
        for other in ["a", "c"] {
            assert_eq!(
                card_variant(&node, &format!("f:card:{other}")),
                acme_widgets::CardVariant::Outlined
            );
        }
        assert_eq!(button(&node, "f:option:b").label, "▸ ☐ Item b · 項目");
        assert_eq!(button(&node, "f:option:a").label, "☐ Item a · 項目");
        assert_eq!(button(&node, "f:option:c").label, "☑ Item c · 項目");
        // Focus does not change the checked variant.
        assert_eq!(
            button(&node, "f:option:b").variant,
            crate::ButtonVariant::Ghost
        );

        let node: WidgetNode<Msg> = group("f", &["a", "b", "c"], &["c"], Some("c")).into();
        assert_eq!(button(&node, "f:option:c").label, "▸ ☑ Item c · 項目");
        assert_eq!(
            button(&node, "f:option:c").variant,
            crate::ButtonVariant::Primary
        );
        assert_eq!(
            card_variant(&node, "f:card:c"),
            acme_widgets::CardVariant::Interactive
        );

        // An unknown focused value marks nothing.
        let node: WidgetNode<Msg> = group("f", &["a", "b"], &[], Some("zz")).into();
        for v in ["a", "b"] {
            assert_eq!(
                card_variant(&node, &format!("f:card:{v}")),
                acme_widgets::CardVariant::Outlined
            );
            assert!(
                !button(&node, &format!("f:option:{v}"))
                    .label
                    .starts_with('▸')
            );
        }
    }

    #[test]
    fn checkbox_cards_keys_stable_across_checked_and_focus() {
        let values = ["a", "b", "c"];
        let keys_of = |checked: &[&str], focused: Option<&str>| {
            let node: WidgetNode<Msg> = group("k", &values, checked, focused).into();
            let mut keys = Vec::new();
            collect_keys(&node, &mut keys);
            keys
        };
        let baseline = keys_of(&[], None);
        assert_eq!(
            baseline,
            [
                "k",
                "k:card:a",
                "k:option:a",
                "k:card:b",
                "k:option:b",
                "k:card:c",
                "k:option:c"
            ]
        );
        let checked_sets: [&[&str]; 4] = [&["a"], &["a", "b"], &["a", "b", "c"], &["c"]];
        for checked in checked_sets {
            for focused in [None, Some("a"), Some("b"), Some("c"), Some("zz")] {
                assert_eq!(keys_of(checked, focused), baseline);
            }
        }
    }

    #[test]
    fn checkbox_cards_navigate_focus_wraps_and_handles_none_and_empty() {
        let b = group("n", &["a", "b", "c"], &["b"], None);
        assert_eq!(b.navigate_focus(Some("a"), true), Some("b"));
        assert_eq!(b.navigate_focus(Some("b"), true), Some("c"));
        assert_eq!(b.navigate_focus(Some("c"), true), Some("a"));
        assert_eq!(b.navigate_focus(Some("a"), false), Some("c"));
        assert_eq!(b.navigate_focus(Some("c"), false), Some("b"));
        assert_eq!(b.navigate_focus(None, true), Some("a"));
        assert_eq!(b.navigate_focus(None, false), Some("c"));
        // Navigation never toggles: checked state is unchanged.
        assert_eq!(b.selected, vec!["b".to_string()]);

        let single = group("n", &["only"], &[], None);
        assert_eq!(single.navigate_focus(Some("only"), true), Some("only"));
        assert_eq!(single.navigate_focus(Some("only"), false), Some("only"));

        let empty = group("n", &[], &[], None);
        assert_eq!(empty.navigate_focus(None, true), None);
        assert_eq!(empty.navigate_focus(Some("a"), false), None);

        // Focus navigation does not require a toggle handler.
        let passive = checkbox_cards::<Msg>()
            .option(CheckboxCardOption::new("x", "X"))
            .option(CheckboxCardOption::new("y", "Y"));
        assert_eq!(passive.navigate_focus(Some("x"), true), Some("y"));
    }

    #[test]
    fn checkbox_cards_navigate_focus_unknown_current_starts_at_edge() {
        let b = group("u", &["a", "b", "c"], &[], Some("a"));
        assert_eq!(b.navigate_focus(Some("zz"), true), Some("a"));
        assert_eq!(b.navigate_focus(Some("zz"), false), Some("c"));
        assert_eq!(b.navigate_focus(Some(""), true), Some("a"));
        assert_eq!(b.navigate_focus(Some(""), false), Some("c"));
    }
}
