# ROADMAP 062: theme token changes

Source of truth: `ROADMAP_062_CHANGES` in `crates/acme-theme/src/lib.rs`, checked by `roadmap_062_pack_retune_reaches_aa`. "Before" ratios are measured on the pre-062 palette (master `ffd7e51`). Changed tokens: **104**. That is 38 for the brief gate, 21 status tokens from the lead's scope extension, and 45 from the review fixes (see "Review fixes" below). The first table is the original pass. Rows whose token moved again in the review are marked **(superseded)**; the review table gives their final values. The brief retune touched 8 of 13 palettes. The review added one `primary_pressed` each in default dark, material dark and windows11 dark for the P1-2 text gate. windows10 and high-contrast are untouched.

## Rules applied (in order)
- **R1 Foreground first:** move the `*_foreground` token by the smallest HSL-lightness step (hue and saturation kept) that reaches >= 4.6. Use a pack or brand palette colour instead when one passes, keeps hue (within about 15 deg), and does not overshoot past about 5.6.
- **R2 Forced fill:** if the foreground is already at the lightness extreme (white text) and cannot reach 4.6, darken the fill by the smallest HSL step.
- **R3 Polarity:** if the R2 fill step would push `primary` below 3.1 vs `background`, switch the text to the pack's own dark ink and keep (or minimally adjust) the brand fill.
- **Status colours (lead extension):** `success`, `warning`, `danger` (the brief's "destructive") and `info` must each reach >= 3:1 vs `surface` and vs `background`. When retuning, I took the smallest HSL step to >= 3.1 against both, away from the background (darker in light mode, lighter in dark mode). A palette colour is used when it passes and keeps hue. Solid status fills get their text from `acme-ui`'s `contrasting_fg` (white or black, always >= 4.5), so no `*_foreground` token moved.
- **Graphic (3:1) rules:** smallest step to >= 3.1. A pack colour that passes (for example `primary_hover`, or the new `primary`) is preferred.

| Pack/mode | Token | Old -> new | Pair | Ratio before -> after | Rationale |
|---|---|---|---|---|---|
| default/light | `muted_foreground` | #6B7280 -> #686F7D | muted_fg/muted | 4.43 -> 4.63 | R1 foreground: HSL lightness -0.011 (hue kept). |
| apple/Light | `primary` | #007AFF -> #0070EB | primary_fg/primary | 4.02 -> 4.65 | R2 forced fill: white text already at max lightness (white tops out at 4.02). systemBlue HSL lightness -0.039, hue kept. Ring stays exact systemBlue. |
| apple/Light | `primary` | #007AFF -> #0070EB | primary/background | 3.60 -> 4.17 | (same change) |
| apple/Light | `accent_foreground` | #007AFF -> #0066D6 | accent_fg/accent | 3.41 -> 4.61 | R1 foreground, palette: #0066D6 is the pack's own `primary_hover`, and it is also the smallest HSL step (-0.080). |
| apple/Light | `muted_foreground` | #8E8E93 -> #6D6D72 | muted_fg/muted | 2.92 -> 4.61 | R1 foreground: systemGray HSL lightness -0.128. Apple's accessible systemGray #6C6C70 is within 1/255 and not in the pack, so I took the minimal step. |
| apple/Light | `success` | #34C759 -> #299D46 | success/surface | 2.22 -> 3.50 | Status graphic (lead scope extension): systemGreen HSL lightness -0.103. Apple's accessible systemGreen is not in the pack and overshoots. |
| apple/Light | `success` | #34C759 -> #299D46 | success/background | 1.99 -> 3.13 | (same change) |
| apple/Light | `warning` | #FF9500 -> #C97500 | warning/surface | 2.20 -> 3.48 | Status graphic: systemOrange HSL lightness -0.106. |
| apple/Light | `warning` | #FF9500 -> #C97500 | warning/background | 1.97 -> 3.12 | (same change) |
| apple/Light | `info` | #5AC8FA -> #0692D2 | info/surface | 1.90 -> 3.47 | Status graphic: systemTeal HSL lightness -0.243. |
| apple/Light | `info` | #5AC8FA -> #0692D2 | info/background | 1.70 -> 3.11 | (same change) |
| apple/Dark | `primary` | #0A84FF -> #0072E6 | primary_fg/primary | 3.65 -> 4.62 | R2 forced fill: white text already at max lightness. HSL lightness -0.069; still 4.55 vs black background. |
| apple/Dark | `primary` | #0A84FF -> #0072E6 | primary/background | 5.76 -> 4.55 | (same change) |
| apple/Dark | `accent_foreground` | #0A84FF -> #409CFF | accent_fg/accent | 4.06 -> 5.23 | R1 foreground, palette: #409CFF is the pack's own `primary_hover` (Apple's accessible dark systemBlue). It is preferred over the minimal #2391FF (4.63). |
| ubuntu/Light | `primary` | #E95420 -> #D04515 | primary_fg/primary | 3.65 -> 4.62 | R2 forced fill: white text already at max lightness. Orange HSL lightness -0.071, hue kept. |
| ubuntu/Light | `primary` | #E95420 -> #D04515 | primary/background | 3.50 -> 4.43 | (same change) |
| ubuntu/Light | `warning` | #F99A00 -> #CB7D00 | warning/surface | 2.18 -> 3.24 | Status graphic: HSL lightness -0.091. |
| ubuntu/Light | `warning` | #F99A00 -> #CB7D00 | warning/background | 2.08 -> 3.11 | (same change) |
| ubuntu/Dark | `primary_foreground` | #FFFFFF -> #111111 | primary_fg/primary | 3.65 -> 5.17 | R3 polarity: darkening the orange for white text tops out at 4.62 text with 3.02 vs background, so both 0.1 margins cannot hold. Brand orange kept; text uses the pack's own ink #111111 (the light foreground). |
| nord/Light | `primary` | #5E81AC -> #4D6D95 | primary_fg/primary | 3.50 -> 4.63 | R2 forced fill: nord6 text reaches only 4.2 even as white. nord10 HSL lightness -0.078. |
| nord/Light | `primary` | #5E81AC -> #4D6D95 | primary/background | 3.50 -> 4.63 | (same change) |
| nord/Light | `accent_foreground` | #5E81AC -> #4C566A | accent_fg/accent | 2.98 -> 5.46 | R1 foreground, palette: nord3 (polar night, hue 220 deg vs 213 deg). The minimal HSL step was #456286 (4.65). |
| nord/Light | `success` | #A3BE8C -> #6B8C4F | success/surface | 1.67 -> 3.14 | Status graphic: nord14 HSL lightness -0.216. Aurora has no darker variant. |
| nord/Light | `success` | #A3BE8C -> #6B8C4F | success/background | 1.77 -> 3.32 | (same change) |
| nord/Light | `warning` | #EBCB8B -> #AA7B1D | warning/surface | 1.28 -> 3.10 | Status graphic: nord13 HSL lightness -0.342 (yellow becomes ochre). |
| nord/Light | `warning` | #EBCB8B -> #AA7B1D | warning/background | 1.35 -> 3.28 | (same change) |
| nord/Light | `info` | #88C0D0 -> #408DA3 | info/surface | 1.64 -> 3.10 | Status graphic: nord8 HSL lightness -0.228. |
| nord/Light | `info` | #88C0D0 -> #408DA3 | info/background | 1.74 -> 3.28 | (same change) |
| nord/Dark | `accent_foreground` | #88C0D0 -> #93C6D4 | accent_fg/accent | 4.31 -> 4.63 | R1 foreground: nord8 HSL lightness +0.029. No frost colour of the same hue passes. |
| nord/Dark | `danger` | #BF616A -> #C97980 | danger/surface | 2.46 -> 3.13 | Status graphic: nord11 HSL lightness +0.065 (2.46 vs nord1 surface). **(superseded)** |
| nord/Dark | `danger` | #BF616A -> #C97980 | danger/background | 3.05 -> 3.88 | (same change) **(superseded)** |
| dracula/Light | `primary` | #BD93F9 -> #A870F7 | primary_fg/primary | 5.90 -> 4.63 | Graphic rule: purple was 2.26 vs background. HSL lightness -0.072 to >= 3.1 vs background and white surface. |
| dracula/Light | `primary` | #BD93F9 -> #A870F7 | primary/background | 2.26 -> 3.11 | (same change) |
| dracula/Light | `primary_foreground` | #282A36 -> #23242F | primary_fg/primary | 5.90 -> 4.63 | Follows from the primary change: the darker purple drops Dracula-bg text to 4.29, so the ink takes HSL lightness -0.025 (text stays dark). |
| dracula/Light | `accent_foreground` | #7850C8 -> #764EC7 | accent_fg/accent | 4.49 -> 4.61 | R1 foreground: HSL lightness -0.006. |
| dracula/Light | `muted_foreground` | #6272A4 -> #586798 | muted_fg/muted | 3.93 -> 4.61 | R1 foreground: comment HSL lightness -0.044. |
| dracula/Light | `ring` | #BD93F9 -> #A870F7 | ring/background | 2.26 -> 3.11 | Graphic rule: set equal to the new primary, which is also the minimal ring step for both bg and surface. |
| dracula/Light | `ring` | #BD93F9 -> #A870F7 | ring/surface | 2.41 -> 3.32 | (same change) |
| dracula/Light | `success` | #50FA7B -> #05A42D | success/surface | 1.37 -> 3.30 | Status graphic: green HSL lightness -0.316. Neon greens cannot reach 3:1 on a light background without a large step. |
| dracula/Light | `success` | #50FA7B -> #05A42D | success/background | 1.29 -> 3.10 | (same change) |
| dracula/Light | `warning` | #F1FA8C -> #899406 | warning/surface | 1.12 -> 3.33 | Status graphic: yellow HSL lightness -0.461 (yellow becomes olive; any 3:1 yellow on a near-white background does). |
| dracula/Light | `warning` | #F1FA8C -> #899406 | warning/background | 1.05 -> 3.13 | (same change) |
| dracula/Light | `danger` | #FF5555 -> #FF4A4A | danger/surface | 3.14 -> 3.32 | Status graphic: red HSL lightness -0.021 (was 2.95 vs background). |
| dracula/Light | `danger` | #FF5555 -> #FF4A4A | danger/background | 2.95 -> 3.11 | (same change) |
| dracula/Light | `info` | #8BE9FD -> #039ABA | info/surface | 1.38 -> 3.31 | Status graphic: cyan HSL lightness -0.397. |
| dracula/Light | `info` | #8BE9FD -> #039ABA | info/background | 1.30 -> 3.11 | (same change) |
| dracula/Dark | `accent_foreground` | #FF79C6 -> #FF97D3 | accent_fg/accent | 3.84 -> 4.63 | R1 foreground: pink HSL lightness +0.058. |
| dracula/Dark | `muted_foreground` | #6272A4 -> #AFB8D1 | muted_fg/muted | 1.94 -> 4.62 | R1 foreground: comment HSL lightness +0.240. No palette colour of that hue passes. |
| dracula/Dark | `danger` | #FF5555 -> #FF6161 | danger/surface | 2.91 -> 3.11 | Status graphic: red HSL lightness +0.023 (2.91 vs current-line surface). |
| dracula/Dark | `danger` | #FF5555 -> #FF6161 | danger/background | 4.53 -> 4.84 | (same change) |
| solarized/Light | `foreground` | #657B83 -> #576A71 | foreground/background | 4.13 -> 5.25 | R1 foreground: base00 HSL lightness -0.062. base01 was tried and fails vs base2 surface (4.39). base02 overshoots (10.6). **(superseded)** |
| solarized/Light | `foreground` | #657B83 -> #576A71 | foreground/surface | 3.64 -> 4.63 | (same change) **(superseded)** |
| solarized/Light | `surface_foreground` | #657B83 -> #576A71 | surface_fg/surface | 3.64 -> 4.63 | Same value as foreground. **(superseded)** |
| solarized/Light | `primary` | #268BD2 -> #2074B0 | primary_fg/primary | 3.41 -> 4.65 | R2 forced fill: base3 text reaches only 3.7 even as white, and base03 text only 4.04. Blue HSL lightness -0.079. |
| solarized/Light | `primary` | #268BD2 -> #2074B0 | primary/background | 3.41 -> 4.65 | (same change) |
| solarized/Light | `secondary` | #2AA198 -> #207C75 | secondary_fg/secondary | 2.93 -> 4.63 | R2 forced fill (GUI finding 1): base3 text maxes at 3.18 as white. Cyan HSL lightness -0.091. A base03 polarity flip (4.75) would also pass but is not a minimal step. |
| solarized/Light | `muted_foreground` | #93A1A1 -> #5C6A6A | muted_fg/muted | 2.18 -> 4.60 | R1 foreground: base1 HSL lightness -0.215. base00 and base01 fail vs base2, and base02 overshoots. |
| solarized/Light | `success` | #859900 -> #798B00 | success/surface | 2.62 -> 3.11 | Status graphic: green HSL lightness -0.027. |
| solarized/Light | `success` | #859900 -> #798B00 | success/background | 2.97 -> 3.54 | (same change) |
| solarized/Light | `warning` | #B58900 -> #A47C00 | warning/surface | 2.62 -> 3.14 | Status graphic: yellow HSL lightness -0.033. |
| solarized/Light | `warning` | #B58900 -> #A47C00 | warning/background | 2.98 -> 3.56 | (same change) |
| solarized/Light | `info` | #2AA198 -> #269189 | info/surface | 2.58 -> 3.12 | Status graphic: cyan HSL lightness -0.039. |
| solarized/Light | `info` | #2AA198 -> #269189 | info/background | 2.93 -> 3.54 | (same change) |
| solarized/Dark | `foreground` | #839496 -> #93A1A1 | foreground/background | 4.75 -> 5.61 | R1 foreground, palette: base1, Solarized's emphasis tone, hue kept. **(superseded)** |
| solarized/Dark | `foreground` | #839496 -> #93A1A1 | foreground/surface | 4.11 -> 4.86 | (same change) **(superseded)** |
| solarized/Dark | `surface_foreground` | #839496 -> #93A1A1 | surface_fg/surface | 4.11 -> 4.86 | R1 foreground, palette: base1. **(superseded)** |
| solarized/Dark | `primary_foreground` | #FDF6E3 -> #002029 | primary_fg/primary | 3.41 -> 4.61 | R3 polarity: darkening the blue for base3 text tops out at 4.65 text with 2.99 vs base03, so both margins cannot hold. Blue kept. Text is base03 at HSL lightness -0.026, because plain base03 is 4.08. |
| solarized/Dark | `secondary_foreground` | #839496 -> #93A1A1 | secondary_fg/secondary | 4.11 -> 4.86 | R1 foreground, palette: base1. |
| solarized/Dark | `accent_foreground` | #2AA198 -> #2DABA1 | accent_fg/accent | 4.12 -> 4.61 | R1 foreground: cyan HSL lightness +0.024. |
| solarized/Dark | `muted_foreground` | #586E75 -> #93A1A1 | muted_fg/muted | 2.42 -> 4.86 | R1 foreground, palette: base1 (base00 2.9 and base0 4.11 fail). Muted fill = surface, so muted text must match body text contrast. |
| solarized/Dark | `danger` | #DC322F -> #DF4340 | danger/surface | 2.81 -> 3.10 | Status graphic: red HSL lightness +0.039 (2.81 vs base02). **(superseded)** |
| solarized/Dark | `danger` | #DC322F -> #DF4340 | danger/background | 3.25 -> 3.58 | (same change) **(superseded)** |
| gruvbox/Light | `primary` | #458588 -> #3D7578 | primary_fg/primary | 3.73 -> 4.61 | R2 forced fill: bg text reaches only 4.27 even as white. Aqua HSL lightness -0.047. Faded blue #076678 overshoots (5.82). |
| gruvbox/Light | `primary` | #458588 -> #3D7578 | primary/background | 3.73 -> 4.61 | (same change) |
| gruvbox/Light | `muted_foreground` | #7C6F64 -> #665C54 | muted_fg/muted | 3.55 -> 4.75 | R1 foreground, palette: fg3 #665C54. |
| gruvbox/Light | `success` | #98971A -> #79740E | success/surface | 2.26 -> 3.55 | Status graphic, palette: gruvbox faded green #79740E. |
| gruvbox/Light | `success` | #98971A -> #79740E | success/background | 2.73 -> 4.29 | (same change) |
| gruvbox/Light | `warning` | #D79921 -> #A17219 | warning/surface | 1.81 -> 3.10 | Status graphic: HSL lightness -0.123. Faded yellow #B57614 fails (2.75 vs bg1). |
| gruvbox/Light | `warning` | #D79921 -> #A17219 | warning/background | 2.19 -> 3.75 | (same change) |
| gruvbox/Dark | `primary` | #458588 -> #519C9F | primary_fg/primary | 3.73 -> 4.64 | R3 + fill: darkening the aqua for light text tops out at 4.61 text with 2.82 vs background. Dark text needs a lighter fill, so aqua HSL lightness +0.069. Bright blue #83A598 rejected (hue 26 deg off). |
| gruvbox/Dark | `primary` | #458588 -> #519C9F | primary/background | 3.48 -> 4.64 | (same change) |
| gruvbox/Dark | `primary_foreground` | #FBF1C7 -> #282828 | primary_fg/primary | 3.73 -> 4.64 | R3 polarity, palette: gruvbox bg #282828, the same ink 060 used for accent_foreground. |
| gruvbox/Dark | `muted_foreground` | #A89984 -> #BDAE93 | muted_fg/muted | 4.17 -> 5.32 | R1 foreground, palette: fg3 #BDAE93. |
| gruvbox/Dark | `ring` | #458588 -> #5A9699 | ring/background | 3.48 -> 4.39 | Graphic rule, palette: the pack's own `primary_hover` #5A9699 (minimal would be #4A8F92 at 3.11). **(superseded)** |
| gruvbox/Dark | `ring` | #458588 -> #5A9699 | ring/surface | 2.74 -> 3.46 | (same change) **(superseded)** |
| gruvbox/Dark | `danger` | #CC241D -> #FB4934 | danger/surface | 2.12 -> 3.37 | Status graphic, palette: gruvbox bright red #FB4934. **(superseded)** |
| gruvbox/Dark | `danger` | #CC241D -> #FB4934 | danger/background | 2.69 -> 4.29 | (same change) **(superseded)** |
| gruvbox/Dark | `info` | #458588 -> #5A9699 | info/surface | 2.74 -> 3.46 | Status graphic, palette: the pack's `primary_hover` #5A9699, the same value as the new ring (minimal #4A8F92 at 3.11). **(superseded)** |
| gruvbox/Dark | `info` | #458588 -> #5A9699 | info/background | 3.48 -> 4.39 | (same change) **(superseded)** |
| one-dark/Light | `primary` | #4078F2 -> #2F6CF1 | primary_fg/primary | 4.05 -> 4.63 | R2 forced fill: white text already at max lightness. Blue HSL lightness -0.035. |
| one-dark/Light | `primary` | #4078F2 -> #2F6CF1 | primary/background | 3.88 -> 4.44 | (same change) |
| one-dark/Light | `muted_foreground` | #A0A1A7 -> #6A6C73 | muted_fg/muted | 2.26 -> 4.61 | R1 foreground: HSL lightness -0.207. One Light mono-2 #696C77 is 4.598, below the 4.6 margin, so I took the next step #6A6C73. |
| one-dark/Dark | `muted_foreground` | #5C6370 -> #858D9B | muted_fg/muted | 2.55 -> 4.60 | R1 foreground: comment HSL lightness +0.164. mono-2 #828997 fails (4.38). **(superseded)** |

## Forced fill changes
- **R2 (white text could not go lighter):** apple L/D `primary`, ubuntu L `primary`, nord L `primary`, solarized L `primary` and `secondary`, gruvbox L `primary`, one-dark L `primary`.
- **Graphic fills:** dracula L `primary` and `ring` (purple was 2.26 vs background). gruvbox D `ring` (2.74 vs surface).
- **R3 plus fill:** gruvbox D `primary` (lighter aqua for dark ink).
- **R3 with brand fill kept:** ubuntu D and solarized D, where only `primary_foreground` changed.

## Not changed (near threshold, already passing)
- solarized/Light `ring` vs surface is 3.004. ubuntu/Dark `ring` vs surface is 3.02. gruvbox/Light `ring` vs surface is 3.08. These rings no longer equal the retuned `primary`, but the leave-passing-tokens rule keeps them.
- gruvbox/Dark `primary_hover` #5A9699 was slightly darker than `primary` #519C9F. The review fixes this (see below).

## Worst remaining ratio per gate rule, first pass (superseded by the review table below)
| Rule | Worst | Where |
|---|---|---|
| AA report pairs >= 4.5 | 4.53 | windows10/Light primary_fg/primary (untouched) |
| selected_fg/selected_bg >= 4.5 | 4.55 | windows11/Light (untouched) |
| ring vs background >= 3 | 3.11 | dracula/Light |
| ring vs surface >= 3 | 3.00 (3.004) | solarized/Light (untouched) |
| primary vs background >= 3 | 3.11 | dracula/Light |
| foreground vs surface >= 4.5 | 4.63 | solarized/Light |
| status vs surface >= 3 | 3.08 | gruvbox/Light info (untouched; worst retuned value is 3.10, gruvbox/Light warning) |
| status vs background >= 3 | 3.06 | one-dark/Light warning (untouched; worst retuned value is 3.10, dracula/Light success) |

## Status soft pair (not gated, reported)
`resolve_tone` paints `soft_fg = <status>` text on `<status>_soft`. Every retuned status colour raises that ratio, but most packs stay below 4.5, as before 062. Soft-badge text AA needs either a soft-pair gate or a separate text token. That is a new token or API, so it is out of scope.

## Review fixes (Fable 5.2 review: P1-2, P1-3, P2, plus the lead's ghost addition)

### New gates (`crates/acme-theme/src/lib.rs`)
- **P1-2 `roadmap_062_primary_states_keep_text_aa`:** `primary_foreground` is >= 4.5 on `primary_hover` and `primary_pressed` in all 24 pack/modes. New values target >= 4.6.
- **`roadmap_062_danger_fills_have_aa_ink`:** each of `danger`, `danger_hover` and `danger_pressed` reaches >= 4.5 against at least one of `primary_foreground`, `background` and `foreground`, in all 24.
- **P1-3 `roadmap_062_state_fills_give_feedback`:** for primary and danger, hover and pressed are each >= 1.15:1 from the rest fill, and pressed is at least as far from rest as hover.
  - **Direction rule:** states step away from the family's ink, so the ink gains contrast on hover and on pressed. Under light ink the states go darker; under dark ink they go lighter. The ink is `primary_foreground` for primary. For danger it is the first candidate reaching 4.5 on the rest fill. One ink therefore stays readable in every state, with no flip on hover.
  - **Scope:** the rule is asserted on the 15 pack/modes 062 retuned. Untouched-pack failures are listed exactly in `STATE_FEEDBACK_UNTOUCHED_EXCEPTIONS` (see below).
- **P2:** `muted_fg/background` >= 4.5 joins `roadmap_062_every_pack_meets_aa_gate` for all 24. The only failure was one-dark dark, so no untouched pack failed.
  - `solarized_muted_text_keeps_a_hierarchy` checks that foreground and muted_fg are >= 1.1:1 apart, that muted is the dimmer one, and that muted_fg is >= 4.6 on muted and on background.
  - `gruvbox_dark_teals_are_unified` checks ring = info = primary, >= 3 vs surface and background, and a hover step >= 1.15.
- **Lead addition:** `foreground` >= 4.5 on `ghost_hover` and `ghost_pressed` joins the AA gate for all 24. The only failure was solarized light ghost_pressed. It is fixed on the foreground side. Solarized dark ghost_pressed already passes (4.98) after the P2 foreground change.
- `apple_primary_is_system_blue` is renamed to `apple_ring_is_system_blue_and_primary_is_darkened_for_aa`.
- Retuned step values target >= 1.17 (the gate is 1.15). Hierarchy values target >= 1.11 (the gate is 1.1). In the table, the margin check is 0.1 for the 4.5 and 3 rules and 0.01 for the 1.15 and 1.1 rules.

### Tokens (45 new, plus 10 tokens from the first pass moved again)
`best_ink/<fill>` is the best of the three danger ink candidates on that fill. `<state>/<base>` is the luminance step between the state and its rest fill. "Before" is the pre-062 palette.

| Pack/mode | Token | Old -> new | Pair | Ratio before -> after | Rationale |
|---|---|---|---|---|---|
| default/dark | `primary_pressed` | #4072E6 -> #4A7AE7 | primary_fg/primary_pressed | 4.20 -> 4.61 | Untouched pack, P1-2 text gate only: smallest lighter step (keeps its darker direction). |
| default/dark | `primary_pressed` | #4072E6 -> #4A7AE7 | primary_pressed/primary | 1.36 -> 1.24 | (same change) |
| apple/Light | `danger_hover` | #D63128 -> #FF6961 | best_ink/danger_hover | 4.85 -> 7.45 | Away from black ink: the pack's dark-mode hover #FF6961 (Apple accessible systemRed). |
| apple/Light | `danger_hover` | #D63128 -> #FF6961 | danger_hover/danger | 1.37 -> 1.26 | (same change) |
| apple/Light | `danger_pressed` | #AD2820 -> #FF8079 | best_ink/danger_pressed | 6.77 -> 8.62 | One more lighter step (same value as dark). |
| apple/Light | `danger_pressed` | #AD2820 -> #FF8079 | danger_pressed/danger | 1.91 -> 1.46 | (same change) |
| apple/Dark | `primary_hover` | #409CFF -> #0066D6 | primary_fg/primary_hover | 2.83 -> 5.42 | White ink, so states go darker: the pack's #0066D6 (old pressed). |
| apple/Dark | `primary_hover` | #409CFF -> #0066D6 | primary_hover/primary | 1.29 -> 1.18 | (same change) |
| apple/Dark | `primary_pressed` | #0066D6 -> #0052AD | primary_fg/primary_pressed | 5.42 -> 7.47 | Darker again: the pack's #0052AD (light pressed). |
| apple/Dark | `primary_pressed` | #0066D6 -> #0052AD | primary_pressed/primary | 1.49 -> 1.62 | (same change) |
| apple/Dark | `danger_pressed` | #D63128 -> #FF8079 | best_ink/danger_pressed | 4.85 -> 8.62 | Was darker than rest under black ink. Now lighter than hover. |
| apple/Dark | `danger_pressed` | #D63128 -> #FF8079 | danger_pressed/danger | 1.42 -> 1.40 | (same change) |
| dracula/Light | `primary_pressed` | #966EDC -> #D2B6FB | primary_fg/primary_pressed | 3.77 -> 8.66 | Dark ink: lighter than hover #C8A5FA. |
| dracula/Light | `primary_pressed` | #966EDC -> #D2B6FB | primary_pressed/primary | 1.57 -> 1.87 | (same change) |
| dracula/Light | `danger_pressed` | #DC3C3C -> #FF8484 | best_ink/danger_pressed | 4.14 -> 6.51 | Dark ink: lighter than hover #FF6E6E. |
| dracula/Light | `danger_pressed` | #DC3C3C -> #FF8484 | danger_pressed/danger | 1.41 -> 1.40 | (same change) |
| dracula/Dark | `primary_pressed` | #966EDC -> #D2B6FB | primary_fg/primary_pressed | 3.77 -> 8.02 | Dark ink: lighter than hover (same value as light). |
| dracula/Dark | `primary_pressed` | #966EDC -> #D2B6FB | primary_pressed/primary | 1.57 -> 1.36 | (same change) |
| dracula/Dark | `danger_hover` | #FF6E6E -> #FF7B7B | best_ink/danger_hover | 5.23 -> 5.68 | Hover was 1.08 from rest. Smallest lighter step to 1.17. |
| dracula/Dark | `danger_hover` | #FF6E6E -> #FF7B7B | danger_hover/danger | 1.15 -> 1.17 | (same change) |
| dracula/Dark | `danger_pressed` | #DC3C3C -> #FF9090 | best_ink/danger_pressed | 4.14 -> 6.54 | Lighter again. |
| dracula/Dark | `danger_pressed` | #DC3C3C -> #FF9090 | danger_pressed/danger | 1.41 -> 1.35 | (same change) |
| gruvbox/Light | `primary_hover` | #5A9699 -> #37696C | primary_fg/primary_hover | 2.96 -> 5.44 | Light ink, so states go darker: old pressed #37696C is the minimal step. |
| gruvbox/Light | `primary_hover` | #5A9699 -> #37696C | primary_hover/primary | 1.26 -> 1.18 | (same change) |
| gruvbox/Light | `primary_pressed` | #37696C -> #325F62 | primary_fg/primary_pressed | 5.44 -> 6.28 | Darker again. |
| gruvbox/Light | `primary_pressed` | #37696C -> #325F62 | primary_pressed/primary | 1.46 -> 1.36 | (same change) |
| gruvbox/Light | `danger_hover` | #D73C34 -> #B8211A | best_ink/danger_hover | 4.04 -> 5.66 | Hover was lighter (4.04 text). Smallest darker step. |
| gruvbox/Light | `danger_hover` | #D73C34 -> #B8211A | danger_hover/danger | 1.19 -> 1.17 | (same change) |
| gruvbox/Dark | `ring` | #458588 -> #519C9F | ring/background | 3.48 -> 4.64 | P2: one teal, so ring = primary #519C9F (3.65 vs bg1). |
| gruvbox/Dark | `ring` | #458588 -> #519C9F | ring/surface | 2.74 -> 3.65 | (same change) |
| gruvbox/Dark | `danger` | #CC241D -> #FB5845 | danger/surface | 2.12 -> 3.64 | Danger ink: bright red #FB4934 was 4.29 with dark ink. HSL lightness +0.033. |
| gruvbox/Dark | `danger` | #CC241D -> #FB5845 | danger/background | 2.69 -> 4.62 | (same change) |
| gruvbox/Dark | `danger` | #CC241D -> #FB5845 | best_ink/danger | 4.82 -> 4.62 | (same change) |
| gruvbox/Dark | `info` | #458588 -> #519C9F | info/surface | 2.74 -> 3.65 | P2: one teal, so info = primary. |
| gruvbox/Dark | `info` | #458588 -> #519C9F | info/background | 3.48 -> 4.64 | (same change) |
| gruvbox/Dark | `primary_hover` | #5A9699 -> #5DA9AC | primary_fg/primary_hover | 2.96 -> 5.43 | P1-3/P2: hover was 1.06 (and darker). Smallest lighter step. |
| gruvbox/Dark | `primary_hover` | #5A9699 -> #5DA9AC | primary_hover/primary | 1.26 -> 1.17 | (same change) |
| gruvbox/Dark | `primary_pressed` | #37696C -> #71B4B6 | primary_fg/primary_pressed | 5.44 -> 6.25 | P1-2: was 2.39 text. Lighter than hover. |
| gruvbox/Dark | `primary_pressed` | #37696C -> #71B4B6 | primary_pressed/primary | 1.46 -> 1.35 | (same change) |
| gruvbox/Dark | `danger_hover` | #D73C34 -> #FC7261 | best_ink/danger_hover | 4.04 -> 5.42 | Away from dark ink. |
| gruvbox/Dark | `danger_hover` | #D73C34 -> #FC7261 | danger_hover/danger | 1.19 -> 1.17 | (same change) |
| gruvbox/Dark | `danger_pressed` | #AA1E18 -> #FC8779 | best_ink/danger_pressed | 6.36 -> 6.23 | Lighter again. |
| gruvbox/Dark | `danger_pressed` | #AA1E18 -> #FC8779 | danger_pressed/danger | 1.32 -> 1.35 | (same change) |
| material/Dark | `primary_pressed` | #6750A4 -> #A091C9 | primary_fg/primary_pressed | 2.04 -> 4.63 | Untouched pack, P1-2 only (was 2.04): primary40 lightened. primary70 #B69DF8 overshoots (5.74). |
| material/Dark | `primary_pressed` | #6750A4 -> #A091C9 | primary_pressed/primary | 3.78 -> 1.67 | (same change) |
| nord/Light | `danger` | #BF616A -> #B24852 | danger/surface | 3.36 -> 4.38 | Danger ink: nord11 was 3.55 with nord6. HSL lightness -0.073. |
| nord/Light | `danger` | #BF616A -> #B24852 | danger/background | 3.55 -> 4.62 | (same change) |
| nord/Light | `danger` | #BF616A -> #B24852 | best_ink/danger | 3.55 -> 4.62 | (same change) |
| nord/Light | `primary_hover` | #81A1C1 -> #466287 | primary_fg/primary_hover | 2.34 -> 5.43 | Hover nord9 #81A1C1 was 2.34 text. Smallest darker step. Pressed stays nord3. |
| nord/Light | `primary_hover` | #81A1C1 -> #466287 | primary_hover/primary | 1.50 -> 1.17 | (same change) |
| nord/Light | `danger_hover` | #B0555E -> #A0414A | best_ink/danger_hover | 4.23 -> 5.44 | Darker step from the new rest. |
| nord/Light | `danger_hover` | #B0555E -> #A0414A | danger_hover/danger | 1.19 -> 1.18 | (same change) |
| nord/Light | `danger_pressed` | #9B4A53 -> #913B43 | best_ink/danger_pressed | 5.21 -> 6.26 | Darker again. |
| nord/Light | `danger_pressed` | #9B4A53 -> #913B43 | danger_pressed/danger | 1.47 -> 1.35 | (same change) |
| nord/Dark | `danger` | #BF616A -> #D08B91 | danger/surface | 2.46 -> 3.73 | Danger ink: 3.88 with dark ink. HSL lightness +0.049. |
| nord/Dark | `danger` | #BF616A -> #D08B91 | danger/background | 3.05 -> 4.63 | (same change) |
| nord/Dark | `danger` | #BF616A -> #D08B91 | best_ink/danger | 3.55 -> 4.63 | (same change) |
| nord/Dark | `primary_hover` | #8FBCBB -> #A2CEDA | primary_fg/primary_hover | 5.99 -> 7.36 | Hover nord7 was 1.04 from nord8. Smallest lighter nord8 step. |
| nord/Dark | `primary_hover` | #8FBCBB -> #A2CEDA | primary_hover/primary | 1.04 -> 1.18 | (same change) |
| nord/Dark | `primary_pressed` | #5E81AC -> #BADBE4 | primary_fg/primary_pressed | 3.10 -> 8.53 | Pressed nord10 was 3.10 text. Lighter again. |
| nord/Dark | `primary_pressed` | #5E81AC -> #BADBE4 | primary_pressed/primary | 2.01 -> 1.37 | (same change) |
| nord/Dark | `danger_hover` | #D07880 -> #D79CA1 | best_ink/danger_hover | 3.97 -> 5.45 | Was 1.02 from rest. Lighter step. |
| nord/Dark | `danger_hover` | #D07880 -> #D79CA1 | danger_hover/danger | 1.30 -> 1.18 | (same change) |
| nord/Dark | `danger_pressed` | #BF616A -> #DDACB0 | best_ink/danger_pressed | 3.55 -> 6.31 | Lighter again. |
| nord/Dark | `danger_pressed` | #BF616A -> #DDACB0 | danger_pressed/danger | 1.00 -> 1.36 | (same change) |
| one-dark/Light | `danger` | #E45649 -> #DD3121 | danger/surface | 3.67 -> 4.63 | Danger ink: 3.67 with white. HSL lightness -0.092. |
| one-dark/Light | `danger` | #E45649 -> #DD3121 | danger/background | 3.51 -> 4.44 | (same change) |
| one-dark/Light | `danger` | #E45649 -> #DD3121 | best_ink/danger | 3.67 -> 4.63 | (same change) |
| one-dark/Light | `primary_hover` | #5A8CF5 -> #1A5DF0 | primary_fg/primary_hover | 3.23 -> 5.43 | Hover was lighter (3.23 text). Smallest darker step. Pressed #3062D2 (1.19) stays. |
| one-dark/Light | `primary_hover` | #5A8CF5 -> #1A5DF0 | primary_hover/primary | 1.25 -> 1.17 | (same change) |
| one-dark/Light | `danger_hover` | #EA6E62 -> #C92C1E | best_ink/danger_hover | 3.73 -> 5.44 | Darker step. |
| one-dark/Light | `danger_hover` | #EA6E62 -> #C92C1E | danger_hover/danger | 1.21 -> 1.17 | (same change) |
| one-dark/Light | `danger_pressed` | #C64439 -> #B8281B | best_ink/danger_pressed | 4.90 -> 6.25 | Darker again. |
| one-dark/Light | `danger_pressed` | #C64439 -> #B8281B | danger_pressed/danger | 1.34 -> 1.35 | (same change) |
| one-dark/Dark | `muted_foreground` | #5C6370 -> #8D95A2 | muted_fg/muted | 2.55 -> 5.10 | P2: captions on background were 4.19. HSL lightness +0.029. |
| one-dark/Dark | `muted_foreground` | #5C6370 -> #8D95A2 | muted_fg/background | 2.32 -> 4.64 | (same change) |
| one-dark/Dark | `primary_pressed` | #4691D2 -> #92CBF7 | primary_fg/primary_pressed | 4.58 -> 8.88 | Was darker than rest under dark ink. Lighter than hover. |
| one-dark/Dark | `primary_pressed` | #4691D2 -> #92CBF7 | primary_pressed/primary | 1.42 -> 1.36 | (same change) |
| one-dark/Dark | `danger_pressed` | #C3555E -> #EB949B | best_ink/danger_pressed | 3.51 -> 6.77 | Was 3.51 text. Lighter than hover. |
| one-dark/Dark | `danger_pressed` | #C3555E -> #EB949B | danger_pressed/danger | 1.37 -> 1.41 | (same change) |
| solarized/Light | `foreground` | #657B83 -> #506168 | foreground/background | 4.13 -> 5.99 | P2 hierarchy and ghost_pressed text: darker by HSL -0.031 so that body text is 1.15 over muted and 4.63 on ghost_pressed. |
| solarized/Light | `foreground` | #657B83 -> #506168 | foreground/surface | 3.64 -> 5.27 | (same change) |
| solarized/Light | `foreground` | #657B83 -> #506168 | foreground/muted_fg | 1.67 -> 1.15 | (same change) |
| solarized/Light | `foreground` | #657B83 -> #506168 | foreground/ghost_pressed | 3.19 -> 4.63 | (same change) |
| solarized/Light | `surface_foreground` | #657B83 -> #506168 | surface_fg/surface | 3.64 -> 5.27 | Same value as foreground. |
| solarized/Light | `danger` | #DC322F -> #D72724 | danger/surface | 3.77 -> 4.08 | Danger ink: 4.29 with base3. HSL lightness -0.030. |
| solarized/Light | `danger` | #DC322F -> #D72724 | danger/background | 4.29 -> 4.63 | (same change) |
| solarized/Light | `danger` | #DC322F -> #D72724 | best_ink/danger | 4.29 -> 4.63 | (same change) |
| solarized/Light | `primary_hover` | #3C9BDC -> #1D699F | primary_fg/primary_hover | 2.82 -> 5.45 | Hover was lighter (2.82 text). Darker step. |
| solarized/Light | `primary_hover` | #3C9BDC -> #1D699F | primary_hover/primary | 1.21 -> 1.17 | (same change) |
| solarized/Light | `primary_pressed` | #1C6EAA -> #1A5F90 | primary_fg/primary_pressed | 5.04 -> 6.31 | Pressed was 1.08 from rest. Darker again. |
| solarized/Light | `primary_pressed` | #1C6EAA -> #1A5F90 | primary_pressed/primary | 1.48 -> 1.36 | (same change) |
| solarized/Light | `danger_hover` | #E4504D -> #BE2825 | best_ink/danger_hover | 3.51 -> 5.53 | Old pressed #BE2825 is the darker step. |
| solarized/Light | `danger_hover` | #E4504D -> #BE2825 | danger_hover/danger | 1.22 -> 1.19 | (same change) |
| solarized/Light | `danger_pressed` | #BE2825 -> #AD2422 | best_ink/danger_pressed | 5.53 -> 6.38 | Darker again. |
| solarized/Light | `danger_pressed` | #BE2825 -> #AD2422 | danger_pressed/danger | 1.29 -> 1.38 | (same change) |
| solarized/Dark | `foreground` | #839496 -> #9DAAAA | foreground/background | 4.75 -> 6.27 | P2 hierarchy: base1 lightened (HSL +0.036). Muted stays base1, 1.12 below body text. |
| solarized/Dark | `foreground` | #839496 -> #9DAAAA | foreground/surface | 4.11 -> 5.43 | (same change) |
| solarized/Dark | `foreground` | #839496 -> #9DAAAA | foreground/muted_fg | 1.70 -> 1.12 | (same change) |
| solarized/Dark | `surface_foreground` | #839496 -> #9DAAAA | surface_fg/surface | 4.11 -> 5.43 | Same value as foreground. |
| solarized/Dark | `danger` | #DC322F -> #E25754 | danger/surface | 2.81 -> 3.54 | Danger ink: 4.05 with the base03 ink. HSL lightness +0.046. |
| solarized/Dark | `danger` | #DC322F -> #E25754 | danger/background | 3.25 -> 4.09 | (same change) |
| solarized/Dark | `danger` | #DC322F -> #E25754 | best_ink/danger | 4.29 -> 4.62 | (same change) |
| solarized/Dark | `primary_pressed` | #1C6EAA -> #52A7E0 | primary_fg/primary_pressed | 5.04 -> 6.43 | P1-2: was 3.12 text. Lighter than hover. |
| solarized/Dark | `primary_pressed` | #1C6EAA -> #52A7E0 | primary_pressed/primary | 1.48 -> 1.40 | (same change) |
| solarized/Dark | `danger_hover` | #E4504D -> #E66C69 | best_ink/danger_hover | 3.97 -> 5.42 | Lighter step. |
| solarized/Dark | `danger_hover` | #E4504D -> #E66C69 | danger_hover/danger | 1.22 -> 1.17 | (same change) |
| solarized/Dark | `danger_pressed` | #BE2825 -> #E97E7B | best_ink/danger_pressed | 5.53 -> 6.24 | Lighter again. |
| solarized/Dark | `danger_pressed` | #BE2825 -> #E97E7B | danger_pressed/danger | 1.29 -> 1.35 | (same change) |
| ubuntu/Light | `primary_hover` | #CD491B -> #BC3E13 | primary_fg/primary_hover | 4.60 -> 5.47 | P1-3: hover was 1.00 from rest. Smallest darker step (the reviewer's #B83D12 is 1.22). Pressed #B13F17 (1.26) stays. |
| ubuntu/Light | `primary_hover` | #CD491B -> #BC3E13 | primary_hover/primary | 1.26 -> 1.18 | (same change) |
| ubuntu/Dark | `primary_pressed` | #CD491B -> #F27D54 | primary_fg/primary_pressed | 4.60 -> 7.05 | P1-2: was 4.10 text. Lighter than hover. |
| ubuntu/Dark | `primary_pressed` | #CD491B -> #F27D54 | primary_pressed/primary | 1.26 -> 1.36 | (same change) |
| ubuntu/Dark | `danger_pressed` | #DA2222 -> #FF9696 | best_ink/danger_pressed | 4.96 -> 9.04 | Was darker than rest under dark ink. Lighter than hover. |
| ubuntu/Dark | `danger_pressed` | #DA2222 -> #FF9696 | danger_pressed/danger | 1.70 -> 1.39 | (same change) |
| windows11/Dark | `primary_pressed` | #0067C0 -> #0078D4 | primary_fg/primary_pressed | 3.70 -> 4.64 | Untouched pack, P1-2 only (was 3.70): Windows blue #0078D4 (windows10 pressed, 4.64). |
| windows11/Dark | `primary_pressed` | #0067C0 -> #0078D4 | primary_pressed/primary | 2.83 -> 2.26 | (same change) |

### Untouched-pack exceptions (P1-3 state feedback, listed rather than retuned)
| Pack/mode | Family | Failure |
|---|---|---|
| default/dark | primary | Hover is lighter and pressed darker under dark ink. Pressed 1.243 is slightly nearer rest than hover 1.247. Pressed was moved only for the P1-2 text gate. |
| default/dark | danger | Hover is 1.13 from rest. Pressed is darker under dark ink. |
| high-contrast/Dark | primary | Hover is 1.09 from rest. Pressed is darker under black ink. |
| high-contrast/Dark | danger | Pressed is darker under dark ink. |
| material/Dark | primary | Hover is 1.13 from rest. Pressed is darker under dark ink (now 4.63 text). |
| material/Dark | danger | Hover is 1.14 from rest. Pressed #B3261E is darker under dark ink. Its text passes only with a flipped ink (foreground). |
| windows10/Dark | primary, danger | Primary hover is 1.12 from rest. In both families pressed is darker under black ink. |
| windows11/Light | primary, danger | Primary hover is lighter under white ink (4.76 text). Danger hover is 1.14 from rest. |
| windows11/Dark | primary, danger | Primary hover is 1.12 from rest. In both families pressed is darker under black ink. |

All exceptions still pass the P1-2 text gate and the danger-ink gate.

### Worst ratio per gate rule after the review fixes (all 24 pack/modes)
| Rule | Worst | Where |
|---|---|---|
| primary_fg on primary_hover >= 4.5 | 4.76 | windows11/Light (untouched) |
| primary_fg on primary_pressed >= 4.5 | 4.61 | default/dark (review value) |
| danger best ink on danger >= 4.5 | 4.62 | solarized/Dark |
| danger best ink on danger_hover >= 4.5 | 5.42 | gruvbox/Dark |
| danger best ink on danger_pressed >= 4.5 | 4.63 | windows10/Dark (untouched) |
| primary state step >= 1.15 (retuned packs) | 1.167 | apple/Light hover (first-pass value, not moved) |
| danger state step >= 1.15 (retuned packs) | 1.172 | gruvbox/Dark hover |
| muted_fg/background >= 4.5 | 4.61 | apple/Light |
| foreground on ghost_hover / ghost_pressed >= 4.5 | 4.58 | one-dark/Dark ghost_pressed (untouched value) |
| solarized foreground vs muted_fg >= 1.1 | 1.12 | solarized/Dark (light 1.15) |
| foreground vs surface >= 4.5 | 5.27 | solarized/Light |
| AA report pairs, ring, primary graphic and status rules | unchanged from the first pass | ring/surface is still 3.00 at solarized/Light (untouched). gruvbox/Dark ring/surface rises to 3.65. |

### Button painted pairs
`cargo test --locked -p acme-widgets button_text_meets_aa -- --nocapture` passes and prints no `TOKEN-LIMITED` lines.
