# ROADMAP 031: render reliability

- Base: 1f23c910f05d7c975094df1e3b2be65ee2437ec9 (origin/master verified).
- Mode: independent agents, isolated author worktrees; lead integrates.
- User authorized implementation, commits, push, PR, mainline merge and release build. Actual default branch is master. No crates.io publication or version rename in this batch.
- Preserve original untracked files. Risk: cross-backend rendering contracts and skipped-frame lifecycle.

## Task graph and leases

1. Lead / A1: reproduce lost one-shot glyph uploads on skipped surface acquisition; fix upload delivery and bounded redraw scheduling. Exact leases: crates/acme-render-wgpu/src/renderer/frame.rs, crates/acme-render-wgpu/src/atlas_pages.rs (test-only COPY_SRC usage for GPU readback), crates/acme-render-wgpu/src/surface.rs, crates/acme-render-wgpu/src/tests.rs, crates/acme-platform/src/lib.rs, crates/acme-platform/src/scheduling.rs. Additional test files only after explicit lease update. Lead owns docs/team/roadmap-031/* and docs/status/todos.md.
2. Scene worker / A4: core structural preflight plus CPU and GPU batch validation. Exact leases: crates/acme-core/src/scene.rs, crates/acme-render-cpu/src/lib.rs, crates/acme-render-wgpu/src/batch.rs. Shared Scene contract has this single writer. Request lead integration for frame.rs guard before any resource mutation. Own worker evidence in docs/team/roadmap-031/scene-*.
3. Independent Rust reviewer: read-only integrated diff and evidence; no implementation lease.
4. Lead: targeted checks, CPU evidence profile, workspace milestone checks, Gallery evidence, PR/CI, merge, exact merged-SHA release build and checksums.

## Acceptance

- A1: skipped frames cannot lose new glyph data; successful later acquisition draws the latest scene without requiring unrelated input. Exercise Timeout/Occluded and Outdated decisions in deterministic regressions; live fault injection explicitly separate.
- A4: common validation rejects clip/layer underflow and unclosed stacks before backend mutations; CPU retains allocation limits. Cover both backends with regression evidence.
- Preserve public signatures where possible, painter order, theme rules, and existing valid scenes.
- No blocking independent findings; required hosted checks successful for candidate; merge only expected PR head without bypass.
- Build release from merged master with pinned Rust 1.88.0 and --locked; record artifact hashes. Native IME/DPI/screen-reader acceptance stays NOT_RUN.

## Rollback / exclusions

Revert the batch commits through a new PR if needed; no destructive reset. GPU group opacity, AccessKit native adapter, input placeholders and crates.io naming remain separate follow-ups. Do not claim project-wide production readiness.

- A6 added within the affected scheduler: current `now + interval` can panic for Duration::MAX and zero interval can spin. Define/test bounded polling (1 ms to 24 h), runtime interval changes, disable, and coexistence with frame retries. This is a concrete adjacent failure, not speculative hardening.

- Gallery regression lease (Lead): apps/gallery/src/retained_ids_tests.rs; execute real frame generation for every page and both themes, including a repeated frame, then structural validation and GPU batch compilation. Desktop screenshot is BLOCKED by agent-desktop private-file DACL check; do not alter host permissions or substitute UI input tools.
