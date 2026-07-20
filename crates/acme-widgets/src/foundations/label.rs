use crate::WidgetNode;
use acme_layout::ShapedText;

/// A text label widget.
#[derive(Clone, Debug)]
pub struct Label {
    pub text: String,
    /// Optional font-size hint for intrinsic sizing.
    /// `None` means the renderer determines the size.
    pub font_size: Option<f32>,
    /// Cached shaped text, populated by the renderer for reuse across frames.
    /// Invalidated when `text` or `font_size` changes.
    pub cached: Option<ShapedText>,
}
/// Manual `PartialEq` — skips `cached` (rendering cache, not identity).
impl PartialEq for Label {
    fn eq(&self, other: &Self) -> bool {
        self.text == other.text && self.font_size == other.font_size
    }
}

/// Create a label widget.
pub fn label<M>(text: impl Into<String>) -> WidgetNode<M> {
    WidgetNode::Label(Label {
        text: text.into(),
        font_size: None,
        cached: None,
    })
}

/// Create a label with an explicit font-size hint.
pub fn label_with_size<M>(text: impl Into<String>, font_size: f32) -> WidgetNode<M> {
    WidgetNode::Label(Label {
        text: text.into(),
        font_size: Some(crate::finite(font_size)),
        cached: None,
    })
}
