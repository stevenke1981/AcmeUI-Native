//! Renderer backend selection policy.
//!
//! This module is pure: it parses the `ACME_RENDER_BACKEND` value and maps a
//! preference plus the compiled feature set onto a [`RendererPlan`]. It never
//! touches a window, a GPU, or a software surface, so every rule is unit-tested
//! without hardware.

use std::env::VarError;

/// Environment variable that selects the renderer backend.
pub const RENDER_BACKEND_ENV: &str = "ACME_RENDER_BACKEND";

/// Accepted values, listed in error messages.
const VALID_VALUES: &str = "auto, gpu, cpu";

/// Error text used when `cpu` is requested but the feature is not compiled in.
pub(crate) const CPU_FEATURE_REQUIRED: &str =
    "CPU backend requires the acme-platform/cpu-renderer feature";

/// Which renderer a window should use, as requested by the environment.
///
/// - `Auto`: GPU first, CPU fallback when the `cpu-renderer` feature is compiled in.
/// - `Gpu`: prefers the GPU; still falls back to CPU when the feature is compiled in
///   (identical to `Auto`).
/// - `Cpu`: forces the CPU software renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RenderBackendPreference {
    #[default]
    Auto,
    Gpu,
    Cpu,
}

impl RenderBackendPreference {
    /// Parse a raw `ACME_RENDER_BACKEND` value.
    ///
    /// Matching is case-insensitive and ignores surrounding whitespace. A missing
    /// or blank value means [`RenderBackendPreference::Auto`]. Any other value is
    /// rejected with a message that lists the valid values.
    pub fn from_env_value(value: Option<&str>) -> Result<Self, String> {
        let Some(raw) = value else {
            return Ok(Self::Auto);
        };
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Ok(Self::Auto);
        }
        if trimmed.eq_ignore_ascii_case("auto") {
            Ok(Self::Auto)
        } else if trimmed.eq_ignore_ascii_case("gpu") {
            Ok(Self::Gpu)
        } else if trimmed.eq_ignore_ascii_case("cpu") {
            Ok(Self::Cpu)
        } else {
            Err(format!(
                "invalid {RENDER_BACKEND_ENV} value {raw:?}; expected one of: {VALID_VALUES}"
            ))
        }
    }

    /// Read `ACME_RENDER_BACKEND` from the process environment.
    ///
    /// An invalid value is logged with `tracing::warn!` and treated as `Auto`,
    /// so a typo never silently forces a backend the user did not ask for.
    pub fn from_env() -> Self {
        Self::from_var_result(std::env::var(RENDER_BACKEND_ENV))
    }

    fn from_var_result(result: Result<String, VarError>) -> Self {
        resolve_render_backend(result, Self::Auto)
    }
}

/// Resolve the effective [`RenderBackendPreference`] from the environment variable
/// `ACME_RENDER_BACKEND` and the application's [` crate::Application::render_backend`]
/// preference.
///
/// - An explicitly set, valid `ACME_RENDER_BACKEND` value (`auto`, `gpu`, `cpu`) wins.
/// - When `ACME_RENDER_BACKEND` is unset or blank, `app_preference` is used.
/// - When `ACME_RENDER_BACKEND` is set to an invalid value (or non-Unicode), a warning
///   is logged and [`RenderBackendPreference::Auto`] is returned.
pub(crate) fn resolve_render_backend(
    env_result: Result<String, VarError>,
    app_preference: RenderBackendPreference,
) -> RenderBackendPreference {
    match env_result {
        Ok(value) => {
            if value.trim().is_empty() {
                return app_preference;
            }
            match RenderBackendPreference::from_env_value(Some(&value)) {
                Ok(preference) => preference,
                Err(message) => {
                    tracing::warn!(%message, "ignoring render backend setting; using auto");
                    RenderBackendPreference::Auto
                }
            }
        }
        Err(VarError::NotPresent) => app_preference,
        Err(VarError::NotUnicode(_)) => {
            tracing::warn!(
                "{RENDER_BACKEND_ENV} is not valid Unicode; expected one of: {VALID_VALUES}; using auto"
            );
            RenderBackendPreference::Auto
        }
    }
}

pub(crate) fn resolve_render_backend_from_env(
    app_preference: RenderBackendPreference,
) -> RenderBackendPreference {
    resolve_render_backend(std::env::var(RENDER_BACKEND_ENV), app_preference)
}

/// The concrete renderer strategy for one window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RendererPlan {
    /// Try the GPU; on failure, use the CPU software renderer.
    GpuThenCpu,
    /// Use the GPU only; a failure is reported as an error.
    GpuOnly,
    /// Use the CPU software renderer directly.
    CpuOnly,
    /// CPU was forced, but this build does not include the `cpu-renderer` feature.
    CpuUnavailable,
}

/// Map a preference and the compiled feature set onto a [`RendererPlan`].
///
/// `cpu_compiled` is `true` when the `cpu-renderer` feature is enabled. It is a
/// parameter so both branches can be tested in any feature configuration.
pub(crate) fn plan_renderer(
    preference: RenderBackendPreference,
    cpu_compiled: bool,
) -> RendererPlan {
    match (preference, cpu_compiled) {
        (RenderBackendPreference::Cpu, true) => RendererPlan::CpuOnly,
        (RenderBackendPreference::Cpu, false) => RendererPlan::CpuUnavailable,
        (RenderBackendPreference::Auto | RenderBackendPreference::Gpu, true) => {
            RendererPlan::GpuThenCpu
        }
        (RenderBackendPreference::Auto | RenderBackendPreference::Gpu, false) => {
            RendererPlan::GpuOnly
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_preference_is_auto() {
        assert_eq!(
            RenderBackendPreference::default(),
            RenderBackendPreference::Auto
        );
    }

    #[test]
    fn missing_or_blank_value_means_auto() {
        assert_eq!(
            RenderBackendPreference::from_env_value(None),
            Ok(RenderBackendPreference::Auto)
        );
        assert_eq!(
            RenderBackendPreference::from_env_value(Some("")),
            Ok(RenderBackendPreference::Auto)
        );
        assert_eq!(
            RenderBackendPreference::from_env_value(Some("   \t ")),
            Ok(RenderBackendPreference::Auto)
        );
    }

    #[test]
    fn accepts_each_backend_case_insensitively_with_whitespace() {
        assert_eq!(
            RenderBackendPreference::from_env_value(Some("auto")),
            Ok(RenderBackendPreference::Auto)
        );
        assert_eq!(
            RenderBackendPreference::from_env_value(Some(" GPU ")),
            Ok(RenderBackendPreference::Gpu)
        );
        assert_eq!(
            RenderBackendPreference::from_env_value(Some("Cpu")),
            Ok(RenderBackendPreference::Cpu)
        );
    }

    #[test]
    fn rejects_typos_and_lists_valid_values() {
        let error = RenderBackendPreference::from_env_value(Some("cpuu"))
            .expect_err("typo must be rejected");
        assert!(
            error.contains("\"cpuu\""),
            "message names the bad value: {error}"
        );
        assert!(
            error.contains("auto, gpu, cpu"),
            "message lists valid values: {error}"
        );
    }

    #[test]
    fn rejects_value_that_is_only_partially_valid() {
        assert!(RenderBackendPreference::from_env_value(Some("cpu gpu")).is_err());
        assert!(RenderBackendPreference::from_env_value(Some("software")).is_err());
    }

    #[test]
    fn var_result_maps_unset_and_invalid_to_auto() {
        assert_eq!(
            RenderBackendPreference::from_var_result(Err(VarError::NotPresent)),
            RenderBackendPreference::Auto
        );
        assert_eq!(
            RenderBackendPreference::from_var_result(Err(VarError::NotUnicode(
                std::ffi::OsString::from("x"),
            ))),
            RenderBackendPreference::Auto
        );
        assert_eq!(
            RenderBackendPreference::from_var_result(Ok("bogus".into())),
            RenderBackendPreference::Auto
        );
    }

    #[test]
    fn var_result_honours_valid_values() {
        assert_eq!(
            RenderBackendPreference::from_var_result(Ok("cpu".into())),
            RenderBackendPreference::Cpu
        );
        assert_eq!(
            RenderBackendPreference::from_var_result(Ok("gpu".into())),
            RenderBackendPreference::Gpu
        );
    }

    #[test]
    fn plan_forces_cpu_when_compiled() {
        assert_eq!(
            plan_renderer(RenderBackendPreference::Cpu, true),
            RendererPlan::CpuOnly
        );
    }

    #[test]
    fn plan_reports_unavailable_cpu_without_feature() {
        assert_eq!(
            plan_renderer(RenderBackendPreference::Cpu, false),
            RendererPlan::CpuUnavailable
        );
    }

    #[test]
    fn plan_auto_and_gpu_fall_back_to_cpu_only_when_compiled() {
        for preference in [RenderBackendPreference::Auto, RenderBackendPreference::Gpu] {
            assert_eq!(plan_renderer(preference, true), RendererPlan::GpuThenCpu);
            assert_eq!(plan_renderer(preference, false), RendererPlan::GpuOnly);
        }
    }

    #[test]
    fn resolve_render_backend_uses_app_preference_when_env_unset_or_blank() {
        for app_pref in [
            RenderBackendPreference::Auto,
            RenderBackendPreference::Gpu,
            RenderBackendPreference::Cpu,
        ] {
            assert_eq!(
                resolve_render_backend(Err(VarError::NotPresent), app_pref),
                app_pref
            );
            assert_eq!(resolve_render_backend(Ok("".into()), app_pref), app_pref);
            assert_eq!(
                resolve_render_backend(Ok("   \t ".into()), app_pref),
                app_pref
            );
        }
    }

    #[test]
    fn resolve_render_backend_explicit_valid_env_overrides_every_app_preference() {
        let env_cases = [
            ("auto", RenderBackendPreference::Auto),
            (" AUTO ", RenderBackendPreference::Auto),
            ("gpu", RenderBackendPreference::Gpu),
            (" GPU ", RenderBackendPreference::Gpu),
            ("cpu", RenderBackendPreference::Cpu),
            (" Cpu ", RenderBackendPreference::Cpu),
        ];
        let app_prefs = [
            RenderBackendPreference::Auto,
            RenderBackendPreference::Gpu,
            RenderBackendPreference::Cpu,
        ];
        for (raw_env, expected) in env_cases {
            for app_pref in app_prefs {
                assert_eq!(
                    resolve_render_backend(Ok(raw_env.into()), app_pref),
                    expected,
                    "env={raw_env:?} must override app_pref={app_pref:?}"
                );
            }
        }
    }

    #[test]
    fn resolve_render_backend_invalid_env_keeps_auto_fallback() {
        for app_pref in [
            RenderBackendPreference::Auto,
            RenderBackendPreference::Gpu,
            RenderBackendPreference::Cpu,
        ] {
            assert_eq!(
                resolve_render_backend(Ok("invalid-backend".into()), app_pref),
                RenderBackendPreference::Auto
            );
            assert_eq!(
                resolve_render_backend(
                    Err(VarError::NotUnicode(std::ffi::OsString::from("bad"))),
                    app_pref
                ),
                RenderBackendPreference::Auto
            );
        }
    }
}
