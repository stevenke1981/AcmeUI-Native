---
name: human-acceptance
description: Human-like acceptance testing of a native Windows desktop app. The agent launches the real build, drives it the way a person would with real pointer glides, clicks, drags, wheel and key presses, reads every screenshot, and records PASS/FAIL evidence with the tested SHA. Use after unit and Gallery tests pass, before merging a UI or interaction change, when a change claims keyboard, pointer, focus, scroll or visual behaviour, or when the user asks to "operate the computer", "verify like a human", run 仿人類驗收, or take screenshot evidence. Windows only. It is DPI-aware and sends arrow keys with the extended-key flag, so results are not false negatives.
---

# Human-like Acceptance

Unit tests prove the logic. This skill proves that a person sitting at the machine gets the intended result. It is the step that turns "manual Gallery NOT_RUN" into evidence, but only for what was actually exercised.

The driver is `scripts/human_input.py`. It needs only the Python standard library and Win32 through ctypes: no PIL, no pyautogui, no external tools.

## When the result can be trusted

A real-window run is valid only if all of the following hold. Check them first and report BLOCKED if any fails:

1. `python scripts/human_input.py --status` reports `desktop unlocked: True`. A locked, minimised-RDP or secure (UAC) desktop cannot be driven.
2. The app under test is the build at the SHA you report. Build it yourself (`cargo build --locked -p <app>`), launch that exact binary, and record the commit.
3. The target window is not elevated (UIPI blocks input into elevated windows), or the driver runs elevated too.
4. Nothing else is fighting for focus. Close or avoid windows you would type into by accident. **Never type into a window you did not launch**, such as the user's editor holding an open file.
   The driver enforces this: before every input or `shot` step it checks the target is still the foreground window, and before every pointer step it checks the point is over the target. If another window came to the front mid-run, it aborts with `aborted: ... no input sent` instead of clicking or capturing someone else's window. Treat an abort as "the desktop is busy": do not add `focus` steps to force your way back; find out what took focus first.

## Why naive automation lies

Two Windows behaviours make generic tools report that a working app "ignores input":

- **DPI scaling.** At 125% or 150%, a process that is not DPI aware moves the cursor in logical pixels while screenshots are in physical pixels, so clicks land at 1/scale of the target. One tool aimed at (161,593) and landed at (107,395) at 150%. The driver makes the whole process per-monitor DPI aware (V2) before using any coordinate, and fails loudly if the cursor does not reach the target.
- **Extended keys.** Arrows, Home/End, PageUp/PageDown, Insert/Delete need `KEYEVENTF_EXTENDEDKEY`. Without it Windows reports numpad keys; with NumLock on, the app receives digits and "arrow navigation is broken". The driver sends scancodes with the correct flag.

If a step "does nothing", first suspect the harness. Reproduce with this driver before filing an app bug, and record the tool problem separately (see `references/evidence-template.md`, "Tooling note").

## Workflow

1. **Plan the journey before touching the mouse.** Write the table you intend to fill in: step, input, expected visible result. Derive it from the task contract's user journey and expected behaviour. Include the negative cases the contract promises, for example "disabled item does not react" or "Esc leaves without scrolling".

2. **Build and launch the exact binary.**
   ```sh
   git rev-parse --short HEAD
   cargo build --locked -p acme-ui-gallery
   ./target/debug/acme-ui-gallery.exe &
   ```

3. **Fit the window to the screen.** Off-screen pixels cannot be clicked or captured:
   ```sh
   python skills/human-acceptance/scripts/human_input.py --list
   python skills/human-acceptance/scripts/human_input.py --title "AcmeUI Gallery" place:0,0,1500,1030 shot:docs/team/<batch>/gui/00-start.png
   ```

4. **Read the screenshot (open the PNG) and pick coordinates from it.** Coordinates are physical pixels relative to the window's top-left, the same grid as the screenshot. Never guess coordinates from source code.

5. **Drive one small group of steps, then capture.**
   ```sh
   H=skills/human-acceptance/scripts/human_input.py
   G=docs/team/<batch>/gui
   python $H --title "AcmeUI Gallery" click:161,342 wait:0.5 park key:f6 shot:$G/10-f6.png key:right shot:$G/11-right.png
   ```
   - Use `park` after clicks: it moves the pointer away so hover styling does not mask the state you want to see.
   - Use `wait:` after actions that animate or load.
   - For longer runs put steps in a file and use `--script steps.txt`.

6. **Look at every screenshot before the next step.** Compare it with the expected column. A step is PASS only if the screenshot shows the expected state. "The command ran" is not evidence.

7. **Look beyond the checklist.** A person notices things the contract did not ask about: content jumping under the pointer, unreadable text, clipped labels, missing glyphs (tofu boxes), focus you cannot see, English strings in zh-TW UI. Record each as a finding with its screenshot. Do not silently fix scope.

8. **Clean up.** Close the app with WM_CLOSE (the window's close button, or `autogui-control window close "<title>"` if available) and confirm the process exited. Delete screenshots that came from a broken harness run, so they cannot be mistaken for evidence.

9. **Write the evidence file** `docs/team/<batch>/GUI.md` from `references/evidence-template.md`, and link it from the batch REPORT. Commit the PNGs you cite.

## Step reference

| Step | Meaning |
|---|---|
| `place:X,Y,W,H` | Move and resize the window (physical px) |
| `move:X,Y` | Glide the pointer (eased, about 170 ms) |
| `click:X,Y` / `dclick:X,Y` / `rclick:X,Y` | Glide, settle, then click |
| `drag:X1,Y1,X2,Y2` | Press, glide, release |
| `wheel:X,Y,N` | N wheel notches (positive = up) |
| `key:NAME` | A key or chord: `key:f6`, `key:down`, `key:ctrl+a`, `key:shift+tab` |
| `type:TEXT` | Unicode text; in a script file quote spaces: `type:"你好 world"` |
| `wait:S`, `park`, `focus` | Pause, move the pointer off the window, re-activate the window |
| `shot:PATH.png` | Screenshot of the window rectangle |

## What this does not prove

Be explicit in the evidence; never round these up to PASS:

- **IME composition** (for example Windows 注音). `type:` injects committed Unicode and bypasses the IME. Composition needs a human at the keyboard. Per AGENTS.md, never claim Traditional Chinese IME works without manual validation.
- **Screen readers**, touch, pen, multi-monitor, or a DPI other than the one tested.
- **Visual review of animation or timing**, except what a still frame shows.
- **Other machines, GPUs or Windows versions.**

Report each as NOT_RUN with the reason.

## AcmeUI-Native specifics

- **Gallery app:** package `acme-ui-gallery` (`apps/acme-gallery`). The window title is `AcmeUI Gallery`. `acme-gallery` is a different, older package.
- **Keyboard groups:** page-scoped (F6/F7 per page). Click the category in the left navigation first, then press the group key.
- **Evidence location:** `docs/team/roadmap-NNN/GUI.md` plus `gui/*.png`. Record the tested SHA (the PR head) even if the branch merges later.
- **Theme coverage:** use the toolbar Dark/Light and theme-pack buttons to cover light, dark and at least one other pack when the change is visual.
