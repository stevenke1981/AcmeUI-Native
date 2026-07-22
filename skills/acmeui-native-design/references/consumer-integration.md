# Consumer application integration

Use this reference when the AcmeUI-Native app lives outside the framework repository.

## Dependency discovery

Inspect the app manifest before changing dependencies:

```powershell
rg -n 'acme-(ui|widgets|theme|platform|layout)' Cargo.toml Cargo.lock
```

Common development form:

```toml
[dependencies]
acme-ui = { path = "../AcmeUI-Native/crates/acme-ui" }
acme-platform = { path = "../AcmeUI-Native/crates/acme-platform" }
```

Common reproducible Git form:

```toml
[dependencies]
acme-ui = { git = "https://github.com/stevenke1981/AcmeUI-Native.git", rev = "<commit>" }
```

Do not replace a pinned `rev` with a moving branch unless the user explicitly requests an
upgrade. When using local path dependencies, report that the build is not independently
reproducible without the sibling checkout.

## Screen implementation

1. Trace the app's `Application` implementation, message enum, and root `WidgetNode` builder.
2. Preserve message wiring while changing composition. Every clickable control must map to
   an existing or intentionally added message.
3. Use `Theme` semantic colors and spacing/radius tokens. Put application art direction in
   a theme pack or app-level semantic mapping, not inside reusable widgets.
4. Prefer occupied layout nodes over floating overlays for permanent navigation, headers,
   and status regions. Test the smallest supported window and long Traditional Chinese text.
5. If background events (tray, hotkey, worker channel) must wake an idle app, confirm the
   platform event loop schedules polling or sends a user event; drawing code alone is not
   an event pump.

## Validation ladder

Read the package name from the manifest (`cargo metadata --no-deps`) rather than guessing it.
Run the smallest relevant checks, then widen:

```powershell
cargo test -p <consumer-package> <targeted_test_name>
cargo check -p <consumer-package> --all-targets
cargo test -p <consumer-package>
```

For framework changes also run the affected AcmeUI crate tests. Finally launch the built
Windows executable and record PID, HWND, title, client size, input result, and screenshot.
Do not claim IME, tray, global hotkey, DPI, or minimized-window behavior from compile tests.

Treat a requested `640x410` as **client area** unless the product specification explicitly
says outer window size. Record both client and outer rectangles during QA. For tray and
global-hotkey acceptance on Windows:

1. Launch the exact executable and lock onto its PID plus exact window title; do not keep
   rediscovering a window by fuzzy title.
2. Record the initial visible, non-zero HWND and verify the client rectangle.
3. Trigger the configured hotkey while the window is focused, unfocused, and hidden to the
   tray. Confirm an observable application state transition for press and release.
4. Hide and restore through the tray. Confirm the same process remains alive, the intended
   main HWND is restored, and taskbar/window styles return correctly.
5. Invoke tray Exit and confirm that exact process terminates. A menu click alone is not
   success.
6. Capture the restored window by its locked HWND. Reject zero-size, title-bar-only, blank,
   or wrong-page images before saving QA evidence.

If the app supports resizing, test the declared minimum client size plus one long
Traditional Chinese sample. If it is fixed-size, verify the resize policy instead of
inventing a smaller breakpoint.
