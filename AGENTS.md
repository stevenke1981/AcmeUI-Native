# AGENTS.md

## Workflow
1. Read spec/plan/todos.
2. Select one bounded task.
3. Define acceptance criteria.
4. Implement without unnecessary public API expansion.
5. Add tests and Gallery evidence.
6. Run targeted crate checks.
7. Run workspace gates only at milestone completion.
8. Update todos/final truthfully.

## Restrictions
- Never add GPUI.
- Never hardcode theme colors inside widgets.
- Never use byte offsets for text cursors.
- Never expose platform-specific types publicly.
- Never claim Traditional Chinese IME works without manual validation.
- Never use cargo clean as a routine fix.
