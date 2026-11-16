//! Sidebar — collapsible application sidebar with variants.
//! Aligns with shadcn/ui Sidebar.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Sidebar variant.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SidebarVariant {
    #[default]
    Sidebar,
    Floating,
    Inset,
}

/// Sidebar collapse state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SidebarState {
    #[default]
    Expanded,
    Collapsed,
}

/// A sidebar navigation item.
#[derive(Clone, Debug)]
pub struct SidebarItem {
    pub label: String,
    pub icon: Option<crate::IconName>,
    pub active: bool,
    /// Value passed to the [`on_select`](SidebarBuilder::on_select) handler;
    /// defaults to the label.
    pub value: String,
    /// Disabled items never emit a selection message.
    pub disabled: bool,
}

impl SidebarItem {
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();
        Self {
            value: label.clone(),
            label,
            icon: None,
            active: false,
            disabled: false,
        }
    }

    /// Set the value passed to the selection handler (defaults to the label).
    ///
    /// Values must be unique within the sidebar and must not contain `:`,
    /// because they are part of the item keys (not enforced).
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    /// Set the disabled state. Only affects the [`on_select`](SidebarBuilder::on_select)
    /// tree; without a handler items are passive labels.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }

    pub fn icon(mut self, value: crate::IconName) -> Self {
        self.icon = Some(value);
        self
    }

    pub fn active(mut self, value: bool) -> Self {
        self.active = value;
        self
    }
}

/// Builder for a sidebar.
pub struct SidebarBuilder<M> {
    pub id: WidgetKey,
    pub variant: SidebarVariant,
    pub state: SidebarState,
    pub items: Vec<SidebarItem>,
    pub header: Option<WidgetNode<M>>,
    pub footer: Option<WidgetNode<M>>,
    pub width: f32,
    pub collapsed_width: f32,
    on_select: Option<fn(&str) -> M>,
}

/// Index of the next enabled entry after (`forward`) or before `current`,
/// wrapping. With no current index, the first (forward) or last (backward)
/// enabled entry is returned. `None` when no entry is enabled.
fn rove(disabled: &[bool], current: Option<usize>, forward: bool) -> Option<usize> {
    let len = disabled.len();
    if len == 0 {
        return None;
    }
    let start = current.unwrap_or(if forward { len - 1 } else { 0 });
    (1..=len)
        .map(|step| {
            if forward {
                (start + step) % len
            } else {
                (start + len - step % len) % len
            }
        })
        .find(|&index| !disabled[index])
}

/// Create a sidebar builder.
pub fn sidebar<M: Clone + 'static>() -> SidebarBuilder<M> {
    SidebarBuilder {
        id: WidgetKey::from("sidebar"),
        variant: SidebarVariant::default(),
        state: SidebarState::default(),
        items: Vec::new(),
        header: None,
        footer: None,
        width: 256.0,
        collapsed_width: 64.0,
        on_select: None,
    }
}

impl<M: Clone + 'static> SidebarBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn variant(mut self, value: SidebarVariant) -> Self {
        self.variant = value;
        self
    }

    pub fn state(mut self, value: SidebarState) -> Self {
        self.state = value;
        self
    }

    pub fn item(mut self, item: SidebarItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn header(mut self, node: WidgetNode<M>) -> Self {
        self.header = Some(node);
        self
    }

    pub fn footer(mut self, node: WidgetNode<M>) -> Self {
        self.footer = Some(node);
        self
    }

    pub fn width(mut self, value: f32) -> Self {
        self.width = value;
        self
    }

    pub fn collapsed_width(mut self, value: f32) -> Self {
        self.collapsed_width = value;
        self
    }

    /// Build each enabled item's typed selection message from its value.
    ///
    /// With a handler, the Column keeps the key `"{id}"` and each item is a
    /// full-width Button keyed `"{id}:item:{value}"` (independent of the
    /// collapse state). The current (`active`) item uses the primary variant
    /// and a `▸ ` marker, the others ghost with a four-space pad that matches
    /// the marker width; collapsed, items show the icon (or the label's first
    /// character) with the marker kept on the current item. Expanded items are
    /// `TextAlign::Left` (label at the button's left padding, see
    /// `Button::label_x`); collapsed items stay centred. Disabled items get no
    /// click message. Values must be unique within the sidebar and must not
    /// contain `:` (not enforced). Without a handler the legacy label tree is
    /// built unchanged.
    pub fn on_select(mut self, f: fn(&str) -> M) -> Self {
        self.on_select = Some(f);
        self
    }

    /// Message selecting the next (`forward`) or previous enabled item,
    /// starting from the first `active` item, wrapping and skipping disabled
    /// items.
    ///
    /// With no current item, returns the first (forward) or last (backward)
    /// enabled item. Returns `None` without an `on_select` handler or when no
    /// item is enabled.
    pub fn navigate(&self, forward: bool) -> Option<M> {
        let handler = self.on_select?;
        let disabled: Vec<bool> = self.items.iter().map(|i| i.disabled).collect();
        let current = self.items.iter().position(|i| i.active);
        let next = rove(&disabled, current, forward)?;
        Some(handler(&self.items[next].value))
    }
}

/// Handler-mode prefix for unmarked expanded items. The `▸ ` marker shapes to
/// ~1.14 em in the Windows default font (14.76px at the 13px label size,
/// measured with `acme-text`; other font stacks may differ), while two spaces are only ~0.55 em (7.12px); four spaces
/// (14.24px) keep every label in the same column within 0.6px at 12–16px.
const UNMARKED_PREFIX: &str = "    ";

/// Button label for an item in the typed (`on_select`) tree.
fn typed_label(item: &SidebarItem, state: SidebarState) -> String {
    match state {
        SidebarState::Expanded => {
            let prefix = if item.active { "▸ " } else { UNMARKED_PREFIX };
            match &item.icon {
                Some(icon) => format!("{}{} {}", prefix, icon.char(), item.label),
                None => format!("{}{}", prefix, item.label),
            }
        }
        SidebarState::Collapsed => {
            let glyph = match &item.icon {
                Some(icon) => icon.char().to_string(),
                None => item
                    .label
                    .chars()
                    .next()
                    .map(String::from)
                    .unwrap_or_default(),
            };
            if item.active {
                format!("▸ {glyph}")
            } else {
                glyph
            }
        }
    }
}

impl<M: Clone + 'static> From<SidebarBuilder<M>> for WidgetNode<M> {
    fn from(b: SidebarBuilder<M>) -> Self {
        let effective_width = match b.state {
            SidebarState::Expanded => b.width,
            SidebarState::Collapsed => b.collapsed_width,
        };

        let prefix_id = b.id.as_str().to_owned();
        let mut col = crate::column::<M>()
            .key(b.id)
            .width(effective_width)
            .gap(4.0)
            .padding(8.0);

        if let Some(header) = b.header {
            col = col.child(header);
            col = col.child(crate::separator::<M>());
        }

        for item in &b.items {
            if let Some(select) = b.on_select {
                let variant = if item.active {
                    crate::ButtonVariant::Primary
                } else {
                    crate::ButtonVariant::Ghost
                };
                let mut btn = crate::button::<M>(
                    format!("{prefix_id}:item:{}", item.value),
                    typed_label(item, b.state),
                )
                .variant(variant)
                .full_width(true)
                .disabled(item.disabled);
                // Expanded labels read as a list; collapsed glyphs stay centred.
                if b.state == SidebarState::Expanded {
                    btn = acme_widgets::style::Styled::text_align(
                        btn,
                        acme_widgets::style::TextAlign::Left,
                    );
                }
                col = col.child(if item.disabled {
                    btn.into()
                } else {
                    btn.on_click(select(&item.value))
                });
                continue;
            }
            let prefix = if item.active { "▸ " } else { "  " };
            let text = match (&item.icon, b.state) {
                (Some(icon), SidebarState::Expanded) => {
                    format!("{}{} {}", prefix, icon.char(), item.label)
                }
                (Some(icon), SidebarState::Collapsed) => icon.char().to_string(),
                (None, _) => format!("{}{}", prefix, item.label),
            };
            col = col.child(crate::label(text));
        }

        if let Some(footer) = b.footer {
            col = col.child(crate::separator::<M>());
            col = col.child(footer);
        }
        col.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn sidebar_produces_column() {
        let node: WidgetNode<Msg> = sidebar().into();
        assert!(matches!(node, WidgetNode::Column(_)));
    }

    #[test]
    fn sidebar_with_items() {
        let node: WidgetNode<Msg> = sidebar()
            .item(SidebarItem::new("Home").icon(crate::IconName::User))
            .item(SidebarItem::new("Settings").icon(crate::IconName::Settings))
            .into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column");
        };
        assert_eq!(c.children.len(), 2);
    }

    #[test]
    fn sidebar_collapsed_uses_narrow_width() {
        let b = sidebar::<Msg>().state(SidebarState::Collapsed);
        assert_eq!(b.state, SidebarState::Collapsed);
    }

    #[test]
    fn sidebar_with_header_and_footer() {
        let node: WidgetNode<Msg> = sidebar()
            .header(crate::label("Logo"))
            .item(SidebarItem::new("Nav"))
            .footer(crate::label("User"))
            .into();
        let WidgetNode::Column(c) = &node else {
            panic!("expected Column");
        };
        // header + separator + item + separator + footer = 5
        assert_eq!(c.children.len(), 5);
    }

    // ---- ROADMAP 057: typed activation and roving keyboard navigation ----

    #[derive(Clone, Debug, PartialEq)]
    enum Nav {
        Go(String),
    }

    fn go(v: &str) -> Nav {
        Nav::Go(v.to_string())
    }

    fn find_by_key<'a>(node: &'a WidgetNode<Nav>, key: &str) -> Option<&'a WidgetNode<Nav>> {
        if node.key().map(|k| k.as_str()) == Some(key) {
            return Some(node);
        }
        node.children().iter().find_map(|c| find_by_key(c, key))
    }

    fn button_at<'a>(node: &'a WidgetNode<Nav>, key: &str) -> &'a crate::Button<Nav> {
        match find_by_key(node, key) {
            Some(WidgetNode::Button(b)) => b,
            _ => panic!("missing button {key}"),
        }
    }

    fn collect_keys(node: &WidgetNode<Nav>, out: &mut Vec<String>) {
        if let Some(k) = node.key() {
            out.push(k.as_str().to_owned());
        }
        for child in node.children() {
            collect_keys(child, out);
        }
    }

    /// Replica of the pre-057 build, used to pin the legacy tree.
    fn legacy_tree(
        id: &str,
        state: SidebarState,
        items: &[SidebarItem],
        header: Option<WidgetNode<Nav>>,
        footer: Option<WidgetNode<Nav>>,
    ) -> WidgetNode<Nav> {
        let width = match state {
            SidebarState::Expanded => 256.0,
            SidebarState::Collapsed => 64.0,
        };
        let mut col = crate::column::<Nav>()
            .key(id)
            .width(width)
            .gap(4.0)
            .padding(8.0);
        if let Some(header) = header {
            col = col.child(header);
            col = col.child(crate::separator::<Nav>());
        }
        for item in items {
            let prefix = if item.active { "▸ " } else { "  " };
            let text = match (&item.icon, state) {
                (Some(icon), SidebarState::Expanded) => {
                    format!("{}{} {}", prefix, icon.char(), item.label)
                }
                (Some(icon), SidebarState::Collapsed) => icon.char().to_string(),
                (None, _) => format!("{}{}", prefix, item.label),
            };
            col = col.child(crate::label(text));
        }
        if let Some(footer) = footer {
            col = col.child(crate::separator::<Nav>());
            col = col.child(footer);
        }
        col.build()
    }

    /// Inbox (icon), Drafts (current, icon), disabled Spam, Archive (no icon).
    fn items() -> Vec<SidebarItem> {
        vec![
            SidebarItem::new("Inbox").icon(crate::IconName::User),
            SidebarItem::new("Drafts")
                .value("drafts")
                .icon(crate::IconName::Settings)
                .active(true),
            SidebarItem::new("Spam").value("spam").disabled(true),
            SidebarItem::new("Archive").value("archive"),
        ]
    }

    fn passive(key: &str, state: SidebarState, items: Vec<SidebarItem>) -> SidebarBuilder<Nav> {
        let mut b = sidebar::<Nav>().key(key).state(state);
        for item in items {
            b = b.item(item);
        }
        b
    }

    fn typed(key: &str, state: SidebarState, items: Vec<SidebarItem>) -> SidebarBuilder<Nav> {
        passive(key, state, items).on_select(go)
    }

    fn with_current(current: Option<&str>) -> Vec<SidebarItem> {
        items()
            .into_iter()
            .map(|i| {
                let on = current == Some(i.value.as_str());
                i.active(on)
            })
            .collect()
    }

    #[test]
    fn sidebar_without_handler_matches_legacy_tree_expanded_collapsed_header_footer() {
        for state in [SidebarState::Expanded, SidebarState::Collapsed] {
            for chrome in [false, true] {
                let mut b = passive("side", state, items());
                let (header, footer) = if chrome {
                    b = b.header(crate::label("Logo")).footer(crate::label("User"));
                    (Some(crate::label("Logo")), Some(crate::label("User")))
                } else {
                    (None, None)
                };
                assert!(b.navigate(true).is_none());
                assert!(b.navigate(false).is_none());
                let expected = legacy_tree("side", state, &items(), header, footer);
                let node: WidgetNode<Nav> = b.into();
                assert_eq!(node, expected, "{state:?} chrome={chrome}");
                let mut keys = Vec::new();
                collect_keys(&node, &mut keys);
                assert_eq!(keys, ["side"]);
            }
        }
        // Default key is unchanged.
        let node: WidgetNode<Nav> = sidebar::<Nav>().item(SidebarItem::new("Nav")).into();
        assert_eq!(node.key().unwrap().as_str(), "sidebar");
    }

    #[test]
    fn sidebar_on_select_builds_instance_keyed_buttons_bound_to_values() {
        let node: WidgetNode<Nav> = typed("side", SidebarState::Expanded, items())
            .header(crate::label("Logo"))
            .footer(crate::label("User"))
            .into();
        assert!(matches!(node, WidgetNode::Column(_)));
        assert_eq!(node.key().unwrap().as_str(), "side");
        let children = node.children();
        // header + separator + 4 items + separator + footer
        assert_eq!(children.len(), 8);
        assert_eq!(children[0], crate::label::<Nav>("Logo"));
        assert_eq!(children[1], crate::separator::<Nav>());
        assert_eq!(children[6], crate::separator::<Nav>());
        assert_eq!(children[7], crate::label::<Nav>("User"));
        // The value defaults to the label.
        assert_eq!(SidebarItem::new("Inbox").value, "Inbox");
        let expected = [
            ("Inbox", true),
            ("drafts", true),
            ("spam", false),
            ("archive", true),
        ];
        for (index, (value, enabled)) in expected.into_iter().enumerate() {
            let WidgetNode::Button(button) = &children[2 + index] else {
                panic!("item {index} is not a Button");
            };
            assert_eq!(button.key.as_str(), format!("side:item:{value}"));
            assert!(button.full_width);
            assert_eq!(button.disabled, !enabled);
            let msg = go(value);
            assert_eq!(button.activate(), enabled.then_some(&msg));
        }
    }

    #[test]
    fn sidebar_disabled_item_never_activates() {
        for state in [SidebarState::Expanded, SidebarState::Collapsed] {
            for current in [
                None,
                Some("Inbox"),
                Some("drafts"),
                Some("spam"),
                Some("archive"),
            ] {
                let b = typed("side", state, with_current(current));
                for forward in [true, false] {
                    assert_ne!(b.navigate(forward), Some(go("spam")));
                }
                let node: WidgetNode<Nav> = b.into();
                let spam = button_at(&node, "side:item:spam");
                assert!(spam.disabled);
                assert!(spam.activate().is_none());
            }
        }
        let only_off = typed(
            "side",
            SidebarState::Expanded,
            vec![SidebarItem::new("Off").disabled(true).active(true)],
        );
        assert_eq!(only_off.navigate(true), None);
    }

    #[test]
    fn sidebar_current_item_is_primary_and_marked_expanded_and_collapsed() {
        use crate::ButtonVariant::{Ghost, Primary};
        let user = crate::IconName::User.char();
        let gear = crate::IconName::Settings.char();
        let label_of = |node: &WidgetNode<Nav>, k: &str| button_at(node, k).label.clone();
        let variant_of = |node: &WidgetNode<Nav>, k: &str| button_at(node, k).variant;

        let expanded: WidgetNode<Nav> = typed("s", SidebarState::Expanded, items()).into();
        assert_eq!(
            label_of(&expanded, "s:item:Inbox"),
            format!("    {user} Inbox")
        );
        assert_eq!(
            label_of(&expanded, "s:item:drafts"),
            format!("▸ {gear} Drafts")
        );
        assert_eq!(label_of(&expanded, "s:item:spam"), "    Spam");
        assert_eq!(label_of(&expanded, "s:item:archive"), "    Archive");
        assert_eq!(variant_of(&expanded, "s:item:drafts"), Primary);
        for k in ["s:item:Inbox", "s:item:spam", "s:item:archive"] {
            assert_eq!(variant_of(&expanded, k), Ghost);
        }

        let collapsed: WidgetNode<Nav> = typed("s", SidebarState::Collapsed, items()).into();
        assert_eq!(label_of(&collapsed, "s:item:Inbox"), user);
        assert_eq!(label_of(&collapsed, "s:item:drafts"), format!("▸ {gear}"));
        assert_eq!(label_of(&collapsed, "s:item:spam"), "S");
        assert_eq!(label_of(&collapsed, "s:item:archive"), "A");
        assert_eq!(variant_of(&collapsed, "s:item:drafts"), Primary);
        for k in ["s:item:Inbox", "s:item:spam", "s:item:archive"] {
            assert_eq!(variant_of(&collapsed, k), Ghost);
        }

        // A current item without an icon keeps its marker, using the first
        // char (not byte) of a multi-byte label.
        let cjk: WidgetNode<Nav> = typed(
            "s",
            SidebarState::Collapsed,
            vec![
                SidebarItem::new("設定").value("settings").active(true),
                SidebarItem::new("首頁").value("home"),
            ],
        )
        .into();
        assert_eq!(label_of(&cjk, "s:item:settings"), "▸ 設");
        assert_eq!(label_of(&cjk, "s:item:home"), "首");
        assert_eq!(variant_of(&cjk, "s:item:settings"), Primary);
        assert_eq!(variant_of(&cjk, "s:item:home"), Ghost);

        // Theme tokens only: production code must not construct colors directly.
        let source = include_str!("sidebar.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        for forbidden in ["Color", "rgb(", "rgba(", "srgb", "0x"] {
            assert!(!production.contains(forbidden), "found {forbidden}");
        }
    }

    #[test]
    fn sidebar_navigate_wraps_and_skips_disabled() {
        let nav = |c: Option<&str>, f: bool| {
            typed("s", SidebarState::Expanded, with_current(c)).navigate(f)
        };
        let sel = |v: &str| Some(go(v));
        assert_eq!(nav(Some("Inbox"), true), sel("drafts"));
        assert_eq!(nav(Some("drafts"), true), sel("archive"));
        assert_eq!(nav(Some("archive"), true), sel("Inbox"));
        assert_eq!(nav(Some("Inbox"), false), sel("archive"));
        assert_eq!(nav(Some("archive"), false), sel("drafts"));
        assert_eq!(nav(Some("drafts"), false), sel("Inbox"));
        // From a disabled current item, roving still moves to a neighbour.
        assert_eq!(nav(Some("spam"), true), sel("archive"));
        assert_eq!(nav(Some("spam"), false), sel("drafts"));
        // The first active item is the current one.
        let two_active: Vec<SidebarItem> = items()
            .into_iter()
            .map(|i| {
                let on = i.value == "Inbox" || i.value == "archive";
                i.active(on)
            })
            .collect();
        let b = typed("s", SidebarState::Collapsed, two_active);
        assert_eq!(b.navigate(true), sel("drafts"));
        assert_eq!(b.navigate(false), sel("archive"));
        // Collapse state does not affect navigation.
        for c in [Some("Inbox"), Some("drafts"), Some("archive")] {
            for f in [true, false] {
                assert_eq!(
                    typed("s", SidebarState::Collapsed, with_current(c)).navigate(f),
                    nav(c, f)
                );
            }
        }
    }

    #[test]
    fn sidebar_navigate_without_current_returns_first_or_last_enabled() {
        let b = typed("s", SidebarState::Expanded, with_current(None));
        assert_eq!(b.navigate(true), Some(go("Inbox")));
        assert_eq!(b.navigate(false), Some(go("archive")));
        let edges_off = typed(
            "s",
            SidebarState::Expanded,
            vec![
                SidebarItem::new("A").disabled(true),
                SidebarItem::new("B"),
                SidebarItem::new("C"),
                SidebarItem::new("D").disabled(true),
            ],
        );
        assert_eq!(edges_off.navigate(true), Some(go("B")));
        assert_eq!(edges_off.navigate(false), Some(go("C")));
    }

    #[test]
    fn sidebar_navigate_none_without_handler_or_enabled_item() {
        for c in [None, Some("Inbox"), Some("drafts")] {
            let b = passive("s", SidebarState::Expanded, with_current(c));
            assert_eq!(b.navigate(true), None);
            assert_eq!(b.navigate(false), None);
        }
        let empty = sidebar::<Nav>().on_select(go);
        assert_eq!(empty.navigate(true), None);
        assert_eq!(empty.navigate(false), None);
        let all_off = typed(
            "s",
            SidebarState::Expanded,
            vec![
                SidebarItem::new("A").disabled(true),
                SidebarItem::new("B").disabled(true).active(true),
            ],
        );
        assert_eq!(all_off.navigate(true), None);
        assert_eq!(all_off.navigate(false), None);
        let single = typed(
            "s",
            SidebarState::Expanded,
            vec![SidebarItem::new("Only").active(true)],
        );
        assert_eq!(single.navigate(true), Some(go("Only")));
        assert_eq!(single.navigate(false), Some(go("Only")));
    }

    #[test]
    fn sidebar_item_keys_stable_across_collapse_and_distinct_per_instance() {
        use acme_core::RetainedTree;
        use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};
        let keys_of = |key: &str, state: SidebarState, current: Option<&str>| {
            let node: WidgetNode<Nav> = typed(key, state, with_current(current))
                .header(crate::label("Logo"))
                .into();
            let mut keys = Vec::new();
            collect_keys(&node, &mut keys);
            keys
        };
        let expected = [
            "a",
            "a:item:Inbox",
            "a:item:drafts",
            "a:item:spam",
            "a:item:archive",
        ];
        for current in [None, Some("drafts"), Some("archive")] {
            assert_eq!(keys_of("a", SidebarState::Expanded, current), expected);
            assert_eq!(keys_of("a", SidebarState::Collapsed, current), expected);
        }
        let a = keys_of("a", SidebarState::Expanded, None);
        let b = keys_of("b", SidebarState::Expanded, None);
        assert!(a.iter().all(|k| !b.contains(k)), "{a:?} vs {b:?}");

        // Retained ids survive collapse toggles and selection changes.
        let values = ["Inbox", "drafts", "spam", "archive"];
        let mut tree = RetainedTree::new();
        let mut saved = None;
        let steps = [
            (SidebarState::Expanded, None, false),
            (SidebarState::Collapsed, Some("drafts"), true),
            (SidebarState::Expanded, Some("archive"), false),
            (SidebarState::Collapsed, None, true),
        ];
        for (state, current, reverse) in steps {
            let build =
                |k: &str| -> WidgetNode<Nav> { typed(k, state, with_current(current)).into() };
            let mut roots = vec![build("x"), build("y")];
            if reverse {
                roots.reverse();
            }
            tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
            let binding = bind_retained(&roots, &tree);
            assert_eq!(binding.mismatches(), 0);
            let ids = ["x", "y"].map(|k| {
                let root = roots
                    .iter()
                    .find(|n| n.key().unwrap().as_str() == k)
                    .unwrap();
                let items = values.map(|v| {
                    let node = find_by_key(root, &format!("{k}:item:{v}")).unwrap();
                    binding.id_for(node).unwrap()
                });
                (binding.id_for(root).unwrap(), items)
            });
            assert_ne!(ids[0], ids[1]);
            if let Some(prev) = saved {
                assert_eq!(ids, prev);
            }
            saved = Some(ids);
        }
    }

    // ---- ROADMAP 063: left-aligned item labels ----

    fn buttons(node: &WidgetNode<Nav>, out: &mut Vec<crate::Button<Nav>>) {
        if let WidgetNode::Button(b) = node {
            out.push(b.clone());
        }
        for child in node.children() {
            buttons(child, out);
        }
    }

    fn any_text_align(node: &WidgetNode<Nav>) -> bool {
        matches!(node, WidgetNode::Button(b) if b.style.text_align.is_some())
            || node.children().iter().any(any_text_align)
    }

    #[test]
    fn sidebar_and_sidenav_handler_items_are_left_aligned_collapsed_centered() {
        use crate::style::TextAlign;
        // Sidebar expanded: every item (current, plain, disabled) is Left and
        // its label starts at rect_x + 12 regardless of label width.
        for current in [None, Some("drafts"), Some("archive")] {
            let node: WidgetNode<Nav> =
                typed("s", SidebarState::Expanded, with_current(current)).into();
            let mut bs = Vec::new();
            buttons(&node, &mut bs);
            assert_eq!(bs.len(), 4);
            for b in &bs {
                assert_eq!(
                    b.style.text_align,
                    Some(TextAlign::Left),
                    "{}",
                    b.key.as_str()
                );
                assert_eq!(b.label_x(8.0, 240.0, 40.0), 20.0);
                assert_eq!(b.label_x(8.0, 240.0, 120.0), 20.0);
            }
        }
        // Sidebar collapsed: glyph-only buttons stay centred (alignment unset).
        let node: WidgetNode<Nav> = typed("s", SidebarState::Collapsed, items()).into();
        let mut bs = Vec::new();
        buttons(&node, &mut bs);
        assert_eq!(bs.len(), 4);
        for b in &bs {
            assert_eq!(b.style.text_align, None, "{}", b.key.as_str());
            assert_eq!(b.label_x(8.0, 48.0, 10.0), 8.0 + 19.0);
        }
        // Sidenav (handler mode): every item is Left; headers stay labels.
        #[cfg(feature = "desktop")]
        {
            use crate::desktop::sidenav::{sidenav, sidenav_item, sidenav_section};
            for selected in [None, Some("profile"), Some("privacy")] {
                let mut b = sidenav::<Nav>("n")
                    .section(
                        sidenav_section("General")
                            .item(sidenav_item("Profile", "profile").icon("@"))
                            .item(sidenav_item("Appearance", "appearance")),
                    )
                    .section(
                        sidenav_section("Advanced")
                            .item(sidenav_item("Privacy", "privacy"))
                            .item(sidenav_item("Security", "security").disabled()),
                    );
                if let Some(v) = selected {
                    b = b.selected(v);
                }
                let node: WidgetNode<Nav> = b.on_select(go).into();
                let mut bs = Vec::new();
                buttons(&node, &mut bs);
                assert_eq!(bs.len(), 4);
                for b in &bs {
                    assert_eq!(
                        b.style.text_align,
                        Some(TextAlign::Left),
                        "{}",
                        b.key.as_str()
                    );
                    assert_eq!(b.label_x(0.0, 200.0, 60.0), 12.0);
                }
            }
        }
    }

    #[test]
    fn sidebar_and_sidenav_without_handler_unchanged() {
        // Passive trees carry no Buttons and no alignment, and match the
        // pinned legacy replica exactly (Debug included).
        for state in [SidebarState::Expanded, SidebarState::Collapsed] {
            let node: WidgetNode<Nav> = passive("side", state, items()).into();
            let expected = legacy_tree("side", state, &items(), None, None);
            assert_eq!(node, expected);
            assert_eq!(format!("{node:?}"), format!("{expected:?}"));
            assert!(!any_text_align(&node));
            let mut bs = Vec::new();
            buttons(&node, &mut bs);
            assert!(bs.is_empty());
        }
        #[cfg(feature = "desktop")]
        {
            use crate::desktop::sidenav::{sidenav, sidenav_item, sidenav_section};
            let node: WidgetNode<Nav> = sidenav::<Nav>("n")
                .section(sidenav_section("General").item(sidenav_item("Profile", "profile")))
                .selected("profile")
                .into();
            assert!(!any_text_align(&node));
            let mut bs = Vec::new();
            buttons(&node, &mut bs);
            assert!(bs.is_empty());
        }
    }

    /// Advance of the shaped glyphs before byte `start` in `text`.
    fn label_start(fonts: &mut acme_text::FontSystem, text: &str, prefix_len: usize) -> f32 {
        let theme = acme_theme::Theme::light();
        let style = acme_text::TextStyle {
            font_size: theme.typography.label,
            ..acme_text::TextStyle::default()
        };
        let layout = fonts.shape(text, &style, acme_text::TextConstraints::default(), 1.0);
        layout
            .glyphs
            .iter()
            .find(|g| g.byte_range.start >= prefix_len)
            .map(|g| g.x)
            .expect("label glyph")
    }

    #[test]
    fn sidebar_marker_and_unmarked_prefix_label_columns_match() {
        // `▸ ` (U+25B8 + space) and two spaces differ by ~7.6px at 13px in the
        // default font, so handler mode pads unmarked items with
        // UNMARKED_PREFIX; the label must then start in the same column
        // (≤ 2px) whether or not the item is current, at both builders' labels.
        let mut fonts = acme_text::FontSystem::new();
        let two = label_start(&mut fonts, "  Inbox", 2);
        let marked = label_start(&mut fonts, "▸ Inbox", "▸ ".len());
        eprintln!("063 prefix width: marker={marked} two_spaces={two}");
        let node: WidgetNode<Nav> =
            typed("s", SidebarState::Expanded, with_current(Some("drafts"))).into();
        let mut bs = Vec::new();
        buttons(&node, &mut bs);
        let starts: Vec<(String, f32)> = bs
            .iter()
            .map(|b| {
                let prefix = if b.label.starts_with('▸') {
                    "▸ "
                } else {
                    UNMARKED_PREFIX
                };
                assert!(b.label.starts_with(prefix), "{}", b.label);
                (
                    b.label.clone(),
                    label_start(&mut fonts, &b.label, prefix.len()),
                )
            })
            .collect();
        #[cfg(feature = "desktop")]
        let starts = {
            use crate::desktop::sidenav::{sidenav, sidenav_item, sidenav_section};
            let node: WidgetNode<Nav> = sidenav::<Nav>("n")
                .section(
                    sidenav_section("General")
                        .item(sidenav_item("Profile", "profile"))
                        .item(sidenav_item("設定", "settings")),
                )
                .selected("profile")
                .on_select(go)
                .into();
            let mut bs = Vec::new();
            buttons(&node, &mut bs);
            let mut starts = starts;
            for b in &bs {
                let prefix = if b.label.starts_with('▸') {
                    "▸ "
                } else {
                    UNMARKED_PREFIX
                };
                assert!(b.label.starts_with(prefix), "{}", b.label);
                starts.push((
                    b.label.clone(),
                    label_start(&mut fonts, &b.label, prefix.len()),
                ));
            }
            starts
        };
        for (label, x) in &starts {
            eprintln!("063 label column: {label:?} -> {x}");
            // The space/marker width ratio depends on the system font stack;
            // the <= 2px column match is verified for the Windows default font
            // only. Other platforms record the measured offset (ROADMAP 063
            // follow-up: paint the marker in a gutter instead of a prefix).
            if cfg!(target_os = "windows") {
                assert!(
                    (x - marked).abs() <= 2.0,
                    "{label:?} starts at {x}, marker column {marked}"
                );
            }
        }
    }
}
