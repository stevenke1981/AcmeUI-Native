//! Foundations category page builders.

use acme_style::FontWeight;
use acme_style::ShadowDef;
use acme_style::prelude::ColorToken;
use acme_style::prelude::Styled;
use acme_widgets::{WidgetNode, card, column, label, label_builder, row};

use crate::types::*;

impl crate::Gallery {
    pub fn foundations_page(&self) -> WidgetNode<GalleryMessage> {
        match self.selected_page {
            5 => self.style_page(),
            _ => {
                let name = crate::types::CATEGORIES[0].pages[self.selected_page.min(5)];
                self.component_page(name)
            }
        }
    }

    /// Style / Tailwind demo page — shows off the new Styled trait + utility API.
    pub fn style_page(&self) -> WidgetNode<GalleryMessage> {
        use acme_layout::Length as L;

        // ── Background colour cards ──
        let bg_cards = row()
            .gap(12.0)
            .child(
                card()
                    .w(L::Px(80.0))
                    .h(L::Px(60.0))
                    .bg(ColorToken::Primary)
                    .rounded_md()
                    .child(label_builder("Primary").text_foreground().build()),
            )
            .child(
                card()
                    .w(L::Px(80.0))
                    .h(L::Px(60.0))
                    .bg(ColorToken::Surface)
                    .rounded_md()
                    .child(label("Surface")),
            )
            .child(
                card()
                    .w(L::Px(80.0))
                    .h(L::Px(60.0))
                    .bg(ColorToken::Muted)
                    .rounded_md()
                    .child(label("Muted")),
            )
            .child(
                card()
                    .w(L::Px(80.0))
                    .h(L::Px(60.0))
                    .bg(ColorToken::Accent)
                    .rounded_md()
                    .child(label_builder("Accent").text_foreground().build()),
            )
            .child(
                card()
                    .w(L::Px(80.0))
                    .h(L::Px(60.0))
                    .bg(ColorToken::Success)
                    .rounded_md()
                    .child(label_builder("Success").text_foreground().build()),
            )
            .child(
                card()
                    .w(L::Px(80.0))
                    .h(L::Px(60.0))
                    .bg(ColorToken::Warning)
                    .rounded_md()
                    .child(label("Warning")),
            )
            .child(
                card()
                    .w(L::Px(80.0))
                    .h(L::Px(60.0))
                    .bg(ColorToken::Danger)
                    .rounded_md()
                    .child(label_builder("Danger").text_foreground().build()),
            )
            .build();

        // ── Border radius demo ──
        let radius_demo = row()
            .gap(8.0)
            .child(
                card()
                    .w(L::Px(50.0))
                    .h(L::Px(50.0))
                    .bg(ColorToken::Surface)
                    .rounded_none()
                    .child(label("0")),
            )
            .child(
                card()
                    .w(L::Px(50.0))
                    .h(L::Px(50.0))
                    .bg(ColorToken::Surface)
                    .rounded_sm()
                    .child(label("sm")),
            )
            .child(
                card()
                    .w(L::Px(50.0))
                    .h(L::Px(50.0))
                    .bg(ColorToken::Surface)
                    .rounded_md()
                    .child(label("md")),
            )
            .child(
                card()
                    .w(L::Px(50.0))
                    .h(L::Px(50.0))
                    .bg(ColorToken::Surface)
                    .rounded_lg()
                    .child(label("lg")),
            )
            .child(
                card()
                    .w(L::Px(50.0))
                    .h(L::Px(50.0))
                    .bg(ColorToken::Surface)
                    .rounded_xl()
                    .child(label("xl")),
            )
            .child(
                card()
                    .w(L::Px(50.0))
                    .h(L::Px(50.0))
                    .bg(ColorToken::Surface)
                    .rounded_full()
                    .child(label("full")),
            )
            .build();

        // ── Shadow demo ──
        let mk_shadow = |ox, oy, blur, a| {
            ShadowDef::new(
                ox,
                oy,
                blur,
                ColorToken::Direct(acme_theme::ThemeColor::rgba(0.0, 0.0, 0.0, a)),
            )
        };
        let shadow_demo = row()
            .gap(16.0)
            .child(
                card()
                    .w(L::Px(100.0))
                    .h(L::Px(60.0))
                    .bg(ColorToken::Surface)
                    .rounded_md()
                    .shadow(mk_shadow(0.0, 1.0, 2.0, 0.1))
                    .child(label("sm")),
            )
            .child(
                card()
                    .w(L::Px(100.0))
                    .h(L::Px(60.0))
                    .bg(ColorToken::Surface)
                    .rounded_md()
                    .shadow(mk_shadow(0.0, 4.0, 6.0, 0.15))
                    .child(label("md")),
            )
            .child(
                card()
                    .w(L::Px(100.0))
                    .h(L::Px(60.0))
                    .bg(ColorToken::Surface)
                    .rounded_md()
                    .shadow(mk_shadow(0.0, 10.0, 15.0, 0.2))
                    .child(label("lg")),
            )
            .build();

        // ── Combined card ──
        let combined = row()
            .gap(16.0)
            .child(
                card()
                    .w(L::Px(160.0))
                    .p(16.0)
                    .bg(ColorToken::Surface)
                    .rounded_lg()
                    .shadow(mk_shadow(0.0, 4.0, 12.0, 0.12))
                    .child(
                        column()
                            .gap(8.0)
                            .child(
                                label_builder("Card Title")
                                    .font_size(16.0)
                                    .text_primary()
                                    .build(),
                            )
                            .child(
                                label_builder(
                                    "This card uses .bg_surface() .rounded_lg() .shadow() .p(16)",
                                )
                                .font_size(13.0)
                                .build(),
                            )
                            .build(),
                    ),
            )
            .child(
                card()
                    .w(L::Px(160.0))
                    .p(16.0)
                    .bg(ColorToken::Primary)
                    .rounded_lg()
                    .shadow(mk_shadow(0.0, 4.0, 12.0, 0.15))
                    .child(
                        column()
                            .gap(8.0)
                            .child(
                                label_builder("Accent Card")
                                    .font_size(16.0)
                                    .text_foreground()
                                    .font_weight(FontWeight::Bold)
                                    .build(),
                            )
                            .child(
                                label_builder("With .bg_primary() .text_foreground()")
                                    .font_size(13.0)
                                    .text_foreground()
                                    .build(),
                            )
                            .build(),
                    ),
            )
            .build();

        column()
            .gap(20.0)
            .padding(16.0)
            .child(label_builder("Style & Tailwind API").font_size(22.0).font_weight(FontWeight::Bold).build())
            .child(label_builder("Background colour tokens — .bg_primary() .bg_surface() .bg_muted() .bg_accent() .bg_success() .bg_warning() .bg_danger()").font_size(13.0).build())
            .child(bg_cards)
            .child(label_builder("Border radius — .rounded_none() .rounded_sm() .rounded_md() .rounded_lg() .rounded_xl() .rounded_full()").font_size(13.0).build())
            .child(radius_demo)
            .child(label_builder("Drop shadows — .shadow(ShadowDef)").font_size(13.0).build())
            .child(shadow_demo)
            .child(label_builder("Combined — chaining utility methods").font_size(13.0).build())
            .child(combined)
            .build()
    }
}
