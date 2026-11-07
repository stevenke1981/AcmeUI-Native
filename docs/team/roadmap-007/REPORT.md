# ROADMAP 007 — Report

Branch `sdk/roadmap-007-packaging` (on ROADMAP 006). Windows 10, Rust 1.88.0 MSVC.

## Execution mode (actual)

| ID | Role / model | Outcome |
|---|---|---|
| Lead | Opus 5.5 | brief, `[workspace.dependencies]` contract, CI steps, CHANGELOG, SDK_GUIDE, review fixes |
| P1 | team-build-engineer / Haiku (high) | 18 manifests → `workspace = true`, apps `publish = false`, `scripts/check_package.py` |
| P2 | team-license-reviewer / Haiku (high) | `LICENSE-MIT`, `LICENSE-APACHE` (official text, sha256 `cfc7749b…`), `deny.toml [licenses]` (9 terms, each proven required), `LICENSES.md` |
| P3 | team-technical-writer / Haiku (high) | `MIGRATION.md` (M1–M13 with file:line), `RELEASE.md` (gates, publish order, version drift) |
| Review | team-independent-reviewer / Sonnet, read-only | APPROVE_WITH_FIXES |

## Evidence

- `cargo check --locked --workspace --all-targets`: 0 warnings; `Cargo.lock` unchanged.
- `cargo test --locked --workspace`: 1429 passed, 0 failed.
- `python scripts/check_package.py --allow-dirty`: 14/14 libraries PASS (reviewer: also PASS on a clean clone without the flag; negative test exits 1).
- `cargo deny check bans licenses sources`: `bans ok, licenses ok, sources ok`.
- Git dependency (`file://` URL, fixed rev) from a scratch crate: `cargo check` PASS (P1 and reviewer).
- `cargo package --workspace` does not work on cargo 1.88 with unpublished internal deps; per-crate packaging with `--config patch.crates-io.*` does (that is what the script does).

## Review findings

| ID | Sev | Disposition |
|---|---|---|
| F1 `acme-core` name taken on crates.io by an unrelated project | HIGH | documented as owner decision (RELEASE.md §8, gate G20); not renamed |
| F2 stale "not in CI / not verified" statements | MED | fixed |
| F3 `.crate` files contain no license text | MED | documented (LICENSES.md open item 7) — owner decision |
| F4 CHANGELOG route_action wording | LOW | fixed |
| F5 local temp paths in LICENSES.md; dev-dep false positive; line drift | LOW | paths removed; others accepted |

## Owner decisions open

- Next version (proposed 0.3.0) and CHANGELOG history option (RELEASE.md §2–3).
- `acme-core` crate name collision; license files inside packages.
- `move_cursor_to_offset(byte_offset)` vs AGENTS.md "no byte offsets for text cursors".
- Hero image provenance, `gallery-smoke.png` content, theme pack brand names, cargo-deny pin.

## NOT_RUN

`cargo publish` (never), registry consumer build, GitHub-URL git dependency,
MIGRATION.md snippets compile check, native GUI/IME/DPI/screen-reader validation.
