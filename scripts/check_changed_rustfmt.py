#!/usr/bin/env python3
"""Check rustfmt only for Rust files changed by the current revision.

The repository contains historical rustfmt drift. Formatting the whole tree and
then checking only the changed paths keeps CI strict for new work without forcing
an unrelated repository-wide formatting commit.
"""

from __future__ import annotations

import os
import subprocess
import sys
from pathlib import Path


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


def main() -> int:
    base = resolve_base()
    output = git(
        "diff",
        "--name-only",
        "--diff-filter=ACMRT",
        base,
        "HEAD",
        "--",
        "*.rs",
    ).stdout
    changed = [path for path in output.splitlines() if Path(path).suffix == ".rs"]
    if not changed:
        print("No changed Rust files require rustfmt validation.")
        return 0

    print("Checking rustfmt for:")
    for path in changed:
        print(f"  {path}")

    subprocess.run(["cargo", "fmt", "--all"], check=True)
    diff = subprocess.run(["git", "diff", "--exit-code", "--", *changed])
    if diff.returncode != 0:
        print("Changed Rust files are not rustfmt-clean.", file=sys.stderr)
        return diff.returncode
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
