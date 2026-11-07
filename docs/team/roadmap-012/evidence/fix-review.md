# ROADMAP 012 P2 fix-wave independent recheck

Reviewer: existing `/root/review_b` (code-reviewer, Codex gpt-6.1-terra).
Scope: only A1/A2/B1/B2/B3 under `TASK-FIX.md`; no new whole-project review.
Candidate: HEAD `271e5e552c6f02d0856ca8fdd7b0a502c6eb8a09` plus the dirty
source files hashed in `fix-source-manifest.json`.

Final reviewer result: **APPROVE**. Both public-definition files are real
modules and covered by formatter traversal. Renderer state fields are private
to the implementation subtree. LRU selection requires an entry on the page;
demand is independent of stale age, while the caller guard still prevents
current-frame eviction. Pending-request and entry/page counting documentation
matches the implementation and focused tests. No unresolved concrete defect.
No checks were rerun by the reviewer; it inspected source and supplied evidence.
Real GPU runtime: NOT_RUN.

## Reproduction boundaries

- A1 RED: coverage assertion over Cargo formatter stdout fails because neither
  types.rs nor state.rs appears; GREEN includes both after module conversion.
- A2: direct pinned-toolchain check passed before the change. The review's
  isolated formatting failure was not reproduced; this is recorded as a
  passing control, not fabricated RED (`fix-a2-before.json`). Both modules
  subsequently pass Cargo and direct formatter checks.
- B1: the old existence-of-aged-entry predicate already excludes empty pages.
  The new empty-page/earlier-frame control passes before the fix. The RED test
  additionally pins the requested private demand selector as occupancy-only,
  independent of entry age (None versus Some(1), `fix-b-red.log`). Current-frame
  protection remains the caller's responsibility and existing tests pass.
- B2/B3 require documentation alignment; their focused contract tests pass both
  before and after. No behavior failure is claimed for these documentation fixes.
