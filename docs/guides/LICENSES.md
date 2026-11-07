# Licensing and third-party notices

Status: ROADMAP 007 / P2 (license review). Date of data: 2026-11-07, branch
`sdk/roadmap-007-packaging`. This guide describes the repository as it is; items
marked **UNKNOWN** or **OWNER DECISION** are not resolved by this document.

This document is an engineering license inventory. It is not legal advice. Items
flagged for legal or rights-holder review are listed in
[Open items](#open-items).

## 1. Project license

AcmeUI-Native is dual-licensed under **MIT OR Apache-2.0**, at your option.

| File | Content | Status |
|---|---|---|
| [`LICENSE-MIT`](../../LICENSE-MIT) | MIT License, copyright line `Copyright (c) 2026 AcmeUI-Native contributors` | Added in P2. Body compared word-for-word with a registry copy of the standard MIT text. |
| [`LICENSE-APACHE`](../../LICENSE-APACHE) | Full Apache License, Version 2.0 text including the appendix | Added in P2, unmodified. SHA-256 `cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30`, 11358 bytes, byte-identical to `https://www.apache.org/licenses/LICENSE-2.0.txt` and to 26 cached registry copies. |

The workspace declares `license = "MIT OR Apache-2.0"` in `[workspace.package]`.
Every workspace crate and app inherits it (`license.workspace = true`), except
`examples/sdk-consumer`, which sets the same string directly.

The README (`License` section) already states `MIT OR Apache-2.0`. Nothing in
the repository ships a copyright notice beyond the two license files above.

## 2. Third-party dependencies

### 2.1 Inventory method

- Source: `cargo metadata --format-version 1 --locked` with
  `CARGO_TARGET_DIR=target/p2`. The lock file is not modified.
- Scope: all 382 non-workspace package versions in `Cargo.lock` (all platforms).
- Cross-check: `cargo deny list` for the seven `[graph]` targets in `deny.toml`
  reports 345 registry crate versions. The 37 extra lock entries are
  target-specific crates outside those seven targets (for example Redox, Hermit
  and Orbital crates). They are covered by the all-targets run in section 3.
- Machine-readable inventory (generated, not committed): 626 rows, one per
  crate and license term, third-party only. Regenerate with
  `cargo deny list --format json` for the current state.

### 2.2 License expressions in the lock file

Counts are unique crate versions for each license expression as declared in the
`license` field of each crate's manifest (from `cargo metadata`). An expression
with `OR` is satisfied by any one listed alternative.

| Expression | Crates | Notable crates |
|---|---:|---|
| `MIT OR Apache-2.0` | 174 | accesskit, accesskit_consumer, accesskit_windows, and most of the Rust ecosystem |
| `MIT` | 88 | calloop, bytes, block2, cfg_aliases |
| `Apache-2.0 OR MIT` | 37 | async-channel, async-executor, async-io, async-process |
| `MIT/Apache-2.0` | 15 | android_system_properties, bitflags, foreign-types |
| `Apache-2.0` | 12 | ab_glyph, ab_glyph_rasterizer, accesskit_winit, winit (0.30), gethostname, khronos_api |
| `Zlib OR Apache-2.0 OR MIT` | 10 | bytemuck, bytemuck_derive, objc2-* (Apple), tinyvec |
| `MIT OR Apache-2.0 OR Zlib` | 8 | cursor-icon, glow, raw-window-handle, xkeysym, zune-jpeg |
| `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` | 7 | linux-raw-sys, rustix, wasip2, wit-bindgen |
| `Unlicense OR MIT` | 4 | byteorder-lite, memchr, termcolor, winapi-util |
| `Zlib` | 3 | foldhash, slotmap |
| `BSL-1.0` | 2 | clipboard-win, error-code |
| `BSD-3-Clause OR Apache-2.0` | 2 | moxcms, pxfm |
| `BSD-3-Clause OR MIT OR Apache-2.0` | 2 | num_enum, num_enum_derive |
| `Apache-2.0/MIT` | 2 | pollster, rustc-hash |
| `MIT OR Apache-2.0 OR LGPL-2.1-or-later` | 2 | r-efi |
| `Unlicense/MIT` | 2 | same-file, walkdir |
| `BSD-3-Clause` | 2 | tiny-skia, tiny-skia-path |
| `BSD-2-Clause OR Apache-2.0 OR MIT` | 2 | zerocopy, zerocopy-derive |
| `0BSD OR MIT OR Apache-2.0` | 1 | adler2 |
| `BSD-2-Clause` | 1 | arrayref |
| `Apache-2.0 AND MIT` | 1 | dpi |
| `CC0-1.0` | 1 | hexf-parse |
| `ISC` | 1 | libloading |
| `MIT OR Zlib OR Apache-2.0` | 1 | miniz_oxide |
| `Apache-2.0 OR GPL-2.0-only` | 1 | self_cell |
| `(MIT OR Apache-2.0) AND Unicode-3.0` | 1 | unicode-ident |

Total: 26 expressions, 382 crate versions.

### 2.3 Copyleft

- No dependency requires a copyleft license. The only copyleft terms are
  alternatives in `OR` expressions:
  - `self_cell`: `Apache-2.0 OR GPL-2.0-only`. The Apache-2.0 alternative is used.
  - `r-efi`: `MIT OR Apache-2.0 OR LGPL-2.1-or-later`. The MIT or Apache-2.0
    alternative is used.
- No dependency is licensed under AGPL, SSPL, or a GPL-only license without a
  permissive alternative.

## 3. License policy enforced by `deny.toml`

`deny.toml` contains a `[licenses]` section (cargo-deny 0.20.2 schema, which has
no `version` key). It allows exactly nine license terms. Each term is needed by
at least one crate with no permissive `OR` alternative:

| Allowed | Required by | Why it is needed |
|---|---|---|
| `MIT` | Most crates | Dual-license default (`MIT OR Apache-2.0`) and single-license MIT crates |
| `Apache-2.0` | Most crates | Dual-license default; ab_glyph, winit, accesskit_winit |
| `Unicode-3.0` | unicode-ident | Required term; combined with `MIT OR Apache-2.0` by AND |
| `BSD-3-Clause` | tiny-skia, tiny-skia-path | Single-license BSD |
| `BSD-2-Clause` | arrayref | Single-license BSD |
| `BSL-1.0` | clipboard-win, error-code | Single-license Boost |
| `Zlib` | foldhash, slotmap | Single-license Zlib |
| `ISC` | libloading | Single-license ISC |
| `CC0-1.0` | hexf-parse | Single-license public-domain dedication |

Not allowed, and not needed: `Apache-2.0 WITH LLVM-exception`, `Unlicense`,
`GPL-*`, `LGPL-*`. The crates that list them have permissive `OR` alternatives
that cargo-deny accepts. `Unlicense` appears only in `Unlicense OR MIT` and
`Unlicense/MIT`.

Minimality check (run by P2): removing any single allowed term makes
`cargo deny check licenses` fail with exit 4. The nine terms are therefore all
required.

No `exceptions` (per-crate overrides) and no `clarify` entries were needed. No
crate's license text had to be guessed. `confidence-threshold = 0.8` is the
cargo-deny default, stated explicitly.

## 4. Notice obligations

Permissive licenses in use (MIT, Apache-2.0, BSD-2/3-Clause, BSL-1.0, ISC, Zlib,
Unicode-3.0, 0BSD, CC0-1.0) mostly require the copyright and license notice to
travel with copies or substantial portions of the software. These are the
obligations that apply to a binary release:

| Obligation | Applies to | Status |
|---|---|---|
| Keep copyright and license notice with redistributed source | Source crates on crates.io, git dependency copies | Normally met by each upstream crate package, which carries its own license files. Not verified crate by crate. |
| Include license notice in binary or documentation distributions | Built `acme-gallery` / `acme-playground` / app binaries, any SDK binary bundle | **GAP.** No generated third-party notice file exists yet. This must be produced before a binary is distributed. |
| Apache-2.0 section 4(d): propagate upstream `NOTICE` content | Dependencies that ship a `NOTICE` file | Checked against the cargo registry cache for lock crates. Only `cfg_aliases` 0.2.2 ships `NOTICES.md`. It attributes code from tectonic_cfg_support (MIT). No Apache-2.0 `NOTICE` file is present in the lock set. |
| Unicode-3.0 notice in documentation of binary distributions | unicode-ident | Same as the notice gap above. |

Recommended action (owner decision, not done in P2): generate a
`THIRD_PARTY_NOTICES` file at release time from the lock file, with the full
license text for each license term in section 2.2, and include it in every
binary release. A tool such as `cargo-about` would do this. It is not installed,
and adding it is a new dependency, so it needs an owner decision.

## 5. Bundled assets

`git ls-files` shows no font files (`*.ttf`, `*.otf`, `*.woff`, `*.woff2`) and no
`include_bytes!` or `include_str!` of binary data. Text is rendered with the
operating system's fonts through `cosmic-text` and `fontdb`. The repository does
not redistribute any font. The Gallery screenshot shows CJK and emoji glyphs. These
are presumably rendered from system fonts, since no bundled font exists in the
repository.

Tracked binary and image assets:

| Path | Type | Origin (from git history) | License status |
|---|---|---|---|
| `docs/assets/acmeui-native-hero.webp` | README concept illustration, WebP | Commit `32bbacb` by Steven Ke, "docs: add generated README hero". `docs/status/final.md` describes it as a project-owned, text-free concept hero. | **UNKNOWN** for the generator. The docs call it project-owned, but the generating tool and its terms of use are not recorded. Owner should record this or replace the image. |
| `docs/gallery-smoke.png` | README screenshot of the Gallery app, PNG | Commit `b6db918` (initial commit). Real capture of the Gallery running on Windows. | Project-owned capture, as described in the README and `docs/reports/final.md`. Note: it shows Windows window chrome and a terminal fragment with local paths (`D:\AcmeUI-Native\target\debug\acme-gallery.exe`). Check before any publication. |
| `skills/acmeui-native-design/assets/layout-gallery.svg` | Hand-authored SVG | Commit `9a36373` | Project source. The SVG only names `Segoe UI` in `font-family`. No font file, no external reference, no embedded raster. |
| `skills/acmeui-native-design/assets/palette-gallery.svg` | Hand-authored SVG | Commit `9a36373` | Same as above. |
| `crates/acme-render-cpu/tests/golden/*.ppm` (6 files) | Golden pixel fixtures, PPM | Commit `ebe4d29`. Generated by this repository's own renderer for regression tests. | Project output. |
| `crates/acme-render-wgpu/src/*.wgsl` | Shader source | Project source | Project source. |

Also: `docs/gallery-smoke.png` and the hero are shipped as repository files, not
as crate contents. `cargo package` includes only crate directories, so these
images are not in any `.crate` file.

## 6. Theme pack palettes

`crates/acme-theme/src/packs/` contains eleven packs named after third-party
designs: Apple, Dracula, Gruvbox, Material (Material Design 3), Nord, One Dark,
Solarized, Ubuntu (Yaru), Windows 10 (Fluent), and Windows 11. Several color
values carry comments naming an upstream palette, for example `// nord6`,
`// Dracula bg`, and the Nord "Snow Storm" note.

- Color values are facts and are not copyrightable on their own. Upstream
  palettes for Nord, Dracula, Gruvbox, Solarized, and One Dark are published
  under permissive licenses. Apple, Microsoft, Canonical, and Google publish
  design guidance, not reusable code. **Not verified** for each pack.
- Brand and trademark names (Apple, Windows, Ubuntu, Material, Dracula, Nord,
  Solarized) appear in pack names and doc comments. Use as descriptive
  references to palettes is a trademark question, not a license question.
  **OWNER / LEGAL DECISION.**
- Recommended: add an attribution block with upstream links and license per pack
  (owner decision), or rename packs to generic descriptors.

## 7. How the checks run

From the repository root, with the pinned toolchain (`rust-toolchain.toml`
pins 1.88.0). `cargo-deny` is installed with `cargo install --locked cargo-deny`
(CI uses the same command, unpinned). Use `CARGO_TARGET_DIR=target/p2` to keep
build output out of the default target directory.

```sh
CARGO_TARGET_DIR=target/p2 cargo deny check licenses
CARGO_TARGET_DIR=target/p2 cargo deny check bans sources
```

Both must exit 0. `bans` reports duplicate-version warnings for many crates. This
is set to `multiple-versions = "warn"` and is not a failure.

CI state: the `audit` job in `.github/workflows/ci.yml` runs
`cargo deny check bans licenses sources`, so the license policy is enforced in CI.

## Open items

| # | Item | Owner | Status |
|---|---|---|---|
| 1 | Add `cargo deny check licenses` to the CI `audit` job. | Lead (`.github/workflows/ci.yml`) | DONE (ROADMAP 007) |
| 2 | Generate a third-party notice file (`THIRD_PARTY_NOTICES`) for binary releases. | Release owner | OPEN, GAP |
| 3 | Record the generator and its terms for `docs/assets/acmeui-native-hero.webp`, or replace it. | Owner / rights holder | UNKNOWN |
| 4 | Check that `docs/gallery-smoke.png` contains no private paths or unrelated window content before publication. | Owner | OPEN |
| 5 | Theme pack names and palette attribution (section 6). | Owner / legal | OWNER DECISION |
| 6 | Pin the cargo-deny version in CI (currently unpinned). | Lead | OWNER DECISION |
| 7 | Packaged `.crate` files contain no license text: `LICENSE-MIT` and `LICENSE-APACHE` are at the repository root, outside every crate directory. Copy them into each crate (or add a `check_package.py` rule) before publishing. | Release owner | OPEN, GAP |

## 變更歷史

| 版本 | 日期 | 內容 | 影響範圍 |
|---|---|---|---|
| v1.0 | 2026-11-07 | Initial license inventory for ROADMAP 007 P2: project license, third-party table from `cargo metadata`, deny policy, bundled asset status, open items. | `docs/guides/LICENSES.md`, `LICENSE-MIT`, `LICENSE-APACHE`, `deny.toml` |
