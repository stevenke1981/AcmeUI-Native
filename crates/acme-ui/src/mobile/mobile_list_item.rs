//! Mobile list item — title + detail + optional trailing icon, tappable.
//!
//! Hosts dispatch the existing Row click message. Text wraps inside the flexible
//! content column while the trailing icon keeps a fixed box.

use crate::WidgetNode;
use acme_core::WidgetKey;
use acme_layout::Length;
use acme_widgets::style::Styled;

/// Builder for a mobile list item.
pub struct MobileListItemBuilder<M> {
    pub id: WidgetKey,
    pub title: String,
    pub detail: Option<String>,
    pub trailing_icon: Option<crate::IconName>,
    pub on_tap: Option<M>,
}

/// Create a mobile list item builder.
pub fn mobile_list_item<M: Clone + 'static>(
    title: impl Into<String>,
    detail: Option<impl Into<String>>,
) -> MobileListItemBuilder<M> {
    MobileListItemBuilder {
        id: WidgetKey::from("mobile_list_item"),
        title: title.into(),
        detail: detail.map(Into::into),
        trailing_icon: None,
        on_tap: None,
    }
}

impl<M: Clone + 'static> MobileListItemBuilder<M> {
    pub fn key(mut self, key: impl Into<WidgetKey>) -> Self {
        self.id = key.into();
        self
    }

    pub fn trailing_icon(mut self, icon: crate::IconName) -> Self {
        self.trailing_icon = Some(icon);
        self
    }

    pub fn on_tap(mut self, msg: M) -> Self {
        self.on_tap = Some(msg);
        self
    }
}

impl<M: Clone + 'static> From<MobileListItemBuilder<M>> for WidgetNode<M> {
    fn from(b: MobileListItemBuilder<M>) -> Self {
        let mut text_col = crate::column::<M>()
            .key(format!("{}__text", b.id.as_str()))
            .min_w(Length::px(0.0))
            .flex_1()
            .flex_shrink(1.0)
            .gap(2.0)
            .child(
                crate::label_builder(b.title)
                    .wrap(crate::LabelWrap::Character)
                    .build(),
            );
        if let Some(d) = b.detail {
            text_col = text_col.child(
                crate::label_builder(d)
                    .wrap(crate::LabelWrap::Character)
                    .build(),
            );
        }

        let mut row = crate::row::<M>()
            .key(b.id)
            .w(Length::Percent(1.0))
            .min_h(Length::px(44.0))
            .bg_surface()
            .rounded(8.0)
            .gap(12.0)
            .padding(14.0)
            .child(text_col.build());

        if let Some(icon_name) = b.trailing_icon {
            row = row.child(
                crate::label_builder(icon_name.char())
                    .font_size(16.0)
                    .w(Length::px(20.0))
                    .flex_shrink(0.0)
                    .build(),
            );
        }
        if let Some(msg) = b.on_tap {
            row = row.on_click(msg);
        }
        row.build()
    }
}

#[cfg(test)]
#[path = "mobile_list_item_tests.rs"]
mod tests;
