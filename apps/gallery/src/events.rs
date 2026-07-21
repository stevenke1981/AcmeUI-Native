//! Event handling methods for the Gallery struct.

use acme_platform::PlatformKey;
use acme_text::TextStyle;
use acme_theme::Theme;

use crate::render::{point_in_rect, scrolled_hit_rect};
use crate::types::*;

impl crate::Gallery {
    /// Reverse-iterate hit regions to find the top-most match under cursor.
    pub fn hit(&self) -> Option<usize> {
        self.button_info
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, hr)| {
                let r = if hr.scrolled {
                    if self.scroll_clip_rect[2] > 0.0
                        && !point_in_rect(self.cursor.0, self.cursor.1, self.scroll_clip_rect)
                    {
                        return None;
                    }
                    scrolled_hit_rect(hr.rect, self.scroll)
                } else {
                    hr.rect
                };
                if point_in_rect(self.cursor.0, self.cursor.1, r) {
                    Some(i)
                } else {
                    None
                }
            })
    }

    /// Dispatch a message from the given hit-region index.
    pub fn activate(&mut self, index: usize) -> bool {
        let Some(hr) = self.button_info.get(index) else {
            return false;
        };
        let message = hr.message;
        match message {
            GalleryMessage::ToggleTheme => {
                self.dark = !self.dark;
                true
            }
            GalleryMessage::ToggleDensity => {
                self.density = self.density.toggle();
                true
            }
            GalleryMessage::ToggleFocusRings => {
                self.show_focus_rings = !self.show_focus_rings;
                true
            }
            GalleryMessage::SelectCategory(i) => {
                let changed = self.selected_category != i;
                self.selected_category = i;
                self.selected_page = 0;
                if changed {
                    self.scroll = 0.0;
                }
                true
            }
            GalleryMessage::SelectPage(i) => {
                self.selected_page = i;
                self.scroll = 0.0;
                true
            }
            GalleryMessage::NavRailSelect(i) => {
                self.nav_rail_selected = i;
                true
            }
            GalleryMessage::TabBarSelect(i) => {
                self.tab_bar_selected = i;
                true
            }
            GalleryMessage::TabBarZhSelect(i) => {
                self.tab_bar_zh_selected = i;
                true
            }
            GalleryMessage::TreeSelectKey(key) => {
                if self.tree_selected == Some(key) {
                    self.tree_toggle_expanded(key);
                }
                self.tree_selected = Some(key);
                true
            }
            GalleryMessage::TreeToggleKey(key) => {
                self.tree_toggle_expanded(key);
                self.tree_selected = Some(key);
                true
            }
            GalleryMessage::TableSort(col) => {
                if self.table_sort_col == Some(col) {
                    self.table_sort_asc = !self.table_sort_asc;
                } else {
                    self.table_sort_col = Some(col);
                    self.table_sort_asc = true;
                }
                true
            }
            GalleryMessage::TableSelectRow(orig) => {
                self.table_selected_row = Some(orig);
                true
            }
            GalleryMessage::FocusDemo | GalleryMessage::DpiInfo => true,
        }
    }

    /// Recompute window-client IME caret rect from field origin + content-local
    /// caret geometry. Called after focus/text changes and each text-input frame.
    pub fn refresh_ime_caret_cache(&mut self) {
        if !self.text_input.focused {
            self.ime_caret_window_rect = None;
            return;
        }
        let [fx, fy, fw, fh] = self.text_input_rect;
        if fw <= 0.0 || fh <= 0.0 {
            self.ime_caret_window_rect = None;
            return;
        }
        let theme = if self.dark {
            Theme::dark()
        } else {
            Theme::light()
        };
        let font_size = theme.typography.body;
        let style = TextStyle {
            font_size,
            line_height: font_size * theme.typography.line_height,
            ..TextStyle::default()
        };
        let padding = theme.spacing.px2;
        let [cx, cy, cw, ch] =
            self.text_input
                .ime_caret_area(&mut self.fonts, &style, self.last_scale_factor);
        self.ime_caret_window_rect = Some([fx + padding + cx, fy + padding + cy, cw, ch.max(1.0)]);
    }
}
