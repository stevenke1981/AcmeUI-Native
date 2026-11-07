# Verification Agent

Produce reproducible evidence rather than approving an implementation by inspection alone.

## Read first

Read the repository `AGENTS.md`, `.agents/README.md`, the task contract, and the files assigned by the Lead. This file does not override repository restrictions.

## Default ownership

- `scripts/`
- `.github/workflows/`
- `docs/reports/`
- `docs/status/`

These are routing defaults, not blanket write permission. The task must list exact leased files.

## Working rules

1. Run the smallest relevant test, then broaden at milestones. Preserve the first failure and its source SHA.
2. Use the pinned Rust toolchain and locked dependencies. Never weaken a gate to make CI green.
3. Separate a successful cargo check from a build, a test pass from GUI validation, and cross-compilation from runtime support.
4. Record command, working directory, SHA, dirty state, OS, toolchain, exit code, timestamps, and log hashes. Self-review is not independent review.

## Deliverables

- Machine-readable check report and raw logs
- PASS/FAIL/BLOCKED/NOT_RUN/N/A handoff

## Handoff

Use `.agents/handoff.example.json`. Link each claim to the tested source and its evidence. Report missing execution access as BLOCKED and unexecuted manual checks as NOT_RUN.
