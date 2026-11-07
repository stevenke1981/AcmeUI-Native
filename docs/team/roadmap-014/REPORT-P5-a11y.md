# REPORT P5 — AccessKit request mapping and the TreeItem press point (ROADMAP 014)

Branch `sdk/roadmap-014-a11y`. Only `crates/acme-accessibility/**` changed, plus this report. Nothing was added, committed, stashed or pushed.

## Design
- **ID inverse.** New public `from_access_id(AccessNodeId) -> NodeId`, the exact inverse of `to_access_id`. It is lossless for every `u64`, so virtual ids (>= 2^56) round-trip.
- **Request mapping.** New public, pure `AccessibilityAction::from_request(&ActionRequest) -> Option<Self>`:

  | AccessKit action | Result |
  |---|---|
  | `Focus` | `Focus` |
  | `Click` | `Click` |
  | `Expand` | `Expand` |
  | `Collapse` | `Collapse` |
  | `ScrollIntoView` | `ScrollIntoView` |
  | `SetValue` | `SetValue`, only with `ActionData::Value`; a missing or non-string value gives `None` |
  | any other action | `None` (never a panic) |

  AccessKit 0.21.1 has no separate default action, so `Click` is the default.
- **`AccessibilityAdapter::handle_request(&ActionRequest) -> Vec<PlatformEvent>`** composes `from_request` and `route_action`. An unsupported request gives an empty vec. The doc comment explains the wiring: an `ActionHandler::do_action` forwards each request to the UI thread, calls `handle_request`, and dispatches the events in order. No new dependency; `accesskit_winit` was not added.
- **Press point (revised after review).**
  - Private `press_point(id)`. If the node in the stored tree has `Role::TreeItem` and has bounds, it presses `(row.right - inset, row centre y)`, with `inset = min(2px, row width / 2)`. The first version pressed the leading edge. The review pointed out that row content (checkbox, disclosure arrow, icon) sits at the start, so the trailing edge, which is usually empty, is safer.
  - Every other node keeps the old rule: snapshot centre, then tree-bounds fallback.
  - `Click`, `Activate`, `Expand` and `Collapse` all use it.
  - Limitation: a widget that fills the whole row width could still be hit at the trailing edge. Row content has no layout rectangle of its own, so this cannot be hit-tested here.
## Files changed
- `crates/acme-accessibility/src/lib.rs`
- `crates/acme-accessibility/src/tests_requests.rs` (new, 6 tests)
- `crates/acme-accessibility/src/structure/tests_tree_actions.rs`: two ROADMAP 013 tests now expect the trailing-edge press point instead of the centre. That change is intended.

## AccessKit types used
`ActionRequest`, `Action` (`Focus`, `Click`, `Expand`, `Collapse`, `ScrollIntoView`, `SetValue`), `ActionData::Value`, `NodeId`, `Role::TreeItem`, `Rect`.

## Tests
- **First round (leading edge):**
  - RED: 5 of the 6 new tests failed with stubs (a stub returning `None`, an empty vec and id 0); 1 passed.
  - GREEN: `cargo test` ran 124 tests and 124 passed, after updating two ROADMAP 013 tests.
  - Mutations: disabling the TreeItem rule failed 3 tests, and mapping `Expand` to `Collapse` failed 1 test. Both were reverted.
- **Revision (trailing edge):**
  - I switched `press_point` to the trailing edge in `lib.rs` and updated the tests in the same step. A separate RED run for the revision was not taken. The mutation below is the failing-test evidence.
  - `tests_requests.rs`: `tree_item_press_is_at_the_trailing_edge_not_the_centre_or_leading_content` now asserts `x > row centre` and `x == right - min(2, width/2)`, with `y` at the row's centre, for Click, Activate, Expand and Collapse on all 4 items on both ID paths.
    - Items: three with a Button at the start (the original centre-button case, which still passes) and one new row `d`.
    - Row `d` starts with a Row holding a Button labelled "Select", then a Label. This repository has no `Checkbox` widget, so the button stands in for it. Content has no rectangle of its own, so the leading half of the row stands for the checkbox's area, and the press must not land in it.
  - `ordinary_button_click_still_presses_the_centre` is unchanged and still passes.
  - The two ROADMAP 013 routing tests now expect `(right - 2, centre y)`.
  - GREEN: 124 passed, 0 failed.
- **Mutation (revision):** I changed the press point back to the leading edge (`b.x0 + inset`). 3 tests failed: the new trailing-edge test and the two ROADMAP 013 routing tests. I reverted it, and 124 passed.
## Commands
| Command | Result |
|---|---|
| `cargo fmt -p acme-accessibility -- --check` | exit 0 |
| `cargo clippy -p acme-accessibility --all-targets -- -D warnings` | exit 0 |
| `cargo test -p acme-accessibility` | 124 passed, 0 failed |
| `cargo check --workspace --all-targets` | exit 0 |

## NOT_RUN
- A real AccessKit platform adapter, `accesskit_winit` wiring and a real screen reader. `handle_request` is only unit-tested.
- Hit-testing the press point against rendered row content, and a real `Checkbox` widget (none exists in this repository).
- Other AccessKit actions (`Blur`, `Increment`, `ShowTooltip` and so on). They map to `None`.
