//! End-to-end check: the headless quick suite runs to completion and writes JSON
//! of the documented schema. The reader below is a small RFC 8259 parser,
//! because this crate uses no serde.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

/// Benchmark names the quick suite must emit. Keep in sync with `src/cases`.
const EXPECTED_NAMES: [&str; 14] = [
    "layout/100_nodes",
    "layout/500_nodes",
    "layout/1000_nodes",
    "reconcile/100_keys_5_rounds_fresh_tree",
    "retained/widget_to_views_1011_widgets",
    "retained/reconcile_steady_1011_widgets",
    "retained/bind_1011_widgets",
    "retained/to_layout_with_binding_1011_widgets",
    "frame_build/1200_quads_100_clipped_50_text",
    "widget_tree/build_50_rows",
    "text/shape_prepare_4_strings",
    "text/atlas_pressure_multipage",
    "render_cpu/2000_quads_scale_1.0",
    "render_cpu/2000_quads_scale_2.0",
];

const QUICK_SAMPLES: f64 = 9.0;

#[test]
fn quick_suite_writes_schema_v1_json() {
    let out_path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("bench-quick-test.json");
    let _ = fs::remove_file(&out_path);
    let out_arg = out_path.to_str().expect("UTF-8 temp path");

    let output = run_benchmark(&["--quick", "--json", out_arg]);
    assert!(
        output.status.success(),
        "exit {:?}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );

    let text = fs::read_to_string(&out_path).expect("JSON file must be written");
    let document = parse(&text).unwrap_or_else(|error| panic!("invalid JSON: {error}"));
    check_document(&document);
}

#[test]
fn window_flag_is_rejected_because_the_runner_is_headless() {
    let output = run_benchmark(&["--window"]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("headless"));
}

fn run_benchmark(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_acme-benchmark"))
        .args(args)
        .output()
        .expect("benchmark binary must start")
}

fn check_document(document: &Json) {
    assert_eq!(text_field(document, "schema"), "acmeui-bench");
    assert_eq!(number_field(document, "schema_version"), 1.0);
    assert_eq!(text_field(document, "package"), "acme-benchmark");
    assert_eq!(text_field(document, "mode"), "quick");
    assert!(!text_field(document, "package_version").is_empty());

    let config = field(document, "config");
    assert_eq!(number_field(config, "samples"), QUICK_SAMPLES);

    let environment = field(document, "environment");
    for key in ["os", "arch", "timestamp_utc"] {
        assert!(
            !text_field(environment, key).is_empty(),
            "environment.{key}"
        );
    }
    let profile = text_field(environment, "profile");
    assert!(
        profile == "debug" || profile == "release",
        "profile {profile}"
    );
    assert!(number_field(environment, "cpu_count") >= 1.0);
    for key in ["git_sha", "rustc", "cpu_model"] {
        assert!(
            matches!(field(environment, key), Json::Null | Json::Str(_)),
            "environment.{key} must be a string or null"
        );
    }
    let timestamp = text_field(environment, "timestamp_utc");
    assert_eq!(timestamp.len(), 20, "timestamp {timestamp}");
    assert!(timestamp.ends_with('Z') && timestamp.as_bytes()[10] == b'T');

    let benchmarks = field(document, "benchmarks").as_array().expect("array");
    let names: Vec<&str> = benchmarks
        .iter()
        .map(|bench| text_field(bench, "name"))
        .collect();
    for expected in EXPECTED_NAMES {
        assert!(names.contains(&expected), "missing benchmark {expected}");
    }
    let mut unique = names.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), names.len(), "benchmark names must be unique");
    for bench in benchmarks {
        check_case(bench);
    }
}

fn check_case(bench: &Json) {
    let name = text_field(bench, "name");
    let stats = field(bench, "stats");
    assert_eq!(number_field(stats, "samples"), QUICK_SAMPLES, "{name}");
    let min = number_field(stats, "min_ns");
    let median = number_field(stats, "median_ns");
    let p95 = number_field(stats, "p95_ns");
    let mean = number_field(stats, "mean_ns");
    let max = number_field(stats, "max_ns");
    assert!(min > 0.0, "{name}: min_ns must be positive");
    assert!(
        min <= median && median <= p95 && p95 <= max,
        "{name}: expected min <= median <= p95 <= max"
    );
    assert!(
        min <= mean && mean <= max,
        "{name}: mean outside [min, max]"
    );

    let meta = field(bench, "meta");
    assert!(
        matches!(meta, Json::Object(_)),
        "{name}: meta must be an object"
    );
    if name.starts_with("render_cpu/") {
        let checksum = text_field(meta, "checksum_fnv1a64");
        assert_eq!(checksum.len(), 16, "{name}: checksum");
        assert!(checksum.bytes().all(|b| b.is_ascii_hexdigit()));
    }
}

fn field<'a>(object: &'a Json, key: &str) -> &'a Json {
    object
        .get(key)
        .unwrap_or_else(|| panic!("missing field `{key}`"))
}

fn text_field<'a>(object: &'a Json, key: &str) -> &'a str {
    field(object, key)
        .as_str()
        .unwrap_or_else(|| panic!("field `{key}` must be a string"))
}

fn number_field(object: &Json, key: &str) -> f64 {
    field(object, key)
        .as_number()
        .unwrap_or_else(|| panic!("field `{key}` must be a number"))
}

#[derive(Debug, Clone, PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Number(f64),
    Str(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

impl Json {
    fn get(&self, key: &str) -> Option<&Json> {
        match self {
            Json::Object(fields) => fields.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
    fn as_str(&self) -> Option<&str> {
        match self {
            Json::Str(text) => Some(text),
            _ => None,
        }
    }
    fn as_number(&self) -> Option<f64> {
        match self {
            Json::Number(value) => Some(*value),
            _ => None,
        }
    }
    fn as_array(&self) -> Option<&[Json]> {
        match self {
            Json::Array(items) => Some(items),
            _ => None,
        }
    }
}

fn parse(text: &str) -> Result<Json, String> {
    let mut parser = Parser {
        bytes: text.as_bytes(),
        pos: 0,
    };
    let value = parser.value()?;
    parser.skip_ws();
    if parser.pos != parser.bytes.len() {
        return Err(format!("trailing data at byte {}", parser.pos));
    }
    Ok(value)
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl Parser<'_> {
    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn eat(&mut self, expected: u8) -> Result<(), String> {
        if self.peek() == Some(expected) {
            self.pos += 1;
            Ok(())
        } else {
            Err(format!(
                "expected '{}' at byte {}",
                expected as char, self.pos
            ))
        }
    }

    fn literal(&mut self, word: &str, value: Json) -> Result<Json, String> {
        if self
            .bytes
            .get(self.pos..)
            .is_some_and(|rest| rest.starts_with(word.as_bytes()))
        {
            self.pos += word.len();
            Ok(value)
        } else {
            Err(format!("bad literal at byte {}", self.pos))
        }
    }

    fn value(&mut self) -> Result<Json, String> {
        self.skip_ws();
        match self.peek() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b'"') => self.string().map(Json::Str),
            Some(b't') => self.literal("true", Json::Bool(true)),
            Some(b'f') => self.literal("false", Json::Bool(false)),
            Some(b'n') => self.literal("null", Json::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(format!("unexpected input at byte {}", self.pos)),
        }
    }

    fn object(&mut self) -> Result<Json, String> {
        self.eat(b'{')?;
        let mut fields = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(Json::Object(fields));
        }
        loop {
            self.skip_ws();
            let key = self.string()?;
            self.skip_ws();
            self.eat(b':')?;
            let value = self.value()?;
            fields.push((key, value));
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(Json::Object(fields));
                }
                _ => return Err(format!("expected ',' or '}}' at byte {}", self.pos)),
            }
        }
    }

    fn array(&mut self) -> Result<Json, String> {
        self.eat(b'[')?;
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(Json::Array(items));
        }
        loop {
            items.push(self.value()?);
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(Json::Array(items));
                }
                _ => return Err(format!("expected ',' or ']' at byte {}", self.pos)),
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.eat(b'"')?;
        let mut out: Vec<u8> = Vec::new();
        loop {
            let byte = self.peek().ok_or("unterminated string")?;
            self.pos += 1;
            match byte {
                b'"' => return String::from_utf8(out).map_err(|error| error.to_string()),
                b'\\' => self.escape(&mut out)?,
                0..=0x1f => return Err(format!("raw control byte at {}", self.pos - 1)),
                _ => out.push(byte),
            }
        }
    }

    fn escape(&mut self, out: &mut Vec<u8>) -> Result<(), String> {
        let code = self.peek().ok_or("unterminated escape")?;
        self.pos += 1;
        let plain = match code {
            b'"' => b'"',
            b'\\' => b'\\',
            b'/' => b'/',
            b'b' => 0x08,
            b'f' => 0x0c,
            b'n' => b'\n',
            b'r' => b'\r',
            b't' => b'\t',
            b'u' => {
                let hex = self
                    .bytes
                    .get(self.pos..self.pos + 4)
                    .ok_or("short \\u escape")?;
                let hex = std::str::from_utf8(hex).map_err(|error| error.to_string())?;
                let scalar = u32::from_str_radix(hex, 16).map_err(|error| error.to_string())?;
                self.pos += 4;
                let ch = char::from_u32(scalar).ok_or("invalid code point")?;
                let mut buf = [0u8; 4];
                out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                return Ok(());
            }
            _ => return Err(format!("bad escape at byte {}", self.pos - 1)),
        };
        out.push(plain);
        Ok(())
    }

    fn number(&mut self) -> Result<Json, String> {
        let start = self.pos;
        while matches!(
            self.peek(),
            Some(b'-' | b'+' | b'.' | b'e' | b'E' | b'0'..=b'9')
        ) {
            self.pos += 1;
        }
        let text = std::str::from_utf8(&self.bytes[start..self.pos]).map_err(|e| e.to_string())?;
        text.parse::<f64>()
            .map(Json::Number)
            .map_err(|error| format!("bad number `{text}`: {error}"))
    }
}
