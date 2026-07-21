//! Patterns category page builders (Settings, Dashboard, IDE Layout, SpeakType).

use acme_widgets::{
    ButtonVariant, WidgetNode, button, column, label, label_with_size, row, separator,
};

use crate::helpers::spacing;
use crate::types::*;

impl crate::Gallery {
    pub fn patterns_page(&self) -> WidgetNode<GalleryMessage> {
        match self.selected_page {
            0 => self.settings_page(),
            1 => self.dashboard_page(),
            2 => self.ide_layout_page(),
            3 => self.speaktype_page(),
            _ => label("Unknown template"),
        }
    }

    /// Settings: sidebar-style with form sections and a danger zone.
    pub fn settings_page(&self) -> WidgetNode<GalleryMessage> {
        column()
            .gap(spacing(self.density, 16.0))
            .padding(spacing(self.density, 24.0))
            .child(label_with_size("Settings", 24.0))
            .child(label("Configure your application preferences"))
            .child(separator())
            .child(
                column()
                    .gap(8.0)
                    .child(label_with_size("General", 18.0))
                    .child(crate::helpers::sf("Username", "eda"))
                    .child(crate::helpers::sf("Language", "繁體中文"))
                    .child(crate::helpers::sf("Theme", "System"))
                    .build(),
            )
            .child(separator())
            .child(
                column()
                    .gap(8.0)
                    .child(label_with_size("Notifications", 18.0))
                    .child(label("☐  Email notifications"))
                    .child(label("☐  Push notifications"))
                    .child(label("☑  Weekly digest"))
                    .build(),
            )
            .child(separator())
            .child(
                column()
                    .gap(8.0)
                    .padding(16.0)
                    .child(label_with_size("Danger Zone", 18.0))
                    .child(label("These actions cannot be undone."))
                    .child(
                        button("delete_account", "Delete Account")
                            .variant(ButtonVariant::Danger)
                            .on_click(GalleryMessage::DpiInfo),
                    )
                    .build(),
            )
            .build()
    }

    /// Dashboard: KPI row, insight card, and activity list.
    pub fn dashboard_page(&self) -> WidgetNode<GalleryMessage> {
        column()
            .gap(spacing(self.density, 16.0))
            .padding(spacing(self.density, 24.0))
            .child(
                row()
                    .gap(16.0)
                    .child(label_with_size("Dashboard", 24.0))
                    .child(button("refresh_btn", "↻ Refresh").on_click(GalleryMessage::DpiInfo))
                    .build(),
            )
            .child(
                row()
                    .gap(12.0)
                    .child(crate::helpers::kpi_card("$48,290", "Revenue"))
                    .child(crate::helpers::kpi_card("2,847", "Users"))
                    .child(crate::helpers::kpi_card("1,203", "Active"))
                    .child(crate::helpers::kpi_card("+12.5%", "Growth"))
                    .build(),
            )
            .child(
                column()
                    .gap(8.0)
                    .child(label_with_size("Revenue Overview", 16.0))
                    .child(label(
                        "[ Chart placeholder — area chart would render here ]",
                    ))
                    .build(),
            )
            .child(
                column()
                    .gap(6.0)
                    .child(label_with_size("Recent Activity", 16.0))
                    .child(label("•  New user registered — 2m ago"))
                    .child(label("•  Order #3842 completed — 15m ago"))
                    .child(label("•  Server deployment finished — 1h ago"))
                    .child(label("•  Payment received — 2h ago"))
                    .build(),
            )
            .build()
    }

    /// Desktop IDE: menu bar, nav rail, file tree, editor, terminal, status bar.
    pub fn ide_layout_page(&self) -> WidgetNode<GalleryMessage> {
        column()
            .gap(0.0)
            .child(
                row()
                    .gap(12.0)
                    .padding(8.0)
                    .child(label("File"))
                    .child(label("Edit"))
                    .child(label("View"))
                    .child(label("Help"))
                    .build(),
            )
            .child(separator())
            .child(
                column()
                    .gap(0.0)
                    .child(
                        row()
                            .gap(0.0)
                            .child(self.ide_nav_rail())
                            .child(self.ide_file_tree())
                            .child(self.ide_editor())
                            .build(),
                    )
                    .child(self.ide_terminal())
                    .build(),
            )
            .child(separator())
            .child(
                row()
                    .gap(16.0)
                    .padding(6.0)
                    .child(label("Ln 42, Col 8"))
                    .child(label("Rust"))
                    .child(label("main"))
                    .child(label("UTF-8"))
                    .build(),
            )
            .build()
    }

    pub fn ide_nav_rail(&self) -> WidgetNode<GalleryMessage> {
        column()
            .gap(8.0)
            .padding(8.0)
            .child(label("📁"))
            .child(label("🔍"))
            .child(label("🔧"))
            .child(label("📦"))
            .child(label("🧪"))
            .build()
    }

    pub fn ide_file_tree(&self) -> WidgetNode<GalleryMessage> {
        column()
            .gap(4.0)
            .padding(8.0)
            .child(label("src/"))
            .child(label("  ├─ main.rs"))
            .child(label("  ├─ lib.rs"))
            .child(label("  └─ render/"))
            .child(label("Cargo.toml"))
            .child(label("README.md"))
            .build()
    }

    pub fn ide_editor(&self) -> WidgetNode<GalleryMessage> {
        column()
            .gap(4.0)
            .padding(12.0)
            .child(label_with_size("fn main() {", 14.0))
            .child(label_with_size("    let msg = \"Hello, AcmeUI!\";", 14.0))
            .child(label_with_size("    println!(\"{}\", msg);", 14.0))
            .child(label_with_size("}", 14.0))
            .build()
    }

    pub fn ide_terminal(&self) -> WidgetNode<GalleryMessage> {
        column()
            .gap(2.0)
            .padding(6.0)
            .child(label("$ cargo build --release"))
            .child(label("   Compiling acme-core v0.1.0"))
            .child(label("   Compiling acme-widgets v0.1.0"))
            .child(label("    Finished release [optimized] target"))
            .build()
    }

    /// SpeakType: recording status, provider dots, transcript, big record button.
    pub fn speaktype_page(&self) -> WidgetNode<GalleryMessage> {
        column()
            .gap(spacing(self.density, 16.0))
            .padding(spacing(self.density, 24.0))
            .child(
                row()
                    .gap(16.0)
                    .child(label("⚙ Settings"))
                    .child(label("📊 History"))
                    .child(label("📝 Notes"))
                    .build(),
            )
            .child(separator())
            .child(
                row()
                    .gap(8.0)
                    .child(label("🔴 Recording"))
                    .child(label("00:42"))
                    .build(),
            )
            .child(
                row()
                    .gap(16.0)
                    .child(label("🟢 OpenAI"))
                    .child(label("🟡 Anthropic"))
                    .child(label("⚪ Local"))
                    .build(),
            )
            .child(label("⌘⇧R  Start/Stop  ·  ⌘⇧S  Save transcript"))
            .child(
                column()
                    .gap(4.0)
                    .child(label_with_size("Recent Transcript", 16.0))
                    .child(label(
                        "Hello, this is a test of the speech recognition system.",
                    ))
                    .child(label("The quick brown fox jumps over the lazy dog."))
                    .child(label("今日的天氣真好，適合出門散步。"))
                    .build(),
            )
            .child(
                button("record_btn", "⏺ Start Recording")
                    .primary()
                    .on_click(GalleryMessage::DpiInfo),
            )
            .build()
    }
}
