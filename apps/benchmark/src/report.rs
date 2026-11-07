//! Serialises a run to JSON (schema v1) and prints a human-readable summary.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::environment::Environment;
use crate::harness::{CaseResult, MetaValue, RunConfig};
use crate::json::{number, optional, quote};

pub const SCHEMA_NAME: &str = "acmeui-bench";
pub const SCHEMA_VERSION: u32 = 1;

/// Everything one run produced.
pub struct Report {
    pub quick: bool,
    pub config: RunConfig,
    pub environment: Environment,
    pub cases: Vec<CaseResult>,
}

/// Render the report as JSON. Top-level fields and benchmark entries each sit
/// on their own line; everything nested is compact.
pub fn to_json(report: &Report) -> String {
    let benchmarks: Vec<String> = report
        .cases
        .iter()
        .map(|case| format!("    {}", case_json(case)))
        .collect();
    let mode = if report.quick { "quick" } else { "full" };
    let fields = [
        format!("  \"schema\": {}", quote(SCHEMA_NAME)),
        format!("  \"schema_version\": {SCHEMA_VERSION}"),
        format!("  \"package\": {}", quote(env!("CARGO_PKG_NAME"))),
        format!(
            "  \"package_version\": {}",
            quote(env!("CARGO_PKG_VERSION"))
        ),
        format!("  \"mode\": {}", quote(mode)),
        format!(
            "  \"config\": {{\"warmup\": {}, \"samples\": {}}}",
            report.config.warmup, report.config.samples
        ),
        format!(
            "  \"environment\": {}",
            environment_json(&report.environment)
        ),
        format!("  \"benchmarks\": [\n{}\n  ]", benchmarks.join(",\n")),
    ];
    format!("{{\n{}\n}}\n", fields.join(",\n"))
}

/// Write the JSON report, creating parent directories as needed.
/// Create the parent directory and check that  can be written, so a bad
/// path fails before the (minutes-long) suite runs rather than after it.
pub fn prepare_json_path(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent().filter(|dir| !dir.as_os_str().is_empty()) {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map(drop)
        .with_context(|| format!("opening {} for writing", path.display()))
}

pub fn write_json(path: &Path, report: &Report) -> Result<()> {
    prepare_json_path(path)?;
    fs::write(path, to_json(report)).with_context(|| format!("writing {}", path.display()))
}

/// Print the environment header and one line per case.
pub fn print_summary(report: &Report) {
    let env = &report.environment;
    let mode = if report.quick { "quick" } else { "full" };
    println!(
        "AcmeUI headless benchmarks ({mode}): {} warm-up runs, {} samples per case",
        report.config.warmup, report.config.samples
    );
    println!(
        "{} {} | {} CPUs | {} build | {} | git {}",
        env.os,
        env.arch,
        env.cpu_count,
        env.profile,
        env.rustc.as_deref().unwrap_or("rustc unknown"),
        env.git_sha.as_deref().unwrap_or("unknown"),
    );
    if let Some(model) = &env.cpu_model {
        println!("cpu: {model}");
    }
    println!();
    println!(
        "{:<56} {:>8} {:>12} {:>12} {:>12} {:>12}",
        "benchmark", "samples", "min", "median", "p95", "mean"
    );
    for case in &report.cases {
        let s = &case.stats;
        println!(
            "{:<56} {:>8} {:>12} {:>12} {:>12} {:>12}",
            case.name,
            s.samples,
            format_ns(s.min_ns as f64),
            format_ns(s.median_ns),
            format_ns(s.p95_ns as f64),
            format_ns(s.mean_ns),
        );
    }
}

fn format_ns(ns: f64) -> String {
    if ns < 1_000.0 {
        format!("{ns:.0} ns")
    } else if ns < 1_000_000.0 {
        format!("{:.2} us", ns / 1_000.0)
    } else if ns < 1_000_000_000.0 {
        format!("{:.2} ms", ns / 1_000_000.0)
    } else {
        format!("{:.3} s", ns / 1_000_000_000.0)
    }
}

fn environment_json(env: &Environment) -> String {
    format!(
        "{{\"git_sha\": {}, \"rustc\": {}, \"os\": {}, \"arch\": {}, \"profile\": {}, \
         \"cpu_count\": {}, \"cpu_model\": {}, \"timestamp_utc\": {}}}",
        optional(env.git_sha.as_deref()),
        optional(env.rustc.as_deref()),
        quote(env.os),
        quote(env.arch),
        quote(env.profile),
        env.cpu_count,
        optional(env.cpu_model.as_deref()),
        quote(&env.timestamp_utc),
    )
}

fn case_json(case: &CaseResult) -> String {
    let s = &case.stats;
    let meta: Vec<String> = case
        .meta
        .iter()
        .map(|(key, value)| format!("{}: {}", quote(key), meta_json(value)))
        .collect();
    format!(
        "{{\"name\": {}, \"stats\": {{\"samples\": {}, \"min_ns\": {}, \"median_ns\": {}, \
         \"p95_ns\": {}, \"mean_ns\": {}, \"max_ns\": {}}}, \"meta\": {{{}}}}}",
        quote(&case.name),
        s.samples,
        s.min_ns,
        number(s.median_ns),
        s.p95_ns,
        number(s.mean_ns),
        s.max_ns,
        meta.join(", "),
    )
}

fn meta_json(value: &MetaValue) -> String {
    match value {
        MetaValue::Count(count) => count.to_string(),
        MetaValue::Text(text) => quote(text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::summarize;

    fn sample_report(rustc: &str) -> Report {
        Report {
            quick: true,
            config: RunConfig::QUICK,
            environment: Environment {
                git_sha: None,
                rustc: Some(rustc.to_string()),
                os: "windows",
                arch: "x86_64",
                profile: "release",
                cpu_count: 8,
                cpu_model: None,
                timestamp_utc: "2026-11-07T00:00:00Z".to_string(),
            },
            cases: vec![CaseResult {
                name: "layout/100_nodes".to_string(),
                stats: summarize(&[10, 20, 30]).expect("samples"),
                meta: vec![("layout_results", MetaValue::Count(100))],
            }],
        }
    }

    #[test]
    fn json_contains_schema_fields_and_escaped_strings() {
        let json = to_json(&sample_report("rustc \"1.88.0\""));
        assert!(json.contains("\"schema_version\": 1"));
        assert!(json.contains("\"mode\": \"quick\""));
        assert!(json.contains("\"git_sha\": null"));
        assert!(json.contains("\"rustc\": \"rustc \\\"1.88.0\\\"\""));
        assert!(json.contains("\"median_ns\": 20"));
        assert!(json.contains("\"layout_results\": 100"));
    }

    #[test]
    fn format_ns_picks_a_readable_unit() {
        assert_eq!(format_ns(950.0), "950 ns");
        assert_eq!(format_ns(1_500.0), "1.50 us");
        assert_eq!(format_ns(2_000_000.0), "2.00 ms");
    }
}
