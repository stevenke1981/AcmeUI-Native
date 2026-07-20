# Text and IME

Dependencies: cosmic-text, fontdb, unicode-segmentation, arboard, winit IME events.

P0 rules:
- Preedit text is visually distinct.
- Candidate window tracks caret.
- Commit is one undo transaction.
- Escape cancels composition.
- Cursor works on grapheme clusters, never UTF-8 bytes.
- CJK fallback is tested before TextInput release.
