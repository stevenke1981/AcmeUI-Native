//! Clipboard integration for AcmeUI Native.
#![forbid(unsafe_op_in_unsafe_fn)]

use arboard::Clipboard as ArboardClipboard;

/// A thread-safe wrapper around the system clipboard.
pub struct Clipboard {
    inner: std::sync::Mutex<ArboardClipboard>,
}

impl Clipboard {
    /// Create a new clipboard connection.
    pub fn new() -> Result<Self, String> {
        ArboardClipboard::new()
            .map(|inner| Self {
                inner: std::sync::Mutex::new(inner),
            })
            .map_err(|e| e.to_string())
    }

    /// Get the current text from the clipboard.
    pub fn get_text(&self) -> Result<String, String> {
        self.inner
            .lock()
            .map_err(|e| e.to_string())?
            .get_text()
            .map_err(|e| e.to_string())
    }

    /// Set the clipboard text.
    pub fn set_text(&self, text: &str) -> Result<(), String> {
        self.inner
            .lock()
            .map_err(|e| e.to_string())?
            .set_text(text)
            .map_err(|e| e.to_string())
    }

    /// Check if clipboard is available
    pub fn is_available() -> bool {
        ArboardClipboard::new().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipboard_new_succeeds_or_fails_gracefully() {
        let result = Clipboard::new();
        match result {
            Ok(_) => { /* clipboard available — construction alone is sufficient */ }
            Err(err) => assert!(!err.is_empty(), "error message must not be empty"),
        }
    }

    #[test]
    fn clipboard_set_get_roundtrip() {
        if !Clipboard::is_available() {
            eprintln!("skipping roundtrip: clipboard not available on this platform");
            return;
        }
        let clipboard = Clipboard::new().expect("clipboard should be available");
        let original = "AcmeUI Native clipboard test";
        clipboard
            .set_text(original)
            .expect("set_text should succeed");
        let result = clipboard.get_text().expect("get_text should succeed");
        assert_eq!(result, original);
    }
}
