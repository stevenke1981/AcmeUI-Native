# ADR 0006: Typed retained activation of existing containers

2026-11-11. Accepted for ROADMAP 041 before implementation.

Add `AccessibilityAction::activation_message<M: Clone>(root, binding) -> Option<M>`.
Only Click/Activate resolves a current Button or Row/Column/Stack handler using
the exact retained ID. Require a complete binding from the same current,
successfully reconciled description; reject synthetic and missing IDs. The
caller dispatches the returned typed message once. No coordinate synthesis,
parent bubbling or fallback to another child/handler occurs. Existing platform
event routing remains source-compatible for consumers that still use it.

Containers with a handler become retained focus candidates and AccessKit
Buttons advertising Focus/Click. Passive containers stay Groups (or existing
parent-specific Cell roles). Names concatenate nonempty descendant Labels
through passive containers; nested controls keep independent names/actions.
There is no new disabled-container field: builders that suppress handlers
remain passive. Existing Button disabled behavior is preserved.

Use the existing original Gallery (already dependent on accessibility) for
typed keyboard activation on its Focus demo. Fix its matching container hit/
paint index contract. No public platform type, new WidgetNode variant, blanket
input router, dependency or cross-crate type erasure is needed. Real platform
AccessKit wiring and screen-reader acceptance remain separate.
