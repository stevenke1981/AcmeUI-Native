# AcmeUI-Native Agent Team

English · Agent-first · Human-friendly

## Mission

Evolve the existing Rust-native UI framework into a production-ready SDK through bounded, verified changes. Preserve the current architecture, public compatibility, and the human user's ability to understand and control the application. Audit before extending. A role file is a work contract, not proof that an autonomous worker was launched.

## Start here

1. Read `AGENTS.md`, `docs/specs/spec.md`, `docs/specs/plan.md`, the relevant entries in `docs/status/todos.md`, and the current PR/CI state.
2. Assign the Lead and only the specialists needed for this batch. Load their files under `.agents/roles/`.
3. Read `docs/agent-team/WORKFLOW.md`, select a bounded task, and reserve exact files using `ownership.json` as routing guidance.
4. Implement and verify. Hand off with the fields in `handoff.example.json` and attach actual logs.

## Roles

| Role | Contract | Primary responsibility |
|---|---|---|
| Lead | `roles/lead.md` | Scope, ownership, integration, evidence |
| Architecture | `roles/architecture.md` | Contracts, compatibility, ADRs |
| Rust Core | `roles/rust-core.md` | Identity, reconciliation, layout, text |
| GPU/Rendering | `roles/rendering.md` | CPU/GPU rendering and resource safety |
| UI/UX | `roles/ui-ux.md` | Human workflows and component behavior |
| Accessibility | `roles/accessibility.md` | Semantics, keyboard, assistive technology |
| Verification | `roles/verification.md` | Reproduction, tests, CI, truthful handoff |

## Execution modes

Use independent workers only when the host provides them and their write sets do not overlap. Otherwise execute named role passes sequentially and record `single-agent-role-passes`. Do not claim seven independent agents or an independent review merely because seven role files exist. These files do not install workers or alter model routing, providers, accounts, permissions, or billing.

## Human control

Accept Chinese requests; retain identifiers, commands, and source documentation in English. Summarize progress in the user's language. Keep destructive changes, releases, new paid services, and account/permission changes explicit. Never trade keyboard usability, clarity, or accessibility for automation convenience.

## First batch

See `docs/agent-team/BATCH-001.md` for the bounded CPU hardening audit. The roadmap is in `docs/agent-team/ROADMAP.md`; uninspected areas remain unaudited, not implicitly complete.
