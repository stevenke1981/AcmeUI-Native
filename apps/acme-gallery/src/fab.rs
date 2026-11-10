//! FABs use the existing Button host; viewport anchoring belongs to the app.
use super::*;
use acme_ui::overlay::{FloatButtonShape, FloatButtonSize, float_button};
impl Gallery {
    pub(super) fn fab_examples(&self) -> WidgetNode<GalleryMessage> {
        column()
            .key("gallery_fabs")
            .gap(12.0)
            .child(
                row()
                    .gap(16.0)
                    .child(
                        float_button()
                            .key("gallery_fab_mini")
                            .size(FloatButtonSize::Mini)
                            .icon(acme_ui::IconName::Plus)
                            .on_click(GalleryMessage::FabPress(0)),
                    )
                    .child(
                        float_button()
                            .key("gallery_fab_normal")
                            .icon(acme_ui::IconName::Plus)
                            .on_click(GalleryMessage::FabPress(1)),
                    )
                    .child(
                        float_button()
                            .key("gallery_fab_extended")
                            .size(FloatButtonSize::Extended)
                            .icon(acme_ui::IconName::Plus)
                            .label("Create · 新增")
                            .on_click(GalleryMessage::FabPress(2)),
                    ),
            )
            .child(
                row()
                    .gap(16.0)
                    .child(
                        float_button()
                            .key("gallery_fab_square")
                            .shape(FloatButtonShape::Square)
                            .icon(acme_ui::IconName::Star)
                            .on_click(GalleryMessage::FabPress(3)),
                    )
                    .child(
                        float_button()
                            .key("gallery_fab_passive")
                            .icon(acme_ui::IconName::Info),
                    ),
            )
            .child(label(format!("FAB action: {}", self.fab_status)))
            .build()
    }
}
