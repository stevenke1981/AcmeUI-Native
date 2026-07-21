# P0-001 — Ordered Display List Migration

| Field | Value |
|---|---|
| ID | P0-001 |
| Title | Ordered Display List (Scene → adjacent batches → wgpu) |
| Status | Planned (design complete) |
| Owner | render / core |
| Risk | T2 Controlled |
| Depends on | none |
| Blocks | correct scroll/overlay stacking, future layers, multi-backend |

## 1. Current State

### 1.1 Canonical-but-unused Scene (`acme-core`)

File: `crates/acme-core/src/scene.rs`

- `PaintCommand`: `SolidRect`, `RoundedRect`, `Border`, `Text { glyphs }`, `PushClip`, `PopClip`
- `Scene { commands: Vec<PaintCommand> }` — append-only ordered list
- `ClipStack` with intersect / underflow / unbalanced checks
- Geometry is logical (`Rect<Logical>`, `Color`, `Radius`)
- **No live consumer** in Gallery or `Renderer`

### 1.2 Live display list: `Frame` (`acme-render-wgpu`)

```rust
Frame {
  clear: [f32; 4],
  quads: Vec<Quad>,                 // unclipped
  clipped_quads: Vec<ClippedQuad>,  // per-item clip rect
  text: Vec<TextRun>,               // optional clip + PreparedText
}
```

`Renderer::render(&Frame)` pipeline:

1. Map all `quads` → instances (Vec order OK within this bucket only)
2. Group `clipped_quads` in `HashMap<[u32;4], …>` → **non-deterministic order**
3. Group `text` in `HashMap<(AtlasFormat, Option<clip>), …>` → **non-deterministic**
4. Draw: **all regular quads → all clip batches → all text batches**

This destroys painter's order. Overlay quads can end up below text, tooltip quads mixed with background.

### 1.3 Call graph (live)

```
Application::frame(FrameContext) -> Frame
  gallery / playground / acme-gallery / benchmark
  acme-textinput::render_text_input(&mut Frame)
  gallery RenderCtx { frame: &mut Frame }
acme-platform Runtime
  -> renderer.render(&frame)
```

## 2. Target Design

### 2.1 Goals

1. **Painter's algorithm**: draw order == command order (except adjacent merges)
2. **Deterministic batching**: no `HashMap` iteration for draw order
3. **Backend-neutral Scene** stays in `acme-core` (no wgpu types)
4. **Incremental migration**; each phase independently verifiable

### 2.2 Target types (`acme-core`)

```rust
/// Backend-neutral ordered draw command (logical pixels).
pub enum DrawCommand {
    Quad(QuadPrimitive),
    Text(TextPrimitive),
    PushClip(Rect<Logical>),
    PopClip,
    BeginLayer(LayerParams),
    EndLayer,
}

pub struct QuadPrimitive {
    pub rect: Rect<Logical>,
    pub color: Color,
    pub radius: Radius,
    pub border_width: f32,
    pub border_color: Color,
}

pub struct TextPrimitive {
    pub origin: Point<Logical>,
    pub color: Color,
    pub glyphs: Vec<GlyphDraw>,
    pub uploads: Vec<AtlasUpload>,
}

pub struct Scene {
    clear: Color,
    commands: Vec<DrawCommand>,
}
```

### 2.3 Adjacent batch compiler

New file `crates/acme-render-wgpu/src/batch.rs`:

- Walk `DrawCommand` list in order
- Maintain `ClipStack` (reuse `acme_core::ClipStack`)
- Merge adjacent compatible commands into same `RenderBatch`
- NEVER merge across clip/layer boundaries
- Output: `Vec<RenderBatch>` in deterministic order

```rust
pub struct RenderBatch {
    pub pipeline: BatchPipeline, // Quad | TextAlpha | TextRgba
    pub clip: Option<[f32; 4]>,
    pub start: u32,
    pub count: u32,
    pub layer: u32,
}
```

## 3. Migration Phases

### Phase 1 — Backend-neutral DrawCommand (additive)

- Add `DrawCommand`, `QuadPrimitive`, `TextPrimitive`, `GlyphDraw`, `GlyphFormat` to `acme-core`
- Keep `PaintCommand` working via conversion helpers
- Extend `Scene` with `clear` field and `push(DrawCommand)`
- **No behavior change** in apps

### Phase 2 — Adjacent batching

- New `batch.rs` with `compile_scene()` function
- `scene_from_frame()` bridge converting Frame → Scene (legacy bucket order)
- Unit tests for batching rules, determinism, clip boundaries

### Phase 3 — Renderer consumes Scene

- `Renderer::render_scene(&Scene)` — new hot path
- `Renderer::render(&Frame)` → bridge → `render_scene` (deprecated)
- Remove `HashMap` grouping from hot path
- Gallery still builds `Frame` (visual parity via bridge)

### Phase 4 — Gallery emits Scene

- Gallery `RenderCtx` uses `&mut Scene` instead of `&mut Frame`
- scroll content emits proper `PushClip`/`PopClip`
- `acme-textinput::render_text_input` emits ordered commands
- Remove `Frame` public API after all callers migrated

## 4. Acceptance Gates

| Phase | Automated | Manual |
|---|---|---|
| 1 | `cargo test -p acme-core` | — |
| 2 | batch unit tests (determinism, order, clip) | — |
| 3 | render-wgpu + platform tests | Gallery launch, visual parity |
| 4 | workspace app checks + textinput tests | scroll stacking, overlays, text input |

## 5. Rollback

- Phases 1-2 are additive (no behavior change)
- Phase 3: keep `render(&Frame)` bridge
- Phase 4: single commit revert for apps
- No data migration required

## 6. Risks

| Risk | Mitigation |
|---|---|
| More draw calls from adjacent-only batching | Measure RenderStats.draw_calls; later optional same-state rejoin |
| Glyph coordinate space confusion | Document in GlyphDraw; match current TextRun math exactly |
| Core vs acme-text glyph type duplication | Single converter module with tests |
| Gallery scroll quads currently unclipped | Intentional fix in Phase 4; screenshot before/after |
