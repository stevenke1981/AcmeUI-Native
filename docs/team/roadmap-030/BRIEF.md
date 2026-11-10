# ROADMAP 030 — Mobile builder contract coverage

Task MOBILE-CONTRACTS-030, P2, 2026-11-10 Asia/Taipei. Base
`913abe681b690bf69568395ec823b2dec2304d5f`, fetched clean master after PR #38
merged with ten hosted checks PASS. Branch sdk/roadmap-030-mobile-contract-tests.
Execution **single-agent-role-passes** (Lead, UI/UX, Verification), sole writer,
author self-review; independent review NOT_RUN. Owner authorizes PR/merge after
local checks, hosted CI and MERGEABLE; third/final slice. Read AGENTS/team/roles,
spec/plan/todos/roadmap, 021/027/028/029 reports and every mobile builder's source.

OBSERVED: historical P3 says “16 modules”, but lists 15 mobile_* builders. The
actual public mobile/mod.rs inventory is 20 modules: those 15 plus action_sheet,
bottom_nav, bottom_sheet, pull_to_refresh and search_bar. Tests exist but default
fields/layout-kind/child-count checks are inconsistent and have no inventory
guard. This task certifies structural/default contracts, not legacy interaction
or rendering completeness.

Acceptance:
1. One named default/shape test for every public module (all public default
   fields, layout kind and widget/layout root child count), plus an inventory
   guard generated from the same test registry to catch future omissions.
2. Gallery regression confirms all 15 P3 examples are present once, bound to real
   retained layout and keep representative root IDs across live theme changes.
   Preserve existing behavior suites. Fresh native CPU Gallery smoke evidence.
3. Correct todo counts and add an explicit coverage matrix in REPORT.md, with
   manual/legacy limitations unchanged. No production widget/API expansion.
4. Locked UI all-feature/mobile-only and Gallery tests, strict Clippy/fmt,
   actual CPU profile and workspace check/test at completed-P3-test milestone.

Exact sole-writer leases: crates/acme-ui/src/mobile/mod.rs,
crates/acme-ui/src/mobile/mobile_contract_tests.rs,
apps/acme-gallery/src/main.rs, apps/acme-gallery/src/mobile_contract_tests.rs,
docs/status/todos.md, docs/agent-team/ROADMAP.md,
docs/team/roadmap-030/{BRIEF.md,REPORT.md,evidence.json,gallery-mobile-contracts.png},
docs/team/roadmap-030/logs/* (generated evidence only, serialized).
Read dependencies: all twenty component modules, widgets/layout/retained/theme
and Gallery demos; no shared contracts/manifests/deps. Rollback: revert batch.
Non-goals: legacy component rewrites, general w_full percentage-unit migration,
Row focus/typed semantics, animation/drag, runtime/release/performance claims.
Manual human Gallery, Windows 注音, physical DPI/touch, GPU and screen readers
NOT_RUN. No behavioral RED is expected for a coverage-only batch.
