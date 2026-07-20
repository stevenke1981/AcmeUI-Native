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
pub struct SidenavBuilder<M> {
    pub id: WidgetKey,
    pub sections: Vec<SidenavSection<M>>,
    pub selected_value: Option<String>,
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
}

// ---------------------------------------------------------------------------
// Conversion into WidgetNode
// ---------------------------------------------------------------------------

impl<M: Clone + 'static> From<SidenavBuilder<M>> for WidgetNode<M> {
    fn from(b: SidenavBuilder<M>) -> Self {
        let mut col = column::<M>().key(b.id);

        for section in &b.sections {
            // Section header label
            col = col.child(crate::label(&section.label));

            // Section items as Cards
            for item in &section.items {
                let is_selected =
                    b.selected_value.as_deref() == Some(item.value.as_str());
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
            .section(
                sidenav_section("Advanced")
                    .item(sidenav_item("Security", "security")),
            )
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
}
