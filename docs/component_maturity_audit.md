# Component Maturity Audit — ACME-P0-003

**Date**: 2026-07-21  
**Scope**: All 106 component files in crates/acme-ui/src/  
**Method**: Heuristic grep-based classification (builder pattern, interaction dispatch, accessibility, test quality)  
**Auditor**: OpenCode Explore agent  

---

## 1. Overall Summary

| Level | Count | Definition |
|-------|-------|------------|
| **S0 Scaffold** | 0 | No component is a pure data struct — all produce a widget tree via build() or From<Builder> |
| **S1 Visual** | 56 | Builder with styling/variant methods, renders correctly, but NO on_* interaction dispatch |
| **S2 Interactive** | 50 | Has at least one on_click, on_change, on_select, etc. for message dispatch |
| **S3 Accessible** | 0 | **No component uses accesskit, ARIA roles, or explicit a11y attributes** |
| **S4 Production** | 0 | No component has golden tests, benchmarks, or manual validation evidence |

**Key findings:**
- 100% of components produce visual output (all at least S1)
- 47% have interaction dispatch (S2)
- 0% have accessibility metadata (S3) — **systemic gap**
- 0% meet production bar (S4) — **no golden tests or benchmarks exist**

---

## 2. Module-by-Module Breakdown

### 2.1 inputs/ (28 files) — Core Interactive Components

| File | Lines | Tests | Maturity | Evidence |
|------|-------|-------|----------|----------|
| autocomplete.rs | 190 | 5 | **S2** | on_select, on_input, on_blur |
| button_group.rs | 224 | 6 | **S2** | on_click per button |
| cascader.rs | 257 | 5 | **S2** | on_change |
| checkbox.rs | 106 | 3 | **S2** | on_click |
| color_picker.rs | 202 | 5 | **S2** | on_change |
| combobox.rs | 126 | 3 | **S2** | on_select |
| date_picker.rs | 350 | 14 | **S2** | on_change, on_open, strong edge-case tests (leap year, underflow) |
| form_message.rs | 176 | 6 | **S1** | No on_*; tone-based visual messages only |
| icon_button.rs | 135 | 4 | **S2** | on_click, variant/size styling |
| mentions.rs | 184 | 6 | **S2** | on_change |
| multi_select.rs | 210 | 5 | **S2** | on_change |
| number_input.rs | 182 | 5 | **S2** | on_change, clamping |
| password_input.rs | 181 | 5 | **S2** | on_change |
| pin_input.rs | 163 | 5 | **S2** | on_complete |
| radio.rs | 177 | 6 | **S2** | on_change |
| range_slider.rs | 201 | 5 | **S2** | on_change, min/max range |
| rating.rs | 155 | 5 | **S2** | on_change |
| search_input.rs | 108 | 3 | **S2** | on_change, on_submit |
| segmented_control.rs | 120 | 4 | **S1** | Visual tabs, but **NO** on_select dispatch |
| select.rs | 155 | 3 | **S2** | on_change |
| slider.rs | 236 | 6 | **S2** | on_change, step rounding, clamping edge-case tests |
| switch.rs | 114 | 3 | **S2** | on_toggle |
| tag_input.rs | 207 | 5 | **S2** | on_change, on_remove |
| textarea.rs | 120 | 4 | **S2** | on_change |
| time_picker.rs | 176 | 4 | **S2** | on_change |
| toggle_button.rs | 146 | 5 | **S2** | on_toggle |
| transfer.rs | 199 | 4 | **S2** | on_change |
| tree_select.rs | 212 | 5 | **S2** | on_select |

**Summary**: 27/28 S2, 1/28 S1 (segmented_control). This is the strongest module.

---

### 2.2 overlay/ (10 files) — Popups, Dialogs, Overlays

| File | Lines | Tests | Maturity | Evidence |
|------|-------|-------|----------|----------|
| about_dialog.rs | 191 | 5 | **S2** | on_close |
| command_palette.rs | 109 | 3 | **S2** | on_submit, on_close |
| confirm_dialog.rs | 152 | 3 | **S2** | on_confirm, on_cancel |
| context_menu.rs | 169 | 7 | **S1** | Only open state bool; no dispatch |
| drawer.rs | 96 | 3 | **S1** | Only open state bool; no dispatch |
| dropdown_menu.rs | 166 | 7 | **S1** | Only open state bool; no dispatch |
| fullscreen.rs | 140 | 6 | **S2** | on_close |
| hover_card.rs | 160 | 8 | **S1** | Only open state bool; no dispatch |
| modal.rs | 156 | 5 | **S2** | on_close |
| toast.rs | 109 | 4 | **S1** | No on_*; tone/duration/dismissible only |

**Summary**: 5/10 S2, 5/10 S1. Half are visual-only with no interaction dispatch.

---

### 2.3 desktop/ (13 files) — Desktop-Specific Components

| File | Lines | Tests | Maturity | Evidence |
|------|-------|-------|----------|----------|
| command_bar.rs | 157 | 5 | **S2** | on_submit |
| dock.rs | 136 | 3 | **S2** | on_select |
| image_view.rs | 123 | 3 | **S1** | No on_* |
| markdown_view.rs | 171 | 5 | **S1** | No on_* (renders Markdown) |
| menubar.rs | 221 | 8 | **S2** | on_click per item |
| navigation_view.rs | 113 | 3 | **S1** | No on_* |
| property_grid.rs | 116 | 3 | **S2** | on_change |
| resize_handle.rs | 112 | 3 | **S1** | No on_* |
| sidenav.rs | 269 | 10 | **S1** | **No interaction dispatch** despite 269 lines & 10 tests — purely visual navigation display |
| status_tray.rs | 164 | 6 | **S1** | No on_* |
| taskbar.rs | 176 | 6 | **S1** | No on_* |
| title_bar.rs | 130 | 3 | **S2** | on_close, on_minimize, on_maximize |
| window_controls.rs | 193 | 7 | **S2** | on_close, on_minimize, on_maximize |

**Summary**: 6/13 S2, 7/13 S1. Desktop has many visual-only shells.

---

### 2.4 foundations/ (29 files) — Primitive Building Blocks

| File | Lines | Tests | Maturity | Evidence |
|------|-------|-------|----------|----------|
| accordion.rs | 110 | 3 | **S2** | Section open/close via on_toggle |
| alert.rs | 240 | 8 | **S1** | No on_*; tone-based visual banner |
| aspect_ratio.rs | 122 | 6 | **S1** | Pure layout container |
| avatar.rs | 106 | 3 | **S1** | No on_* |
| badge.rs | 220 | 7 | **S1** | Tone/size/outlined styling only |
| banner.rs | 213 | 5 | **S2** | on_dismiss |
| calendar.rs | 269 | 10 | **S1** | **No on_select** — renders calendar grid visually only |
| collapsible.rs | 189 | 6 | **S2** | on_toggle |
| descriptions.rs | 142 | 5 | **S1** | Display-only key-value list |
| diff_viewer.rs | 173 | 6 | **S1** | Display-only diff rendering |
| drag_region.rs | 125 | 5 | **S1** | No on_* (visual drag indicator) |
| drop_zone.rs | 176 | 6 | **S2** | on_drop, on_drag_over, on_drag_leave |
| empty_state.rs | 99 | 2 | **S1** | No on_* |
| flex.rs | 166 | 5 | **S1** | Pure layout container |
| focus_ring.rs | 117 | 6 | **S1** | Visual indicator only |
| hero.rs | 144 | 5 | **S1** | No on_* |
| icon.rs | 139 | 2 | **S1** | No on_* |
| kbd.rs | 68 | 2 | **S1** | Display-only shortcut label |
| line_numbers.rs | 152 | 5 | **S1** | No on_* |
| link.rs | 168 | 8 | **S2** | on_click |
| list.rs | 245 | 7 | **S2** | on_select |
| progress.rs | 219 | 9 | **S1** | No on_*; visual only |
| result.rs | 133 | 5 | **S1** | No on_* |
| skeleton.rs | 89 | 3 | **S1** | No on_* |
| spinner.rs | 87 | 2 | **S1** | No on_*; size/tone styling |
| statistic.rs | 135 | 5 | **S1** | No on_* |
| tag.rs | 147 | 3 | **S1** | No on_* |
| timeline.rs | 193 | 5 | **S1** | No on_* |
| watermark.rs | 155 | 6 | **S1** | No on_* |

**Summary**: 6/29 S2, 23/29 S1. Most are visual primitives. **calendar.rs** is the standout — 269 lines, 10 tests, but no on_select dispatch.

---

### 2.5 layout/ (14 files) — Page Structure

| File | Lines | Tests | Maturity | Evidence |
|------|-------|-------|----------|----------|
| breadcrumb.rs | 140 | 4 | **S2** | on_click per breadcrumb item |
| form.rs | 260 | 8 | **S1** | No on_submit; visual field layout container |
| grid.rs | 97 | 3 | **S1** | Pure layout container |
| navigation_menu.rs | 186 | 7 | **S2** | on_click per item |
| page_header.rs | 159 | 4 | **S2** | on_back |
| pagination.rs | 154 | 3 | **S2** | on_page_change |
| scroll_area.rs | 109 | 7 | **S1** | No on_*; scroll container |
| section.rs | 166 | 6 | **S1** | No on_* |
| settings_page.rs | 110 | 3 | **S1** | No on_* |
| split_panel.rs | 92 | 4 | **S1** | No on_*; pure layout |
| status_bar.rs | 82 | 3 | **S1** | No on_* |
| stepper.rs | 80 | 3 | **S1** | No on_*; visual step indicator |
| tabs.rs | 124 | 3 | **S1** | No on_*; visual tab display |
| toolbar.rs | 65 | 3 | **S1** | No on_*; tool button container |

**Summary**: 4/14 S2, 10/14 S1. Layout components are predominantly structural.

---

### 2.6 charts/ (8 files) — Data Visualization

| File | Lines | Tests | Maturity | Evidence |
|------|-------|-------|----------|----------|
| area_chart.rs | 77 | 1 | **S1** | No on_*; static visual |
| bar_chart.rs | 89 | 1 | **S1** | No on_* |
| donut_chart.rs | 190 | 5 | **S1** | No on_* |
| gauge.rs | 80 | 1 | **S1** | No on_* |
| line_chart.rs | 100 | 1 | **S1** | No on_* |
| pie_chart.rs | 90 | 1 | **S1** | No on_* |
| scatter_chart.rs | 154 | 5 | **S1** | No on_* |
| sparkline.rs | 64 | 1 | **S1** | No on_* |

**Summary**: 0/8 S2, 8/8 S1. All charts are static visual renderings with no interaction.

---

### 2.7 mobile/ (5 files) — Touch-Optimized Components

| File | Lines | Tests | Maturity | Evidence |
|------|-------|-------|----------|----------|
| action_sheet.rs | 182 | 6 | **S2** | on_select |
| bottom_nav.rs | 141 | 3 | **S2** | on_select |
| bottom_sheet.rs | 126 | 3 | **S2** | on_close |
| pull_to_refresh.rs | 131 | 3 | **S2** | on_refresh |
| search_bar.rs | 174 | 6 | **S2** | on_change, on_submit, auto_focus |

**Summary**: 5/5 S2. Strongest module ratio-wise — all are interactive.

---

### 2.8 browser/ (4 files) — Web-like Components

| File | Lines | Tests | Maturity | Evidence |
|------|-------|-------|----------|----------|
| carousel.rs | 209 | 3 | **S2** | on_change (page index) |
| image_gallery.rs | 198 | 6 | **S1** | No on_*; static image grid |
| lightbox.rs | 120 | 3 | **S2** | on_close |
| zoom_view.rs | 164 | 3 | **S2** | on_zoom_change |

**Summary**: 3/4 S2, 1/4 S1. Strong except image_gallery.

---

## 3. Top 5 Components Needing Urgent Improvement

### 🔴 #1: calendar.rs (foundations) — 269 lines, 10 tests, but NO interaction
A full calendar grid renders with days, months, years, but dispatches **no message** when a day is clicked. This is the largest S1 component with the widest gap between apparent complexity and actual capability.

### 🔴 #2: segmented_control.rs (inputs) — 120 lines, in inputs/ module, but NO interaction
Sits in the inputs module but has no on_select or on_change. Creates button children internally but doesn't wire them to dispatchers. Users cannot react to selection changes.

### 🔴 #3: sidenav.rs (desktop) — 269 lines, 10 tests, but NO interaction dispatch
Large navigation component with sections, items, icons, and selected state — but no on_select or on_navigate handler. The selected state is set at construction only.

### 🔴 #4: context_menu.rs, dropdown_menu.rs, drawer.rs, hover_card.rs (overlay) — **Systemic S1 gap**
All four overlay components accept open/closed state but none dispatch on_open/on_close messages. They are passive visual shells.

### 🔴 #5: image_gallery.rs (browser) — 198 lines, 6 tests, but no interaction
Gallery renders image thumbnails and a grid layout but has no on_select for choosing an image and no on_zoom handler.

---

## 4. S2/S3/S4 Assessment

### Genuine S2 Components (interactive, with message dispatch)
The strongest examples of S2 components with both interaction and solid test coverage:

| Component | Why |
|-----------|-----|
| **date_picker.rs** | on_change, on_open, 14 tests including leap year, month-start weekday, underflow edge cases |
| **slider.rs** | on_change, step rounding, value clamping, 6 tests with behavioral verification |
| **drop_zone.rs** | on_drop, on_drag_over, on_drag_leave — three distinct interaction handlers |
| **cascader.rs** | on_change with hierarchical data dispatch |
| **	oggle_button.rs** | on_toggle with selected state management |
| **	itle_bar.rs / window_controls.rs** | on_close, on_minimize, on_maximize — full desktop window control |
| **button_group.rs** | Per-button on_click dispatch |

### S0/S1 Scaffolds (no interaction — visual shells)
The following components have full visual rendering but **no way for users to react to interaction**:

| Component | Module | Lines | Impact |
|-----------|--------|-------|--------|
| calendar.rs | foundations | 269 | Large, complex grid — no date selection dispatch |
| segmented_control.rs | inputs | 120 | Lives in inputs but can't report selection |
| sidenav.rs | desktop | 269 | Full nav tree — no selection dispatch |
| context_menu.rs | overlay | 169 | Right-click menu — no close/select dispatch |
| dropdown_menu.rs | overlay | 166 | Dropdown — no change dispatch |
| hover_card.rs | overlay | 160 | Hover card — no open/close dispatch |
| drawer.rs | overlay | 96 | Slide-out panel — no close dispatch |
| toast.rs | overlay | 109 | Notification — no dismiss dispatch |
| image_gallery.rs | browser | 198 | Image selection — no select dispatch |
| All 8 charts | charts | 64-190 | Static visualizations — no hover/click dispatch |
| All 10 layout S1s | layout | 65-260 | Structural containers (expected) |

---

## 5. Accessibility Gap Analysis (S3)

**No component in the entire codebase meets S3.** Specific findings:

| Requirement | Found? | Evidence |
|-------------|--------|----------|
| AccessKit integration | ❌ | No accesskit dependency or import anywhere |
| Explicit role attribute | ❌ | 0 occurrences of role() or accessibility role setting |
| Explicit name/label | ❌ | label() is used for display text, not a11y labels |
| Focus management | ⚠️ | Only search_bar.rs has auto_focus — not a11y focus management |
| Keyboard navigation | ❌ | No 	ab_index, on_key, Focusable, or keyboard_nav |
| ARIA-like attributes | ❌ | No aria-*, described_by, labelled_by patterns |

---

## 6. Production Readiness (S4 Gap)

No component meets the full S4 bar. The closest candidates:

| Component | Tests | What's Missing |
|-----------|-------|----------------|
| **date_picker.rs** | 14 (leap year, underflow, weekday) | Golden tests, benchmarks, manual validation |
| **slider.rs** | 6 (step rounding, clamping) | Golden tests, benchmarks, manual validation |
| **sidenav.rs** | 10 (layout, defaults) | Interaction dispatch, golden tests, benchmarks |
| **calendar.rs** | 10 (layout, month calc) | Interaction dispatch, golden tests, benchmarks |
| **alert.rs** | 8 (tone, structure) | Edge-case interaction tests, golden tests |

**Systemic gaps:**
- ❌ No golden/snapshot tests exist anywhere
- ❌ No benchmarks exist anywhere
- ❌ No manual validation evidence documented
- ❌ No performance tests

---

## 7. Recommendations

### Immediate (P0)
1. **Add S3 across all S2 components** — Implement AccessKit or a lightweight Accessible trait with role, label, action, focus for all interactive components
2. **Fix the 5 urgent S1 scaffolds** — Add on_select to calendar.rs, on_select to segmented_control.rs, on_select to sidenav.rs, on_open/on_close to overlay components, on_select to image_gallery.rs

### Short-term (P1)
3. **Add golden tests** — Use insta or custom snapshot for widget tree structure verification
4. **Add benchmarks** — At minimum for the 10 most-used components (checkbox, button, input, slider, select, date_picker, modal, form, list, menu)

### Medium-term (P2)
5. **Move S1 layout/chart components to S2** where applicable (chart hover/click, tab change, scroll events)
6. **Establish S4 criteria** — Define golden test format, benchmark regression gates, and manual validation checklist

---

## 8. Final Summary

`
Module        Total    S0   S1   S2   S3   S4
──────        ─────    ──   ──   ──   ──   ──
inputs           28     0    1   27    0    0
overlay          10     0    5    5    0    0
desktop          13     0    7    6    0    0
foundations      29     0   23    6    0    0
layout           14     0   10    4    0    0
charts            8     0    8    0    0    0
mobile            5     0    0    5    0    0
browser           4     0    1    3    0    0
──────        ─────    ──   ──   ──   ──   ──
Total           111     0   56   50    0    0
               (100%)  (0%) (50%) (45%) (0%) (0%)
`

**Urgency rating**: 🟡 **Moderate** — 50% of components are interactive (S2+), but 0% have accessibility or production verification. The scaffold ratio (50% S1) is acceptable for a young codebase, but the accessibility gap is critical before this can be used in production applications.
