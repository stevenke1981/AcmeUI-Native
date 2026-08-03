#!/usr/bin/env python3
"""Run Clippy for the workspace and reject diagnostics in changed Rust files.

AcmeUI Native has historical Clippy warnings that predate incremental CI. This
script still compiles and lints the complete workspace with all features, but it
only turns warnings into a CI failure when their primary span belongs to a Rust
file changed by the current revision. New code therefore cannot add warnings,
while existing warnings can be paid down independently.
"""

from __future__ import annotations

import json
import os
import subprocess
import sys
from pathlib import Path
from typing import Any


def git(*args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["git", *args],
        check=check,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )


def valid_commit(value: str | None) -> bool:
    if not value or set(value) == {"0"}:
        return False
    return git("cat-file", "-e", f"{value}^{{commit}}", check=False).returncode == 0


def resolve_base() -> str:
    requested = os.environ.get("BASE_SHA")
    if valid_commit(requested):
        return requested or ""

    default_branch = os.environ.get("DEFAULT_BRANCH", "master")
    remote_default = f"origin/{default_branch}"
    if valid_commit(remote_default):
        merge_base = git("merge-base", "HEAD", remote_default).stdout.strip()
        if valid_commit(merge_base):
            return merge_base

    if valid_commit("HEAD^"):
        return "HEAD^"
    return "HEAD"


def normalize_path(value: str) -> str:
    return Path(value).as_posix().removeprefix("./")


def changed_rust_files(base: str) -> set[str]:
    output = git(
        "diff",
        "--name-only",
        "--diff-filter=ACMRT",
        base,
        "HEAD",
        "--",
        "*.rs",
    ).stdout
    return {normalize_path(path) for path in output.splitlines() if path.endswith(".rs")}


def diagnostic_files(message: dict[str, Any]) -> set[str]:
    spans = message.get("spans") or []
    primary = [span for span in spans if span.get("is_primary")]
    selected = primary or spans
    return {
        normalize_path(str(span["file_name"]))
        for span in selected
        if span.get("file_name")
    }


def main() -> int:
    base = resolve_base()
    changed = changed_rust_files(base)
    if not changed:
        print("No changed Rust files require Clippy validation.")
        return 0

    print("Running workspace Clippy; changed-file warning gate applies to:")
    for path in sorted(changed):
        print(f"  {path}")

    result = subprocess.run(
        [
            "cargo",
            "clippy",
            "--workspace",
            "--all-targets",
            "--all-features",
            "--message-format=json",
        ],
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )

    relevant: list[str] = []
    baseline_diagnostics = 0
    compiler_errors = 0

    for line in result.stdout.splitlines():
        try:
            record = json.loads(line)
        except json.JSONDecodeError:
            continue
        if record.get("reason") != "compiler-message":
            continue
        message = record.get("message") or {}
        level = message.get("level")
        if level not in {"warning", "error"}:
            continue
        if level == "error":
            compiler_errors += 1

        files = diagnostic_files(message)
        if files & changed:
            rendered = message.get("rendered") or message.get("message") or str(message)
            relevant.append(str(rendered).rstrip())
        else:
            baseline_diagnostics += 1

    if relevant:
        print("\nClippy diagnostics in changed files:\n", file=sys.stderr)
        print("\n\n".join(relevant), file=sys.stderr)
        return 1

    if result.returncode != 0:
        print(result.stderr, file=sys.stderr)
        print(
            "Clippy did not complete successfully despite finding no diagnostic "
            "in a changed file.",
            file=sys.stderr,
        )
        return result.returncode

    print(
        "Changed Rust files are Clippy-clean "
        f"({baseline_diagnostics} pre-existing workspace diagnostics ignored)."
    )
    if compiler_errors:
        print(
            f"Unexpectedly observed {compiler_errors} compiler errors.",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
