//! Navigation category page builders.

use acme_widgets::{
    WidgetNode, breadcrumb, button, column, label, nav_item, nav_rail,
    sidebar, tab_bar,
};
use acme_widgets::TabItem;

use crate::helpers::{density_demo, long_text_section, spacing};
use crate::types::*;

impl crate::Gallery {
    pub fn navigation_page(&self) -> WidgetNode<GalleryMessage> {
        match self.selected_page.min(3) {
            0 => self.nav_rail_page(),
            1 => self.sidebar_widget_page(),
            2 => self.tab_bar_page(),
            _ => self.breadcrumb_page(),
        }
    }

    pub fn nav_rail_page(&self) -> WidgetNode<GalleryMessage> {
        let g = spacing(self.density, 8.0);
        let sel = self.nav_rail_selected;
        let names = ["Home", "Search", "Library", "Settings"];
        let sel_name = names.get(sel).copied().unwrap_or("?");
        let expanded = nav_rail::<GalleryMessage>("demo_rail")
            .item(
                nav_item("Home")
                    .icon("⌂")
                    .on_click(GalleryMessage::NavRailSelect(0)),
            )
            .item(
                nav_item("Search")
                    .icon("⌕")
                    .on_click(GalleryMessage::NavRailSelect(1)),
            )
            .item(
                nav_item("Library")
                    .icon("☰")
                    .on_click(GalleryMessage::NavRailSelect(2)),
            )
            .item(nav_item("Settings").icon("⚙").disabled(true))
            .selected(sel)
            .collapsed(false)
            .build();
        let collapsed = nav_rail::<GalleryMessage>("demo_rail_c")
            .item(
                nav_item("Home")
                    .icon("⌂")
                    .on_click(GalleryMessage::NavRailSelect(0)),
            )
            .item(
                nav_item("Search")
                    .icon("⌕")
                    .on_click(GalleryMessage::NavRailSelect(1)),
            )
            .item(
                nav_item("Library")
                    .icon("☰")
                    .on_click(GalleryMessage::NavRailSelect(2)),
            )
            .selected(sel.min(2))
            .collapsed(true)
            .build();
        let secs = vec![
            (
                "Anatomy",
                column()
                    .gap(8.0)
                    .child(label("NavRail — vertical destinations"))
                    .child(label("  key, items[], selected, collapsed"))
                    .child(label("  item: label + optional icon / message / disabled"))
                    .build(),
            ),
            (
                "Expanded",
                column()
                    .gap(g)
                    .child(label(format!("Selected: {sel_name} (click items)")))
                    .child(expanded)
                    .build(),
            ),
            (
                "Collapsed",
                column()
                    .gap(g)
                    .child(label("Icons / short labels only — shares selection state"))
                    .child(collapsed)
                    .build(),
            ),
            ("Density", density_demo()),
            ("Long Traditional Chinese Text", long_text_section()),
        ];
        self.build_component_page("NavRail", secs)
    }

    pub fn sidebar_widget_page(&self) -> WidgetNode<GalleryMessage> {
        let g = spacing(self.density, 8.0);
        let demo = sidebar::<GalleryMessage>("demo_sidebar")
            .width(224.0)
            .header("Explorer")
            .child(button("sb_files", "Files").on_click(GalleryMessage::DpiInfo))
            .child(button("sb_search", "Search").on_click(GalleryMessage::DpiInfo))
            .child(label("— Recent —"))
            .child(label("readme.md"))
            .child(label("main.rs"))
            .build();
        let secs = vec![
            (
                "Anatomy",
                column()
                    .gap(8.0)
                    .child(label("Sidebar — fixed-width panel"))
                    .child(label("  key, width (default 224), header, children[]"))
                    .build(),
            ),
            (
                "Demo",
                column()
                    .gap(g)
                    .child(label("width = 224px, header + body"))
                    .child(demo)
                    .build(),
            ),
            ("Density", density_demo()),
            ("Long Traditional Chinese Text", long_text_section()),
        ];
        self.build_component_page("Sidebar", secs)
    }

    pub fn tab_bar_page(&self) -> WidgetNode<GalleryMessage> {
        let g = spacing(self.density, 8.0);
        let primary_labels = ["Overview", "Details", "History", "Settings"];
        let zh_labels = ["日", "週", "月"];
        let mut tabs = tab_bar::<GalleryMessage>("demo_tabs").selected(self.tab_bar_selected);
        for (i, label_text) in primary_labels.iter().enumerate() {
            tabs = tabs.item(TabItem::new(*label_text).on_click(GalleryMessage::TabBarSelect(i)));
        }
        let tabs = tabs.build();
        let mut tabs_sel =
            tab_bar::<GalleryMessage>("demo_tabs_2").selected(self.tab_bar_zh_selected);
        for (i, label_text) in zh_labels.iter().enumerate() {
            tabs_sel = tabs_sel
                .item(TabItem::new(*label_text).on_click(GalleryMessage::TabBarZhSelect(i)));
        }
        let tabs_sel = tabs_sel.build();
        let primary_name = primary_labels
            .get(self.tab_bar_selected)
            .copied()
            .unwrap_or("?");
        let zh_name = zh_labels
            .get(self.tab_bar_zh_selected)
            .copied()
            .unwrap_or("?");
        let secs = vec![
            (
                "Anatomy",
                column()
                    .gap(8.0)
                    .child(label("TabBar — horizontal tab strip"))
                    .child(label("  key, tabs[], selected index"))
                    .child(label("  selected tab rendered as [Label]"))
                    .build(),
            ),
            (
                "Demo",
                column()
                    .gap(g)
                    .child(label(format!("selected = {primary_name} (click tabs)")))
                    .child(tabs)
                    .child(label(format!("selected = {zh_name}")))
                    .child(tabs_sel)
                    .build(),
            ),
            ("Density", density_demo()),
            ("Long Traditional Chinese Text", long_text_section()),
        ];
        self.build_component_page("TabBar", secs)
    }

    pub fn breadcrumb_page(&self) -> WidgetNode<GalleryMessage> {
        let g = spacing(self.density, 8.0);
        let trail = breadcrumb::<GalleryMessage>("demo_bc")
            .segment("Home")
            .segment("Library")
            .segment("Data")
            .segment("表單")
            .build();
        let trail_gt = breadcrumb::<GalleryMessage>("demo_bc_gt")
            .separator(">")
            .segment("Root")
            .segment("src")
            .segment("main.rs")
            .build();
        let secs = vec![
            (
                "Anatomy",
                column()
                    .gap(8.0)
                    .child(label("Breadcrumb — path trail with separators"))
                    .child(label("  key, segments[], separator (default \"/\")"))
                    .build(),
            ),
            (
                "Demo",
                column()
                    .gap(g)
                    .child(label("separator = /"))
                    .child(trail)
                    .child(label("separator = >"))
                    .child(trail_gt)
                    .build(),
            ),
            ("Density", density_demo()),
            ("Long Traditional Chinese Text", long_text_section()),
        ];
        self.build_component_page("Breadcrumb", secs)
    }
}
