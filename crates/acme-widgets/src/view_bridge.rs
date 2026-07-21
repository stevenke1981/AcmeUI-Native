//! Bridge from [`WidgetNode`] to [`ViewNode`] for the runtime shadow tree.
//!
//! This module converts the declarative widget tree into a flat `Vec<ViewNode>`
//! forest that can be fed to [`RetainedTree::reconcile_roots`] for diagnostics.
//! It is purely diagnostic (Phase 1) — no functional changes to layout,
//! paint, hit-testing, or accessibility.

use acme_core::ViewNode;
use acme_core::WidgetKey;

use crate::WidgetNode;

/// Auto-key prefix used for widgets that do not have an explicit user-supplied
/// key (e.g. `Label`, `Separator`).
const AUTO_KEY_PREFIX: &str = "__auto";

/// Convert a [`WidgetNode`] forest into a `Vec<ViewNode>` forest.
///
/// Each widget variant is mapped to a `ViewNode` with:
/// - **key**: the widget's explicit key, or an auto-generated key based on the
///   parent path and child index.
/// - **kind**: a short descriptive string matching the variant name.
/// - **children**: recursively converted child/children.
/// - **focusable**: `true` for interactive widgets (Button, TextInput, etc.).
/// - **disabled**: `true` when the widget exposes a disabled flag.
///
/// # Auto-keying
///
/// Widgets without an explicit `WidgetKey` (`Label`, `Separator`) receive a
/// synthetic key `__auto:{parent_key}:{index}:{kind}`. This is **not** stable
/// across structural changes (insert/remove/reorder), but is sufficient for
/// Phase 1 diagnostics.
pub fn widget_to_views<M>(roots: &[WidgetNode<M>]) -> Vec<ViewNode> {
    let mut views = Vec::with_capacity(roots.len());
    for (i, w) in roots.iter().enumerate() {
        views.push(widget_to_view(w, "", i));
    }
    views
}

fn widget_to_view<M>(widget: &WidgetNode<M>, parent_path: &str, index: usize) -> ViewNode {
    match widget {
        // ── Containers ──────────────────────────────────────────────────────
        WidgetNode::Row(v) => {
            container_view(v.key.as_ref(), "row", &v.children, parent_path, index)
        }
        WidgetNode::Column(v) => {
            container_view(v.key.as_ref(), "column", &v.children, parent_path, index)
        }
        WidgetNode::Stack(v) => {
            container_view(v.key.as_ref(), "stack", &v.children, parent_path, index)
        }
        WidgetNode::Card(v) => {
            container_view(v.key.as_ref(), "card", &v.children, parent_path, index)
        }
        WidgetNode::ScrollView(v) => {
            container_view(Some(&v.key), "scroll_view", &v.children, parent_path, index)
        }

        // ── Labels (no explicit key) ────────────────────────────────────────
        WidgetNode::Label(_l) => ViewNode {
            key: auto_key(parent_path, index, "label"),
            kind: "label".into(),
            children: vec![],
            focusable: false,
            disabled: false,
            // Store the label text in a secondary field is not possible with
            // ViewNode, so we log via the variant name. The text itself is
            // available from the original WidgetNode.
        },

        // ── Buttons ─────────────────────────────────────────────────────────
        WidgetNode::Button(v) => ViewNode {
            key: v.key.clone(),
            kind: "button".into(),
            children: vec![],
            focusable: true,
            disabled: v.disabled,
        },

        // ── Separator (no explicit key) ─────────────────────────────────────
        WidgetNode::Separator(_) => ViewNode {
            key: auto_key(parent_path, index, "separator"),
            kind: "separator".into(),
            children: vec![],
            focusable: false,
            disabled: false,
        },

        // ── TextInput ───────────────────────────────────────────────────────
        WidgetNode::TextInput(v) => ViewNode {
            key: v.key.clone(),
            kind: "text_input".into(),
            children: vec![],
            focusable: true,
            disabled: v.disabled,
        },

        // ── Overlays ────────────────────────────────────────────────────────
        WidgetNode::Tooltip(v) => {
            // Tooltip has a single child; treat transparently but keep in tree.
            let child = widget_to_view(&v.child, v.key.as_str(), 0);
            ViewNode {
                key: v.key.clone(),
                kind: "tooltip".into(),
                children: vec![child],
                focusable: false,
                disabled: false,
            }
        }
        WidgetNode::Popover(v) => {
            // Popover has [anchor, content] children.
            let children: Vec<ViewNode> = v
                .children
                .iter()
                .enumerate()
                .map(|(i, c)| widget_to_view(c, v.key.as_str(), i))
                .collect();
            ViewNode {
                key: v.key.clone(),
                kind: "popover".into(),
                children,
                focusable: false,
                disabled: false,
            }
        }
        WidgetNode::Menu(v) => ViewNode {
            key: v.key.clone(),
            kind: "menu".into(),
            // Menu items are MenuItem structs, not WidgetNodes — no children
            // in the view tree for Phase 1.
            children: vec![],
            focusable: false,
            disabled: false,
        },
        WidgetNode::Dialog(v) => {
            let content = widget_to_view(&v.content, v.key.as_str(), 0);
            ViewNode {
                key: v.key.clone(),
                kind: "dialog".into(),
                children: vec![content],
                focusable: false,
                disabled: false,
            }
        }

        // ── Data widgets ────────────────────────────────────────────────────
        WidgetNode::VirtualList(v) => {
            // Children are the full list (visible-range selection is not
            // replicated in the view tree for Phase 1).
            container_view(
                Some(&v.key),
                "virtual_list",
                &v.children,
                parent_path,
                index,
            )
        }
        WidgetNode::Tree(v) => {
            // Tree uses visible_nodes() for layout; for the shadow view we
            // just use the top-level TreeNode children's content widgets.
            let children: Vec<ViewNode> = v
                .children
                .iter()
                .enumerate()
                .map(|(i, tn)| {
                    // Each visible tree node's content widget.
                    let mut vn = widget_to_view(&tn.content, v.key.as_str(), i);
                    vn.kind = "tree_node".into();
                    vn
                })
                .collect();
            ViewNode {
                key: v.key.clone(),
                kind: "tree".into(),
                children,
                focusable: true,
                disabled: false,
            }
        }
        WidgetNode::Table(v) => {
            // all_cells is a flat list of all cell widgets.
            container_view(Some(&v.key), "table", &v.all_cells, parent_path, index)
        }
        WidgetNode::DataGrid(v) => {
            container_view(Some(&v.key), "data_grid", &v.all_cells, parent_path, index)
        }

        // ── Navigation widgets ──────────────────────────────────────────────
        WidgetNode::NavRail(v) => {
            container_view(Some(&v.key), "nav_rail", &v.children, parent_path, index)
        }
        WidgetNode::Sidebar(v) => {
            container_view(Some(&v.key), "sidebar", &v.children, parent_path, index)
        }
        WidgetNode::TabBar(v) => {
            container_view(Some(&v.key), "tab_bar", &v.children, parent_path, index)
        }
        WidgetNode::Breadcrumb(v) => {
            container_view(Some(&v.key), "breadcrumb", &v.children, parent_path, index)
        }
    }
}

/// Helper: build a `ViewNode` for a container-style widget.
fn container_view<M>(
    key: Option<&WidgetKey>,
    kind: &str,
    children: &[WidgetNode<M>],
    parent_path: &str,
    index: usize,
) -> ViewNode {
    let actual_key = match key {
        Some(k) => k.clone(),
        None => auto_key(parent_path, index, kind),
    };
    let child_views: Vec<ViewNode> = children
        .iter()
        .enumerate()
        .map(|(i, c)| widget_to_view(c, actual_key.as_str(), i))
        .collect();
    ViewNode {
        key: actual_key,
        kind: kind.into(),
        children: child_views,
        focusable: false,
        disabled: false,
    }
}

/// Generate an auto-key for widgets without an explicit key.
fn auto_key(parent_path: &str, index: usize, kind: &str) -> WidgetKey {
    if parent_path.is_empty() {
        WidgetKey::new(format!("{AUTO_KEY_PREFIX}:{index}:{kind}"))
    } else {
        WidgetKey::new(format!("{AUTO_KEY_PREFIX}:{parent_path}:{index}:{kind}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{button, column, label, row, scroll_view, separator, text_input};

    #[test]
    fn empty_root_produces_empty_views() {
        let views = widget_to_views::<()>(&[]);
        assert!(views.is_empty());
    }

    #[test]
    fn single_label_has_auto_key() {
        let views = widget_to_views(&[label::<()>("Hello")]);
        assert_eq!(views.len(), 1);
        assert!(views[0].key.as_str().starts_with(AUTO_KEY_PREFIX));
        assert_eq!(views[0].kind, "label");
        assert!(!views[0].focusable);
    }

    #[test]
    fn button_has_explicit_key() {
        let views = widget_to_views(&[WidgetNode::from(button::<()>("ok_btn", "OK"))]);
        assert_eq!(views[0].key.as_str(), "ok_btn");
        assert_eq!(views[0].kind, "button");
        assert!(views[0].focusable);
    }

    #[test]
    fn disabled_button_reflected() {
        let views = widget_to_views(&[WidgetNode::from(
            button::<()>("disabled_btn", "Nope").disabled(true),
        )]);
        assert!(views[0].disabled);
    }

    #[test]
    fn column_contains_child_labels() {
        let col = column::<()>().child(label("A")).child(label("B")).build();
        let views = widget_to_views(&[col]);
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].kind, "column");
        // Column has an auto-key because Container has key=None.
        assert!(views[0].key.as_str().starts_with(AUTO_KEY_PREFIX));
        assert_eq!(views[0].children.len(), 2);
        assert_eq!(views[0].children[0].kind, "label");
        assert_eq!(views[0].children[1].kind, "label");
    }

    #[test]
    fn row_with_keyed_container() {
        let row = row::<()>().key("my_row").child(label("X")).build();
        let views = widget_to_views(&[row]);
        assert_eq!(views[0].key.as_str(), "my_row");
        assert_eq!(views[0].kind, "row");
    }

    #[test]
    fn text_input_is_focusable() {
        let views = widget_to_views(&[text_input::<()>("name_input").build()]);
        assert!(views[0].focusable);
        assert_eq!(views[0].key.as_str(), "name_input");
    }

    #[test]
    fn separator_has_auto_key() {
        let views = widget_to_views(&[separator::<()>()]);
        assert_eq!(views[0].kind, "separator");
        assert!(views[0].key.as_str().starts_with(AUTO_KEY_PREFIX));
    }

    #[test]
    fn scroll_view_children_are_preserved() {
        let sv = scroll_view::<()>("sv").child(label("inner")).build();
        let views = widget_to_views(&[sv]);
        assert_eq!(views[0].key.as_str(), "sv");
        assert_eq!(views[0].kind, "scroll_view");
        assert_eq!(views[0].children.len(), 1);
        assert_eq!(views[0].children[0].kind, "label");
    }

    #[test]
    fn duplicate_label_keys_detected() {
        // Two unkeyed Labels side by side will have distinct auto-keys
        // because they have different indices.
        let col = column::<()>().child(label("A")).child(label("B")).build();
        let views = widget_to_views(&[col]);
        assert_eq!(views[0].children.len(), 2);
        assert_ne!(
            views[0].children[0].key.as_str(),
            views[0].children[1].key.as_str()
        );
    }

    #[test]
    fn container_auto_keys_differ_by_index() {
        let col = column::<()>()
            .child(separator::<()>())
            .child(separator::<()>())
            .build();
        let views = widget_to_views(&[col]);
        assert_ne!(
            views[0].children[0].key.as_str(),
            views[0].children[1].key.as_str()
        );
    }
}
