//! Cascader component — cascading dropdown selector.
//!
//! Select an option at each level, optionally exposing the next level of children.
//! When open, renders a Row of Columns — one Column of options per level.

use acme_core::WidgetKey;
use acme_widgets::*;

/// An option in the cascader hierarchy.
#[derive(Clone, Debug)]
pub struct CascaderOption {
    pub label: String,
    pub value: String,
    pub children: Vec<CascaderOption>,
}

/// Builder for a Cascader component.
pub struct CascaderBuilder<M> {
    pub id: WidgetKey,
    pub placeholder: String,
    pub options: Vec<CascaderOption>,
    pub selected_path: Vec<String>,
    pub open: bool,
    pub on_change: Option<M>,
}

/// Create a new Cascader builder.
pub fn cascader<M: Clone + 'static>(id: impl Into<WidgetKey>) -> CascaderBuilder<M> {
    CascaderBuilder {
        id: id.into(),
        placeholder: String::new(),
        options: vec![],
        selected_path: vec![],
        open: false,
        on_change: None,
    }
}

/// Create a cascader option node.
pub fn cascader_option(label: impl Into<String>, value: impl Into<String>) -> CascaderOption {
    CascaderOption {
        label: label.into(),
        value: value.into(),
        children: vec![],
    }
}

impl CascaderOption {
    /// Add a child option.
    pub fn child(mut self, node: CascaderOption) -> Self {
        self.children.push(node);
        self
    }

    /// Set all children at once.
    pub fn children(mut self, nodes: Vec<CascaderOption>) -> Self {
        self.children = nodes;
        self
    }
}

impl<M: Clone + 'static> CascaderBuilder<M> {
    /// Set the placeholder text.
    pub fn placeholder(mut self, value: impl Into<String>) -> Self {
        self.placeholder = value.into();
        self
    }

    /// Add a root-level option.
    pub fn option(mut self, node: CascaderOption) -> Self {
        self.options.push(node);
        self
    }

    /// Set the selected path (values at each level).
    pub fn selected_path(mut self, values: Vec<String>) -> Self {
        self.selected_path = values;
        self
    }

    /// Set whether the dropdown is open.
    pub fn open(mut self, value: bool) -> Self {
        self.open = value;
        self
    }

    /// Set the message dispatched when the selection changes.
    pub fn on_change(mut self, msg: M) -> Self {
        self.on_change = Some(msg);
        self
    }
}

/// Find the label(s) for a selected path, returning "label1 / label2 / ...".
fn format_selected_path(options: &[CascaderOption], path: &[String]) -> String {
    let mut labels = Vec::new();
    let mut current = options;
    for p in path {
        if let Some(found) = current.iter().find(|o| o.value == *p) {
            labels.push(found.label.clone());
            current = &found.children;
        } else {
            break;
        }
    }
    if labels.is_empty() {
        String::new()
    } else {
        labels.join(" / ")
    }
}

/// Render a level of options as a Column, highlighting the selected one.
fn render_level<M: Clone + 'static>(
    options: &[CascaderOption],
    selected: Option<&str>,
    id_prefix: &str,
    level: usize,
) -> WidgetNode<M> {
    let mut col = column::<M>().gap(2.0);
    for (i, opt) in options.iter().enumerate() {
        let is_selected = selected.is_some_and(|s| s == opt.value);
        let opt_key = format!("{}-l{}-o{}", id_prefix, level, i);
        let opt_card = card::<M>()
            .key(opt_key.as_str())
            .padding(6.0)
            .variant(if is_selected {
                CardVariant::Interactive
            } else {
                CardVariant::Plain
            })
            .child(label::<M>(opt.label.clone()));
        col = col.child(opt_card);
    }
    col.build()
}

/// Build the cascade of level Columns based on the selected path.
fn build_cascade_columns<M: Clone + 'static>(
    options: &[CascaderOption],
    path: &[String],
    id_prefix: &str,
) -> Vec<WidgetNode<M>> {
    let mut columns = Vec::new();
    let mut current = options;
    let mut depth = 0usize;

    // Always render the first level
    let selected_first = path.first().map(|s| s.as_str());
    columns.push(render_level::<M>(current, selected_first, id_prefix, depth));

    // For each selected path entry, render the next level if there are children
    for p in path {
        if let Some(found) = current.iter().find(|o| o.value == *p) {
            if !found.children.is_empty() {
                depth += 1;
                let next_selected = path.get(depth).map(|s| s.as_str());
                columns.push(render_level::<M>(
                    &found.children,
                    next_selected,
                    id_prefix,
                    depth,
                ));
                current = &found.children;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    columns
}

impl<M: Clone + 'static> From<CascaderBuilder<M>> for WidgetNode<M> {
    fn from(b: CascaderBuilder<M>) -> Self {
        // Closed: show formatted path or placeholder.
        if !b.open {
            let display = if b.selected_path.is_empty() {
                b.placeholder.clone()
            } else {
                format_selected_path(&b.options, &b.selected_path)
            };
            return card::<M>()
                .key(b.id)
                .variant(CardVariant::Outlined)
                .padding(8.0)
                .child(label::<M>(display))
                .build();
        }

        // Open: top label + Row of level Columns
        let display = if b.selected_path.is_empty() {
            b.placeholder.clone()
        } else {
            format_selected_path(&b.options, &b.selected_path)
        };

        let header = label::<M>(display);
        let cascade = build_cascade_columns::<M>(&b.options, &b.selected_path, b.id.as_str());

        let mut level_row = row::<M>().gap(8.0);
        for col in cascade {
            level_row = level_row.child(col);
        }

        column::<M>()
            .key(b.id)
            .gap(4.0)
            .child(header)
            .child(level_row)
            .build()
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

    #[test]
    fn cascader_builder_defaults() {
        let c = cascader::<TestMsg>("c");
        assert!(c.placeholder.is_empty());
        assert!(c.options.is_empty());
        assert!(c.selected_path.is_empty());
        assert!(!c.open);
    }

    #[test]
    fn cascader_closed_renders_card() {
        let node: WidgetNode<TestMsg> = cascader("c").placeholder("Choose...").into();
        let WidgetNode::Card(card) = &node else {
            panic!("expected Card variant when closed");
        };
        assert_eq!(card.variant, CardVariant::Outlined);
    }

    #[test]
    fn cascader_open_renders_column_with_header_and_levels() {
        let node: WidgetNode<TestMsg> = cascader("c")
            .option(cascader_option("Fruits", "fruits").child(cascader_option("Apple", "apple")))
            .open(true)
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant when open");
        };
        // header + level row
        assert_eq!(col.children.len(), 2);
    }

    #[test]
    fn cascader_format_selected_path() {
        let opts =
            vec![cascader_option("Fruits", "fruits").child(cascader_option("Apple", "apple"))];
        let path = vec!["fruits".into(), "apple".into()];
        let formatted = format_selected_path(&opts, &path);
        assert_eq!(formatted, "Fruits / Apple");
    }

    #[test]
    fn cascader_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = cascader("c")
            .option(cascader_option("Fruits", "fruits"))
            .open(true)
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        assert!(!layout.children.is_empty());
    }
}
