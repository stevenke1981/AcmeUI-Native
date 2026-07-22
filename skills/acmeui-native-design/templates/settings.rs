//! Settings and preferences starter.

use acme_ui::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum SettingsMessage {
    SelectGeneral,
    SelectPrivacy,
    SelectShortcuts,
    ToggleLaunchAtLogin,
    Save,
    Reset,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SettingsState {
    pub selected_section: String,
    pub launch_at_login: bool,
    pub is_dirty: bool,
    pub save_status: String,
}

pub fn settings_view(state: &SettingsState) -> WidgetNode<SettingsMessage> {
    default_template("Settings")
        .subtitle("Preferences are grouped by task, not by component type")
        .child(
            row::<SettingsMessage>()
                .key("settings-shell")
                .gap(20.0)
                .child(
                    column::<SettingsMessage>()
                        .key("settings-navigation")
                        .gap(8.0)
                        .child(button("general", "General").on_click(SettingsMessage::SelectGeneral))
                        .child(button("privacy", "Privacy").on_click(SettingsMessage::SelectPrivacy))
                        .child(button("shortcuts", "Shortcuts").on_click(SettingsMessage::SelectShortcuts))
                        .build(),
                )
                .child(
                    column::<SettingsMessage>()
                        .key("settings-content")
                        .gap(12.0)
                        .child(label(format!("{} settings", state.selected_section)))
                        .child(separator())
                        .child(label(format!(
                            "Launch at login: {}",
                            if state.launch_at_login { "On" } else { "Off" }
                        )))
                        .child(
                            button("toggle-launch", "Toggle launch at login")
                                .on_click(SettingsMessage::ToggleLaunchAtLogin),
                        )
                        .child(label(state.save_status.as_str()))
                        .child(
                            row::<SettingsMessage>()
                                .gap(8.0)
                                .child(button("reset", "Reset").on_click(SettingsMessage::Reset))
                                .child(
                                    button(
                                        "save",
                                        if state.is_dirty { "Save changes" } else { "Saved" },
                                    )
                                    .primary()
                                    .disabled(!state.is_dirty)
                                    .on_click(SettingsMessage::Save),
                                )
                                .build(),
                        )
                        .build(),
                )
                .build(),
        )
        .build()
}
