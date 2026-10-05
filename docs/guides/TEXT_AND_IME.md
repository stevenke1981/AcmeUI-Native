# Text and IME

Dependencies: cosmic-text, fontdb, unicode-segmentation, arboard, winit IME events.

## Edit history in the Gallery

Open Inputs > TextInput, click the field, and enter text. Ctrl+Z undoes the last
edit; Ctrl+Y or Ctrl+Shift+Z reapplies it. Ctrl+A followed by Ctrl+Backspace or
Ctrl+Delete removes the selection as one undoable edit. A new edit discards the
old redo branch. Key release does not apply the shortcut again.

Readonly inputs reject undo/redo without consuming history. The application can
enable editing again and continue using the same history. Automated Gallery
event tests cover this path without a native window; Windows visual and IME
validation still require the manual checklist.

P0 rules:
- Preedit text is visually distinct.
- Candidate window tracks caret.
- Commit is one undo transaction.
- Escape cancels composition.
- Cursor works on grapheme clusters, never UTF-8 bytes.
- CJK fallback is tested before TextInput release.
