//! MarkdownView — a simple markdown renderer that converts markdown text to a
//! tree of styled widgets.

use crate::*;

/// Builder for a MarkdownView component.
pub struct MarkdownViewBuilder<M> {
    pub id: WidgetKey,
    pub text: String,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a new MarkdownView builder with initial markdown text.
pub fn markdown_view<M: Clone + 'static>(
    id: impl Into<WidgetKey>,
    text: impl Into<String>,
) -> MarkdownViewBuilder<M> {
    MarkdownViewBuilder::<M> {
        id: id.into(),
        text: text.into(),
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> MarkdownViewBuilder<M> {
    /// Set the markdown source text.
    pub fn text(mut self, value: impl Into<String>) -> Self {
        self.text = value.into();
        self
    }
}

/// Parse a single line of markdown and return zero or more widget nodes.
fn parse_line<M: Clone + 'static>(
    line: &str,
    nodes: &mut Vec<WidgetNode<M>>,
) {
    let trimmed = line.trim();

    if trimmed.is_empty() {
        // Spacer for empty lines
        nodes.push(column::<M>().height(8.0).build());
        return;
    }

    // Headings
    if trimmed.starts_with("### ") {
        let text = trimmed.trim_start_matches("### ");
        nodes.push(label_with_size::<M>(text, 18.0));
        return;
    }
    if trimmed.starts_with("## ") {
        let text = trimmed.trim_start_matches("## ");
        nodes.push(label_with_size::<M>(text, 22.0));
        return;
    }
    if trimmed.starts_with("# ") {
        let text = trimmed.trim_start_matches("# ");
        nodes.push(label_with_size::<M>(text, 28.0));
        return;
    }

    // Unordered list
    if trimmed.starts_with("- ") {
        let text = trimmed.trim_start_matches("- ");
        nodes.push(
            row::<M>()
                .child(label::<M>("•"))
                .child(label::<M>(text))
                .build(),
        );
        return;
    }

    // Blockquote
    if trimmed.starts_with("> ") {
        let text = trimmed.trim_start_matches("> ");
        nodes.push(
            card::<M>()
                .variant(CardVariant::Outlined)
                .child(label::<M>(text))
                .build(),
        );
        return;
    }

    // Regular body text (14px)
    nodes.push(label::<M>(line.to_string()));
}

impl<M: Clone + 'static> From<MarkdownViewBuilder<M>> for WidgetNode<M> {
    fn from(b: MarkdownViewBuilder<M>) -> Self {
        let mut col = column::<M>().key(b.id).gap(0.0);
        let mut nodes: Vec<WidgetNode<M>> = Vec::new();
        let mut in_code_block = false;
        let mut code_buffer: Vec<String> = Vec::new();

        for line in b.text.lines() {
            let trimmed = line.trim();

            // Toggle code block mode on ``` delimiters
            if trimmed.starts_with("```") {
                if in_code_block {
                    // End code block — emit buffered lines as a card
                    let code_text = code_buffer.join("\n");
                    nodes.push(
                        card::<M>()
                            .variant(CardVariant::Outlined)
                            .child(label::<M>(code_text))
                            .build(),
                    );
                    code_buffer.clear();
                }
                in_code_block = !in_code_block;
                continue;
            }

            if in_code_block {
                code_buffer.push(line.to_string());
                continue;
            }

            parse_line(line, &mut nodes);
        }

        // Flush any code block that was never closed
        if in_code_block && !code_buffer.is_empty() {
            let code_text = code_buffer.join("\n");
            nodes.push(
                card::<M>()
                    .variant(CardVariant::Outlined)
                    .child(label::<M>(code_text))
                    .build(),
            );
        }

        for node in nodes {
            col = col.child(node);
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
    fn markdown_view_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> =
            markdown_view("md", "# Hello\n\nThis is a paragraph.").into();
        let layout = node.to_layout(NodeId::new(1));
        assert_eq!(layout.style.kind, LayoutKind::Column);
        // h1 + spacer + paragraph = 3 children
        assert_eq!(layout.children.len(), 3);
    }

    #[test]
    fn markdown_view_builder_defaults() {
        let mv = markdown_view::<TestMsg>("md", "default text");
        assert_eq!(mv.text, "default text");
    }

    #[test]
    fn markdown_view_parses_headings() {
        let mv = markdown_view::<TestMsg>("md", "# H1\n## H2\n### H3");
        assert_eq!(mv.text, "# H1\n## H2\n### H3");
    }

    #[test]
    fn markdown_view_code_block() {
        let md = "before\n```\ncode block\n```\nafter";
        let node: WidgetNode<TestMsg> = markdown_view("md", md).into();
        let layout = node.to_layout(NodeId::new(1));
        // before + code card + after = 3 children
        assert_eq!(layout.children.len(), 3);
    }

    #[test]
    fn markdown_view_list_and_blockquote() {
        let md = "- item\n> quote";
        let node: WidgetNode<TestMsg> = markdown_view("md", md).into();
        let layout = node.to_layout(NodeId::new(1));
        // list row + blockquote card = 2 children
        assert_eq!(layout.children.len(), 2);
    }
}
