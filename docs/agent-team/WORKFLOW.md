# Delivery Workflow

## States

`AUDIT -> PLAN -> WORK -> VERIFY -> REVIEW -> DRAFT_PR -> MERGE_READY`

An existing Draft PR may precede verification; its presence is not acceptance. Failed verification returns the task to WORK. Environmental blockers remain BLOCKED. The Lead integrates only the files assigned to the batch. A maintainer decides whether to merge and release.

## Task contract

Each task records an ID, observed problem, reproduction or explicit hypothesis, priority, owner, exact base SHA, exact write set, read dependencies, expected behavior, non-goals, checks, manual scenarios, and rollback. Prefer one observable end-to-end improvement to a broad rewrite.

Severity: P0 blocks builds, loses user data, or exposes a confirmed critical safety failure; P1 breaks a normal workflow or a required reliability contract; P2 improves maintainability, ergonomics, or measured performance. Assess actual impact rather than inflating severity from labels.

## File ownership and concurrency

Use `.agents/ownership.json` for routing. Before editing, the Lead records a lease for exact files and verifies the current branch head. Shared files include root manifests/lockfile, the Scene contract, platform event loop, and the core Gallery entrypoint. Only one writer holds a shared-file lease at a time. Other roles submit a patch or design note; they must not overwrite the owner. Separate worktrees do not eliminate integration conflicts. Update refs with an expected SHA; on a mismatch, inspect and reconcile, never force blindly.

## Audit

Read source and tests, not only README claims. Inventory APIs, dependency edges, CI, and known manual gaps. Label each finding OBSERVED, REPRODUCED, HYPOTHESIS, HISTORICAL, or NOT_INSPECTED. Record files/lines and an immutable source SHA. Old TODOs may already be fixed; check the current source and reports before reopening them.

## Verification

Start with the modified crate and direct dependents. Keep new checks separate from the existing changed-file gates. At a milestone, run workspace checks/tests and release compilation under the pinned toolchain. Use `--locked` so tests cannot silently alter dependency resolution. Do not replace a failing lockfile guard with a lockfile rewrite in CI.

Suggested commands for this CPU slice:

```sh
cargo fmt -p acme-render-cpu -- --check
cargo test --locked -p acme-render-cpu
cargo clippy --locked -p acme-render-cpu --all-targets --no-deps -- -D warnings
cargo check --locked -p acme-platform --no-default-features
cargo check --locked --workspace --all-targets
cargo test --locked --workspace
```

Use `python scripts/verify_sdk.py --profile cpu` for machine-readable logs. `--dry-run` lists commands and records NOT_RUN; it never generates a PASS. A missing tool is BLOCKED, not skipped. Keep pre-existing unrelated warnings separate from new warnings; never silently disable diagnostics.

## Evidence contract

Attach the exact tested commit (and merge SHA when CI tests a merge), dirty status, OS, toolchain, command argument vector, timestamps, exit code, and raw log. Preserve initial failing logs and matching regression-test names. Use PASS, FAIL, BLOCKED, NOT_RUN, or N/A with a reason. A script syntax check is not a Rust compilation; a CPU snapshot is not a native-window or accessibility test; cross-target library compilation is not runtime support. Do not copy a pass from an ancestor onto a newer commit.

## Review and delivery

Review bounds, state transitions, allocation behavior, API compatibility, regressions, documentation, and the evidence itself. A different worker or maintainer provides independent review; author self-review must be labeled. Deliver files, commit/PR, checks, remaining risks, and one concrete next batch. Keep the PR draft when manual acceptance is still missing. Never merge, publish binaries, alter branch protection, or add security exceptions as an incidental test fix.

## Practical acceptance

For a normal patch: one relevant regression, targeted checks, readable failure behavior, and a small self-review are sufficient to submit a Draft PR. Require broader checks at integration boundaries, not on every keystroke. Add human Windows/DPI/IME/screen-reader testing only where the changed behavior needs it, but do not represent untested behavior as proven.
