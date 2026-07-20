use crate::WidgetNode;
use acme_layout::ShapedText;
use acme_layout::TextWrapMode;

/// Controls how text wraps when the label's content exceeds its available width.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LabelWrap {
    /// No wrapping — text may overflow.
    #[default]
    None,
    /// Break at word boundaries.
    Word,
    /// Break at character boundaries.
    Character,
}

/// Convert a [`LabelWrap`] to the layout engine's [`TextWrapMode`].
pub fn map_label_wrap(wrap: LabelWrap) -> TextWrapMode {
    match wrap {
        LabelWrap::None => TextWrapMode::None,
        LabelWrap::Word => TextWrapMode::Word,
        LabelWrap::Character => TextWrapMode::Character,
    }
}

/// A text label widget.
#[derive(Clone, Debug)]
pub struct Label {
    pub text: String,
    /// Optional font-size hint for intrinsic sizing.
    /// `None` means the renderer determines the size.
    pub font_size: Option<f32>,
    /// Optional line-height override. Falls back to the theme's body line-height
    /// during layout when `None`.
    pub line_height: Option<f32>,
    /// Text wrapping behaviour.
    pub wrap: LabelWrap,
    /// Maximum visible lines before truncation. `None` means unlimited.
    pub max_lines: Option<usize>,
    /// Cached shaped text, populated by the renderer for reuse across frames.
    /// Invalidated when `text` or `font_size` changes.
    pub cached: Option<ShapedText>,
}
/// Manual `PartialEq` — skips `cached` (rendering cache, not identity).
impl PartialEq for Label {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text
            && self.font_size == other.font_size
            && self.line_height == other.line_height
            && self.wrap == other.wrap
            && self.max_lines == other.max_lines
    }
}

/// Create a label widget.
pub fn label<M>(text: impl Into<String>) -> WidgetNode<M> {
    WidgetNode::Label(Label {
        text: text.into(),
        font_size: None,
        line_height: None,
        wrap: LabelWrap::default(),
        max_lines: None,
        cached: None,
    })
}

/// Create a label with an explicit font-size hint.
pub fn label_with_size<M>(text: impl Into<String>, font_size: f32) -> WidgetNode<M> {
    WidgetNode::Label(Label {
        text: text.into(),
        font_size: Some(crate::finite(font_size)),
        line_height: None,
        wrap: LabelWrap::default(),
        max_lines: None,
        cached: None,
    })
}

/// Builder for constructing a [`Label`] widget with full control over
/// typography and wrapping properties.
#[derive(Clone, Debug)]
pub struct LabelBuilder<M> {
    label: Label,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a label builder with the given text content.
pub fn label_builder<M>(text: impl Into<String>) -> LabelBuilder<M> {
    LabelBuilder {
        label: Label {
            text: text.into(),
            font_size: None,
            line_height: None,
            wrap: LabelWrap::default(),
            max_lines: None,
            cached: None,
        },
        _phantom: std::marker::PhantomData,
    }
}

impl<M> LabelBuilder<M> {
    /// Set an explicit font-size hint.
    pub fn font_size(mut self, value: f32) -> Self {
        self.label.font_size = Some(crate::finite(value));
        self
    }

    /// Set an explicit line-height override.
    pub fn line_height(mut self, value: f32) -> Self {
        self.label.line_height = Some(value);
        self
    }

    /// Set the text-wrapping mode.
    pub fn wrap(mut self, value: LabelWrap) -> Self {
        self.label.wrap = value;
        self
    }

    /// Set the maximum number of visible lines before truncation.
    pub fn max_lines(mut self, value: usize) -> Self {
        self.label.max_lines = Some(value);
        self
    }

    /// Consume the builder and produce a [`WidgetNode::Label`].
    pub fn build(self) -> WidgetNode<M> {
        WidgetNode::Label(self.label)
    }
}
