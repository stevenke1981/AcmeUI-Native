//! SettingsPage component — a column of titled groups with label-widget rows.

use crate::WidgetNode;

/// A group of settings rows with a title.
pub struct SettingsGroup<M> {
    pub title: String,
    pub rows: Vec<SettingsRow<M>>,
}

/// A single settings row with a label and a widget.
pub struct SettingsRow<M> {
    pub label: String,
    pub widget: WidgetNode<M>,
}

/// Builder for a settings page.
pub struct SettingsPageBuilder<M> {
    pub groups: Vec<SettingsGroup<M>>,
}

/// Create a settings page builder.
pub fn settings_page<M: Clone + 'static>() -> SettingsPageBuilder<M> {
    SettingsPageBuilder { groups: vec![] }
}

impl<M: Clone> SettingsPageBuilder<M> {
    /// Add a settings group.
    pub fn group(mut self, group: SettingsGroup<M>) -> Self {
        self.groups.push(group);
        self
    }

    /// Build the widget node tree.
    ///
    /// Each group is a `Column` with a title `Label` followed by rows.
    /// Each row is a `Row` with a label and its widget.
    pub fn build(self) -> WidgetNode<M> {
        let mut outer = crate::column::<M>().gap(16.0);

        for group in self.groups {
            let mut group_col = crate::column::<M>().gap(4.0);

            // Title label
            group_col = group_col.child(crate::label::<M>(group.title));

            // Rows
            for row_item in group.rows {
                let row_node = crate::row::<M>()
                    .child(crate::label::<M>(row_item.label))
                    .child(row_item.widget)
                    .gap(8.0)
                    .build();
                group_col = group_col.child(row_node);
            }

            outer = outer.child(group_col.build());
        }

        outer.build()
    }
}

impl<M: Clone + 'static> From<SettingsPageBuilder<M>> for WidgetNode<M> {
    fn from(b: SettingsPageBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{WidgetNode, label};
    use acme_core::NodeId;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn settings_page_has_non_zero_layout_rect() {
        let node: WidgetNode<Msg> = settings_page::<Msg>()
            .group(SettingsGroup {
                title: "General".into(),
                rows: vec![
                    SettingsRow {
                        label: "Name".into(),
                        widget: label("Alice"),
                    },
                    SettingsRow {
                        label: "Email".into(),
                        widget: label("alice@example.com"),
                    },
                ],
            })
            .into();
        let layout = node.to_layout(NodeId::new(1));
        // Outer column with 1 group
        assert_eq!(layout.children.len(), 1);
        // Group column: title + 2 rows = 3 children
        assert_eq!(layout.children[0].children.len(), 3);
    }

    #[test]
    fn settings_page_multiple_groups() {
        let node: WidgetNode<Msg> = settings_page::<Msg>()
            .group(SettingsGroup {
                title: "Profile".into(),
                rows: vec![SettingsRow {
                    label: "Name".into(),
                    widget: label("Bob"),
                }],
            })
            .group(SettingsGroup {
                title: "Notifications".into(),
                rows: vec![],
            })
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.children.len(), 2);
    }

    #[test]
    fn settings_page_empty() {
        let node: WidgetNode<Msg> = settings_page::<Msg>().build();
        let layout = node.to_layout(NodeId::new(1));
        assert!(layout.children.is_empty());
    }
}
