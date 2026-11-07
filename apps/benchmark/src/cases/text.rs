//! Text cases.
//!
//! `text/shape_prepare_4_strings`: `FontSystem::shape` followed by
//! `FontSystem::prepare` for four mixed Latin / CJK / symbol strings. The result
//! depends on the fonts the OS provides, so compare runs only on the same machine.
//!
//! `text/atlas_pressure_multipage`: a rotation workload that cycles through
//! `ROTATION_GROUPS` glyph sets (one font size each) through small atlas pages.
//! The timed loop uses a 4-page atlas. Counters for a 1-page and a 4-page atlas
//! are replayed untimed, so the meta compares the two budgets. "refused" is
//! `FallbackDiagnostics::atlas_full_glyphs`, which also counts the two
//! zero-size space glyphs of each frame; compare budgets, not absolute values.

use acme_text::{FontSystem, GlyphAtlas, GlyphAtlasConfig, TextConstraints, TextStyle};
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

const ROTATION_TEXT: &str = "Rotate 轮换 ABC";
/// Glyph sets in the rotation. Each set has its own font size, so its glyphs
/// are distinct cache keys. A period longer than the 60-frame stale age lets
/// idle pages become evictable.
const ROTATION_GROUPS: usize = 72;
const ROTATION_BASE_SIZE: f32 = 9.0;
const ROTATION_SIZE_STEP: f32 = 0.25;
/// Two full rotations per timed sample.
const ROTATION_FRAMES: usize = ROTATION_GROUPS * 2;
const PRESSURE_PAGE_EDGE: u32 = 256;
const SINGLE_PAGE: u32 = 1;
const MULTI_PAGES: u32 = 4;

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

    Ok(vec![
        CaseResult::from_samples(
            format!("text/shape_prepare_{}_strings", SAMPLE_TEXTS.len()),
            &samples,
            vec![("atlas_len", MetaValue::Count(atlas.len() as u64))],
        )?,
        run_atlas_pressure(config)?,
    ])
}

/// Times the multi-page rotation and records the counters of both page budgets.
fn run_atlas_pressure(config: RunConfig) -> Result<CaseResult> {
    let mut fonts = FontSystem::new();
    let single = replay(&mut fonts, pressure_atlas(SINGLE_PAGE), ROTATION_FRAMES);
    let multi = replay(&mut fonts, pressure_atlas(MULTI_PAGES), ROTATION_FRAMES);

    // Persistent atlas, as in an application: the sample measures steady state.
    let mut timed_atlas = pressure_atlas(MULTI_PAGES);
    let mut frame = 0usize;
    let samples = measure(config, || {
        for _ in 0..ROTATION_FRAMES {
            rotation_frame(&mut fonts, &mut timed_atlas, frame);
            frame += 1;
        }
        Ok(())
    })?;

    CaseResult::from_samples(
        "text/atlas_pressure_multipage",
        &samples,
        vec![
            (
                "frames_per_sample",
                MetaValue::Count(ROTATION_FRAMES as u64),
            ),
            ("page_edge", MetaValue::Count(u64::from(PRESSURE_PAGE_EDGE))),
            ("single_max_pages", MetaValue::Count(u64::from(SINGLE_PAGE))),
            ("single_pages", MetaValue::Count(single.pages as u64)),
            ("single_evictions", MetaValue::Count(single.evictions)),
            ("single_generation", MetaValue::Count(single.generation)),
            ("single_drawn_glyphs", MetaValue::Count(single.drawn)),
            ("single_refused_glyphs", MetaValue::Count(single.refused)),
            ("multi_max_pages", MetaValue::Count(u64::from(MULTI_PAGES))),
            ("multi_pages", MetaValue::Count(multi.pages as u64)),
            ("multi_evictions", MetaValue::Count(multi.evictions)),
            ("multi_generation", MetaValue::Count(multi.generation)),
            ("multi_drawn_glyphs", MetaValue::Count(multi.drawn)),
            ("multi_refused_glyphs", MetaValue::Count(multi.refused)),
        ],
    )
}

/// Counters after one replay of the rotation on a fresh atlas.
#[derive(Debug, Default, PartialEq, Eq)]
struct Replay {
    pages: usize,
    evictions: u64,
    generation: u64,
    drawn: u64,
    refused: u64,
}

fn pressure_atlas(max_pages: u32) -> GlyphAtlas {
    GlyphAtlas::with_config(GlyphAtlasConfig {
        page_width: PRESSURE_PAGE_EDGE,
        page_height: PRESSURE_PAGE_EDGE,
        max_pages,
    })
}

fn replay(fonts: &mut FontSystem, mut atlas: GlyphAtlas, frames: usize) -> Replay {
    let (mut drawn, mut refused) = (0, 0);
    for frame in 0..frames {
        let (frame_drawn, frame_refused) = rotation_frame(fonts, &mut atlas, frame);
        drawn += frame_drawn;
        refused += frame_refused;
    }
    Replay {
        pages: atlas.page_count(),
        evictions: atlas.page_evictions(),
        generation: atlas.generation(),
        drawn,
        refused,
    }
}

/// Prepares one glyph set; returns `(drawn, refused)` glyph counts for it.
fn rotation_frame(fonts: &mut FontSystem, atlas: &mut GlyphAtlas, frame: usize) -> (u64, u64) {
    let group = frame % ROTATION_GROUPS;
    let style = TextStyle {
        font_size: ROTATION_BASE_SIZE + group as f32 * ROTATION_SIZE_STEP,
        ..TextStyle::default()
    };
    let layout = fonts.shape(ROTATION_TEXT, &style, TextConstraints::default(), 1.0);
    atlas.begin_frame();
    let prepared = fonts.prepare(&layout, atlas);
    let refused = fonts.diagnostics().atlas_full_glyphs as u64;
    (prepared.glyphs.len() as u64, refused)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_page_budget_is_under_pressure_in_the_rotation() {
        let mut fonts = FontSystem::new();
        let single = replay(&mut fonts, pressure_atlas(SINGLE_PAGE), ROTATION_FRAMES);
        assert_eq!(single.pages, 1);
        if single.generation == 0 {
            // No host font: nothing was rasterized, so there is no pressure to measure.
            eprintln!("SKIP single_page_budget: host has no glyphs");
            return;
        }
        assert!(single.evictions > 0, "rotation must overflow one page");
    }
}
