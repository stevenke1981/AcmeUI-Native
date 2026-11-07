//! Machine and build facts recorded with every run. Each probe is optional: a
//! missing `git` or `rustc` yields `null`, never a failure.

use std::env;
use std::process::Command;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

const SECONDS_PER_DAY: i64 = 86_400;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    pub git_sha: Option<String>,
    pub rustc: Option<String>,
    pub os: &'static str,
    pub arch: &'static str,
    pub profile: &'static str,
    pub cpu_count: usize,
    /// `PROCESSOR_IDENTIFIER` on Windows; `None` elsewhere.
    pub cpu_model: Option<String>,
    pub timestamp_utc: String,
}

impl Environment {
    pub fn collect() -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_secs() as i64);
        Self {
            git_sha: command_stdout("git", &["rev-parse", "HEAD"]),
            rustc: command_stdout("rustc", &["-V"]),
            os: env::consts::OS,
            arch: env::consts::ARCH,
            profile: if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            },
            cpu_count: thread::available_parallelism().map_or(0, |count| count.get()),
            cpu_model: env::var("PROCESSOR_IDENTIFIER")
                .ok()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            timestamp_utc: format_utc(now),
        }
    }
}

/// Trimmed stdout of a successful command, or `None`.
fn command_stdout(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let trimmed = text.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

/// ISO 8601 UTC timestamp (`YYYY-MM-DDThh:mm:ssZ`) for Unix seconds.
pub fn format_utc(unix_secs: i64) -> String {
    let days = unix_secs.div_euclid(SECONDS_PER_DAY);
    let seconds_of_day = unix_secs.rem_euclid(SECONDS_PER_DAY);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        seconds_of_day / 3_600,
        (seconds_of_day % 3_600) / 60,
        seconds_of_day % 60,
    )
}

/// Proleptic Gregorian date for days since 1970-01-01 (H. Hinnant's algorithm).
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_utc_matches_known_instants() {
        assert_eq!(format_utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_utc(-1), "1969-12-31T23:59:59Z");
        assert_eq!(format_utc(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(format_utc(1_700_000_000), "2023-11-14T22:13:20Z");
    }

    #[test]
    fn collect_always_produces_a_timestamp_and_platform_facts() {
        let env = Environment::collect();
        assert_eq!(env.timestamp_utc.len(), 20);
        assert!(env.timestamp_utc.ends_with('Z'));
        assert!(!env.os.is_empty() && !env.arch.is_empty());
        assert!(matches!(env.profile, "debug" | "release"));
    }
}
