//! Public API regressions for grapheme-aware text editing.

use acme_platform::PlatformKey;
use acme_textinput::{TextInputState, handle_key, handle_text};
use std::hint::black_box;
use std::time::{Duration, Instant};
use unicode_segmentation::UnicodeSegmentation;

fn focused(text: &str) -> TextInputState {
    let mut state = TextInputState::new();
    state.focused = true;
    handle_text(&mut state, text);
    state
}

fn key(state: &mut TextInputState, key: PlatformKey, shift: bool) {
    handle_key(state, &key, true, None, false, shift);
}

#[test]
fn arrows_and_backspace_treat_combining_flag_and_zwj_sequences_as_graphemes() {
    let text = "Ae\u{301}🇹🇼👩‍👩‍👧‍👦Z";
    let mut state = focused(text);

    key(&mut state, PlatformKey::ArrowLeft, false);
    let family_end = state.cursor();
    key(&mut state, PlatformKey::ArrowLeft, false);
    assert_eq!(state.text, text);
    assert_eq!(&state.text[state.cursor()..family_end], "👩‍👩‍👧‍👦");
    key(&mut state, PlatformKey::ArrowRight, false);
    key(&mut state, PlatformKey::Backspace, false);
    assert_eq!(state.text, "Ae\u{301}🇹🇼Z");

    let flag_end = state.cursor();
    key(&mut state, PlatformKey::ArrowLeft, false);
    assert_eq!(&state.text[state.cursor()..flag_end], "🇹🇼");
    key(&mut state, PlatformKey::ArrowRight, false);
    key(&mut state, PlatformKey::Backspace, false);
    assert_eq!(state.text, "Ae\u{301}Z");
}

#[test]
fn extending_selection_and_forward_delete_preserve_exact_unicode() {
    let mut state = focused("xe\u{301}🇹🇼👩‍👩‍👧‍👦y");
    key(&mut state, PlatformKey::ArrowLeft, false);
    key(&mut state, PlatformKey::ArrowLeft, true);
    key(&mut state, PlatformKey::ArrowLeft, true);
    let (start, end) = state.selection().expect("shift arrows select graphemes");
    assert_eq!(&state.text[start..end], "🇹🇼👩‍👩‍👧‍👦");
    key(&mut state, PlatformKey::Delete, false);
    assert_eq!(state.text, "xe\u{301}y");

    state.cursor_home();
    key(&mut state, PlatformKey::ArrowRight, false);
    key(&mut state, PlatformKey::Delete, false);
    assert_eq!(state.text, "xy");
}

/// Run with `cargo test --release --locked -p acme-textinput --test grapheme_navigation -- --ignored --nocapture`.
/// Prints median nanoseconds per prev/next pair for exact text byte/grapheme counts.
#[test]
#[ignore = "reproducible local microbenchmark; run explicitly with --ignored"]
fn benchmark_grapheme_navigation_near_end() {
    const SAMPLES: usize = 7;
    const ITERATIONS: usize = 300;
    const SIZES: [usize; 3] = [256, 4_096, 16_384];
    let unit = "a e\u{301}🇹🇼👩‍👩‍👧‍👦繁";

    for target_graphemes in SIZES {
        let repeats = target_graphemes / unit.graphemes(true).count();
        let text = unit.repeat(repeats);
        let graphemes = text.graphemes(true).count();
        let bytes = text.len();
        let mut samples = Vec::with_capacity(SAMPLES);
        for _ in 0..SAMPLES {
            let mut state = TextInputState::new();
            state.text = text.clone();
            state.cursor_end();
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                state.cursor_prev();
                black_box(state.cursor());
                state.cursor_next();
                black_box(state.cursor());
            }
            samples.push(started.elapsed());
        }
        samples.sort_unstable();
        let median: Duration = samples[SAMPLES / 2];
        println!(
            "bytes={bytes}, graphemes={graphemes}: median {:?} total, {:.1} ns/pair ({ITERATIONS} pairs, {SAMPLES} samples)",
            median,
            median.as_nanos() as f64 / ITERATIONS as f64
        );
    }
}
