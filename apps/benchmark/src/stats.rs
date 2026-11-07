//! Summary statistics over timed samples. All durations are nanoseconds.

/// Summary of one benchmark's timed samples.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stats {
    pub samples: usize,
    pub min_ns: u64,
    pub median_ns: f64,
    pub p95_ns: u64,
    pub mean_ns: f64,
    pub max_ns: u64,
}

/// Summarise `samples`; returns `None` when there are none.
///
/// The median is the middle sample, or the mean of the two middle samples for
/// an even count. The p95 uses the nearest-rank method, so it is always an
/// observed sample: the smallest value at or above which 95% of samples fall.
pub fn summarize(samples: &[u64]) -> Option<Stats> {
    if samples.is_empty() {
        return None;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let n = sorted.len();
    let sum: f64 = sorted.iter().map(|&ns| ns as f64).sum();
    Some(Stats {
        samples: n,
        min_ns: sorted[0],
        median_ns: median(&sorted),
        p95_ns: sorted[nearest_rank(n, 95) - 1],
        mean_ns: sum / n as f64,
        max_ns: sorted[n - 1],
    })
}

fn median(sorted: &[u64]) -> f64 {
    let n = sorted.len();
    if n % 2 == 1 {
        sorted[n / 2] as f64
    } else {
        (sorted[n / 2 - 1] as f64 + sorted[n / 2] as f64) / 2.0
    }
}

/// 1-based nearest rank of percentile `pct` (1..=100) among `n >= 1` values.
fn nearest_rank(n: usize, pct: usize) -> usize {
    (pct * n).div_ceil(100).max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_samples_have_no_summary() {
        assert_eq!(summarize(&[]), None);
    }

    #[test]
    fn single_sample_is_every_statistic() {
        let stats = summarize(&[7]).expect("one sample");
        assert_eq!(stats.samples, 1);
        assert_eq!((stats.min_ns, stats.p95_ns, stats.max_ns), (7, 7, 7));
        assert_eq!((stats.median_ns, stats.mean_ns), (7.0, 7.0));
    }

    #[test]
    fn odd_count_median_is_the_middle_value_in_any_input_order() {
        let stats = summarize(&[5, 1, 3, 2, 4]).expect("samples");
        assert_eq!(stats.median_ns, 3.0);
        assert_eq!(stats.mean_ns, 3.0);
        assert_eq!((stats.min_ns, stats.max_ns), (1, 5));
        assert_eq!(stats.p95_ns, 5);
    }

    #[test]
    fn even_count_median_averages_the_middle_pair() {
        let stats = summarize(&[4, 1, 3, 2]).expect("samples");
        assert_eq!(stats.median_ns, 2.5);
    }

    #[test]
    fn p95_on_ten_samples_is_nearest_rank_ten() {
        let values: Vec<u64> = (1..=10).collect();
        let stats = summarize(&values).expect("samples");
        assert_eq!(stats.p95_ns, 10);
        assert_eq!(stats.median_ns, 5.5);
        assert_eq!(stats.mean_ns, 5.5);
    }

    #[test]
    fn p95_on_twenty_samples_is_nearest_rank_nineteen() {
        let values: Vec<u64> = (1..=20).rev().collect();
        let stats = summarize(&values).expect("samples");
        assert_eq!(stats.p95_ns, 19);
        assert_eq!(stats.median_ns, 10.5);
    }

    #[test]
    fn p95_on_hundred_samples_is_ninety_fifth_value() {
        let values: Vec<u64> = (1..=100).collect();
        let stats = summarize(&values).expect("samples");
        assert_eq!(stats.p95_ns, 95);
        assert_eq!(stats.median_ns, 50.5);
    }

    #[test]
    fn nearest_rank_never_drops_below_one() {
        assert_eq!(nearest_rank(1, 95), 1);
        assert_eq!(nearest_rank(2, 95), 2);
    }
}
