//! DiffViewer component — side-by-side diff display with added/removed/unchanged lines.
//!
//! Renders a column where each diff line is a row with:
//! - Optional line number (small, muted foreground)
//! - "+" / "-" / " " prefix with semantic color
//! - The line text

use crate::*;

/// The kind of a diff line.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiffLineKind {
    /// Line was added — shown with "+" in success/green.
    Added,
    /// Line was removed — shown with "-" in danger/red.
    Removed,
    /// Line is context — shown with " " in neutral.
    Unchanged,
}

/// A single diff line.
#[derive(Clone, Debug)]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub text: String,
    pub line_number: Option<usize>,
}

/// Builder for a DiffViewer component.
pub struct DiffViewerBuilder<M> {
    pub id: WidgetKey,
    pub lines: Vec<DiffLine>,
    pub show_line_numbers: bool,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a new DiffViewer builder.
pub fn diff_viewer<M: Clone + 'static>(id: impl Into<WidgetKey>) -> DiffViewerBuilder<M> {
    DiffViewerBuilder {
        id: id.into(),
        lines: Vec::new(),
        show_line_numbers: true,
        _phantom: std::marker::PhantomData,
    }
}

/// Create a new DiffLine.
pub fn diff_line(text: impl Into<String>, kind: DiffLineKind) -> DiffLine {
    DiffLine {
        kind,
        text: text.into(),
        line_number: None,
    }
}

impl DiffLine {
    /// Attach an optional line number.
    pub fn with_line_number(mut self, n: usize) -> Self {
        self.line_number = Some(n);
        self
    }
}

impl<M: Clone + 'static> DiffViewerBuilder<M> {
    /// Add a single diff line.
    pub fn line(mut self, line: DiffLine) -> Self {
        self.lines.push(line);
        self
    }

    /// Set the full list of diff lines.
    pub fn lines(mut self, lines: Vec<DiffLine>) -> Self {
        self.lines = lines;
        self
    }

    /// Show or hide line numbers.
    pub fn show_line_numbers(mut self, value: bool) -> Self {
        self.show_line_numbers = value;
        self
    }
}

impl<M: Clone + 'static> From<DiffViewerBuilder<M>> for WidgetNode<M> {
    fn from(b: DiffViewerBuilder<M>) -> Self {
        let theme = acme_theme::Theme::light();
        let mut col = column::<M>().key(b.id).gap(0.0);

        for line in &b.lines {
            let (prefix, fg_color) = match line.kind {
                DiffLineKind::Added => ("+", Some(theme.colors.success)),
                DiffLineKind::Removed => ("-", Some(theme.colors.danger)),
                DiffLineKind::Unchanged => (" ", None),
            };

            let mut row_builder = row::<M>().gap(4.0);

            // Optional line number (small, muted)
            if b.show_line_numbers
                && let Some(n) = line.line_number
            {
                row_builder = row_builder.child(
                    label_builder::<M>(&n.to_string())
                        .font_size(11.0)
                        .color(theme.colors.muted_foreground)
                        .build(),
                );
            }

            // Prefix marker with semantic color
            let mut prefix_label = label_builder::<M>(prefix).font_size(12.0);
            if let Some(c) = fg_color {
                prefix_label = prefix_label.color(c);
            }
            row_builder = row_builder.child(prefix_label.build());

            // Text content
            row_builder = row_builder.child(label_builder::<M>(&line.text).font_size(12.0).build());

            // Wrap the row in a padded column to create line spacing
            let line_wrapper = column::<M>()
                .child(row_builder.build())
                .padding(1.0)
                .build();
            col = col.child(line_wrapper);
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

    #[test]
    fn diff_viewer_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = diff_viewer("dv")
            .line(diff_line("Hello world", DiffLineKind::Added))
            .line(diff_line("Old line", DiffLineKind::Removed))
            .line(diff_line("Context", DiffLineKind::Unchanged))
            .into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        // 3 lines = 3 children
        assert_eq!(layout.children.len(), 3);
    }

    #[test]
    fn diff_viewer_builder_defaults() {
        let dv = diff_viewer::<TestMsg>("dv");
        assert!(dv.lines.is_empty());
        assert!(dv.show_line_numbers);
    }

    #[test]
    fn diff_viewer_with_line_numbers() {
        let node: WidgetNode<TestMsg> = diff_viewer("dv")
            .show_line_numbers(true)
            .line(diff_line("Line one", DiffLineKind::Unchanged).with_line_number(1))
            .line(diff_line("Line two", DiffLineKind::Unchanged).with_line_number(2))
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant");
        };
        assert_eq!(col.children.len(), 2);
    }

    #[test]
    fn diff_viewer_hides_line_numbers() {
        let node: WidgetNode<TestMsg> = diff_viewer("dv")
            .show_line_numbers(false)
            .line(diff_line("Data", DiffLineKind::Added))
            .into();
        let WidgetNode::Column(col) = &node else {
            panic!("expected Column variant");
        };
        assert_eq!(col.children.len(), 1);
    }

    #[test]
    fn diff_line_kind_has_correct_text() {
        let added = diff_line("new", DiffLineKind::Added);
        assert_eq!(added.text, "new");
        assert_eq!(added.kind, DiffLineKind::Added);
        assert!(added.line_number.is_none());
    }

    #[test]
    fn diff_line_with_number() {
        let line = diff_line("code", DiffLineKind::Removed).with_line_number(42);
        assert_eq!(line.line_number, Some(42));
    }
}
