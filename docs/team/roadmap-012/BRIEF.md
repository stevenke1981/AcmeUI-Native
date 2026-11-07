# ROADMAP 012 — Brief for implementers

Read `AGENTS.md` first. Base: master `84c7290` (ROADMAP 011 merged).
Branch `sdk/roadmap-012-split-atlas-preedit`; do not switch branches.

Scope (owner-approved, 2026-11-07):

- **A** Split `crates/acme-render-wgpu/src/lib.rs` (~1690 lines) into modules. Pure move.
- **B** Glyph atlas follow-ups from `docs/status/todos.md` (ROADMAP 011 section):
  per-format allocators, mid-frame eviction of untouched pages, starvation when
  every page holds only fresh entries. Design first: `docs/team/roadmap-012/SPEC-B.md`.
- **D** TextInput: an IME preedit wider than the remaining field does not scroll
  into view (`docs/team/roadmap-005/REPORT.md` line 36).

## Hard rules

- Edit only your write set. No `git add/commit/stash/checkout/switch`. No new
  dependencies, no `Cargo.toml`/`Cargo.lock` edits. Never `cargo clean`.
- Public API: no removals or signature changes. Additions only when your task
  section or the SPEC lists them.
- Test-first for behavior changes (B, D): add the failing test, run it, show
  RED, then implement GREEN. A is behavior-neutral: no test edits except moving
  the inline `mod tests` to a file unchanged.
- Run only your crate: `cargo test --locked -p <crate>`,
  `cargo clippy --locked -p <crate> --all-targets -- -D warnings`,
  `cargo fmt -p <crate>`. Other agents edit other crates concurrently; if their
  in-progress code breaks your build, report it rather than editing it.
- Files < 800 lines, functions < 50 lines where practical. No hardcoded theme
  colors in widgets. No byte offsets as text-cursor API.
- Report (< 50 lines): files, RED excerpt (B, D), GREEN command + exit code +
  test count, deviations with reasons, NOT_RUN items.

## A — wgpu split (team-rust-engineer, Haiku)

Write set: `crates/acme-render-wgpu/src/**` except `atlas_pages.rs`, `batch.rs`,
`golden.rs`, `*.wgsl` (do not edit those).

- Move items out of `lib.rs` into cohesive private modules (suggested: `types.rs`
  for the public data types and errors, `surface.rs` for surface/device-loss
  helpers, `images.rs`, `instances.rs` for primitive→instance conversion,
  `resources.rs`, `renderer/*.rs` with several `impl Renderer` blocks, and
  `tests.rs` for the inline test module). `lib.rs` keeps `mod` declarations and
  `pub use` re-exports so every public path stays identical.
- Every file < 800 lines; `lib.rs` < 150 lines.
- Acceptance: identical public API. Prove it: before editing, run
  `cargo doc --locked -p acme-render-wgpu --no-deps` and save the sorted list of
  generated HTML file names under `target/doc/acme_render_wgpu/` to
  `docs/team/roadmap-012/evidence/a-doc-before.txt`; after the split save
  `a-doc-after.txt`; they must be identical. Same test count before/after
  (record both). `cargo check --locked -p acme-render-wgpu --all-targets` and
  `-p acme-ui` must pass.

## D — preedit scroll (team-rust-engineer, Haiku)

Write set: `crates/acme-textinput/src/lib.rs`, `crates/acme-textinput/tests/scroll_caret.rs`,
new `crates/acme-textinput/tests/preedit_scroll.rs`.

- `scroll_caret_into_view` only considers the committed caret; during
  composition the preedit (inserted at `state.cursor`) can extend past the right
  edge and is clipped. Make it keep the composition visible: the preedit
  cursor (`preedit_cursor` end, or preedit end when `None`) must lie inside
  `[0, visible]` content-local, and the scroll range must account for the
  preedit width. When the preedit is empty, behavior must be byte-for-byte as
  today (existing `scroll_caret.rs` and `ime_dpi.rs` pass unchanged).
- Also check `render_text_input` and `ime_caret_area` use the same geometry so
  the IME candidate window follows the scrolled position. Investigate and REPORT
  (do not silently change) whether preedit glyphs themselves are drawn; section
  6 of `render_text_input` only draws underlines.
- Tests at scales 1.0/1.25/1.5/2.0 like `ime_dpi.rs`; include a long
  Traditional Chinese preedit (e.g. 注音 composition string) in a narrow field.

## B — atlas follow-ups

Wave 1: design only (team-solution-architect, Sonnet, read-only) → `SPEC-B.md`.
Lead approves the spec, then implementers start after A lands (B may touch
wgpu code that A moves).
