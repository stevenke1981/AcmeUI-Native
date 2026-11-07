# ROADMAP 011 — Brief for implementers

Read `AGENTS.md` and `docs/team/roadmap-011/SPEC.md` (the design) first. The
wave-0 contract is committed (`04881aa`): acme-core constants and
`GlyphDraw::atlas_page`, acme-text `page`/`atlas_page` fields,
`GlyphAtlasConfig`, `with_config`, `Default`, stub getters. Behavior is still
single-page. Branch `sdk/roadmap-011-multipage-atlas`; do not switch.

## Hard rules

- Edit only your write set. No `git add/commit/stash/checkout`. No new
  dependencies, no Cargo.toml/Cargo.lock edits. Never `cargo clean`.
- Do not change the wave-0 public signatures; additions only if the SPEC lists them.
- Test-first: add the failing tests, run them, show RED, then implement GREEN.
- Run only: `cargo test --locked -p <your crate>`,
  `cargo clippy --locked -p <your crate> --all-targets -- -D warnings`,
  `cargo fmt -p <your crate>`. Other agents edit other crates concurrently; if
  their in-progress code breaks your build, report it rather than editing it.
- Files < 800 lines, functions < 50 lines where practical.
- Report (< 50 lines): files, RED excerpt, GREEN command + result, deviations
  from SPEC with reasons, NOT_RUN.

## Wave 1

| ID | Role | Write set |
|---|---|---|
| P1 | team-rust-engineer | `crates/acme-text/src/**` (you may move the atlas into `src/atlas.rs`; public API unchanged). `crates/acme-text/tests/atlas_lifecycle.rs` must pass UNCHANGED. |
| P3 | team-rust-engineer | `crates/acme-render-cpu/src/lib.rs`, `crates/acme-render-cpu/tests/regression.rs`, new `crates/acme-render-cpu/tests/atlas_pages.rs` |
| P4 | team-graphics-engineer | `crates/acme-render-wgpu/src/atlas_pages.rs` only (Lead already declared `mod atlas_pages;`). Until P5 wires it, put `#![allow(dead_code)]` at the top of the module with a `// P5 removes` comment. |

## Wave 2 (after wave 1): P2 lifecycle integration test, P5 wgpu integration.
## Wave 3: P6 apps / e2e / benchmark. Lead: docs.
