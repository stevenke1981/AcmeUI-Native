//! Headless benchmark runner for AcmeUI's layout, retained-tree, frame-build,
//! text-shaping and CPU-rasterizer paths. It never opens a window.
//! Methodology and comparison rules: `docs/guides/BENCHMARKS.md`.

mod cases;
mod environment;
mod harness;
mod json;
mod options;
mod report;
mod stats;

use anyhow::{Result, anyhow};

use environment::Environment;
use harness::RunConfig;
use report::Report;

fn main() -> Result<()> {
    let options = options::parse(std::env::args().skip(1))
        .map_err(|message| anyhow!("{message}\n\n{}", options::USAGE))?;
    if options.help {
        print!("{}", options::USAGE);
        return Ok(());
    }

    let config = if options.quick {
        RunConfig::QUICK
    } else {
        RunConfig::FULL
    };
    if let Some(path) = &options.json {
        report::prepare_json_path(path)?;
    }
    if cfg!(debug_assertions) {
        eprintln!("warning: debug build; numbers are not comparable. Use --release.");
    }
    let environment = Environment::collect();
    let cases = cases::run_suite(config)?;
    let report = Report {
        quick: options.quick,
        config,
        environment,
        cases,
    };

    report::print_summary(&report);
    if let Some(path) = &options.json {
        report::write_json(path, &report)?;
        println!("json: {}", path.display());
    }
    Ok(())
}
