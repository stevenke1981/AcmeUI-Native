# ROADMAP NNN — Real-window GUI verification

- **Tested source:** `<short SHA>` (`<branch>` / PR #<n>). Build: `cargo build --locked -p <package>` (debug|release).
- **Environment:** Windows <edition> <build>; <W>x<H> at <scale>% (`human_input.py --status`); NumLock <on/off>; console session unlocked.
- **Operator:** <model or person>, driving real pointer and keyboard input with `skills/human-acceptance/scripts/human_input.py`. The driver is DPI-aware and sends scancodes with extended-key flags.
- **Run date (UTC):** <timestamp>

## Results

| # | Input | Expected | Observed | Screenshot |
|---|---|---|---|---|
| 1 | Mouse click "<target>" | <visible result> | PASS / FAIL: <what you saw> | `gui/01-....png` |
| 2 | <key or chord> | <visible result> | PASS / FAIL | `gui/02-....png` |

Mark a row PASS only when its screenshot shows the expected state.

## Findings

Record things a person would notice that the contract did not ask about, such as jumps, contrast, clipping, glyphs, focus visibility, and language mix.

1. **<title>** — <what happens, where (screenshot), likely cause, severity>. Follow-up: <ROADMAP item or "none">.

## Tooling note (only if the harness misbehaved)

<The tool, the symptom, how it was isolated, and which runs were excluded from the evidence. This is not an app defect.>

## NOT_RUN

<IME composition (e.g. Windows 注音), screen reader, touch, other DPI, other monitors or GPUs, and anything else not exercised, each with the reason.>
