# ADR 0004: DataList density option

2026-11-11. Accepted for ROADMAP 038 before implementation.

Add DataListDensity::{Compact, Default, Relaxed}, public builder density field
and density(value) setter, following the existing orientation API. Default
preserves the current item gap/padding/vertical pair gap/horizontal pair gap
of 8/4/2/12 logical pixels. Compact uses 4/2/2/8; Relaxed uses 12/8/4/16.
Only spacing varies; content, orientation, keys, colors and typography do not.
No raw spacing API or widget/Scene/platform contract is added.

DataListBuilder already has a private phantom field, so external struct
literals are unavailable. Existing constructor/setter users retain the same
default geometry; no existing public field or setter changes meaning. The
new enum and setter are re-exported through the existing foundations glob.
An explicit density avoids coupling this content component to platform-native
control density or inventing a workspace-wide density system.
