//! Warm-up and timed-sample harness shared by every benchmark case.

use std::hint::black_box;
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};

use crate::stats::{Stats, summarize};

/// Warm-up and sample counts for one run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunConfig {
    pub warmup: usize,
    pub samples: usize,
}

impl RunConfig {
    /// Default run: enough samples for a stable median on a quiet machine.
    pub const FULL: Self = Self {
        warmup: 10,
        samples: 51,
    };
    /// Smoke run for `--quick`: proves the suite works; its numbers are noisy.
    pub const QUICK: Self = Self {
        warmup: 2,
        samples: 9,
    };
}

/// A fact recorded next to a case's timings, such as a count or checksum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetaValue {
    Count(u64),
    Text(String),
}

/// One finished benchmark case.
#[derive(Debug, Clone, PartialEq)]
pub struct CaseResult {
    pub name: String,
    pub stats: Stats,
    pub meta: Vec<(&'static str, MetaValue)>,
}

impl CaseResult {
    /// Build a result from raw samples. Fails when no sample was recorded.
    pub fn from_samples(
        name: impl Into<String>,
        samples: &[u64],
        meta: Vec<(&'static str, MetaValue)>,
    ) -> Result<Self> {
        let name = name.into();
        let stats = summarize(samples).ok_or_else(|| anyhow!("{name}: no samples recorded"))?;
        Ok(Self { name, stats, meta })
    }
}

/// Run `op` untimed `warmup` times, then time `samples` runs. Each timed run
/// covers the call and the drop of nothing else: the returned value is dropped
/// after the clock stops. The first error aborts the case.
pub fn measure<T>(config: RunConfig, mut op: impl FnMut() -> Result<T>) -> Result<Vec<u64>> {
    for _ in 0..config.warmup {
        black_box(op()?);
    }
    let mut samples = Vec::with_capacity(config.samples);
    for _ in 0..config.samples {
        let start = Instant::now();
        let value = op()?;
        samples.push(saturating_nanos(start.elapsed()));
        black_box(value);
    }
    Ok(samples)
}

/// Convert a foreign `Result` (the engines use `Debug` errors) into `anyhow`.
pub fn check_ok<T, E: std::fmt::Debug>(result: Result<T, E>, what: &str) -> Result<T> {
    result.map_err(|error| anyhow!("{what}: {error:?}"))
}

fn saturating_nanos(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measure_returns_one_sample_per_timed_run() {
        let mut calls = 0usize;
        let config = RunConfig {
            warmup: 3,
            samples: 4,
        };
        let samples = measure(config, || {
            calls += 1;
            Ok(calls)
        })
        .expect("measure");
        assert_eq!(samples.len(), 4);
        assert_eq!(calls, 7, "warm-up runs are untimed but still executed");
    }

    #[test]
    fn measure_propagates_the_first_error() {
        let config = RunConfig {
            warmup: 0,
            samples: 3,
        };
        let result = measure(config, || Err::<(), _>(anyhow!("boom")));
        assert!(result.is_err());
    }

    #[test]
    fn from_samples_rejects_an_empty_case() {
        assert!(CaseResult::from_samples("empty", &[], Vec::new()).is_err());
    }
}
