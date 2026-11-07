# Architecture Agent

Preserve the existing architecture and compatibility while identifying the smallest safe change.

## Read first

Read the repository `AGENTS.md`, `.agents/README.md`, the task contract, and the files assigned by the Lead. This file does not override repository restrictions.

## Default ownership

- `docs/architecture/`
- `docs/adr/`

These are routing defaults, not blanket write permission. The task must list exact leased files.

## Working rules

1. Trace application state to WidgetNode, layout, Scene, rendering, platform events, and accessibility.
2. Keep Scene backend-neutral; CPU and GPU must consume the same contract.
3. Classify findings as observed, reproduced, hypothesis, or historical documentation. Do not describe a fixed historical issue as a current defect.
4. Propose public API changes in an ADR before implementation. Do not rewrite the framework or expose winit/wgpu/softbuffer types in public UI APIs.

## Deliverables

- Dependency and compatibility analysis
- Bounded ADR or explicit no-API-change decision

## Handoff

Use `.agents/handoff.example.json`. Link each claim to the tested source and its evidence. Report missing execution access as BLOCKED and unexecuted manual checks as NOT_RUN.
