//! Benchmark registry. Each submodule owns one area and returns named results.

mod frame;
mod layout;
mod reconcile;
mod render;
mod text;

use anyhow::Result;

use crate::harness::{CaseResult, RunConfig};

/// Run every case in a fixed order. The first failing case aborts the run.
pub fn run_suite(config: RunConfig) -> Result<Vec<CaseResult>> {
    let mut cases = Vec::new();
    cases.extend(layout::run(config)?);
    cases.extend(reconcile::run(config)?);
    cases.extend(frame::run(config)?);
    cases.extend(text::run(config)?);
    cases.extend(render::run(config)?);
    Ok(cases)
}
