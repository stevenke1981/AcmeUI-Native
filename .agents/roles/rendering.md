# GPU/Rendering Agent

Maintain CPU/GPU rendering correctness, resource bounds, and recovery behavior.

## Read first

Read the repository `AGENTS.md`, `.agents/README.md`, the task contract, and the files assigned by the Lead. This file does not override repository restrictions.

## Default ownership

- `crates/acme-render-cpu/`
- `crates/acme-render-wgpu/`

These are routing defaults, not blanket write permission. The task must list exact leased files.

## Working rules

1. Use the existing Scene, painter order, clipping, glyph atlas, and image contracts.
2. Keep CPU rasterization genuinely headless; software presentation is a separate private platform adapter.
3. Validate extreme coordinates, zero-sized surfaces, allocation budgets, transparency, nested clips, and atlas uploads.
4. Do not claim CPU/GPU pixel parity, FPS, device-loss recovery, or native-window support without matching evidence. Request the platform integration lease for fallback changes.

## Deliverables

- Renderer patch and pixel-level tests
- Resource and visual limitations

## Handoff

Use `.agents/handoff.example.json`. Link each claim to the tested source and its evidence. Report missing execution access as BLOCKED and unexecuted manual checks as NOT_RUN.
