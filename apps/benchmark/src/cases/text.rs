//! Text case: `FontSystem::shape` followed by `FontSystem::prepare` for four
//! mixed Latin / CJK / symbol strings. The result depends on the fonts the OS
//! provides, so compare runs only on the same machine.

use acme_text::{FontSystem, GlyphAtlas, TextConstraints, TextStyle};
use anyhow::Result;

use crate::harness::{CaseResult, MetaValue, RunConfig, measure};

const SAMPLE_TEXTS: [&str; 4] = [
    "Hello, world! The quick brown fox jumps over the lazy dog.",
    "在一個寧靜的午後，古老的書架上擺滿了泛黃的書籍",
    "Rust is a systems programming language focused on safety.",
    "1234567890 !@#$%^&*() 特殊字符テスト",
];
const ATLAS_SIZE: u32 = 2048;
const FONT_SIZE: f32 = 14.0;

pub fn run(config: RunConfig) -> Result<Vec<CaseResult>> {
    let mut fonts = FontSystem::new();
    // One atlas for the whole case: after warm-up the glyph cache is hot, so the
    // sample measures shaping plus cache lookups, as an application frame would.
    let mut atlas = GlyphAtlas::new(ATLAS_SIZE, ATLAS_SIZE);
    let style = TextStyle {
        font_size: FONT_SIZE,
        ..TextStyle::default()
    };
    let constraints = TextConstraints::default();

    let samples = measure(config, || {
        for text in SAMPLE_TEXTS {
            let layout = fonts.shape(text, &style, constraints, 1.0);
            let _prepared = fonts.prepare(&layout, &mut atlas);
        }
        Ok(())
    })?;

    Ok(vec![CaseResult::from_samples(
        format!("text/shape_prepare_{}_strings", SAMPLE_TEXTS.len()),
        &samples,
        vec![("atlas_len", MetaValue::Count(atlas.len() as u64))],
    )?])
}
