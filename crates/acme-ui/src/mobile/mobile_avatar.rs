//! Mobile avatar — circular initials or image placeholder with size variants.

use crate::WidgetNode;
use acme_core::WidgetKey;

/// Size preset for mobile avatars.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MobileAvatarSize {
    Sm,
    #[default]
    Md,
    Lg,
}

impl MobileAvatarSize {
    pub fn diameter(&self) -> f32 {
        match self {
            Self::Sm => 32.0,
            Self::Md => 40.0,
            Self::Lg => 56.0,
        }
    }
}

/// Builder for a mobile avatar.
pub struct MobileAvatarBuilder<M> {
    pub id: WidgetKey,
    pub initials: String,
    pub size: MobileAvatarSize,
    _phantom: std::marker::PhantomData<M>,
}

/// Create a mobile avatar builder.
pub fn mobile_avatar<M: Clone + 'static>(
    initials: impl Into<String>,
) -> MobileAvatarBuilder<M> {
    MobileAvatarBuilder {
        id: WidgetKey::from("mobile_avatar"),
        initials: initials.into(),
        size: MobileAvatarSize::default(),
        _phantom: std::marker::PhantomData,
    }
}

impl<M: Clone + 'static> MobileAvatarBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn size(mut self, value: MobileAvatarSize) -> Self {
        self.size = value;
        self
    }
}

impl<M: Clone + 'static> From<MobileAvatarBuilder<M>> for WidgetNode<M> {
    fn from(b: MobileAvatarBuilder<M>) -> Self {
        let d = b.size.diameter();
        crate::stack::<M>()
            .key(b.id)
            .size(d, d)
            .child(crate::label(b.initials))
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    enum Msg {}

    #[test]
    fn mobile_avatar_produces_stack() {
        let node: WidgetNode<Msg> = mobile_avatar("AB").into();
        assert!(matches!(node, WidgetNode::Stack(_)));
    }

    #[test]
    fn mobile_avatar_has_child_label() {
        let node: WidgetNode<Msg> = mobile_avatar("XY").into();
        let WidgetNode::Stack(s) = &node else {
            panic!("expected Stack");
        };
        assert_eq!(s.children.len(), 1);
    }

    #[test]
    fn mobile_avatar_size_diameter() {
        assert_eq!(MobileAvatarSize::Sm.diameter(), 32.0);
        assert_eq!(MobileAvatarSize::Lg.diameter(), 56.0);
    }
}
