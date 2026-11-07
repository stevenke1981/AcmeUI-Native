# ROADMAP 005 Report — DPI and IME acceptance, automated part (2026-11-07)

Brief: [BRIEF.md](BRIEF.md). **Manual 注音 sessions at 100/125/150/200 %
Windows scaling remain NOT_RUN and require human sign-off**
(`docs/guides/MANUAL_VALIDATION.md` checklist B).

## Team

| Item | Owner | Result |
|---|---|---|
| D1 | team-test-engineer / Haiku | Gallery at 5 scales, 32 pages: logical drift 0.0 px; found 2 field bugs |
| D2 | team-test-engineer / Haiku | CPU renderer exact physical mapping; 1 AA finding kept ignored |
| D3 | team-test-engineer / Haiku | Mixed 注音/ASCII caret geometry; found 3 IME bugs |
| E1 | team-rust-engineer / Haiku | Fixed the 2 Gallery field bugs; `scroll_caret_into_view` |
| Lead | Opus 5.5 | Fixed the 3 IME bugs, docs, commits |
| Review | team-independent-reviewer / Sonnet | APPROVE_WITH_FIXES: HIGH-1 (caret not drawn at the end of overflowing text), MEDIUM-1 (other cursor setters mid-char), MEDIUM-2 (password masked offset panic) fixed by Lead with mutation-checked tests; AA corner finding relabelled as design choice |

## Bugs found by the new tests and fixed

1. `move_cursor_to_offset(1)` on "注音" stored a mid-character byte offset;
   `ime_caret_area` then panicked (`byte index 1 is not a char boundary`).
   Now snaps to a grapheme boundary.
2. `handle_text` left the preedit set after committing; a stale composition
   stayed drawn. Now cleared.
3. Escape blurred without cancelling the composition (TEXT_AND_IME.md P0).
   Now the first Escape cancels the preedit.
4. Gallery TextInput was laid out 21 px tall (one-line marker label), clipping
   text to ~5 px; caret bottom 8 px below the field. Now control height (40).
5. `scroll_offset` was never written; with 300 characters the caret sat at
   x=2197 past the field edge at x=1076. Now kept in view.

## Not fixed (tracked)

- CPU renderer sharp-corner pixel coverage uses `max` instead of the area
  product (`dpi.rs`, ignored test); AA rule needs an owner decision.
- A preedit wider than the remaining field does not scroll into view.

## Measured

Logical layout and IME caret drift vs 1.0: 0.0000 px at 1.25/1.5/1.75/2.0 on
all 32 Gallery pages (shaping uses logical metrics; scale only reaches raster
placement — this does not prove physical glyph quality).
