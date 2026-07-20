# Manual Validation Checklists

Honest human sign-off for behaviors that **unit tests cannot prove** on real
Windows hardware / IME UI.

**Rules**

- Do **not** mark either checklist done without a dated human sign-off below.
- Automated coverage listed here is **evidence of wiring only**, not product
  completion of the manual path.
- `AGENTS.md`: never claim Traditional Chinese IME works without checklist B
  signed off.
- STATUS / todos / CHANGELOG must stay aligned with the status lines in this
  file until a human completes the steps.

| Area | Status |
|------|--------|
| A. GPU device loss recovery | **NOT YET MANUALLY VALIDATED** |
| B. Traditional Chinese 注音 IME | **NOT YET MANUALLY VALIDATED** |

---

## A. GPU device loss recovery

### Status

**NOT YET MANUALLY VALIDATED**

### Prerequisites

- Windows 10/11 host (local, not WSL-only).
- Gallery app builds and runs (`cargo run -p acme-gallery` or project equivalent).
- Discrete and/or integrated GPU present and driving the Gallery window.
- Prefer a machine where a brief GPU reset is acceptable (see warnings).

### What automated tests already cover

These prove pure logic and app wiring. They do **not** replace interactive
device-loss on real hardware.

| Coverage | Where (indicative) |
|----------|--------------------|
| Pure surface status machine (`suspended` / `device_lost` / acquire → Skip, DeviceLost, Reconfigure, Rendered) | `acme-render-wgpu` `resolve_surface_action` unit tests |
| Recovery bookkeeping (`gpu_epoch`, clear device-lost flag after recovery) | `complete_recovery_state` / related unit tests |
| Atlas clear contract (cache hit → `clear()` → re-upload + generation bump) | `acme-text` `atlas_clear_forces_reupload*` |
| `Application::on_gpu_recovered` default noop + override registration | `acme-platform` unit tests |
| Gallery / Playground override clears CPU `GlyphAtlas` after recovery | app `on_gpu_recovered` implementations |
| Device-lost callback / uncaptured Internal+OOM → `AtomicBool` wiring | `register_device_error_handlers` + related tests |
| Ignored GPU smoke (adapter/device + handler registration when GPU available) | `#[ignore]` smoke in render crate — run only with intentional `--ignored` |

### Manual steps (candidates)

> **Warning:** Forcing a GPU reset or disabling the display adapter is
> **disruptive**. It may blank the desktop, reset other apps, or require a
> reboot. Prefer a non-production machine. Do not run these steps on a shared
> or critical workstation without expectation of interruption.

1. Start Gallery; open a page with visible text (labels + TextInput content).
2. Confirm baseline: text glyphs render; note any log sink used by the app.
3. Trigger real device loss using **one** of:
   - **Device Manager**: disable the GPU adapter driving the window mid-run,
     then re-enable (highly disruptive).
   - **TDR / driver reset**: if your environment can force a Timeout Detection
     and Recovery event safely (vendor tools / known repro); still disruptive.
   - **Future debug hotkey**: only if a documented in-app “simulate device lost”
     hotpath exists in the build under test — do not invent one for this
     checklist. Prefer real loss when validating end-to-end recovery.
4. Observe process behavior during and after the event (no silent exit, no hard
   crash preferred).
5. After recovery path runs, confirm:
   - Window still alive or cleanly recoverable.
   - Text glyphs visible again (not permanently blank rectangles).
   - Logs (if enabled) show device-lost detection, recovery / `on_device_lost`
     path, and app `on_gpu_recovered` side effects (e.g. atlas clear).

### Pass / fail criteria

| Criterion | Pass | Fail |
|-----------|------|------|
| Process stability | No crash / panic abort during loss + recovery | Crash, hang requiring kill, or uncaught panic |
| Text after recovery | Glyphs readable on previously text-heavy UI | Blank text persists after recovery settles |
| Recovery signals | Logs or debug output show lost → recover → `on_gpu_recovered` (or equivalent documented markers) | No recovery path observed; app stuck black/frozen without recovery |
| User agency | Operator can continue interacting after recovery | Permanent unusable surface without restart (unless restart is an explicitly accepted fallback and documented) |

Optional notes (not required for pass): multi-monitor, hybrid GPU switch,
sleep/resume combined with loss.

### Sign-off (human only)

| Field | Value |
|-------|-------|
| Date | |
| Operator | |
| Machine / GPU | |
| Trigger method | |
| Build / commit | |
| Result (PASS/FAIL) | |
| Notes / logs path | |

Until this table is filled with a real PASS, status remains
**NOT YET MANUALLY VALIDATED**.

---

## B. Traditional Chinese IME (注音)

### Status

**NOT YET MANUALLY VALIDATED**

### Prerequisites

- Windows 10/11 with **Microsoft 注音** (or equivalent Traditional Chinese
  Bopomofo IME) installed and selectable in the language bar / Win+.
- Gallery running; navigate to the **Text Input / IME** demo page.
- Focus the Gallery `TextInput` (click field so it accepts keyboard/IME).
- Prefer a build that wires `ime_cursor_area` / `set_ime_cursor_area` (current
  Gallery does field-relative caret rect caching).

### What automated tests already cover

Architecture and geometry only — **not** real 注音 candidate UI placement.

| Coverage | Where (indicative) |
|----------|--------------------|
| IME preedit / commit event model (incl. `WindowId` on detailed variants) | `acme-platform` event types + dispatch tests |
| TextInput preedit/commit / password masking / grapheme cursor | `acme-textinput` unit tests |
| Caret geometry: `ime_caret_area` (scroll, line height, insert advances x) | `acme-textinput` unit tests |
| `resolve_ime_cursor_area` prefers app rect over mouse fallback | `acme-platform` unit tests |
| Gallery `ime_cursor_area` wiring (field origin + padding + caret cache) | Gallery `refresh_ime_caret_cache` / `Application::ime_cursor_area` |

### Manual steps

1. Switch OS input method to **注音** (Bopomofo).
2. Focus Gallery TextInput; confirm OS IME is active for that window.
3. Type a composition sequence (e.g. 注音 keys that show preedit before commit).
4. Confirm:
   - Preedit appears in-field (visually distinct if the demo styles it).
   - **Candidate window** tracks near the **text caret**, not the mouse pointer
     (move mouse far from the caret and re-compose to check).
5. Commit a **繁體** character/word from candidates; confirm committed text is
   correct and editable.
6. Backspace: delete by **grapheme** (one 字 per backspace when appropriate),
   not by raw UTF-8 bytes mid-cluster.
7. If a password field exists in the demo: mask committed text; IME should not
   leak composition into an unmasked side channel in the UI.
8. Optional stress: mixed 中英, cancel composition (Esc if supported), refocus
   away/back, second window if multi-window build is under test.

### Explicit project rule (`AGENTS.md`)

> Never claim Traditional Chinese IME works without manual validation.

Until this checklist is signed off PASS:

- Do not say “注音 works”, “IME complete”, or “manually validated TC IME” in
  README, STATUS, CHANGELOG, release notes, or agent summaries.
- Todos may keep architecture / wiring items checked; the **manual** todo must
  stay open.
- Prefer phrasing: “IME architecture + caret wiring; **manual 注音 validation
  still pending**.”

### Pass / fail criteria

| Criterion | Pass | Fail |
|-----------|------|------|
| Composition | Preedit visible and updates during 注音 input | No preedit / wrong field / crash |
| Candidate placement | Candidates near caret; not glued to mouse when mouse is elsewhere | Candidates only at mouse or off-window useless position |
| Commit | 繁體 commit lands in field as one logical edit | Garbage, missing chars, or wrong target widget |
| Backspace | Grapheme-safe delete of committed CJK | Splits code units / corrupts string |
| Password (if present) | Masked display for committed secrets | Plaintext leak of password content in the field UI |
| Honesty | Sign-off table completed only after above | Claiming success without this checklist |

### Sign-off (human only)

| Field | Value |
|-------|-------|
| Date | |
| Operator | |
| Windows version / IME name | |
| Build / commit | |
| Result (PASS/FAIL) | |
| Notes / screenshots path | |

Until this table is filled with a real PASS, status remains
**NOT YET MANUALLY VALIDATED**.

---

## Related references

- `AGENTS.md` — TC IME claim restriction
- `STATUS.md` — “Automated Only (manual still pending)”
- `todos.md` — open manual 注音 item; surface recovery automated vs manual
- `TEXT_AND_IME.md` — product rules for preedit/caret/commit
- `final.md` — hardening notes (recovery + IME caret wiring, unvalidated 注音)
