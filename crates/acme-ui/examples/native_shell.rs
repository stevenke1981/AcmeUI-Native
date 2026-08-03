use acme_ui::prelude::*;

#[derive(Clone, Debug, PartialEq)]
enum AppMessage {
    Save,
}

fn main() {
    let profile = native_profile();
    let _theme = profile.theme(ThemeMode::Light);
    let _layout_context = profile.layout_context(1.0);

    let view = native_template("Settings")
        .subtitle(format!(
            "{} · Save with {}",
            profile.platform.display_name(),
            profile.shortcut_label("S")
        ))
        .child(
            native_button("save", "Save")
                .primary()
                .on_click(AppMessage::Save),
        )
        .build();

    assert_eq!(
        view.key().expect("native template key").as_str(),
        "acmeui-native-template"
    );
}
