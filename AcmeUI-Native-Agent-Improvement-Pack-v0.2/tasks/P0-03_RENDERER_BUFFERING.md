# P0-03 Renderer Buffering and Batching

## 實作

- Persistent quad/glyph instance buffers。
- 容量不足時 1.5–2 倍增長，不每幀 recreate。
- Frame ring 或 staging belt，避免覆蓋 GPU in-flight data。
- clipped quads 按 clip rect batch，不再一 quad 一 buffer。
- text runs 按 atlas format + clip batch。
- atlas 只上傳 dirty regions。
- 新增 RenderStats：buffers_created、bytes_uploaded、draw_calls、quads、glyphs、atlas_hit_rate。
- `eprintln!` 改 tracing。

## 驗收

- 穩定畫面第二幀不建立新的 instance buffer。
- 1000 quads draw calls 顯著低於 1000。
- 同 clip quads 單一 batch。
- device/surface loss 測試可重建 pipeline、atlas、buffers。
- benchmark 記錄 cold/warm frame-build 與 render preparation。
