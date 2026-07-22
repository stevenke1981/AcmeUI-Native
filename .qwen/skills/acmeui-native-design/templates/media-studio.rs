//! Media/editor workspace starter.

use acme_ui::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum MediaStudioMessage {
    NewProject,
    ImportAsset,
    TogglePlayback,
    Export,
    SelectAsset,
    OpenProperties,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MediaStudioState {
    pub project_name: String,
    pub selected_asset: Option<String>,
    pub is_playing: bool,
    pub is_exporting: bool,
    pub timeline_summary: String,
}

pub fn media_studio_view(state: &MediaStudioState) -> WidgetNode<MediaStudioMessage> {
    ubuntu25_template("Media Studio")
        .subtitle(state.project_name.as_str())
        .child(
            row::<MediaStudioMessage>()
                .key("studio-command-bar")
                .gap(8.0)
                .child(button("new", "New project").on_click(MediaStudioMessage::NewProject))
                .child(button("import", "Import").on_click(MediaStudioMessage::ImportAsset))
                .child(
                    button(
                        "play",
                        if state.is_playing { "Pause" } else { "Play" },
                    )
                    .primary()
                    .on_click(MediaStudioMessage::TogglePlayback),
                )
                .child(
                    button(
                        "export",
                        if state.is_exporting { "Exporting…" } else { "Export" },
                    )
                    .on_click(MediaStudioMessage::Export),
                )
                .build(),
        )
        .child(
            row::<MediaStudioMessage>()
                .key("studio-workspace")
                .gap(12.0)
                .child(
                    card::<MediaStudioMessage>()
                        .key("asset-rail")
                        .gap(8.0)
                        .padding(12.0)
                        .child(label("Assets"))
                        .child(label("Imported files and project media"))
                        .child(button("select-asset", "Select asset").on_click(MediaStudioMessage::SelectAsset))
                        .build(),
                )
                .child(
                    card::<MediaStudioMessage>()
                        .key("preview-canvas")
                        .gap(8.0)
                        .padding(20.0)
                        .child(label("Preview / Canvas"))
                        .child(label(if state.is_playing { "Playing" } else { "Paused" }))
                        .build(),
                )
                .child(
                    card::<MediaStudioMessage>()
                        .key("property-inspector")
                        .gap(8.0)
                        .padding(12.0)
                        .child(label("Inspector"))
                        .child(label(
                            state.selected_asset.as_deref().unwrap_or("Nothing selected"),
                        ))
                        .child(button("properties", "Open properties").on_click(MediaStudioMessage::OpenProperties))
                        .build(),
                )
                .build(),
        )
        .child(
            column::<MediaStudioMessage>()
                .key("timeline")
                .gap(8.0)
                .child(label("Timeline"))
                .child(separator())
                .child(label(state.timeline_summary.as_str()))
                .build(),
        )
        .build()
}
