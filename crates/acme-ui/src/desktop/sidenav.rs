//! Sidenav — vertical side navigation menu with sections and items.
//!
//! Used in desktop applications for settings/configuration navigation.
//! Follows the AcmeUIKit alignment builder pattern.
//!
//! # Example
//!
//! ```rust,ignore
//! sidenav("sidenav1")
//!     .section(
//!         sidenav_section("General")
//!             .item(sidenav_item("Profile", "profile"))
//!             .item(sidenav_item("Appearance", "appearance")),
//!     )
//!     .section(
//!         sidenav_section("Advanced")
//!             .item(sidenav_item("Privacy", "privacy"))
//!             .item(sidenav_item("Security", "security")),
//!     )
//!     .selected("profile")
//! ```

use crate::*;

// ---------------------------------------------------------------------------
// SidenavItem
// ---------------------------------------------------------------------------

/// A single navigation item within a sidenav section.
#[derive(Clone, Debug)]
pub struct SidenavItem {
    pub label: String,
    pub value: String,
    pub icon: Option<String>,
    pub disabled: bool,
}

impl SidenavItem {
    /// Set an optional icon name for this item.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Mark this item as disabled.
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

// ---------------------------------------------------------------------------
// SidenavSection
// ---------------------------------------------------------------------------

/// A named section containing a list of navigation items.
#[derive(Clone, Debug)]
pub struct SidenavSection<M> {
    pub label: String,
    pub items: Vec<SidenavItem>,
    _phantom: std::marker::PhantomData<M>,
}

impl<M> SidenavSection<M> {
    /// Add a navigation item to this section.
    pub fn item(mut self, item: SidenavItem) -> Self {
        self.items.push(item);
        self
    }
}

// ---------------------------------------------------------------------------
// SidenavBuilder
// ---------------------------------------------------------------------------

/// Builder for a vertical side navigation menu.
///
/// Construct it with [`sidenav`]. The private `on_select` handler field means
/// a `SidenavBuilder` can no longer be built with a struct literal outside
/// this module (a source-compatibility change from earlier releases); the
/// public fields remain readable and assignable.
pub struct SidenavBuilder<M> {
    pub id: WidgetKey,
    pub sections: Vec<SidenavSection<M>>,
    pub selected_value: Option<String>,
    on_select: Option<fn(&str) -> M>,
}

// ---------------------------------------------------------------------------
// Factory functions
// ---------------------------------------------------------------------------

/// Create a new Sidenav builder.
pub fn sidenav<M: Clone + 'static>(id: impl Into<WidgetKey>) -> SidenavBuilder<M> {
    SidenavBuilder {
        id: id.into(),
        sections: vec![],
        selected_value: None,
        on_select: None,
    }
}

/// Create a single navigation item.
pub fn sidenav_item(label: impl Into<String>, value: impl Into<String>) -> SidenavItem {
    SidenavItem {
        label: label.into(),
        value: value.into(),
        icon: None,
        disabled: false,
    }
}

/// Create a new sidenav section with the given label.
pub fn sidenav_section<M>(label: impl Into<String>) -> SidenavSection<M> {
    SidenavSection {
        label: label.into(),
        items: vec![],
        _phantom: std::marker::PhantomData,
    }
}

// ---------------------------------------------------------------------------
// Builder methods
// ---------------------------------------------------------------------------

impl<M: Clone + 'static> SidenavBuilder<M> {
    /// Add a section containing navigation items.
    pub fn section(mut self, section: SidenavSection<M>) -> Self {
        self.sections.push(section);
        self
    }

    /// Set the value of the currently selected item.
    pub fn selected(mut self, value: impl Into<String>) -> Self {
        self.selected_value = Some(value.into());
        self
    }

    /// Build each enabled item's typed selection message from its value.
    ///
    /// With a handler, the Column keeps the key `"{id}"`, section headers stay
    /// labels, and each item is a full-width Button keyed `"{id}:item:{value}"`.
    /// The selected item uses the primary variant and a `▸ ` marker, the
    /// others ghost with a four-space pad that matches the marker width; an
    /// icon is shown as a text prefix before the label. Items are
    /// `TextAlign::Left` (label at the button's left padding, see
    /// `Button::label_x`). Disabled items get no click message. Values must be
    /// unique across all sections and must not contain `:` (not enforced).
    /// Without a handler the legacy Card tree is built unchanged.
    pub fn on_select(mut self, f: fn(&str) -> M) -> Self {
        self.on_select = Some(f);
        self
    }

    /// Message selecting the next (`forward`) or previous enabled item across
    /// all sections, starting from the selected item, wrapping and skipping
    /// disabled items.
    ///
    /// With no (or an unknown) selected value, returns the first (forward) or
    /// last (backward) enabled item. Returns `None` without an `on_select`
    /// handler or when no item is enabled.
    pub fn navigate(&self, forward: bool) -> Option<M> {
        let handler = self.on_select?;
        let items: Vec<&SidenavItem> = self.sections.iter().flat_map(|s| &s.items).collect();
        let disabled: Vec<bool> = items.iter().map(|i| i.disabled).collect();
        let current = self
            .selected_value
            .as_deref()
            .and_then(|v| items.iter().position(|i| i.value == v));
        let next = rove(&disabled, current, forward)?;
        Some(handler(&items[next].value))
    }
}

/// Index of the next enabled entry after (`forward`) or before `current`,
/// wrapping. With no current index, the first (forward) or last (backward)
/// enabled entry is returned. `None` when no entry is enabled.
///
/// Private copy of the layout sidebar's roving helper: the `desktop` feature
/// does not enable `layout`, so it cannot be shared from there.
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

/// Handler-mode prefix for unselected items: four spaces shape to the width of
/// the `▸ ` marker in the Windows default font within 0.6px (other font
/// stacks may differ) (two spaces are ~7.6px
/// narrower at 13px), so labels share one column. Mirrors the Sidebar rule.
const UNMARKED_PREFIX: &str = "    ";

/// Button label for an item in the typed (`on_select`) tree.
fn typed_label(item: &SidenavItem, selected: bool) -> String {
    let prefix = if selected { "▸ " } else { UNMARKED_PREFIX };
    match &item.icon {
        Some(icon) => format!("{prefix}{icon} {}", item.label),
        None => format!("{prefix}{}", item.label),
    }
}

// ---------------------------------------------------------------------------
// Conversion into WidgetNode
// ---------------------------------------------------------------------------

impl<M: Clone + 'static> From<SidenavBuilder<M>> for WidgetNode<M> {
    fn from(b: SidenavBuilder<M>) -> Self {
        let prefix_id = b.id.as_str().to_owned();
        let mut col = column::<M>().key(b.id);

        for section in &b.sections {
            // Section header label
            col = col.child(crate::label(&section.label));

            // Section items: typed Buttons with a handler, else legacy Cards
            for item in &section.items {
                let is_selected = b.selected_value.as_deref() == Some(item.value.as_str());
                if let Some(select) = b.on_select {
                    let variant = if is_selected {
                        ButtonVariant::Primary
                    } else {
                        ButtonVariant::Ghost
                    };
                    let btn = button::<M>(
                        format!("{prefix_id}:item:{}", item.value),
                        typed_label(item, is_selected),
                    )
                    .variant(variant)
                    .full_width(true)
                    .disabled(item.disabled);
                    let btn = acme_widgets::style::Styled::text_align(
                        btn,
                        acme_widgets::style::TextAlign::Left,
                    );
                    col = col.child(if item.disabled {
                        btn.into()
                    } else {
                        btn.on_click(select(&item.value))
                    });
                    continue;
                }
                let variant = if is_selected {
                    CardVariant::Interactive
                } else {
                    CardVariant::Plain
                };

                let card_key = format!("sidenav_item_{}", item.value);
                let card = card::<M>()
                    .key(card_key.as_str())
                    .variant(variant)
                    .child(crate::label(&item.label))
                    .build();

                col = col.child(card);
            }
        }

        col.build()
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

    // -- Roving helper ---------------------------------------------------

    #[test]
    fn sidenav_rove_wraps_skips_disabled_and_handles_none() {
        let d = [false, true, false, false];
        // Wrap forward and backward.
        assert_eq!(rove(&d, Some(3), true), Some(0));
        assert_eq!(rove(&d, Some(0), false), Some(3));
        // Skip disabled.
        assert_eq!(rove(&d, Some(0), true), Some(2));
        assert_eq!(rove(&d, Some(2), false), Some(0));
        // No current: first (forward) / last (backward) enabled.
        assert_eq!(rove(&d, None, true), Some(0));
        assert_eq!(rove(&d, None, false), Some(3));
        let lead_trail = [true, false, false, true];
        assert_eq!(rove(&lead_trail, None, true), Some(1));
        assert_eq!(rove(&lead_trail, None, false), Some(2));
        // Single enabled entry returns itself.
        assert_eq!(rove(&[true, false, true], Some(1), true), Some(1));
        // All disabled or empty.
        assert_eq!(rove(&[true, true], Some(0), true), None);
        assert_eq!(rove(&[true, true], None, false), None);
        assert_eq!(rove(&[], None, true), None);
    }

    // -- Builder defaults ------------------------------------------------

    #[test]
    fn sidenav_builder_defaults() {
        let nav = sidenav::<TestMsg>("nav1");
        assert_eq!(nav.id.as_str(), "nav1");
        assert!(nav.sections.is_empty());
        assert!(nav.selected_value.is_none());
    }

    // -- Sections and items ----------------------------------------------

    #[test]
    fn sidenav_add_sections_and_items() {
        let nav = sidenav::<TestMsg>("nav1")
            .section(
                sidenav_section("General")
                    .item(sidenav_item("Profile", "profile"))
                    .item(sidenav_item("Appearance", "appearance")),
            )
            .section(
                sidenav_section("Advanced")
                    .item(sidenav_item("Privacy", "privacy"))
                    .item(sidenav_item("Security", "security")),
            );
        assert_eq!(nav.sections.len(), 2);
        assert_eq!(nav.sections[0].items.len(), 2);
        assert_eq!(nav.sections[1].items.len(), 2);
        assert_eq!(nav.sections[0].items[0].label, "Profile");
        assert_eq!(nav.sections[0].items[0].value, "profile");
    }

    #[test]
    fn sidenav_selected_value() {
        let nav = sidenav::<TestMsg>("nav1")
            .section(sidenav_section("A").item(sidenav_item("P", "profile")))
            .selected("profile");
        assert_eq!(nav.selected_value.as_deref(), Some("profile"));
    }

    // -- Item builder methods --------------------------------------------

    #[test]
    fn sidenav_item_with_icon() {
        let item = sidenav_item("Profile", "profile").icon("user");
        assert_eq!(item.icon.as_deref(), Some("user"));
    }

    #[test]
    fn sidenav_item_disabled() {
        let item = sidenav_item("Profile", "profile").disabled();
        assert!(item.disabled);
    }

    // -- WidgetNode conversion -------------------------------------------

    #[test]
    fn sidenav_from_converts_to_column() {
        let node: WidgetNode<TestMsg> = sidenav::<TestMsg>("nav1")
            .section(sidenav_section("General").item(sidenav_item("Profile", "profile")))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        // Section header label + 1 card = 2 children
        assert_eq!(layout.children.len(), 2);
        // Second child should be a card container (column in layout)
        assert_eq!(layout.children[1].style.kind, LayoutKind::Column);
    }

    #[test]
    fn sidenav_unselected_item_is_plain_card() {
        let node: WidgetNode<TestMsg> = sidenav::<TestMsg>("nav1")
            .section(sidenav_section("General").item(sidenav_item("Profile", "profile")))
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        // child[0] = section label, child[1] = card for item
        let WidgetNode::Card(card) = &col.children[1] else {
            panic!("expected Card at child[1]");
        };
        assert_eq!(card.variant, CardVariant::Plain);
        assert_eq!(card.children.len(), 1);
    }

    #[test]
    fn sidenav_selected_item_is_interactive_card() {
        let node: WidgetNode<TestMsg> = sidenav::<TestMsg>("nav1")
            .section(sidenav_section("General").item(sidenav_item("Profile", "profile")))
            .selected("profile")
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column");
        };
        let WidgetNode::Card(card) = &col.children[1] else {
            panic!("expected Card at child[1]");
        };
        assert_eq!(card.variant, CardVariant::Interactive);
    }

    #[test]
    fn sidenav_multiple_sections_in_layout() {
        let node: WidgetNode<TestMsg> = sidenav::<TestMsg>("nav1")
            .section(
                sidenav_section("General")
                    .item(sidenav_item("Profile", "profile"))
                    .item(sidenav_item("Appearance", "appearance")),
            )
            .section(sidenav_section("Advanced").item(sidenav_item("Security", "security")))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Section 1: label + profile card + appearance card = 3 children
        // Section 2: label + security card = 2 children
        // Total: 5 children
        assert_eq!(layout.children.len(), 5);
    }

    #[test]
    fn sidenav_disabled_item_field() {
        let item = sidenav_item("Secret", "secret").disabled();
        assert!(item.disabled);
        assert_eq!(item.label, "Secret");
        assert_eq!(item.value, "secret");
    }

    // ---- ROADMAP 061: typed activation and keyboard navigation ----

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

    /// Replica of the pre-061 build, used to pin the legacy tree.
    fn legacy_tree(
        id: &str,
        sections: &[SidenavSection<Nav>],
        selected: Option<&str>,
    ) -> WidgetNode<Nav> {
        let mut col = column::<Nav>().key(id);
        for section in sections {
            col = col.child(crate::label(&section.label));
            for item in &section.items {
                let variant = if selected == Some(item.value.as_str()) {
                    CardVariant::Interactive
                } else {
                    CardVariant::Plain
                };
                let card_key = format!("sidenav_item_{}", item.value);
                let card = card::<Nav>()
                    .key(card_key.as_str())
                    .variant(variant)
                    .child(crate::label(&item.label))
                    .build();
                col = col.child(card);
            }
        }
        col.build()
    }

    /// General: Profile (icon), Appearance; Advanced: Privacy, Security (disabled).
    fn sections() -> Vec<SidenavSection<Nav>> {
        vec![
            sidenav_section("General")
                .item(sidenav_item("Profile", "profile").icon("@"))
                .item(sidenav_item("Appearance", "appearance")),
            sidenav_section("Advanced")
                .item(sidenav_item("Privacy", "privacy"))
                .item(sidenav_item("Security", "security").disabled()),
        ]
    }

    fn passive(
        key: &str,
        sections: Vec<SidenavSection<Nav>>,
        selected: Option<&str>,
    ) -> SidenavBuilder<Nav> {
        let mut b = sidenav::<Nav>(key);
        for s in sections {
            b = b.section(s);
        }
        match selected {
            Some(v) => b.selected(v),
            None => b,
        }
    }

    fn typed(key: &str, selected: Option<&str>) -> SidenavBuilder<Nav> {
        passive(key, sections(), selected).on_select(go)
    }

    #[test]
    fn sidenav_without_handler_matches_legacy_tree() {
        for selected in [None, Some("profile"), Some("privacy"), Some("security")] {
            let b = passive("side", sections(), selected);
            assert!(b.navigate(true).is_none());
            assert!(b.navigate(false).is_none());
            let expected = legacy_tree("side", &sections(), selected);
            let node: WidgetNode<Nav> = b.into();
            assert_eq!(node, expected, "{selected:?}");
            assert_eq!(format!("{node:?}"), format!("{expected:?}"));
            let mut keys = Vec::new();
            collect_keys(&node, &mut keys);
            assert_eq!(
                keys,
                [
                    "side",
                    "sidenav_item_profile",
                    "sidenav_item_appearance",
                    "sidenav_item_privacy",
                    "sidenav_item_security",
                ]
            );
        }
        let empty: WidgetNode<Nav> = sidenav::<Nav>("e").into();
        assert_eq!(empty, legacy_tree("e", &[], None));
    }

    #[test]
    fn sidenav_on_select_builds_instance_keyed_buttons_bound_to_values() {
        let node: WidgetNode<Nav> = typed("side", None).into();
        assert!(matches!(node, WidgetNode::Column(_)));
        assert_eq!(node.key().unwrap().as_str(), "side");
        let children = node.children();
        // General label + 2 items + Advanced label + 2 items
        assert_eq!(children.len(), 6);
        assert_eq!(children[0], crate::label::<Nav>("General"));
        assert_eq!(children[3], crate::label::<Nav>("Advanced"));
        let expected = [
            (1, "profile", true),
            (2, "appearance", true),
            (4, "privacy", true),
            (5, "security", false),
        ];
        for (index, value, enabled) in expected {
            let WidgetNode::Button(button) = &children[index] else {
                panic!("child {index} is not a Button");
            };
            assert_eq!(button.key.as_str(), format!("side:item:{value}"));
            assert!(button.full_width);
            assert_eq!(button.disabled, !enabled);
            let msg = go(value);
            assert_eq!(button.activate(), enabled.then_some(&msg));
        }
    }

    #[test]
    fn sidenav_disabled_item_never_activates() {
        for selected in [
            None,
            Some("profile"),
            Some("appearance"),
            Some("privacy"),
            Some("security"),
        ] {
            let b = typed("side", selected);
            for forward in [true, false] {
                assert_ne!(b.navigate(forward), Some(go("security")));
            }
            let node: WidgetNode<Nav> = b.into();
            let security = button_at(&node, "side:item:security");
            assert!(security.disabled);
            assert!(security.activate().is_none());
        }
        let only_off = passive(
            "side",
            vec![sidenav_section("A").item(sidenav_item("Off", "off").disabled())],
            Some("off"),
        )
        .on_select(go);
        assert_eq!(only_off.navigate(true), None);
        assert_eq!(only_off.navigate(false), None);
    }

    #[test]
    fn sidenav_selected_item_is_primary_and_marked() {
        use crate::ButtonVariant::{Ghost, Primary};
        let label_of = |node: &WidgetNode<Nav>, k: &str| button_at(node, k).label.clone();
        let variant_of = |node: &WidgetNode<Nav>, k: &str| button_at(node, k).variant;

        let node: WidgetNode<Nav> = typed("s", Some("privacy")).into();
        assert_eq!(label_of(&node, "s:item:profile"), "    @ Profile");
        assert_eq!(label_of(&node, "s:item:appearance"), "    Appearance");
        assert_eq!(label_of(&node, "s:item:privacy"), "▸ Privacy");
        assert_eq!(label_of(&node, "s:item:security"), "    Security");
        assert_eq!(variant_of(&node, "s:item:privacy"), Primary);
        for k in ["s:item:profile", "s:item:appearance", "s:item:security"] {
            assert_eq!(variant_of(&node, k), Ghost);
        }

        let node: WidgetNode<Nav> = typed("s", Some("profile")).into();
        assert_eq!(label_of(&node, "s:item:profile"), "▸ @ Profile");
        assert_eq!(variant_of(&node, "s:item:profile"), Primary);
        assert_eq!(variant_of(&node, "s:item:privacy"), Ghost);

        // No selection: every item is Ghost and unmarked.
        let node: WidgetNode<Nav> = typed("s", None).into();
        for v in ["profile", "appearance", "privacy", "security"] {
            let k = format!("s:item:{v}");
            assert_eq!(variant_of(&node, &k), Ghost);
            assert!(label_of(&node, &k).starts_with("    "));
        }

        // Theme tokens only: production code must not construct colors directly.
        let source = include_str!("sidenav.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        for forbidden in ["Color", "rgb(", "rgba(", "srgb", "0x"] {
            assert!(!production.contains(forbidden), "found {forbidden}");
        }
    }

    #[test]
    fn sidenav_navigate_crosses_sections_wraps_and_skips_disabled() {
        let nav = |c: Option<&str>, f: bool| typed("s", c).navigate(f);
        let sel = |v: &str| Some(go(v));
        // Forward: crosses from General into Advanced, skips Security, wraps.
        assert_eq!(nav(Some("profile"), true), sel("appearance"));
        assert_eq!(nav(Some("appearance"), true), sel("privacy"));
        assert_eq!(nav(Some("privacy"), true), sel("profile"));
        // Backward: wraps past disabled Security, crosses back into General.
        assert_eq!(nav(Some("profile"), false), sel("privacy"));
        assert_eq!(nav(Some("privacy"), false), sel("appearance"));
        assert_eq!(nav(Some("appearance"), false), sel("profile"));
        // From a disabled selected item, roving still moves to a neighbour.
        assert_eq!(nav(Some("security"), true), sel("profile"));
        assert_eq!(nav(Some("security"), false), sel("privacy"));
        // An unknown selected value behaves like no selection.
        assert_eq!(nav(Some("missing"), true), sel("profile"));
        assert_eq!(nav(Some("missing"), false), sel("privacy"));
        // Empty sections are skipped transparently.
        let gappy = passive(
            "s",
            vec![
                sidenav_section("A").item(sidenav_item("One", "one")),
                sidenav_section("Empty"),
                sidenav_section("B").item(sidenav_item("Two", "two")),
            ],
            Some("one"),
        )
        .on_select(go);
        assert_eq!(gappy.navigate(true), sel("two"));
        assert_eq!(gappy.navigate(false), sel("two"));
    }

    #[test]
    fn sidenav_navigate_without_selection_returns_first_or_last_enabled() {
        let b = typed("s", None);
        assert_eq!(b.navigate(true), Some(go("profile")));
        // The last item (Security) is disabled, so backward lands on Privacy.
        assert_eq!(b.navigate(false), Some(go("privacy")));
        let edges_off = passive(
            "s",
            vec![
                sidenav_section("A")
                    .item(sidenav_item("A", "a").disabled())
                    .item(sidenav_item("B", "b")),
                sidenav_section("B")
                    .item(sidenav_item("C", "c"))
                    .item(sidenav_item("D", "d").disabled()),
            ],
            None,
        )
        .on_select(go);
        assert_eq!(edges_off.navigate(true), Some(go("b")));
        assert_eq!(edges_off.navigate(false), Some(go("c")));
    }

    #[test]
    fn sidenav_navigate_none_without_handler_or_enabled_item() {
        for c in [None, Some("profile"), Some("privacy")] {
            let b = passive("s", sections(), c);
            assert_eq!(b.navigate(true), None);
            assert_eq!(b.navigate(false), None);
        }
        let empty = sidenav::<Nav>("s").on_select(go);
        assert_eq!(empty.navigate(true), None);
        assert_eq!(empty.navigate(false), None);
        let empty_sections =
            passive("s", vec![sidenav_section("A"), sidenav_section("B")], None).on_select(go);
        assert_eq!(empty_sections.navigate(true), None);
        assert_eq!(empty_sections.navigate(false), None);
        let all_off = passive(
            "s",
            vec![
                sidenav_section("A").item(sidenav_item("A", "a").disabled()),
                sidenav_section("B").item(sidenav_item("B", "b").disabled()),
            ],
            Some("b"),
        )
        .on_select(go);
        assert_eq!(all_off.navigate(true), None);
        assert_eq!(all_off.navigate(false), None);
        let single = passive(
            "s",
            vec![sidenav_section("A").item(sidenav_item("Only", "only"))],
            Some("only"),
        )
        .on_select(go);
        assert_eq!(single.navigate(true), Some(go("only")));
        assert_eq!(single.navigate(false), Some(go("only")));
    }

    #[test]
    fn sidenav_item_keys_distinct_per_instance() {
        use acme_core::RetainedTree;
        use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};
        let keys_of = |key: &str, selected: Option<&str>| {
            let node: WidgetNode<Nav> = typed(key, selected).into();
            let mut keys = Vec::new();
            collect_keys(&node, &mut keys);
            keys
        };
        let expected = [
            "a",
            "a:item:profile",
            "a:item:appearance",
            "a:item:privacy",
            "a:item:security",
        ];
        for selected in [None, Some("profile"), Some("privacy")] {
            assert_eq!(keys_of("a", selected), expected);
        }
        let a = keys_of("a", None);
        let b = keys_of("b", None);
        assert!(a.iter().all(|k| !b.contains(k)), "{a:?} vs {b:?}");

        // Two instances side by side reconcile without collisions, and retained
        // ids survive selection changes and sibling reordering.
        let values = ["profile", "appearance", "privacy", "security"];
        let mut tree = RetainedTree::new();
        let mut saved = None;
        let steps = [
            (None, false),
            (Some("appearance"), true),
            (Some("privacy"), false),
            (None, true),
        ];
        for (selected, reverse) in steps {
            let build = |k: &str| -> WidgetNode<Nav> { typed(k, selected).into() };
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
}
