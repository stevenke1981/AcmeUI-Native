#!/usr/bin/env python3
"""Check that the AcmeUI workspace crates are packageable. Python 3.10+, stdlib only.

Checks, in order:
1. Static manifest rules (no cargo packaging needed):
   - every package under apps/ sets `publish = false`;
   - every package under crates/ leaves `publish` unset;
   - every internal (workspace) path dependency carries a version requirement
     that matches the dependency's own workspace version (no `*`, no drift).
2. Packaging: `cargo package -p <library> --locked --no-verify` for each library.
   `cargo package --workspace` is not usable on cargo 1.88 while internal crates
   are unpublished: it fails at the first dependent crate with
   "no matching package named `acme-text` found" in the crates.io index. Each
   library is therefore packaged on its own, and its internal dependencies
   (transitive closure) are redirected to their local paths with
   `--config patch.crates-io.<dep>.path=...`. This verifies manifest
   normalisation and file inclusion. It does NOT build the packaged crate
   against the registry; `--no-verify` skips that step.
3. Per `.crate`: file count and uncompressed size from the archive, plus a
   hard limit of 10 MiB on the compressed `.crate` file.

Exit codes: 0 = all checks PASS, 1 = at least one FAIL, 2 = environment error
(cargo missing or `cargo metadata` failed).

Usage: python scripts/check_package.py [--allow-dirty]
  --allow-dirty  pass --allow-dirty to cargo package (local runs on an
                 uncommitted tree only; CI must not use it).
"""
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tarfile
from dataclasses import dataclass, field
from pathlib import Path

MAX_CRATE_BYTES = 10 * 1024 * 1024
LIBRARY_AREA = "crates"
APP_AREA = "apps"


@dataclass
class Package:
    name: str
    version: str
    area: str
    publish: list[str] | None
    directory: Path
    dependencies: list[dict] = field(default_factory=list)


@dataclass
class Result:
    name: str
    status: str
    files: int | None = None
    uncompressed: int | None = None
    crate_bytes: int | None = None
    detail: str = ""


def run_cargo(args: list[str], cwd: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["cargo", *args],
        cwd=cwd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=False,
    )


def tail(text: str, lines: int = 12) -> str:
    return "\n".join(text.strip().splitlines()[-lines:])


def load_packages(root: Path) -> tuple[dict[str, Package], str]:
    proc = run_cargo(["metadata", "--locked", "--no-deps", "--format-version", "1"], root)
    if proc.returncode != 0:
        raise RuntimeError("cargo metadata failed:\n" + tail(proc.stderr))
    meta = json.loads(proc.stdout)
    workspace_root = Path(meta["workspace_root"]).resolve()
    packages: dict[str, Package] = {}
    for raw in meta["packages"]:
        manifest_dir = Path(raw["manifest_path"]).resolve().parent
        area = manifest_dir.relative_to(workspace_root).parts[0]
        packages[raw["name"]] = Package(
            name=raw["name"],
            version=raw["version"],
            area=area,
            publish=raw["publish"],
            directory=manifest_dir,
            dependencies=raw["dependencies"],
        )
    return packages, meta["target_directory"]


def internal_deps(pkg: Package, members: dict[str, Package]) -> list[dict]:
    return [d for d in pkg.dependencies if d["name"] in members and d.get("path")]


def static_problems(members: dict[str, Package]) -> list[str]:
    problems: list[str] = []
    for name, pkg in sorted(members.items()):
        if pkg.area == APP_AREA and pkg.publish != []:
            problems.append(f"{name}: apps/* package must set publish = false")
        if pkg.area == LIBRARY_AREA and pkg.publish is not None:
            problems.append(f"{name}: library must leave publish unset")
        for dep in internal_deps(pkg, members):
            target_version = members[dep["name"]].version
            req = dep["req"]
            kind = dep.get("kind") or "normal"
            label = f"{name} -> {dep['name']} ({kind})"
            if req == "*":
                problems.append(f"{label}: internal path dependency has no version")
            elif req not in (f"^{target_version}", f"={target_version}", target_version):
                problems.append(
                    f"{label}: version requirement '{req}' does not match "
                    f"workspace version {target_version}"
                )
    return problems


def closure(name: str, members: dict[str, Package]) -> list[str]:
    seen: set[str] = set()
    stack = [name]
    while stack:
        current = stack.pop()
        for dep in internal_deps(members[current], members):
            dep_name = dep["name"]
            if dep_name != name and dep_name not in seen:
                seen.add(dep_name)
                stack.append(dep_name)
    return sorted(seen)


def package_args(name: str, members: dict[str, Package], allow_dirty: bool) -> list[str]:
    args = ["package", "-p", name, "--locked", "--no-verify"]
    if allow_dirty:
        args.append("--allow-dirty")
    for dep_name in closure(name, members):
        dep_dir = members[dep_name].directory.as_posix()
        args += ["--config", f'patch.crates-io.{dep_name}.path="{dep_dir}"']
    return args


def crate_contents(path: Path) -> tuple[int, int]:
    with tarfile.open(path, "r:gz") as archive:
        files = [m for m in archive.getmembers() if m.isfile()]
    return len(files), sum(m.size for m in files)


def package_one(name: str, members: dict[str, Package], target_dir: Path, allow_dirty: bool, root: Path) -> Result:
    pkg = members[name]
    crate_file = target_dir / "package" / f"{name}-{pkg.version}.crate"
    crate_file.unlink(missing_ok=True)
    proc = run_cargo(package_args(name, members, allow_dirty), root)
    if proc.returncode != 0:
        return Result(name, "FAIL", detail="cargo package exit " + str(proc.returncode) + "\n" + tail(proc.stderr))
    if not crate_file.is_file():
        return Result(name, "FAIL", detail=f"cargo succeeded but {crate_file} is missing")
    files, uncompressed = crate_contents(crate_file)
    size = crate_file.stat().st_size
    status = "PASS" if size <= MAX_CRATE_BYTES else "FAIL"
    detail = "" if status == "PASS" else f".crate is {size} bytes, limit {MAX_CRATE_BYTES}"
    return Result(name, status, files, uncompressed, size, detail)


def kib(value: int | None) -> str:
    return "-" if value is None else f"{value / 1024:.1f} KiB"


def print_table(results: list[Result]) -> None:
    header = f"{'crate':<22}{'files':>7}{'uncompressed':>15}{'.crate':>13}  status"
    print(header)
    print("-" * len(header))
    for r in results:
        print(f"{r.name:<22}{'-' if r.files is None else r.files:>7}{kib(r.uncompressed):>15}{kib(r.crate_bytes):>13}  {r.status}")
        if r.detail:
            for line in r.detail.splitlines():
                print("    " + line)


def main(argv: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--allow-dirty", action="store_true", help="pass --allow-dirty to cargo package (local use only)")
    args = parser.parse_args(argv)

    root = Path(__file__).resolve().parent.parent
    try:
        members, target_text = load_packages(root)
    except (OSError, RuntimeError, json.JSONDecodeError) as error:
        print(f"ENVIRONMENT ERROR: {error}", file=sys.stderr)
        return 2
    target_dir = Path(target_text)

    problems = static_problems(members)
    print("static manifest checks: " + ("PASS" if not problems else "FAIL"))
    for problem in problems:
        print("  - " + problem)

    libraries = sorted(n for n, p in members.items() if p.area == LIBRARY_AREA)
    results = [package_one(name, members, target_dir, args.allow_dirty, root) for name in libraries]
    print()
    print_table(results)

    failed = bool(problems) or any(r.status != "PASS" for r in results)
    print()
    print("OVERALL: " + ("FAIL" if failed else "PASS") + f" ({len(libraries)} libraries, apps excluded: "
          + ", ".join(sorted(n for n, p in members.items() if p.area == APP_AREA)) + ")")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
