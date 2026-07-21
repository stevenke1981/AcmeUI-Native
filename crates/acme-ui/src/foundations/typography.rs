//! Typography — text formatting components (Title, Paragraph, Text).
//! Aligns with Ant Design Typography component.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Title level (h1–h5).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TitleLevel {
    H1,
    #[default]
    H2,
    H3,
    H4,
    H5,
}

impl TitleLevel {
    pub fn font_size(&self) -> f32 {
        match self {
            Self::H1 => 38.0,
            Self::H2 => 30.0,
            Self::H3 => 24.0,
            Self::H4 => 20.0,
            Self::H5 => 16.0,
        }
    }
}

/// Builder for a typography title.
pub struct TitleBuilder<M> {
    pub id: WidgetKey,
    pub text: String,
    pub level: TitleLevel,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a title builder.
pub fn title<M: Clone + 'static>(text: impl Into<String>) -> TitleBuilder<M> {
    TitleBuilder {
        id: WidgetKey::from("title"),
        text: text.into(),
        level: TitleLevel::default(),
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> TitleBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn level(mut self, value: TitleLevel) -> Self {
        self.level = value;
        self
    }
}

impl<M: Clone + 'static> From<TitleBuilder<M>> for WidgetNode<M> {
    fn from(b: TitleBuilder<M>) -> Self {
        let mut node = crate::label(b.text);
        if let WidgetNode::Label(ref mut l) = node {
            l.font_size = Some(b.level.font_size());
        }
        node
    }
}

/// Builder for a typography paragraph.
pub struct ParagraphBuilder<M> {
    pub id: WidgetKey,
    pub text: String,
    pub ellipsis: bool,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a paragraph builder.
pub fn paragraph<M: Clone + 'static>(text: impl Into<String>) -> ParagraphBuilder<M> {
    ParagraphBuilder {
        id: WidgetKey::from("paragraph"),
        text: text.into(),
        ellipsis: false,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> ParagraphBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn ellipsis(mut self, value: bool) -> Self {
        self.ellipsis = value;
        self
    }
}

impl<M: Clone + 'static> From<ParagraphBuilder<M>> for WidgetNode<M> {
    fn from(b: ParagraphBuilder<M>) -> Self {
        let text = if b.ellipsis && b.text.len() > 80 {
            format!("{}…", &b.text[..77])
        } else {
            b.text
        };
        crate::label(text)
    }
}

/// Text type for inline text styling.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextType {
    #[default]
    Default,
    Secondary,
    Success,
    Warning,
    Danger,
    Disabled,
    Mark,
    Code,
    Underline,
    Delete,
    Strong,
}

/// Builder for inline text.
pub struct TextBuilder<M> {
    pub id: WidgetKey,
    pub text: String,
    pub text_type: TextType,
    _phantom: std::marker::PhantomData<M>,
}

/// Create an inline text builder.
pub fn text<M: Clone + 'static>(content: impl Into<String>) -> TextBuilder<M> {
    TextBuilder {
        id: WidgetKey::from("text"),
        text: content.into(),
        text_type: TextType::default(),
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> TextBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn text_type(mut self, value: TextType) -> Self {
        self.text_type = value;
        self
    }

    pub fn strong(mut self) -> Self {
        self.text_type = TextType::Strong;
        self
    }

    pub fn code(mut self) -> Self {
        self.text_type = TextType::Code;
        self
    }
}

impl<M: Clone + 'static> From<TextBuilder<M>> for WidgetNode<M> {
    fn from(b: TextBuilder<M>) -> Self {
        let display = match b.text_type {
            TextType::Code => format!("`{}`", b.text),
            TextType::Delete => format!("~~{}~~", b.text),
            TextType::Underline => format!("_{}_", b.text),
            TextType::Mark => format!("=={}==", b.text),
            _ => b.text,
        };
        crate::label(display)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn title_produces_label() {
        let node: WidgetNode<Msg> = title("Hello").into();
        assert!(matches!(node, WidgetNode::Label(_)));
    }

    #[test]
    fn title_level_font_sizes() {
        assert_eq!(TitleLevel::H1.font_size(), 38.0);
        assert_eq!(TitleLevel::H5.font_size(), 16.0);
    }

    #[test]
    fn paragraph_produces_label() {
        let node: WidgetNode<Msg> = paragraph("Some text").into();
        assert!(matches!(node, WidgetNode::Label(_)));
    }

    #[test]
    fn paragraph_ellipsis_truncates() {
        let long = "a".repeat(100);
        let node: WidgetNode<Msg> = paragraph(long).ellipsis(true).into();
        let WidgetNode::Label(l) = &node else {
            panic!("expected Label");
        };
        assert!(l.text.len() <= 80);
    }

    #[test]
    fn text_code_wraps_backticks() {
        let node: WidgetNode<Msg> = text("foo").code().into();
        let WidgetNode::Label(l) = &node else {
            panic!("expected Label");
        };
        assert_eq!(l.text, "`foo`");
    }

    #[test]
    fn text_strong_type() {
        let b = text::<Msg>("bold").strong();
        assert_eq!(b.text_type, TextType::Strong);
    }
}
