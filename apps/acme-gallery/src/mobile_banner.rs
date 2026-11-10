use super::*;

impl Gallery {
    pub(super) fn banner_examples(&self) -> WidgetNode<GalleryMessage> {
        let mut examples = column().key("gallery_banners").gap(8.0);
        if self.banner_visible {
            examples = examples.child(
                mobile_banner("Update available · 有可用更新")
                    .key("gallery_banner_info")
                    .action("Retry", GalleryMessage::MobileBannerAction)
                    .on_dismiss(GalleryMessage::MobileBannerDismiss),
            );
        }
        examples
            .child(mobile_banner("Saved · 儲存完成")
                .key("gallery_banner_success").tone(MobileBannerTone::Success))
            .child(column().width(320.0).child(
                mobile_banner("Connection is slow. Please wait while we retry. 連線較慢，請稍候，正在重新嘗試連線。")
                    .key("gallery_banner_warning").tone(MobileBannerTone::Warning)))
            .child(mobile_banner("Unable to sync · 無法同步")
                .key("gallery_banner_danger").tone(MobileBannerTone::Danger))
            .child(row().gap(8.0)
                .child(button("gallery_banner_restore", "Restore banner")
                    .on_click(GalleryMessage::MobileBannerRestore))
                .child(label(format!("Banner action: {}", self.banner_action))))
            .build()
    }
}
