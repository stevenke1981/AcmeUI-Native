//! Declarative MVP widget descriptions.
#![forbid(unsafe_op_in_unsafe_fn)]

pub use acme_core::WidgetKey;
use acme_layout::{Edges, LayoutKind, LayoutNode, LayoutStyle, Length, Overflow};
use acme_theme::{Theme, ThemeColor};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
    Danger,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ButtonState {
    pub hovered: bool,
    pub pressed: bool,
    pub focused: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResolvedButtonStyle {
    pub background: ThemeColor,
    pub foreground: ThemeColor,
    pub border: ThemeColor,
    pub focus: ThemeColor,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WidgetNode<M> {
    Row(Container<M>),
    Column(Container<M>),
    Stack(Container<M>),
    Label(Label),
    Button(Button<M>),
    Card(Container<M>),
    Separator(Separator),
    ScrollView(ScrollView<M>),
}
impl<M> WidgetNode<M> {
    pub fn key(&self) -> Option<&WidgetKey> {
        match self {
            Self::Button(v) => Some(&v.key),
            Self::ScrollView(v) => Some(&v.key),
            Self::Row(v) | Self::Column(v) | Self::Stack(v) | Self::Card(v) => v.key.as_ref(),
            Self::Label(_) | Self::Separator(_) => None,
        }
    }
    pub fn children(&self) -> &[WidgetNode<M>] {
        match self {
            Self::Row(v) | Self::Column(v) | Self::Stack(v) | Self::Card(v) => &v.children,
            Self::ScrollView(v) => &v.children,
            Self::Label(_) | Self::Button(_) | Self::Separator(_) => &[],
        }
    }
    pub fn to_layout(&self, next: &mut u64) -> LayoutNode {
        let id = *next;
        *next += 1;
        match self {
            Self::Row(v) => LayoutNode::container(
                id,
                v.layout(LayoutKind::Row),
                v.children.iter().map(|c| c.to_layout(next)).collect(),
            ),
            Self::Column(v) => LayoutNode::container(
                id,
                v.layout(LayoutKind::Column),
                v.children.iter().map(|c| c.to_layout(next)).collect(),
            ),
            Self::Stack(v) => LayoutNode::container(
                id,
                v.layout(LayoutKind::Stack),
                v.children.iter().map(|c| c.to_layout(next)).collect(),
            ),
            Self::Card(v) => LayoutNode::container(
                id,
                v.layout(LayoutKind::Column),
                v.children.iter().map(|c| c.to_layout(next)).collect(),
            ),
            Self::ScrollView(v) => LayoutNode::container(
                id,
                v.layout(),
                v.children.iter().map(|c| c.to_layout(next)).collect(),
            ),
            Self::Label(_) => LayoutNode::leaf(id, LayoutStyle::default()),
            Self::Button(_) => LayoutNode::leaf(
                id,
                LayoutStyle {
                    height: Length::px(36.0),
                    ..Default::default()
                },
            ),
            Self::Separator(v) => LayoutNode::leaf(
                id,
                LayoutStyle {
                    height: Length::px(v.thickness),
                    ..Default::default()
                },
            ),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Label {
    pub text: String,
}
pub fn label<M>(text: impl Into<String>) -> WidgetNode<M> {
    WidgetNode::Label(Label { text: text.into() })
}

#[derive(Clone, Debug, PartialEq)]
pub struct Button<M> {
    pub key: WidgetKey,
    pub label: String,
    pub variant: ButtonVariant,
    pub disabled: bool,
    message: Option<M>,
}
pub fn button<M>(key: impl Into<WidgetKey>, label: impl Into<String>) -> Button<M> {
    Button {
        key: key.into(),
        label: label.into(),
        variant: ButtonVariant::Secondary,
        disabled: false,
        message: None,
    }
}
impl<M> Button<M> {
    pub fn primary(mut self) -> Self {
        self.variant = ButtonVariant::Primary;
        self
    }
    pub fn variant(mut self, value: ButtonVariant) -> Self {
        self.variant = value;
        self
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
    pub fn on_click(mut self, message: M) -> WidgetNode<M> {
        self.message = Some(message);
        WidgetNode::Button(self)
    }
    pub fn activate(&self) -> Option<&M> {
        if self.disabled {
            None
        } else {
            self.message.as_ref()
        }
    }
    pub fn resolve_style(&self, theme: &Theme, state: ButtonState) -> ResolvedButtonStyle {
        let c = theme.colors;
        let background = if self.disabled {
            c.surface
        } else {
            match self.variant {
                ButtonVariant::Primary => {
                    if state.hovered {
                        c.accent_hover
                    } else {
                        c.accent
                    }
                }
                ButtonVariant::Danger => c.danger,
                ButtonVariant::Secondary => {
                    if state.hovered {
                        c.surface_hover
                    } else {
                        c.surface
                    }
                }
                ButtonVariant::Ghost => c.background,
            }
        };
        ResolvedButtonStyle {
            background,
            foreground: if self.disabled {
                c.disabled_text
            } else if self.variant == ButtonVariant::Primary {
                c.on_accent
            } else if self.variant == ButtonVariant::Danger {
                c.on_danger
            } else {
                c.text
            },
            border: c.border,
            focus: c.focus,
        }
    }
}
impl<M> From<Button<M>> for WidgetNode<M> {
    fn from(value: Button<M>) -> Self {
        WidgetNode::Button(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Container<M> {
    pub key: Option<WidgetKey>,
    pub children: Vec<WidgetNode<M>>,
    pub gap: f32,
    pub padding: Edges,
}
impl<M> Container<M> {
    fn new() -> Self {
        Self {
            key: None,
            children: vec![],
            gap: 0.0,
            padding: Edges::default(),
        }
    }
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.key = Some(key.into());
        self
    }
    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.children.push(child.into());
        self
    }
    pub fn gap(mut self, value: f32) -> Self {
        self.gap = finite(value);
        self
    }
    pub fn padding(mut self, value: f32) -> Self {
        self.padding = Edges::all(value);
        self
    }
    fn layout(&self, kind: LayoutKind) -> LayoutStyle {
        LayoutStyle {
            kind,
            gap: self.gap,
            padding: self.padding,
            ..Default::default()
        }
    }
}

pub struct ContainerBuilder<M> {
    container: Container<M>,
    kind: LayoutKind,
}
impl<M> ContainerBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.container = self.container.key(key);
        self
    }
    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.container = self.container.child(child);
        self
    }
    pub fn gap(mut self, value: f32) -> Self {
        self.container = self.container.gap(value);
        self
    }
    pub fn padding(mut self, value: f32) -> Self {
        self.container = self.container.padding(value);
        self
    }
    pub fn build(self) -> WidgetNode<M> {
        match self.kind {
            LayoutKind::Row => WidgetNode::Row(self.container),
            LayoutKind::Column => WidgetNode::Column(self.container),
            LayoutKind::Stack => WidgetNode::Stack(self.container),
            LayoutKind::Leaf => WidgetNode::Card(self.container),
        }
    }
}
impl<M> From<ContainerBuilder<M>> for WidgetNode<M> {
    fn from(value: ContainerBuilder<M>) -> Self {
        value.build()
    }
}
pub fn row<M>() -> ContainerBuilder<M> {
    ContainerBuilder {
        container: Container::new(),
        kind: LayoutKind::Row,
    }
}
pub fn column<M>() -> ContainerBuilder<M> {
    ContainerBuilder {
        container: Container::new(),
        kind: LayoutKind::Column,
    }
}
pub fn stack<M>() -> ContainerBuilder<M> {
    ContainerBuilder {
        container: Container::new(),
        kind: LayoutKind::Stack,
    }
}
pub fn card<M>() -> ContainerBuilder<M> {
    ContainerBuilder {
        container: Container::new(),
        kind: LayoutKind::Leaf,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Separator {
    pub thickness: f32,
}
pub fn separator<M>() -> WidgetNode<M> {
    WidgetNode::Separator(Separator { thickness: 1.0 })
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollView<M> {
    pub key: WidgetKey,
    pub children: Vec<WidgetNode<M>>,
    pub viewport_height: Length,
}
impl<M> ScrollView<M> {
    pub fn child(mut self, child: impl Into<WidgetNode<M>>) -> Self {
        self.children.push(child.into());
        self
    }
    pub fn viewport_height(mut self, value: f32) -> Self {
        self.viewport_height = Length::px(value);
        self
    }
    pub fn build(self) -> WidgetNode<M> {
        WidgetNode::ScrollView(self)
    }
    fn layout(&self) -> LayoutStyle {
        LayoutStyle {
            kind: LayoutKind::Column,
            height: self.viewport_height,
            overflow: Overflow::Scroll,
            ..Default::default()
        }
    }
}
pub fn scroll_view<M>(key: impl Into<WidgetKey>) -> ScrollView<M> {
    ScrollView {
        key: key.into(),
        children: vec![],
        viewport_height: Length::Auto,
    }
}

fn finite(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Clone, Debug, PartialEq)]
    enum Msg {
        Save,
    }
    #[test]
    fn builders_make_expected_tree() {
        let tree = column::<Msg>()
            .key("root")
            .gap(8.0)
            .child(label("標題"))
            .child(button("save", "儲存").primary().on_click(Msg::Save))
            .build();
        assert_eq!(tree.children().len(), 2);
        assert_eq!(tree.key().unwrap().as_str(), "root");
    }
    #[test]
    fn disabled_button_does_not_activate() {
        let node = button("save", "save").disabled(true).on_click(Msg::Save);
        let WidgetNode::Button(b) = node else {
            panic!()
        };
        assert_eq!(b.activate(), None);
    }
    #[test]
    fn button_uses_theme_tokens() {
        let b = button::<Msg>("save", "save").primary();
        let theme = Theme::dark();
        assert_eq!(
            b.resolve_style(&theme, ButtonState::default()).background,
            theme.colors.accent
        );
        assert_eq!(
            b.resolve_style(
                &theme,
                ButtonState {
                    hovered: true,
                    ..Default::default()
                }
            )
            .background,
            theme.colors.accent_hover
        );
    }
    #[test]
    fn scroll_layout_is_clipped() {
        let tree = scroll_view::<Msg>("scroll")
            .viewport_height(100.0)
            .child(label("內容"))
            .build();
        let mut id = 1;
        let layout = tree.to_layout(&mut id);
        assert_eq!(layout.style.overflow, Overflow::Scroll);
        assert_eq!(layout.style.height, Length::px(100.0));
    }
}
