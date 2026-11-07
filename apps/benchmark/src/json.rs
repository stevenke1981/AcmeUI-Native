//! Hand-written JSON helpers. The crate deliberately avoids serde.

/// Quote `value` as a JSON string literal (RFC 8259 escaping).
pub fn quote(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A finite `f64` as a JSON number; non-finite values become `null`.
pub fn number(value: f64) -> String {
    if value.is_finite() {
        format!("{value}")
    } else {
        "null".to_string()
    }
}

/// Quote an optional string, or emit `null`.
pub fn optional(value: Option<&str>) -> String {
    value.map_or_else(|| "null".to_string(), quote)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_escapes_quotes_backslashes_and_control_characters() {
        assert_eq!(quote("a\"b\\c\n\t"), "\"a\\\"b\\\\c\\n\\t\"");
        assert_eq!(quote("\u{1}\u{8}\u{c}\r"), "\"\\u0001\\b\\f\\r\"");
    }

    #[test]
    fn quote_keeps_non_ascii_text_verbatim() {
        assert_eq!(quote("繁體 テスト"), "\"繁體 テスト\"");
    }

    #[test]
    fn non_finite_numbers_become_null() {
        assert_eq!(number(f64::NAN), "null");
        assert_eq!(number(f64::INFINITY), "null");
        assert_eq!(number(2.5), "2.5");
    }

    #[test]
    fn optional_emits_null_for_none() {
        assert_eq!(optional(None), "null");
        assert_eq!(optional(Some("x")), "\"x\"");
    }
}
