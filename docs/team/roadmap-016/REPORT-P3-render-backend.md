# ROADMAP 016 — Track P3 Report: App-Selected Renderer Backend

## Changed Files

- `crates/acme-platform/src/backend.rs`
- `crates/acme-platform/src/lib.rs`

## Public API Diff (`acme-platform`)

```rust
pub trait Application: 'static {
    // ...
    /// Preferred rendering backend when `ACME_RENDER_BACKEND` is not explicitly set.
    fn render_backend(&self) -> RenderBackendPreference {
        RenderBackendPreference::Auto
    }
    // ...
}
```

## Summary of Changes

1. **`Application::render_backend`**:
   - Added `fn render_backend(&self) -> RenderBackendPreference` to `Application` defaulting to `RenderBackendPreference::Auto`.
2. **Environment vs. App Preference Resolution (`resolve_render_backend`)**:
   - Added pure resolver `resolve_render_backend(env_result, app_preference)` in `crates/acme-platform/src/backend.rs`:
     - Explicitly set, valid `ACME_RENDER_BACKEND` (`auto`, `gpu`, `cpu`, case-insensitive) wins over `app_preference`.
     - Unset (`VarError::NotPresent`) or blank (`""` / whitespace) uses `app_preference`.
     - Invalid or non-Unicode `ACME_RENDER_BACKEND` logs `tracing::warn!` and falls back to `RenderBackendPreference::Auto` (preserving existing behaviour).
   - Wired `resolve_render_backend_from_env(self.app.render_backend())` into `Runtime::resumed` and `initialize_renderer`.
   - When `RenderBackendPreference::Cpu` is selected without the `cpu-renderer` feature compiled in, `plan_renderer(RenderBackendPreference::Cpu, false)` returns `RendererPlan::CpuUnavailable`, surfacing `CPU_FEATURE_REQUIRED` via `PlatformError::NoWindows` without panicking.

## Verification & Test Counts

| Command / Check | Status | Result |
| --- | --- | --- |
| `cargo test -p acme-platform` | PASS | 46 passed, 0 failed (unit tests); 0 failed, 1 ignored (doc-test) |
| `cargo test -p acme-platform --no-default-features` | PASS | 46 passed, 0 failed (unit tests); 0 failed, 1 ignored (doc-test) |
| `cargo clippy -p acme-platform --all-targets -- -D warnings` | PASS | 0 warnings, 0 errors |
| `cargo check --workspace --all-targets` | PASS | All workspace crates and app targets compiled cleanly |
| `rustfmt --edition 2024 --check <changed files>` | PASS | 0 formatting diffs |
