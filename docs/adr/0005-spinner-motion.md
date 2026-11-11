# ADR 0005: Host-driven spinner motion

2026-11-11. Accepted for ROADMAP 040 before implementation.

Add `animation(elapsed: std::time::Duration, reduced_motion: bool)` to the
foundation SpinnerBuilder and MobileLoaderBuilder, forwarding one clock/policy
pair through composition. Existing constructors and public fields remain.
Private clock/policy fields are compatible with builders' existing private
phantom fields (external struct literals were already unavailable).

Replace the font-dependent arrow with seven themed circular Row surfaces in
a fixed-size Stack. Dot positions rotate once per second, with a tapered tail
and a gap. Reduced motion ignores elapsed time and renders the same zero pose.
Defaults use zero elapsed time until the host supplies a clock. This is a
documented representation/geometry change from Label to Stack; consumers must
not rely on the old internal Label variant. Tone now resolves semantic tokens.

The host owns scheduling and motion preference; widgets create no threads or
timers. Gallery demonstrates Application background polling at 33 ms only on
the Mobile page in normal motion. Its explicit preference is an application
demo, not an OS accessibility-preference detector. No new WidgetNode variant,
Scene transform, animation-engine dependency or public platform type is needed.
