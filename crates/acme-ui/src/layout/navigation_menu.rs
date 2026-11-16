//! NavigationMenu component — a horizontal navigation bar with items that can
//! have dropdown sub-items (like a website nav bar with dropdowns).

use crate::WidgetNode;

/// A single navigation menu item, optionally with child items for a dropdown.
#[derive(Clone, Debug, PartialEq)]
pub struct NavMenuItem {
    pub label: String,
    pub value: String,
    pub children: Vec<NavMenuItem>,
    pub disabled: bool,
}

/// Create a navigation menu item.
pub fn nav_menu_item(label: impl Into<String>, value: impl Into<String>) -> NavMenuItem {
    NavMenuItem {
        label: label.into(),
        value: value.into(),
        children: vec![],
        disabled: false,
    }
}

impl NavMenuItem {
    /// Add a child item (for dropdown sub-menus).
    pub fn child(mut self, item: NavMenuItem) -> Self {
        self.children.push(item);
        self
    }

    /// Set the disabled state.
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
}

/// Builder for a horizontal navigation menu.
pub struct NavigationMenuBuilder<M> {
    id: crate::WidgetKey,
    items: Vec<NavMenuItem>,
    selected_value: String,
    active_child: Option<String>,
    on_select: Option<fn(&str) -> M>,
    _phantom: std::marker::PhantomData<M>,
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

/// Create a navigation menu builder.
pub fn navigation_menu<M: Clone + 'static>(
    id: impl Into<crate::WidgetKey>,
) -> NavigationMenuBuilder<M> {
    NavigationMenuBuilder {
        id: id.into(),
        items: vec![],
        selected_value: String::new(),
        active_child: None,
        on_select: None,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone> NavigationMenuBuilder<M> {
    /// Add a navigation menu item.
    pub fn item(mut self, item: NavMenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Set the selected item's value. The item with this value will be
    /// visually highlighted; if it has children, a dropdown card is shown.
    pub fn selected(mut self, value: impl Into<String>) -> Self {
        self.selected_value = value.into();
        self
    }

    /// Mark a child of the selected item as the active (focused) child.
    ///
    /// Only used together with [`on_select`](Self::on_select): that child is
    /// rendered with the primary variant and a `▸ ` marker. Ignored without
    /// a handler, so the legacy tree is unchanged.
    pub fn active_child(mut self, value: impl Into<String>) -> Self {
        self.active_child = Some(value.into());
        self
    }

    /// Build each enabled item's and child's typed selection message from its
    /// value.
    ///
    /// With a handler, the Row is keyed `"{id}"`, each item Column
    /// `"{id}:group:{value}"`, each item Button `"{id}:item:{value}"`, the open
    /// dropdown Card `"{id}:menu:{value}"` and each child Button
    /// `"{id}:sub:{parent}:{child}"`. The selected item uses the primary
    /// variant and the others ghost; items with children carry a ` ▾` marker.
    /// Disabled items, disabled children and children of a disabled item never
    /// emit and get no click message. Values must be globally unique across
    /// all items and children (not just within a level), because the handler
    /// receives only the value, and must not contain `:` because they are part
    /// of those keys (not enforced). Without a handler the legacy
    /// `"{id}_{value}"` / `"{id}_{value}_{child}"` tree is built unchanged.
    pub fn on_select(mut self, f: fn(&str) -> M) -> Self {
        self.on_select = Some(f);
        self
    }

    /// Message selecting the next (`forward`) or previous enabled top-level
    /// item, wrapping and skipping disabled items.
    ///
    /// With no or an unknown selection, returns the first (forward) or last
    /// (backward) enabled item. Returns `None` without an `on_select` handler
    /// or when no item is enabled.
    pub fn navigate(&self, forward: bool) -> Option<M> {
        let handler = self.on_select?;
        let disabled: Vec<bool> = self.items.iter().map(|i| i.disabled).collect();
        let current = self
            .items
            .iter()
            .position(|i| i.value == self.selected_value);
        let next = rove(&disabled, current, forward)?;
        Some(handler(&self.items[next].value))
    }

    /// Value of the next (`forward`) or previous enabled child of the selected
    /// item, wrapping and skipping disabled children (one level only).
    ///
    /// With no or an unknown `current`, returns the first (forward) or last
    /// (backward) enabled child. Returns `None` when the selected item is
    /// missing, disabled or has no enabled child.
    pub fn navigate_child(&self, current: Option<&str>, forward: bool) -> Option<&str> {
        let parent = self
            .items
            .iter()
            .find(|i| i.value == self.selected_value)
            .filter(|i| !i.disabled)?;
        let disabled: Vec<bool> = parent.children.iter().map(|c| c.disabled).collect();
        let position = current.and_then(|v| parent.children.iter().position(|c| c.value == v));
        let next = rove(&disabled, position, forward)?;
        Some(parent.children[next].value.as_str())
    }

    /// Typed build used when an `on_select` handler is set.
    fn build_typed(self, select: fn(&str) -> M) -> WidgetNode<M> {
        let id = self.id.as_str().to_owned();
        let variant = |active: bool| {
            if active {
                crate::ButtonVariant::Primary
            } else {
                crate::ButtonVariant::Ghost
            }
        };
        let mut row = crate::row::<M>().key(self.id.clone()).gap(0.0);
        for item in &self.items {
            let is_selected = item.value == self.selected_value;
            let label = if item.children.is_empty() {
                item.label.clone()
            } else {
                format!("{} ▾", item.label)
            };
            let btn = crate::button::<M>(format!("{id}:item:{}", item.value), label)
                .variant(variant(is_selected))
                .disabled(item.disabled);
            let btn: WidgetNode<M> = if item.disabled {
                btn.into()
            } else {
                btn.on_click(select(&item.value))
            };
            let mut col = crate::column::<M>()
                .key(format!("{id}:group:{}", item.value))
                .child(btn)
                .gap(0.0);

            if is_selected && !item.children.is_empty() {
                let mut dropdown = crate::card::<M>()
                    .key(format!("{id}:menu:{}", item.value))
                    .variant(crate::CardVariant::Outlined)
                    .padding(4.0)
                    .gap(2.0);
                for child in &item.children {
                    let is_active = self.active_child.as_deref() == Some(child.value.as_str());
                    let label = if is_active {
                        format!("▸ {}", child.label)
                    } else {
                        child.label.clone()
                    };
                    let off = item.disabled || child.disabled;
                    let child_btn = crate::button::<M>(
                        format!("{id}:sub:{}:{}", item.value, child.value),
                        label,
                    )
                    .variant(variant(is_active))
                    .disabled(off);
                    let child_btn: WidgetNode<M> = if off {
                        child_btn.into()
                    } else {
                        child_btn.on_click(select(&child.value))
                    };
                    dropdown = dropdown.child(child_btn);
                }
                col = col.child(dropdown.build());
            }

            row = row.child(col.build());
        }
        row.build()
    }

    /// Build the widget node tree.
    ///
    /// Produces a horizontal [`Row`] where each menu item is a [`Column`]
    /// containing a [`Button`]. If the item is selected and has children, a
    /// [`Card`] with child buttons is appended inside that column. See
    /// [`on_select`](Self::on_select) for the keyed, interactive variant.
    pub fn build(self) -> WidgetNode<M> {
        if let Some(select) = self.on_select {
            return self.build_typed(select);
        }
        let mut row = crate::row::<M>().gap(0.0);

        for item in &self.items {
            let is_selected = item.value == self.selected_value;
            let item_key = format!("{}_{}", self.id.as_str(), item.value);
            let btn = crate::button::<M>(item_key.as_str(), &item.label).disabled(item.disabled);
            let mut col = crate::column::<M>().child(btn).gap(0.0);

            if is_selected && !item.children.is_empty() {
                let mut dropdown = crate::card::<M>()
                    .variant(crate::CardVariant::Outlined)
                    .padding(4.0)
                    .gap(2.0);
                for child in &item.children {
                    let child_key = format!("{}_{}_{}", self.id.as_str(), item.value, child.value);
                    let child_btn = crate::button::<M>(child_key.as_str(), &child.label)
                        .disabled(child.disabled);
                    dropdown = dropdown.child(child_btn);
                }
                col = col.child(dropdown.build());
            }

            row = row.child(col.build());
        }

        row.build()
    }
}

impl<M: Clone + 'static> From<NavigationMenuBuilder<M>> for WidgetNode<M> {
    fn from(b: NavigationMenuBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WidgetNode;
    use acme_core::NodeId;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn navigation_menu_builds_row() {
        let node: WidgetNode<Msg> = navigation_menu::<Msg>("nav")
            .item(nav_menu_item("Home", "home"))
            .item(nav_menu_item("About", "about"))
            .build();
        assert!(matches!(node, WidgetNode::Row(_)));
    }

    #[test]
    fn navigation_menu_has_correct_item_count() {
        let node: WidgetNode<Msg> = navigation_menu::<Msg>("nav")
            .item(nav_menu_item("Home", "home"))
            .item(nav_menu_item("Products", "products"))
            .item(nav_menu_item("About", "about"))
            .selected("home")
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Row contains 3 columns
        assert_eq!(layout.children.len(), 3);
    }

    #[test]
    fn navigation_menu_selected_shows_dropdown() {
        let node: WidgetNode<Msg> = navigation_menu::<Msg>("nav")
            .item(nav_menu_item("Home", "home"))
            .item(
                nav_menu_item("Products", "products")
                    .child(nav_menu_item("Sub A", "sub-a"))
                    .child(nav_menu_item("Sub B", "sub-b")),
            )
            .item(nav_menu_item("About", "about"))
            .selected("products")
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Row contains 3 columns
        assert_eq!(layout.children.len(), 3);
        // First item (not selected, no children): 1 child (button only)
        assert_eq!(layout.children[0].children.len(), 1);
        // Second item (selected, has children): button + card dropdown = 2
        assert_eq!(layout.children[1].children.len(), 2);
        // Third item (not selected, no children): 1 child (button only)
        assert_eq!(layout.children[2].children.len(), 1);
    }

    #[test]
    fn navigation_menu_non_selected_no_dropdown() {
        let node: WidgetNode<Msg> = navigation_menu::<Msg>("nav")
            .item(nav_menu_item("Products", "products"))
            .item(nav_menu_item("About", "about"))
            .selected("about")
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // First item (not selected, no children): 1 child
        assert_eq!(layout.children[0].children.len(), 1);
        // Second item (selected, no children): 1 child (button only)
        assert_eq!(layout.children[1].children.len(), 1);
    }

    #[test]
    fn navigation_menu_no_children_no_card() {
        let node: WidgetNode<Msg> = navigation_menu::<Msg>("nav")
            .item(nav_menu_item("Alone", "alone"))
            .selected("alone")
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Selected but no children -> still only the button
        assert_eq!(layout.children[0].children.len(), 1);
    }

    #[test]
    fn navigation_menu_empty() {
        let node: WidgetNode<Msg> = navigation_menu::<Msg>("nav").build();
        let layout = node.to_layout(NodeId::new(1));
        assert!(layout.children.is_empty());
    }

    #[test]
    fn navigation_menu_from_conversion() {
        let menu = navigation_menu::<Msg>("nav").item(nav_menu_item("Home", "home"));
        let node: WidgetNode<Msg> = menu.into();
        assert!(matches!(node, WidgetNode::Row(_)));
    }

    // ---- ROADMAP 056: typed activation and roving keyboard navigation ----

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

    fn collect_buttons<'a>(node: &'a WidgetNode<Nav>, out: &mut Vec<&'a crate::Button<Nav>>) {
        if let WidgetNode::Button(b) = node {
            out.push(b);
        }
        for child in node.children() {
            collect_buttons(child, out);
        }
    }

    /// Home, Products (Sub A, disabled Sub B, Sub C), disabled Legal (Terms), About.
    fn base(key: &str, selected: Option<&str>) -> NavigationMenuBuilder<Nav> {
        let mut b = navigation_menu::<Nav>(key)
            .item(nav_menu_item("Home", "home"))
            .item(
                nav_menu_item("Products", "products")
                    .child(nav_menu_item("Sub A", "sub-a"))
                    .child(nav_menu_item("Sub B", "sub-b").disabled(true))
                    .child(nav_menu_item("Sub C", "sub-c")),
            )
            .item(
                nav_menu_item("Legal", "legal")
                    .child(nav_menu_item("Terms", "terms"))
                    .disabled(true),
            )
            .item(nav_menu_item("About", "about"));
        if let Some(s) = selected {
            b = b.selected(s);
        }
        b
    }

    fn menu(key: &str, selected: Option<&str>) -> NavigationMenuBuilder<Nav> {
        base(key, selected).on_select(go)
    }

    /// Replica of the pre-056 passive build, used to pin the legacy tree.
    fn legacy_tree(id: &str, items: &[NavMenuItem], selected: &str) -> WidgetNode<Nav> {
        let mut row = crate::row::<Nav>().gap(0.0);
        for item in items {
            let item_key = format!("{}_{}", id, item.value);
            let btn = crate::button::<Nav>(item_key.as_str(), &item.label).disabled(item.disabled);
            let mut col = crate::column::<Nav>().child(btn).gap(0.0);
            if item.value == selected && !item.children.is_empty() {
                let mut dropdown = crate::card::<Nav>()
                    .variant(crate::CardVariant::Outlined)
                    .padding(4.0)
                    .gap(2.0);
                for child in &item.children {
                    let child_key = format!("{}_{}_{}", id, item.value, child.value);
                    dropdown = dropdown.child(
                        crate::button::<Nav>(child_key.as_str(), &child.label)
                            .disabled(child.disabled),
                    );
                }
                col = col.child(dropdown.build());
            }
            row = row.child(col.build());
        }
        row.build()
    }

    #[test]
    fn navigation_menu_items_bind_typed_select_messages_to_instance_keyed_buttons() {
        let node: WidgetNode<Nav> = menu("nav", Some("products")).active_child("sub-c").build();
        assert!(matches!(node, WidgetNode::Row(_)));
        assert_eq!(node.key().unwrap().as_str(), "nav");
        assert_eq!(node.children().len(), 4);
        let items = [
            ("home", "Home", true),
            ("products", "Products ▾", true),
            ("legal", "Legal ▾", false),
            ("about", "About", true),
        ];
        for (index, (value, label, enabled)) in items.into_iter().enumerate() {
            let group = &node.children()[index];
            assert!(matches!(group, WidgetNode::Column(_)));
            assert_eq!(group.key().unwrap().as_str(), format!("nav:group:{value}"));
            let button = button_at(group, &format!("nav:item:{value}"));
            assert_eq!(button.label, label);
            let expected = go(value);
            assert_eq!(button.activate(), enabled.then_some(&expected));
        }
        let products = &node.children()[1];
        assert_eq!(products.children().len(), 2);
        let card = &products.children()[1];
        assert!(matches!(card, WidgetNode::Card(_)));
        assert_eq!(card.key().unwrap().as_str(), "nav:menu:products");
        let children = [
            ("sub-a", "Sub A", true),
            ("sub-b", "Sub B", false),
            ("sub-c", "▸ Sub C", true),
        ];
        for (value, label, enabled) in children {
            let button = button_at(card, &format!("nav:sub:products:{value}"));
            assert_eq!(button.label, label);
            let expected = go(value);
            assert_eq!(button.activate(), enabled.then_some(&expected));
        }
        for closed in [0, 2, 3] {
            assert_eq!(node.children()[closed].children().len(), 1);
        }
    }

    #[test]
    fn navigation_menu_without_handler_keeps_legacy_tree_and_keys() {
        let passive = base("nav", Some("products")).active_child("sub-c");
        let expected = legacy_tree("nav", &passive.items, "products");
        assert!(passive.navigate(true).is_none());
        assert!(passive.navigate(false).is_none());
        let node = passive.build();
        assert_eq!(node, expected);
        let mut keys = Vec::new();
        collect_keys(&node, &mut keys);
        assert_eq!(
            keys,
            [
                "nav_home",
                "nav_products",
                "nav_products_sub-a",
                "nav_products_sub-b",
                "nav_products_sub-c",
                "nav_legal",
                "nav_about",
            ]
        );
        let mut buttons = Vec::new();
        collect_buttons(&node, &mut buttons);
        assert!(buttons.iter().all(|b| b.activate().is_none()));
        let labels: Vec<&str> = buttons.iter().map(|b| b.label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "Home", "Products", "Sub A", "Sub B", "Sub C", "Legal", "About"
            ]
        );
        let closed = base("nav", None);
        let expected = legacy_tree("nav", &closed.items, "");
        assert_eq!(closed.build(), expected);
    }

    #[test]
    fn navigation_menu_disabled_items_and_children_never_emit() {
        for selected in ["products", "legal"] {
            let node = menu("nav", Some(selected)).active_child("sub-b").build();
            let mut buttons = Vec::new();
            collect_buttons(&node, &mut buttons);
            assert!(!buttons.is_empty());
            for button in &buttons {
                if button.disabled {
                    assert!(button.activate().is_none(), "{} emitted", button.label);
                } else {
                    assert!(button.activate().is_some(), "{} silent", button.label);
                }
            }
            let legal = button_at(&node, "nav:item:legal");
            assert!(legal.disabled && legal.activate().is_none());
            if selected == "products" {
                let sub_b = button_at(&node, "nav:sub:products:sub-b");
                assert!(sub_b.disabled && sub_b.activate().is_none());
            } else {
                // Children of a disabled parent inherit the disabled state.
                let terms = button_at(&node, "nav:sub:legal:terms");
                assert!(terms.disabled && terms.activate().is_none());
            }
        }
        for selected in [
            None,
            Some("home"),
            Some("products"),
            Some("legal"),
            Some("about"),
        ] {
            for forward in [true, false] {
                assert_ne!(menu("nav", selected).navigate(forward), Some(go("legal")));
            }
        }
    }

    #[test]
    fn navigation_menu_navigate_wraps_skips_disabled_and_handles_empty_unknown() {
        let nav = |s: Option<&str>, f: bool| menu("n", s).navigate(f);
        let sel = |v: &str| Some(go(v));
        assert_eq!(nav(Some("home"), true), sel("products"));
        assert_eq!(nav(Some("products"), true), sel("about"));
        assert_eq!(nav(Some("about"), true), sel("home"));
        assert_eq!(nav(Some("home"), false), sel("about"));
        assert_eq!(nav(Some("about"), false), sel("products"));
        assert_eq!(nav(Some("products"), false), sel("home"));
        assert_eq!(nav(None, true), sel("home"));
        assert_eq!(nav(None, false), sel("about"));
        assert_eq!(nav(Some("zz"), true), sel("home"));
        assert_eq!(nav(Some("zz"), false), sel("about"));
        assert_eq!(nav(Some("legal"), true), sel("about"));
        assert_eq!(nav(Some("legal"), false), sel("products"));
        let single = navigation_menu::<Nav>("n")
            .item(nav_menu_item("Only", "only"))
            .item(nav_menu_item("Off", "off").disabled(true))
            .selected("only")
            .on_select(go);
        assert_eq!(single.navigate(true), sel("only"));
        assert_eq!(single.navigate(false), sel("only"));
        let all_disabled = navigation_menu::<Nav>("n")
            .item(nav_menu_item("A", "a").disabled(true))
            .item(nav_menu_item("B", "b").disabled(true))
            .on_select(go);
        assert_eq!(all_disabled.navigate(true), None);
        assert_eq!(all_disabled.navigate(false), None);
        let empty = navigation_menu::<Nav>("n").selected("a").on_select(go);
        assert_eq!(empty.navigate(true), None);
        assert_eq!(base("n", Some("home")).navigate(true), None);
    }

    #[test]
    fn navigation_menu_navigate_child_wraps_skips_disabled_and_requires_open_parent() {
        let open = menu("n", Some("products"));
        assert_eq!(open.navigate_child(None, true), Some("sub-a"));
        assert_eq!(open.navigate_child(None, false), Some("sub-c"));
        assert_eq!(open.navigate_child(Some("sub-a"), true), Some("sub-c"));
        assert_eq!(open.navigate_child(Some("sub-c"), true), Some("sub-a"));
        assert_eq!(open.navigate_child(Some("sub-c"), false), Some("sub-a"));
        assert_eq!(open.navigate_child(Some("sub-a"), false), Some("sub-c"));
        assert_eq!(open.navigate_child(Some("sub-b"), true), Some("sub-c"));
        assert_eq!(open.navigate_child(Some("sub-b"), false), Some("sub-a"));
        assert_eq!(open.navigate_child(Some("zz"), true), Some("sub-a"));
        assert_eq!(open.navigate_child(Some("zz"), false), Some("sub-c"));
        assert_eq!(
            base("n", Some("products")).navigate_child(None, true),
            Some("sub-a")
        );
        for selected in [None, Some("home"), Some("about"), Some("legal"), Some("zz")] {
            for forward in [true, false] {
                assert_eq!(menu("n", selected).navigate_child(None, forward), None);
                assert_eq!(
                    menu("n", selected).navigate_child(Some("sub-a"), forward),
                    None
                );
            }
        }
        let dead = navigation_menu::<Nav>("n")
            .item(
                nav_menu_item("P", "p")
                    .child(nav_menu_item("X", "x").disabled(true))
                    .child(nav_menu_item("Y", "y").disabled(true)),
            )
            .selected("p")
            .on_select(go);
        assert_eq!(dead.navigate_child(None, true), None);
        assert_eq!(dead.navigate_child(Some("x"), false), None);
    }

    #[test]
    fn navigation_menu_retained_ids_survive_selection_change_and_dropdown_toggle() {
        use acme_core::RetainedTree;
        use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};
        let values = ["home", "products", "legal", "about"];
        let mut tree = RetainedTree::new();
        let mut saved_items = None;
        let mut saved_children = None;
        let mut open_streak = 0;
        let mut longest_streak = 0;
        let steps: [(Option<&str>, Option<&str>, bool); 7] = [
            (None, None, false),
            (Some("products"), Some("sub-a"), true),
            (Some("products"), Some("sub-c"), false),
            (Some("products"), None, true),
            (Some("about"), None, false),
            (Some("products"), Some("sub-c"), true),
            (Some("home"), Some("sub-c"), false),
        ];
        for (selected, active, reverse) in steps {
            let build = |key: &str| {
                let b = menu(key, selected);
                match active {
                    Some(a) => b.active_child(a).build(),
                    None => b.build(),
                }
            };
            let mut roots = vec![build("x"), build("y")];
            if reverse {
                roots.reverse();
            }
            tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
            let binding = bind_retained(&roots, &tree);
            assert_eq!(binding.mismatches(), 0);
            let root = |k: &str| {
                roots
                    .iter()
                    .find(|n| n.key().unwrap().as_str() == k)
                    .unwrap()
            };
            let ids = ["x", "y"].map(|k| {
                let r = root(k);
                let row = binding.id_for(r).unwrap();
                let items = values.map(|v| {
                    let group = find_by_key(r, &format!("{k}:group:{v}")).unwrap();
                    let item = find_by_key(r, &format!("{k}:item:{v}")).unwrap();
                    (
                        binding.id_for(group).unwrap(),
                        binding.id_for(item).unwrap(),
                    )
                });
                (row, items)
            });
            if let Some(saved) = saved_items {
                assert_eq!(ids, saved);
            }
            assert_ne!(ids[0], ids[1]);
            saved_items = Some(ids);
            let is_open = selected == Some("products");
            for k in ["x", "y"] {
                let card = find_by_key(root(k), &format!("{k}:menu:products"));
                assert_eq!(card.is_some(), is_open);
            }
            if is_open {
                let children = ["x", "y"].map(|k| {
                    let r = root(k);
                    let card = find_by_key(r, &format!("{k}:menu:products")).unwrap();
                    let subs = ["sub-a", "sub-b", "sub-c"].map(|c| {
                        let node = find_by_key(r, &format!("{k}:sub:products:{c}")).unwrap();
                        binding.id_for(node).unwrap()
                    });
                    (binding.id_for(card).unwrap(), subs)
                });
                // Child ids are stable while the dropdown stays open; closing
                // it unmounts the Card, so a reopen mounts fresh child ids.
                if let Some(saved) = saved_children {
                    assert_eq!(children, saved);
                }
                assert_ne!(children[0], children[1]);
                saved_children = Some(children);
                open_streak += 1;
                longest_streak = longest_streak.max(open_streak);
            } else {
                saved_children = None;
                open_streak = 0;
            }
        }
        assert_eq!(longest_streak, 3);
    }

    #[test]
    fn navigation_menu_keys_unique_and_variants_use_theme_tokens() {
        let node = menu("dup", Some("products")).active_child("sub-c").build();
        let mut keys = Vec::new();
        collect_keys(&node, &mut keys);
        // Row + 4 groups + 4 items + 1 card + 3 children.
        assert_eq!(keys.len(), 13);
        let mut unique = keys.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), keys.len(), "duplicate keys: {keys:?}");
        let variant = |k: &str| button_at(&node, k).variant;
        assert_eq!(variant("dup:item:products"), crate::ButtonVariant::Primary);
        for v in ["home", "legal", "about"] {
            assert_eq!(
                variant(&format!("dup:item:{v}")),
                crate::ButtonVariant::Ghost
            );
        }
        assert_eq!(
            variant("dup:sub:products:sub-c"),
            crate::ButtonVariant::Primary
        );
        for c in ["sub-a", "sub-b"] {
            assert_eq!(
                variant(&format!("dup:sub:products:{c}")),
                crate::ButtonVariant::Ghost
            );
        }
        let idle = menu("idle", None).build();
        let mut buttons = Vec::new();
        collect_buttons(&idle, &mut buttons);
        assert!(
            buttons
                .iter()
                .all(|b| b.variant == crate::ButtonVariant::Ghost)
        );
        // Theme tokens only: production code must not construct colors directly.
        let source = include_str!("navigation_menu.rs");
        let production = source.split("#[cfg(test)]").next().unwrap();
        for forbidden in ["Color", "rgb(", "rgba(", "srgb", "0x"] {
            assert!(!production.contains(forbidden), "found {forbidden}");
        }
    }
}
