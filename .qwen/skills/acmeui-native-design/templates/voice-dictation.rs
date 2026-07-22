//! Voice-dictation / Typeless-like product workflow starter.
//! This is not a third-party brand clone. Replace all placeholder copy and wire real services.

use acme_ui::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VoiceTone {
    #[default]
    Auto,
    Concise,
    Professional,
    Casual,
}

#[derive(Clone, Debug, PartialEq)]
pub enum VoiceDictationMessage {
    ToggleRecording,
    CopyTranscript,
    ClearTranscript,
    OpenHistory,
    OpenSettings,
    SelectTone(VoiceTone),
}

#[derive(Clone, Debug, PartialEq)]
pub struct VoiceDictationState {
    pub is_recording: bool,
    pub is_processing: bool,
    pub microphone_allowed: bool,
    pub transcript: String,
    pub language_label: String,
    pub tone: VoiceTone,
}

impl Default for VoiceDictationState {
    fn default() -> Self {
        Self {
            is_recording: false,
            is_processing: false,
            microphone_allowed: true,
            transcript: String::new(),
            language_label: "Auto detect".into(),
            tone: VoiceTone::Auto,
        }
    }
}

pub fn voice_dictation_view(
    state: &VoiceDictationState,
) -> WidgetNode<VoiceDictationMessage> {
    let status = if !state.microphone_allowed {
        "Microphone permission required"
    } else if state.is_processing {
        "Cleaning and formatting transcript…"
    } else if state.is_recording {
        "Listening… speak naturally"
    } else {
        "Ready"
    };

    let transcript = if state.transcript.trim().is_empty() {
        "Your polished transcript will appear here."
    } else {
        state.transcript.as_str()
    };

    let recording_label = if state.is_recording {
        "Stop recording"
    } else {
        "Start recording"
    };

    apple_template("Voice Dictation")
        .subtitle("Speak naturally, then review clear structured text")
        .child(
            row::<VoiceDictationMessage>()
                .key("voice-app-bar")
                .gap(8.0)
                .child(button("history", "History").on_click(VoiceDictationMessage::OpenHistory))
                .child(button("settings", "Settings").on_click(VoiceDictationMessage::OpenSettings))
                .child(label(format!("{} · {}", state.language_label, status)))
                .build(),
        )
        .child(separator())
        .child(
            column::<VoiceDictationMessage>()
                .key("recording-focus")
                .gap(12.0)
                .child(label(status))
                .child(
                    button("record", recording_label)
                        .primary()
                        .on_click(VoiceDictationMessage::ToggleRecording),
                )
                .build(),
        )
        .child(
            card::<VoiceDictationMessage>()
                .key("transcript-workspace")
                .gap(12.0)
                .padding(20.0)
                .child(label("Transcript"))
                .child(label(transcript))
                .child(
                    row::<VoiceDictationMessage>()
                        .gap(8.0)
                        .child(button("copy", "Copy").on_click(VoiceDictationMessage::CopyTranscript))
                        .child(button("clear", "Clear").on_click(VoiceDictationMessage::ClearTranscript))
                        .build(),
                )
                .build(),
        )
        .child(
            column::<VoiceDictationMessage>()
                .key("tone-controls")
                .gap(8.0)
                .child(label(format!("Writing tone: {:?}", state.tone)))
                .child(
                    row::<VoiceDictationMessage>()
                        .gap(8.0)
                        .child(button("tone-auto", "Auto").on_click(VoiceDictationMessage::SelectTone(VoiceTone::Auto)))
                        .child(button("tone-concise", "Concise").on_click(VoiceDictationMessage::SelectTone(VoiceTone::Concise)))
                        .child(button("tone-pro", "Professional").on_click(VoiceDictationMessage::SelectTone(VoiceTone::Professional)))
                        .child(button("tone-casual", "Casual").on_click(VoiceDictationMessage::SelectTone(VoiceTone::Casual)))
                        .build(),
                )
                .build(),
        )
        .build()
}
