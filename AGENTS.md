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

## Agent team
- Load `.agents/README.md` and the assigned role contract before a multi-role task.
- Follow `docs/agent-team/WORKFLOW.md`; reserve exact files before parallel edits.
- Keep shared manifests, Scene contracts, and platform integration single-writer.
- Record actual execution mode and tested SHA; role files are not independent-agent evidence.
- Use `python scripts/verify_sdk.py --profile cpu` for CPU evidence. A dry run is NOT_RUN.
