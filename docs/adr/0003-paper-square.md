# ADR 0003: Independent Paper square option

2026-11-11. Accepted for ROADMAP 036 before implementation.

Paper currently derives corner radius from elevation. E0 has zero corners,
but using E0 to request square corners discards the selected elevation. Add
`PaperBuilder::square(bool)` and a public `square: bool` field, default false,
consistent with the existing public options. Conversion forces radius zero
when square; otherwise it preserves the existing elevation-derived radius.
Outline, children, key and 16 px padding are unchanged. No new WidgetNode,
renderer, Scene or platform API is needed.

A new public field affects external PaperBuilder struct literals: add
`square: false` to preserve behavior, or use `paper()` and setters. Existing
constructor/setter users retain their defaults. This is an unreleased SDK
change, documented in the changelog. Adding a private field would also break
literals and unnecessarily prevent existing public-field access. No additional
public radius/shadow API is introduced. Current elevation remains the existing
Card semantic surface and corner mapping; this does not implement physical
shadow levels or establish native MUI parity.
