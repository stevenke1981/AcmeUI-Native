# ROADMAP 056 — Real-window GUI verification

- **Tested source:** `ad96f13` (the PR #72 head, later merged as `8735e43`), debug build of `acme-ui-gallery`, launched in a real native window.
- **Environment:** Windows 11 Pro, 1920×1080 at 150% scaling, NumLock on, unlocked console session (`autogui-control session status`: ready).
- **Operator:** Claude Opus 5.5 (Lead), driving the app like a person: real cursor movement, real mouse clicks, real key presses.
  - Input came from a DPI-aware Win32 `SendInput` helper with scancodes and the extended-key flag.
  - Screenshots came from `rsautogui` `autogui-control capture`.

## Results

| Step | Input | Expected | Observed | Screenshot |
|---|---|---|---|---|
| 1 | Mouse click on "Layout" | Layout page opens | PASS | `gui/01-click-layout.png` |
| 2 | F6 | Menu revealed, "Keyboard menu: active", 首頁 selected, 支援 disabled | PASS | `gui/10-f6.png` |
| 3 | → | 產品 selected (Primary), dropdown opens | PASS | `gui/11-right-products.png` |
| 4 | ↓ ↓ | Child focus 桌面版 → 行動版 (`▸` marker, Primary); 企業版 disabled | PASS | `gui/12-down-down.png` |
| 5 | Enter | Status "目前目的地：行動版" | PASS | `gui/13-enter.png` |
| 6 | Esc, Esc | Child focus cleared, then "Keyboard menu: inactive"; destination kept | PASS | `gui/14-esc-esc.png` |
| 7 | Mouse click on 文件 | Status "目前目的地：文件", 文件 Primary | PASS | `gui/15-click-docs.png` |
| — | Window loses focus (another app opened) | Menu deactivates | PASS (observed incidentally) | not saved |

The ▾ and ▸ glyphs render correctly, which resolves the open glyph-coverage risk.

## Findings
1. **Scroll jumps on reveal (confirms review P2 #1).** F6, every arrow press, and even a mouse click on an already-visible menu scroll the content. In step 7 the menu row moved from y≈143 to y≈232 under the pointer. The F6 reveal puts the row flush under the toolbar, which hides the section title and instructions (steps 3–6). Fixed in ROADMAP 057: reveal only when the menu is out of view.
2. **New: the selected Gallery category is nearly unreadable.** The current category ("Layout" / "Mobile") renders as white text on a very light blue fill, visible in every screenshot. It is a likely WCAG AA contrast failure in the Gallery shell's category navigation, not in NavigationMenu. Queued as ROADMAP 060.

## Tooling note (not an app defect)
The first attempt drove input through rsautogui's MCP `hotkey` and `click_at`. Arrow keys had no effect, and pointer moves landed at about 2/3 of the target (aimed at 161,593, landed at 107,395).

- **Pointer cause:** coordinate scaling that is not DPI-aware at 150%.
- **Arrow-key cause:** arrows sent without the extended-key flag. With NumLock on, Windows reports them as numpad digits.

The same scenario passes with correctly flagged real input (table above). Those runs are excluded from the evidence. The behaviour should be reported to rsautogui.

## NOT_RUN
Windows 注音 IME composition, screen reader, touch, a second monitor or a different DPI, and the GPU-specific renderer path.
