use winit::keyboard::{Key, NamedKey};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlatformKey {
    Tab,
    Enter,
    Space,
    Escape,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    ArrowDown,
    PageUp,
    PageDown,
    Insert,
    Backspace,
    Delete,
    Home,
    End,
    Function(u8),
    Other,
}

/// Map a winit [`NamedKey`] to a backend-agnostic [`PlatformKey`].
pub(crate) fn map_named_key(named: NamedKey) -> PlatformKey {
    match named {
        NamedKey::Tab => PlatformKey::Tab,
        NamedKey::Enter => PlatformKey::Enter,
        NamedKey::Space => PlatformKey::Space,
        NamedKey::Escape => PlatformKey::Escape,
        NamedKey::ArrowLeft => PlatformKey::ArrowLeft,
        NamedKey::ArrowRight => PlatformKey::ArrowRight,
        NamedKey::ArrowUp => PlatformKey::ArrowUp,
        NamedKey::ArrowDown => PlatformKey::ArrowDown,
        NamedKey::PageUp => PlatformKey::PageUp,
        NamedKey::PageDown => PlatformKey::PageDown,
        NamedKey::Insert => PlatformKey::Insert,
        NamedKey::Backspace => PlatformKey::Backspace,
        NamedKey::Delete => PlatformKey::Delete,
        NamedKey::Home => PlatformKey::Home,
        NamedKey::End => PlatformKey::End,
        NamedKey::F1 => PlatformKey::Function(1),
        NamedKey::F2 => PlatformKey::Function(2),
        NamedKey::F3 => PlatformKey::Function(3),
        NamedKey::F4 => PlatformKey::Function(4),
        NamedKey::F5 => PlatformKey::Function(5),
        NamedKey::F6 => PlatformKey::Function(6),
        NamedKey::F7 => PlatformKey::Function(7),
        NamedKey::F8 => PlatformKey::Function(8),
        NamedKey::F9 => PlatformKey::Function(9),
        NamedKey::F10 => PlatformKey::Function(10),
        NamedKey::F11 => PlatformKey::Function(11),
        NamedKey::F12 => PlatformKey::Function(12),
        NamedKey::F13 => PlatformKey::Function(13),
        NamedKey::F14 => PlatformKey::Function(14),
        NamedKey::F15 => PlatformKey::Function(15),
        NamedKey::F16 => PlatformKey::Function(16),
        NamedKey::F17 => PlatformKey::Function(17),
        NamedKey::F18 => PlatformKey::Function(18),
        NamedKey::F19 => PlatformKey::Function(19),
        NamedKey::F20 => PlatformKey::Function(20),
        NamedKey::F21 => PlatformKey::Function(21),
        NamedKey::F22 => PlatformKey::Function(22),
        NamedKey::F23 => PlatformKey::Function(23),
        NamedKey::F24 => PlatformKey::Function(24),
        _ => PlatformKey::Other,
    }
}

/// Map a winit logical [`Key`] to a [`PlatformKey`].
pub(crate) fn map_logical_key(key: &Key) -> PlatformKey {
    match key {
        Key::Named(named) => map_named_key(*named),
        _ => PlatformKey::Other,
    }
}

/// Resolve the text payload for a key event: prefer `KeyEvent::text` when winit
/// provides a non-empty string (covering Space and shifted characters), falling
/// back to `Key::Character`.
pub(crate) fn resolve_key_text(event_text: Option<&str>, logical_key: &Key) -> Option<String> {
    if let Some(text) = event_text.filter(|s| !s.is_empty()) {
        return Some(text.to_string());
    }
    resolve_key_character(logical_key)
}

/// Extract the `Key::Character` payload of `logical_key` (unaffected by Ctrl).
pub(crate) fn resolve_key_character(logical_key: &Key) -> Option<String> {
    match logical_key {
        Key::Character(s) => Some(s.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_named_key_preserves_existing_variants() {
        assert_eq!(map_named_key(NamedKey::Tab), PlatformKey::Tab);
        assert_eq!(map_named_key(NamedKey::Enter), PlatformKey::Enter);
        assert_eq!(map_named_key(NamedKey::Space), PlatformKey::Space);
        assert_eq!(map_named_key(NamedKey::Escape), PlatformKey::Escape);
        assert_eq!(map_named_key(NamedKey::ArrowLeft), PlatformKey::ArrowLeft);
        assert_eq!(map_named_key(NamedKey::ArrowRight), PlatformKey::ArrowRight);
        assert_eq!(map_named_key(NamedKey::Backspace), PlatformKey::Backspace);
        assert_eq!(map_named_key(NamedKey::Delete), PlatformKey::Delete);
        assert_eq!(map_named_key(NamedKey::Home), PlatformKey::Home);
        assert_eq!(map_named_key(NamedKey::End), PlatformKey::End);
    }

    #[test]
    fn map_named_key_maps_navigation_and_insert_variants() {
        assert_eq!(map_named_key(NamedKey::ArrowUp), PlatformKey::ArrowUp);
        assert_eq!(map_named_key(NamedKey::ArrowDown), PlatformKey::ArrowDown);
        assert_eq!(map_named_key(NamedKey::PageUp), PlatformKey::PageUp);
        assert_eq!(map_named_key(NamedKey::PageDown), PlatformKey::PageDown);
        assert_eq!(map_named_key(NamedKey::Insert), PlatformKey::Insert);
    }

    #[test]
    fn map_named_key_maps_function_keys_f1_through_f24() {
        let cases = [
            (NamedKey::F1, 1),
            (NamedKey::F2, 2),
            (NamedKey::F3, 3),
            (NamedKey::F4, 4),
            (NamedKey::F5, 5),
            (NamedKey::F6, 6),
            (NamedKey::F7, 7),
            (NamedKey::F8, 8),
            (NamedKey::F9, 9),
            (NamedKey::F10, 10),
            (NamedKey::F11, 11),
            (NamedKey::F12, 12),
            (NamedKey::F13, 13),
            (NamedKey::F14, 14),
            (NamedKey::F15, 15),
            (NamedKey::F16, 16),
            (NamedKey::F17, 17),
            (NamedKey::F18, 18),
            (NamedKey::F19, 19),
            (NamedKey::F20, 20),
            (NamedKey::F21, 21),
            (NamedKey::F22, 22),
            (NamedKey::F23, 23),
            (NamedKey::F24, 24),
        ];
        for (named, expected) in cases {
            assert_eq!(map_named_key(named), PlatformKey::Function(expected));
        }
        assert_eq!(map_named_key(NamedKey::F1), PlatformKey::Function(1));
        assert_eq!(map_named_key(NamedKey::F12), PlatformKey::Function(12));
        assert_eq!(map_named_key(NamedKey::F24), PlatformKey::Function(24));
        assert_eq!(map_named_key(NamedKey::F25), PlatformKey::Other);
    }

    #[test]
    fn resolve_key_text_prefers_event_text_and_falls_back_to_character() {
        let space_key = Key::Named(NamedKey::Space);
        assert_eq!(
            resolve_key_text(Some(" "), &space_key),
            Some(" ".to_string())
        );

        let char_key = Key::Character("a".into());
        assert_eq!(
            resolve_key_text(Some("A"), &char_key),
            Some("A".to_string())
        );
        assert_eq!(resolve_key_text(None, &char_key), Some("a".to_string()));
        assert_eq!(resolve_key_text(Some(""), &char_key), Some("a".to_string()));
        assert_eq!(resolve_key_text(None, &space_key), None);
    }

    #[test]
    fn resolve_key_character_returns_logical_character_even_under_ctrl() {
        let ctrl_c_logical = Key::Character("c".into());
        assert_eq!(
            resolve_key_character(&ctrl_c_logical),
            Some("c".to_string())
        );
        assert_eq!(
            resolve_key_text(Some("\u{3}"), &ctrl_c_logical),
            Some("\u{3}".to_string())
        );
        assert_eq!(resolve_key_character(&Key::Named(NamedKey::Space)), None);
    }
}
