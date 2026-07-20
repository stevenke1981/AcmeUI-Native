# Architecture

```text
Application
  -> declarative view descriptions
  -> reconciliation
  -> retained node tree
  -> Taffy layout
  -> hit-test + accessibility + scene
  -> paint batching
  -> wgpu render passes
  -> OS surface
```

## Dirty flags
Style, layout, paint, semantics, children. Propagate only as far as required.

## Overlay layers
Main, Floating, Modal, Tooltip, Drag, Debug.

## Coordinates
Use typed PhysicalPixels, LogicalPixels, WindowSpace and LocalSpace. Avoid naked f32 across boundaries.
