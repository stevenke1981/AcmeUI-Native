# API Direction

```rust
fn view(state: &AppState) -> impl View {
    column()
        .gap(8.0)
        .padding(16.0)
        .child(label("AcmeUI Native"))
        .child(button("save", "儲存設定").primary().on_click(AppMessage::Save))
}
```

Rules: stable IDs, message-driven events, semantic theme tokens, typed lengths, no public wgpu/platform types.
