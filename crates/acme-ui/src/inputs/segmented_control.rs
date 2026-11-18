//! SegmentedControl component.
//!
//! A horizontal group of mutually exclusive options. The selected option uses
//! the Primary button variant; other options use Ghost.
//!
//! The application owns selection. Build with its current index, handle the
//! typed message from `on_select`, and rebuild with the updated index. A host
//! can call `navigate` for Left/Right while the group owns keyboard focus.

use acme_core::WidgetKey;
use acme_widgets::*;
use std::collections::BTreeMap;

/// Builder for a SegmentedControl component.
pub struct SegmentedControlBuilder<M> {
    pub id: WidgetKey,
    pub items: Vec<String>,
    pub selected: usize,
    options: BTreeMap<usize, SegmentOptions>,
    on_select: Option<fn(usize) -> M>,
}

#[derive(Default)]
struct SegmentOptions {
    key: Option<WidgetKey>,
    disabled: bool,
}

/// Create a new SegmentedControl builder.
pub fn segmented_control<M: Clone + 'static>(
    id: impl Into<WidgetKey>,
) -> SegmentedControlBuilder<M> {
    SegmentedControlBuilder {
        id: id.into(),
        items: vec![],
        selected: 0,
        options: BTreeMap::new(),
        on_select: None,
    }
}

impl<M: Clone + 'static> SegmentedControlBuilder<M> {
    /// Add an item to the segmented control.
    pub fn item(mut self, label: impl Into<String>) -> Self {
        self.items.push(label.into());
        self
    }

    /// Add an item with a stable identity independent of label and order.
    ///
    /// The key must be unique in the control. The button's key becomes
    /// `"{id}:segment:{key}"`; plain items keep the original `"{id}_{index}"`.
    pub fn keyed_item(mut self, key: impl Into<WidgetKey>, label: impl Into<String>) -> Self {
        self.options.entry(self.items.len()).or_default().key = Some(key.into());
        self.items.push(label.into());
        self
    }

    /// Disable an item by index. An out-of-range index has no effect until
    /// the corresponding item exists. A disabled selection stays app-owned.
    pub fn disable_item(mut self, index: usize) -> Self {
        self.options.entry(index).or_default().disabled = true;
        self
    }

    /// Map an enabled button's index to the application's selection message.
    ///
    /// The index refers to the current declaration order, even for keyed items.
    pub fn on_select(mut self, handler: fn(usize) -> M) -> Self {
        self.on_select = Some(handler);
        self
    }

    /// Build a selection message for the next (forward) or previous enabled
    /// option, wrapping. An out-of-range selection starts at the first or
    /// last enabled option. Returns `None` without a handler or enabled item.
    pub fn navigate(&self, forward: bool) -> Option<M> {
        let select = self.on_select?;
        let len = self.items.len();
        if len == 0 {
            return None;
        }
        let current = if self.selected < len {
            self.selected
        } else if forward {
            len - 1
        } else {
            0
        };
        (1..=len)
            .map(|step| {
                if forward {
                    (current + step) % len
                } else {
                    (current + len - step) % len
                }
            })
            .find(|index| !self.options.get(index).is_some_and(|item| item.disabled))
            .map(select)
    }

    /// Set the index of the currently selected item (default 0).
    pub fn selected(mut self, index: usize) -> Self {
        self.selected = index;
        self
    }
}

impl<M: Clone + 'static> From<SegmentedControlBuilder<M>> for WidgetNode<M> {
    fn from(b: SegmentedControlBuilder<M>) -> Self {
        let id_str = b.id.as_str().to_string();
        let mut seg_row = row::<M>().key(b.id).gap(0.0);
        for (i, item) in b.items.iter().enumerate() {
            let options = b.options.get(&i);
            let btn_key = if let Some(key) = options.and_then(|item| item.key.as_ref()) {
                format!("{id_str}:segment:{}", key.as_str())
            } else {
                format!("{id_str}_{i}")
            };
            let disabled = options.is_some_and(|item| item.disabled);
            let button = button::<M>(btn_key, item.as_str())
                .variant(if i == b.selected {
                    ButtonVariant::Primary
                } else {
                    ButtonVariant::Ghost
                })
                .disabled(disabled);
            let button: WidgetNode<M> = match b.on_select {
                Some(select) if !disabled => button.on_click(select(i)),
                _ => button.into(),
            };
            seg_row = seg_row.child(button);
        }
        seg_row.build()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::NodeId;
    use acme_layout::{LayoutKind, Length};

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {
        Select(usize),
    }

    #[test]
    fn segmented_control_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = segmented_control("seg1")
            .item("Day")
            .item("Week")
            .item("Month")
            .selected(1)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Row);
        assert_eq!(layout.children.len(), 3);
        // Each child is a Button leaf with non-zero height
        for child in &layout.children {
            assert!(child.children.is_empty());
            assert_ne!(child.style.height, Length::px(0.0));
        }
    }

    #[test]
    fn segmented_control_builder_defaults() {
        let s = segmented_control::<TestMsg>("s");
        assert!(s.items.is_empty());
        assert_eq!(s.selected, 0);
    }

    #[test]
    fn segmented_control_selected_index() {
        let s: SegmentedControlBuilder<TestMsg> = segmented_control("seg")
            .item("A")
            .item("B")
            .item("C")
            .selected(2);
        assert_eq!(s.selected, 2);
        assert_eq!(s.items.len(), 3);
    }

    #[test]
    fn segmented_control_buttons_have_variants() {
        let node: WidgetNode<TestMsg> = segmented_control("seg")
            .item("One")
            .item("Two")
            .selected(0)
            .into();
        let WidgetNode::Row(container) = &node else {
            panic!("expected Row");
        };
        assert_eq!(container.children.len(), 2);
        // First button should be Primary (selected)
        let WidgetNode::Button(btn0) = &container.children[0] else {
            panic!("expected Button");
        };
        assert_eq!(btn0.variant, ButtonVariant::Primary);
        // Second button should be Ghost (unselected)
        let WidgetNode::Button(btn1) = &container.children[1] else {
            panic!("expected Button");
        };
        assert_eq!(btn1.variant, ButtonVariant::Ghost);
    }

    #[test]
    fn action_binding_preserves_legacy_tree_and_skips_disabled_buttons() {
        let passive: WidgetNode<TestMsg> =
            segmented_control("period").item("今日").item("本月").into();
        let expected = row()
            .key("period")
            .gap(0.0)
            .child(button("period_0", "今日").variant(ButtonVariant::Primary))
            .child(button("period_1", "本月").variant(ButtonVariant::Ghost))
            .build();
        assert_eq!(passive, expected);

        let active: WidgetNode<TestMsg> = segmented_control("period")
            .keyed_item("today", "今日")
            .keyed_item("month", "本月")
            .on_select(TestMsg::Select)
            .disable_item(1)
            .into();
        let views = acme_widgets::view_bridge::widget_to_views(std::slice::from_ref(&active));
        assert!(!views[0].children[0].disabled);
        assert!(views[0].children[1].disabled);

        for (index, child) in active.children().iter().enumerate() {
            let WidgetNode::Button(button) = child else {
                panic!("expected Button");
            };
            assert_eq!(button.label, ["今日", "本月"][index]);
            assert_eq!(button.disabled, index == 1);
            let message = TestMsg::Select(index);
            assert_eq!(button.activate(), (index == 0).then_some(&message));
        }
    }

    #[test]
    fn navigation_wraps_and_skips_disabled_items_in_both_directions() {
        let control = |selected| {
            segmented_control("period")
                .item("Today")
                .item("Week")
                .item("Month")
                .item("Year")
                .on_select(TestMsg::Select)
                .disable_item(1)
                .disable_item(2)
                .selected(selected)
        };
        for (selected, next, previous) in [(0, 3, 3), (3, 0, 0), (1, 3, 0), (99, 0, 3)] {
            assert_eq!(
                control(selected).navigate(true),
                Some(TestMsg::Select(next))
            );
            assert_eq!(
                control(selected).navigate(false),
                Some(TestMsg::Select(previous))
            );
        }
        assert_eq!(
            control(0).disable_item(3).navigate(false),
            Some(TestMsg::Select(0))
        );
        assert_eq!(
            control(1).disable_item(0).disable_item(3).navigate(true),
            None
        );
        assert_eq!(
            segmented_control("empty")
                .on_select(TestMsg::Select)
                .navigate(false),
            None
        );
        assert_eq!(
            segmented_control::<TestMsg>("unbound")
                .item("Today")
                .navigate(true),
            None
        );
    }

    #[test]
    fn stable_segment_keys_retain_identity_and_bind_current_order() {
        use acme_core::RetainedTree;
        use acme_widgets::{retained_binding::bind_retained, view_bridge::widget_to_views};

        let mut tree = RetainedTree::new();
        let mut previous = None;
        for (index, order) in [["today", "month"], ["month", "today"]]
            .into_iter()
            .enumerate()
        {
            let root = segmented_control("period")
                .keyed_item(order[0], "Same label")
                .keyed_item(order[1], "Same label")
                .on_select(TestMsg::Select)
                .selected(index)
                .into();
            let roots = [root];
            tree.reconcile_roots(&widget_to_views(&roots)).unwrap();
            let binding = bind_retained(&roots, &tree);
            let WidgetNode::Button(today) = &roots[0].children()[index] else {
                panic!("expected Button");
            };
            let id = binding.id_for(&roots[0].children()[index]).unwrap();
            assert_eq!(today.key.as_str(), "period:segment:today");
            assert_eq!(today.variant, ButtonVariant::Primary);
            assert_eq!(today.activate(), Some(&TestMsg::Select(index)));
            assert_eq!(binding.mismatches(), 0);
            if let Some(previous) = previous {
                assert_eq!(id, previous);
            }
            previous = Some(id);
        }
    }
}
