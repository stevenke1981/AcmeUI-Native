# REPORT P5 — TreeItem actions and content walk (ROADMAP 013)

Worktree a separate worktree, branch `sdk/roadmap-013-a11y`. Only `crates/acme-accessibility/**` changed, plus this report. Nothing was added, committed, stashed or pushed.

## Design
- **Actions.** Every `TreeItem` gets `Action::Focus` and `Action::Click`. An item with children also gets `Action::Collapse` when it is expanded, or `Action::Expand` when it is collapsed. AccessKit 0.21.1 has both actions.
- **Routing.**
  - `AccessibilityAction::Focus` and `Click` on a TreeItem id already worked through the existing `route_action`: FocusChanged, then a PointerButton press at the row centre. The press selects the row, the same as a mouse click.
  - Two additive variants are new: `AccessibilityAction::Expand(id)` and `Collapse(id)`. They route to FocusChanged, a PointerButton press (this selects the row), then Key press and release of `ArrowRight` (expand) or `ArrowLeft` (collapse). That is what the keyboard does on the selected row. No new `PlatformEvent` was needed.
  - `resolve_node_center` now falls back to the node's bounds in the stored tree when the snapshot has no entry. On the retained path TreeItem ids are virtual (>= 2^56), so previously a Click on them resolved to (0,0). This is also a bug fix for virtual-id nodes in general.
- **Content walk.**
  - Item content that is more than plain text is walked with `walk(.., Slot::Plain, Lowering::Unlowered)`. "Plain text" means Label, or Row, Column, Stack or Card containing only plain text.
  - The content root becomes the sole child of the TreeItem, with the correct role, name and actions (for example a Button gets Click and Focus).
  - Layout numbers no node for row content, so the walk takes no layout id. The layout-ID order is unchanged.
  - The content root has no layout rectangle, so it reads its row's rectangle. Nested content nodes have no bounds, as with other unlowered widgets.
  - Name rule: the first Label text, as before. Only if the content has no Label, fall back to the first Button label or TextInput name.
- **Plain-text trees.** No children and no extra nodes. Only the new actions are added.

## Files changed
- `crates/acme-accessibility/src/structure.rs`: `tree_item_ids`, `tree_item_node`, `is_plain_content`, `control_name`, and the test module registration.
- `crates/acme-accessibility/src/lib.rs`: the `Expand` and `Collapse` variants, their routing, and the `resolve_node_center` fallback.
- `crates/acme-accessibility/src/structure/tests_tree_actions.rs` (new): 8 tests.
- `docs/team/roadmap-013/TASK-P5-a11y.md` and this report. The task file was supplied by the lead.

## Tests
- **RED** (new tests with only stub enum variants; routing returned an empty list): 6 failed, 2 passed.
  - Failed:
    - `every_tree_item_supports_focus_and_click`
    - `only_items_with_children_expose_expand_or_collapse`
    - `widgets_in_item_content_become_children_of_the_item`
    - `content_root_reads_its_row_rect_and_ids_are_unique`
    - `click_on_a_tree_item_selects_its_row_on_both_paths`
    - `expand_and_collapse_select_the_row_then_press_the_arrow_key`
  - Passed (these guard against regressions, so they were expected to pass before the change):
    - `plain_text_tree_adds_only_the_actions`
    - `walking_item_content_keeps_the_layout_id_order`
- **GREEN:** `cargo test -p acme-accessibility` ran 118 tests (110 existing, 8 new). 118 passed.
- **Mutation:** I changed the content walk to `Lowering::Lowered`. Layout ids were then consumed, and `walking_item_content_keeps_the_layout_id_order` failed ("button after the tree": 117 passed, 1 failed). I reverted the change, and 118 passed again.

## Commands
| Command | Result |
|---|---|
| `cargo fmt -p acme-accessibility -- --check` | exit 0 |
| `cargo clippy -p acme-accessibility --all-targets -- -D warnings` | exit 0 |
| `cargo test -p acme-accessibility` | 118 passed, 0 failed |
| `cargo check --workspace --all-targets` | exit 0 (playground, gallery and acme-gallery compile) |

## NOT_RUN
- A real screen reader (NVDA, Narrator) and a real AccessKit platform adapter.
- Wiring the new `Expand` and `Collapse` variants into the app. No consumer outside this crate matches on `AccessibilityAction`, so the variants are additive. The app or winit bridge must map AccessKit `Action::Expand` and `Action::Collapse` requests to them.
- The `FocusChanged.node_id` for a virtual TreeItem id on the retained path. The app is not checked to resolve it to a row.
