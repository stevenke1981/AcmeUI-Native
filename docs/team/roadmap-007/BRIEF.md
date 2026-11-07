# ROADMAP 007 — Packaging, versioning, migration, license review (BATCH-007)

ROADMAP row 007: "SDK examples, versioning, packaging, migration guidance" with
acceptance "external consumer build, release build, compatibility and license
review". Base: `sdk/roadmap-007-packaging` (on ROADMAP 006). Rust 1.88.0
pinned, Windows MSVC host. Read `AGENTS.md`, `docs/guides/SDK_GUIDE.md`,
`docs/CHANGELOG.md`.

Already done: `examples/sdk-consumer` (path deps, CI job `sdk-consumer`),
crate metadata (description/keywords/categories/license via workspace),
SDK_GUIDE §7 pre-1.0 policy.

## Problems (OBSERVED)

- The workspace declares `license = "MIT OR Apache-2.0"` but the repository
  ships **no license texts** (`git ls-files | grep -i licen` is empty).
- `deny.toml` has `[bans]`/`[sources]` only; no license policy is enforced.
- Internal dependencies are `{ path = ... }` without `version`, so no crate can
  be packaged (`cargo package` rejects path-only deps). Apps are not marked
  `publish = false`.
- Version drift: CHANGELOG has `v0.2.1`–`v0.2.3` headings, the workspace says
  `0.2.0`, and there are no git tags. Several "Unreleased" entries are breaking.
- No migration guide, no release checklist; git dependency use is "not verified".

## Shared contract (Lead, committed before work starts)

Root `Cargo.toml` `[workspace.dependencies]` gains one entry per library crate:
`acme-core = { path = "crates/acme-core", version = "0.2.0" }` (all 14).
**Do not bump the version** — a release/version bump is the owner's decision;
document the proposal instead.

## Work items

| ID | Role | Write set |
|---|---|---|
| P1 | team-build-engineer | `crates/*/Cargo.toml`, `apps/*/Cargo.toml` (manifests only), new `scripts/check_package.py` |
| P2 | team-license-reviewer | new `LICENSE-MIT`, `LICENSE-APACHE`, `deny.toml`, new `docs/guides/LICENSES.md` |
| P3 | team-technical-writer | new `docs/guides/MIGRATION.md`, new `docs/guides/RELEASE.md` |

Lead owns: root `Cargo.toml`, `Cargo.lock`, `.github/workflows/*`,
`docs/CHANGELOG.md`, `docs/status/todos.md`, `docs/guides/SDK_GUIDE.md`, README.

### P1 — packageable crates

- Every internal dependency in `crates/*/Cargo.toml` and `apps/*/Cargo.toml`
  becomes `acme-x.workspace = true` (keep `optional = true` /
  `features` / `default-features` where present; dev-dependencies too).
- `publish = false` on every `apps/*` package. Libraries keep `publish` unset.
- `Cargo.lock` must not change (`git diff --exit-code -- Cargo.lock`).
- `scripts/check_package.py`: runs `cargo package --workspace --locked
  --no-verify --exclude <each app>` (or per-crate in dependency order if the
  multi-package form is unsupported on 1.88 — find out and report), prints the
  file list size per crate, exits non-zero on failure. Must also flag any
  packaged `.crate` > 10 MiB. Only stdlib Python.
- Verify a **git** dependency from a scratch crate outside the repo (in
  `%TEMP%`, not committed): `acme-ui = { git = "file:///D:/AcmeUI-Native", rev =
  "<HEAD sha>" }` + `acme-core`, `cargo check`. Report PASS/FAIL with output.
  (HEAD is the Lead's contract commit.)
- Never run `cargo publish` (not even `--dry-run` against crates.io auth).

### P2 — license review

- Add standard `LICENSE-MIT` (copyright line: `Copyright (c) 2026 AcmeUI-Native
  contributors`) and the full Apache-2.0 text as `LICENSE-APACHE`.
- From `cargo metadata --format-version 1 --locked` (all targets in
  `deny.toml [graph]`), list every third-party license expression and the
  crates using it. Write `[licenses]` in `deny.toml` (cargo-deny 0.18+ schema:
  `version = 2`? check the installed version's docs; `allow = [...]`,
  `confidence-threshold`, exceptions only if needed, each justified in a
  comment). Install `cargo install --locked cargo-deny` if missing (dev tool,
  same as CI) and run `cargo deny check licenses` and `cargo deny check bans
  sources` → must PASS.
- `docs/guides/LICENSES.md`: project license, third-party license table
  (expression → count → notable crates), copyleft/notice obligations (e.g.
  MPL-2.0, Unicode, OFL for any bundled fonts — check whether any font or
  asset is bundled: `git ls-files` for ttf/otf/png/svg), and how the check runs.

### P3 — migration and release docs

- `docs/guides/MIGRATION.md`: from `0.2.0` to the next release. For each
  breaking or behaviour change in the "Unreleased" CHANGELOG sections (e.g.
  `PlatformError::NoWindows/Renderer`, `RenderBackendPreference`, retained
  binding / `to_layout_with_binding`, accessibility semantics changes,
  `move_cursor_to_offset` grapheme snapping, GlyphAtlas deferred eviction in
  `begin_frame`, benchmark CLI): what changed, who is affected, before/after
  code taken from the actual source (verify every identifier with grep), and
  required action. No invented APIs.
- `docs/guides/RELEASE.md`: release checklist (version bump in
  `[workspace.package]` and `[workspace.dependencies]`, CHANGELOG section,
  tag `vX.Y.Z`, CI green incl. sdk-consumer and package check, license check,
  manual validations from `MANUAL_VALIDATION.md` and their status, crates.io
  publish order derived from the dependency graph). Explain the version drift
  (CHANGELOG `v0.2.1`–`v0.2.3` vs workspace `0.2.0`, no tags) and propose
  `0.3.0` as the next version because Unreleased has breaking changes — as a
  proposal for the owner, not done.

## Hard rules

Only your write set. No git add/commit/stash/checkout/reset, no cargo clean,
`CARGO_TARGET_DIR=target/<id>`, pinned toolchain, `--locked`, no new crate
dependencies, no GUI launch, no network publish. Honest PASS/FAIL/NOT_RUN.

## Report format

```
ID / role / files changed
Summary (≤10 lines)
Commands → status (+ output tails)
Patches requested for shared files
Risks / NOT_RUN / follow-ups
```
