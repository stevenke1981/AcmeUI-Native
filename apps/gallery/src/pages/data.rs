//! Data category page builders (Tree, Table, DataGrid, VirtualList).

use acme_widgets::{
    WidgetNode, WidgetKey, button, column, datagrid, label, label_with_size, row, separator, table,
    tree, virtual_list,
};
use acme_widgets::{
    DataGridColumn, DataGridRow, SortDirection, TableColumn, TableRow, TreeNode,
};

use crate::helpers::{spacing, table_cell_text, table_display_order, table_row_cells, tree_key_static};
use crate::types::*;

impl crate::Gallery {
    pub fn data_page(&self) -> WidgetNode<GalleryMessage> {
        let page = self.selected_page.min(3);
        let title = CATEGORIES[4].pages[page];
        let body = match page {
            0 => self.tree_demo(),
            1 => self.table_demo(),
            2 => self.datagrid_demo(),
            _ => self.virtual_list_demo(),
        };
        column()
            .gap(spacing(self.density, 16.0))
            .padding(spacing(self.density, 24.0))
            .child(label_with_size(title, 24.0))
            .child(self.data_page_tabs(page))
            .child(separator())
            .child(body)
            .build()
    }

    /// In-category page switcher for Data demos.
    pub fn data_page_tabs(&self, active: usize) -> WidgetNode<GalleryMessage> {
        const TAB_KEYS: [&str; 4] = ["data_tab_0", "data_tab_1", "data_tab_2", "data_tab_3"];
        let mut tabs = row::<GalleryMessage>().gap(spacing(self.density, 8.0));
        for (i, name) in CATEGORIES[4].pages.iter().enumerate() {
            let mut btn = button::<GalleryMessage>(TAB_KEYS[i], *name);
            if i == active {
                btn = btn.primary();
            }
            tabs = tabs.child(btn.on_click(GalleryMessage::SelectPage(i)));
        }
        tabs.build()
    }

    pub fn tree_demo(&self) -> WidgetNode<GalleryMessage> {
        let gap = spacing(self.density, 8.0);
        let mut tree_widget = tree::<GalleryMessage>("gallery_tree")
            .indent(20.0)
            .viewport_height(320.0)
            .child(
                TreeNode::new("docs", label("Documents"))
                    .expanded(false)
                    .child(TreeNode::new("docs_readme", label("README.md")))
                    .child(TreeNode::new("docs_guide", label("Getting Started")))
                    .child(
                        TreeNode::new("docs_zh", label("繁體中文說明"))
                            .expanded(false)
                            .child(TreeNode::new("docs_zh_ime", label("IME 輸入注意事項")))
                            .child(TreeNode::new("docs_zh_a11y", label("無障礙指南"))),
                    ),
            )
            .child(
                TreeNode::new("images", label("Images"))
                    .expanded(false)
                    .child(TreeNode::new("img_logo", label("logo.png")))
                    .child(TreeNode::new("img_banner", label("banner.webp"))),
            )
            .child(
                TreeNode::new("code", label("Code"))
                    .expanded(false)
                    .child(
                        TreeNode::new("code_src", label("src/"))
                            .expanded(false)
                            .child(TreeNode::new("code_main", label("main.rs")))
                            .child(TreeNode::new("code_lib", label("lib.rs"))),
                    )
                    .child(TreeNode::new("code_toml", label("Cargo.toml"))),
            );
        for (i, &key) in TREE_EXPAND_KEYS.iter().enumerate() {
            if self.tree_expanded & (1u32 << i) != 0 {
                tree_widget.expanded.insert(WidgetKey::from(key));
            }
        }
        if let Some(sel) = self.tree_selected {
            tree_widget.selected = Some(WidgetKey::from(sel));
        }
        let sel_label = self.tree_selected.unwrap_or("(none)");
        column()
            .gap(gap)
            .child(label(
                "Hierarchical Tree with expand/collapse. Nested categories demo:",
            ))
            .child(label(
                "Click row to select · click again (or chevron zone) to toggle · ←/→ collapse/expand",
            ))
            .child(label(format!("Selected: {sel_label}")))
            .child(tree_widget.build())
            .child(label(
                "State lives on Gallery; Tree rebuilds each frame from expand bits + selection.",
            ))
            .build()
    }

    pub fn table_demo(&self) -> WidgetNode<GalleryMessage> {
        let gap = spacing(self.density, 8.0);
        let headers = ["Name", "Status", "Owner", "Updated"];
        let widths = [160.0_f32, 100.0, 100.0, 120.0];
        let mut tbl = table::<GalleryMessage>("gallery_table")
            .sticky_header(true)
            .row_height(28.0);

        for (ci, (header, width)) in headers.iter().zip(widths.iter()).enumerate() {
            let mut title = (*header).to_string();
            if self.table_sort_col == Some(ci) {
                title.push_str(if self.table_sort_asc { " ↑" } else { " ↓" });
            }
            let mut col = TableColumn::new(
                ["name", "status", "owner", "updated"][ci],
                label(title),
                *width,
            )
            .sortable(true)
            .resizable(true);
            if self.table_sort_col == Some(ci) {
                col = col.sort_indicator(if self.table_sort_asc {
                    SortDirection::Ascending
                } else {
                    SortDirection::Descending
                });
            }
            tbl = tbl.column(col);
        }

        let order = table_display_order(self.table_sort_col, self.table_sort_asc);
        let display_selected = self
            .table_selected_row
            .and_then(|orig| order.iter().position(|&o| o == orig));

        for &orig in &order {
            let cells = table_row_cells(orig);
            tbl = tbl.add_row(TableRow::new(vec![
                label(cells[0].clone()),
                label(cells[1].clone()),
                label(cells[2].clone()),
                label(cells[3].clone()),
            ]));
        }

        let mut node = tbl.build();
        if let WidgetNode::Table(ref mut t) = node {
            t.sort_column = self.table_sort_col;
            t.sort_ascending = self.table_sort_asc;
            t.selected_row = display_selected;
        }

        let sort_info = match self.table_sort_col {
            Some(c) => format!(
                "sort = {} ({})",
                headers[c],
                if self.table_sort_asc { "asc" } else { "desc" }
            ),
            None => "sort = none".into(),
        };
        let sel_info = match self.table_selected_row {
            Some(i) => format!("selected row = Project {i:02}"),
            None => "selected row = none".into(),
        };

        column()
            .gap(gap)
            .child(label(
                "Table with sticky header, 4 columns, and 28 sample rows.",
            ))
            .child(label(
                "Click header to sort · click row to select. State lives on Gallery.",
            ))
            .child(label(format!("{sort_info} · {sel_info}")))
            .child(node)
            .build()
    }

    pub fn datagrid_demo(&self) -> WidgetNode<GalleryMessage> {
        let gap = spacing(self.density, 8.0);
        let mut grid = datagrid::<GalleryMessage>("gallery_datagrid")
            .frozen_cols(1)
            .frozen_rows(0)
            .default_row_height(28.0)
            .default_col_width(120.0)
            .viewport_width(640.0)
            .viewport_height(280.0)
            .column(
                DataGridColumn::new("id", label("ID"), 72.0)
                    .frozen(true)
                    .sortable(true),
            )
            .column(DataGridColumn::new("product", label("Product"), 140.0).sortable(true))
            .column(DataGridColumn::new("region", label("Region"), 100.0))
            .column(DataGridColumn::new("qty", label("Qty"), 72.0))
            .column(DataGridColumn::new("total", label("Total"), 100.0));

        let regions = ["APAC", "EMEA", "AMER", "JP"];
        let products = ["Widget A", "Widget B", "Gadget C", "Module D", "Kit E"];
        for i in 0..12 {
            let id = format!("R{i:03}");
            let product = products[i % products.len()];
            let region = regions[i % regions.len()];
            let qty = format!("{}", 10 + i * 3);
            let total = format!("${}.00", 120 + i * 17);
            grid = grid.add_row(
                DataGridRow::new(vec![
                    label(id),
                    label(product),
                    label(region),
                    label(qty),
                    label(total),
                ])
                .row_number(format!("{}", i + 1)),
            );
        }
        grid = grid.merge_cells(0, 1, 2, 1);

        column()
            .gap(gap)
            .child(label(
                "DataGrid with frozen first column, 5 columns × 12 rows, and one cell merge.",
            ))
            .child(label(
                "Frozen cols stay visible during horizontal scroll; merge is declarative (layout still shows cell slots).",
            ))
            .child(grid.build())
            .build()
    }

    pub fn virtual_list_demo(&self) -> WidgetNode<GalleryMessage> {
        let gap = spacing(self.density, 8.0);

        let mut list = virtual_list::<GalleryMessage>("gallery_vlist")
            .item_height(Some(VLIST_ITEM_HEIGHT))
            .viewport_height(VLIST_VIEWPORT_H)
            .overscan(4)
            .scroll_offset(self.vlist_scroll);

        for i in 0..VLIST_ITEM_COUNT {
            list = list.child(label(format!("Item {i}: 項目內容 demo")));
        }

        column()
            .gap(gap)
            .child(label_with_size("VirtualList", 16.0))
            .child(label(
                "Fixed item height path · only the viewport window (+ overscan) is painted.",
            ))
            .child(label(format!(
                "{} items × {}px · viewport {}px · overscan 4 · scroll {:.0}px",
                VLIST_ITEM_COUNT, VLIST_ITEM_HEIGHT, VLIST_VIEWPORT_H, self.vlist_scroll
            )))
            .child(list.build())
            .child(label(
                "Hover the list and scroll to move VirtualList; scroll outside moves the page.",
            ))
            .build()
    }
}
