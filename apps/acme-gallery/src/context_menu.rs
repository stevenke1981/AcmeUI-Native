//! Context menu demo for the Gallery overlay page.
use super::*;
use acme_ui::overlay::context_menu;
use acme_widgets::menu_item;

impl Gallery {
    pub(super) fn context_menu_examples(&self) -> WidgetNode<GalleryMessage> {
        let content: WidgetNode<GalleryMessage> = if self.context_menu_open {
            context_menu("gallery_context_menu")
                .anchor(label("Right-click area · 右鍵區域"))
                .item(menu_item("cut", "Cut · 剪下").on_click(GalleryMessage::ContextMenuCut))
                .item(menu_item("copy", "Copy · 複製").on_click(GalleryMessage::ContextMenuCopy))
                .separator()
                .item(
                    menu_item("lock", "Locked · 鎖定")
                        .disabled(true)
                        .on_click(GalleryMessage::ContextMenuCut),
                )
                .open(true)
                .into()
        } else {
            button(
                "gallery_context_menu_show",
                "Show context menu · 顯示操作選單",
            )
            .on_click(GalleryMessage::ContextMenuShow)
        };
        column()
            .key("gallery_context_menu_demo")
            .gap(8.0)
            .child(content)
            .child(label(format!(
                "Context menu Cut: {}, Copy: {}",
                self.context_menu_cuts, self.context_menu_copies
            )))
            .build()
    }
}
