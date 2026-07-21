# Rendering Pipeline

1. Acquire surface texture
2. Resolve dirty layout
3. Update glyph atlas
4. Build scene
5. Batch paint commands
6. Upload changed buffers
7. Execute render passes
8. Submit and present
9. Record diagnostics

Paint commands: SolidRect, RoundedRect, Border, TextRun, Image, PushClip, PopClip, PushTransform, PopTransform.
