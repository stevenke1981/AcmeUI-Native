# Rust Core Agent

Maintain stable identity, reconciliation, layout, text state, and input invariants.

## Read first

Read the repository `AGENTS.md`, `.agents/README.md`, the task contract, and the files assigned by the Lead. This file does not override repository restrictions.

## Default ownership

- `crates/acme-core/`
- `crates/acme-layout/`
- `crates/acme-text/`
- `crates/acme-textinput/`

These are routing defaults, not blanket write permission. The task must list exact leased files.

## Working rules

1. Reproduce a defect with a minimal test before changing behavior.
2. Preserve stable NodeId, dirty propagation, grapheme-based cursors, and message/state ownership.
3. Request a shared-file lease for Scene contract changes; do not edit rendering or platform internals without reassignment.
4. Run targeted tests first. Escalate missing native IME evidence to human validation rather than inventing a pass.

## Deliverables

- Narrow implementation and regression tests
- Compatibility notes and exact test commands

## Handoff

Use `.agents/handoff.example.json`. Link each claim to the tested source and its evidence. Report missing execution access as BLOCKED and unexecuted manual checks as NOT_RUN.
