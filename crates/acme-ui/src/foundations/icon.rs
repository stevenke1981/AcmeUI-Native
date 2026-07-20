//! Icon component — renders a symbolic character as a Label.

use crate::WidgetNode;

/// Named icon identifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IconName {
    Check,
    Close,
    Menu,
    ChevronDown,
    ChevronLeft,
    ChevronRight,
    Info,
    Warning,
    Error,
    Success,
    Search,
    Plus,
    Minus,
    Settings,
    User,
    Eye,
    EyeOff,
    Star,
    Folder,
    Clock,
    ArrowUp,
    ArrowDown,
    Calendar,
}

impl IconName {
    /// Return the Unicode character for this icon.
    pub fn char(&self) -> &'static str {
        match self {
            Self::Check => "✓",
            Self::Close => "✕",
            Self::Menu => "☰",
            Self::ChevronDown => "▾",
            Self::ChevronLeft => "◂",
            Self::ChevronRight => "▸",
            Self::Info => "ℹ",
            Self::Warning => "⚠",
            Self::Error => "✗",
            Self::Success => "✔",
            Self::Search => "⌕",
            Self::Plus => "+",
            Self::Minus => "−",
            Self::Settings => "⚙",
            Self::User => "👤",
            Self::Eye => "👁",
            Self::EyeOff => "👁‍🗨",
            Self::Star => "★",
            Self::Folder => "📁",
            Self::Clock => "🕐",
            Self::ArrowUp => "↑",
            Self::ArrowDown => "↓",
            Self::Calendar => "📅",
        }
    }
}

/// Builder for an icon widget.
pub struct IconBuilder<M> {
    pub name: IconName,
    pub size: f32,
    pub color: Option<acme_theme::ThemeColor>,
    _phantom: std::marker::PhantomData<M>,
}

/// Create an icon builder.
pub fn icon<M>(name: IconName) -> IconBuilder<M> {
    IconBuilder {
        name,
        size: 16.0,
        color: None,
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> IconBuilder<M> {
    /// Set the icon size in pixels.
    pub fn size(mut self, px: f32) -> Self {
        self.size = px;
        self
    }

    /// Set the icon color.
    pub fn color(mut self, color: acme_theme::ThemeColor) -> Self {
        self.color = Some(color);
        self
    }

    /// Build the icon widget.
    pub fn build(self) -> WidgetNode<M> {
        let mut lbl = crate::label(self.name.char());
        if let WidgetNode::Label(ref mut l) = lbl {
            l.font_size = Some(self.size);
        }
        lbl
    }
}

impl<M: Clone + 'static> From<IconBuilder<M>> for WidgetNode<M> {
    fn from(b: IconBuilder<M>) -> Self {
        b.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    use acme_core::NodeId;
    use acme_layout::{LayoutEngine, WidgetLayoutContext};

    fn test_context() -> WidgetLayoutContext {
        WidgetLayoutContext {
            body_font_size: 16.0,
            body_line_height: 22.0,
            label_font_size: 14.0,
            control_height: 32.0,
            scale_factor: 1.0,
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    enum TestMsg {}

    #[test]
    fn icon_has_non_zero_layout_rect() {
        let node: WidgetNode<TestMsg> = icon(IconName::Check).size(24.0).build();
        let ctx = test_context();
        let layout = node.to_layout_with_context(NodeId::new(1), &ctx);
        let mut fonts = acme_text::FontSystem::new();
        let snapshot = LayoutEngine::new()
            .compute_with_text(&layout, (800.0, 600.0), &mut fonts, 1.0)
            .unwrap();
        let rect = snapshot.get(NodeId::new(1)).unwrap();
        assert!(rect.width > 0.0, "icon width should be > 0");
        assert!(rect.height > 0.0, "icon height should be > 0");
    }

    #[test]
    fn icon_displays_label_text() {
        let node: WidgetNode<TestMsg> = icon(IconName::Check).build();
        let WidgetNode::Label(l) = &node else {
            panic!("expected Label variant");
        };
        assert_eq!(l.text, "✓");
    }
}
