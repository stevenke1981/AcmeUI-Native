#!/usr/bin/env python3
"""Run bounded SDK checks and preserve evidence. Python 3.10+, no dependencies."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import signal
import subprocess
import sys
import time
from datetime import datetime, timezone


def now() -> str:
    return datetime.now(timezone.utc).isoformat()


def profiles() -> dict[str, list[list[str]]]:
    return {
        "cpu": [
            ["cargo", "fmt", "-p", "acme-render-cpu", "--", "--check"],
            ["cargo", "test", "--locked", "-p", "acme-render-cpu"],
            ["cargo", "clippy", "--locked", "-p", "acme-render-cpu", "--all-targets", "--no-deps", "--", "-D", "warnings"],
        ],
        "milestone": [
            ["cargo", "fmt", "--all", "--", "--check"],
            ["cargo", "check", "--locked", "--workspace", "--all-targets"],
            ["cargo", "check", "--locked", "-p", "acme-platform", "--no-default-features"],
            ["cargo", "test", "--locked", "--workspace"],
            ["cargo", "test", "--locked", "-p", "acme-ui", "--all-features"],
            ["cargo", "build", "--locked", "--release", "--workspace"],
        ],
    }


def probe(args: list[str], root: Path) -> str | None:
    try:
        result = subprocess.run(args, cwd=root, capture_output=True, text=True, timeout=10)
        return result.stdout.strip() if result.returncode == 0 else None
    except (OSError, subprocess.TimeoutExpired):
        return None


def run_check(args: list[str], root: Path, log: Path, timeout: float) -> dict:
    record = {"command": args, "cwd": str(root), "started_at": now(),
              "status": "NOT_RUN", "exit_code": None, "log": str(log)}
    started = time.monotonic()
    executable = shutil.which(args[0])
    with log.open("w", encoding="utf-8", newline="\n") as output:
        if executable is None:
            record.update(status="BLOCKED", reason=f"Missing executable: {args[0]}")
            output.write(record["reason"] + "\n")
        else:
            process = None
            try:
                process = subprocess.Popen(
                    [executable, *args[1:]], cwd=root, stdout=output,
                    stderr=subprocess.STDOUT, shell=False,
                    start_new_session=os.name != "nt",
                    creationflags=subprocess.CREATE_NEW_PROCESS_GROUP if os.name == "nt" else 0,
                )
                code = process.wait(timeout=timeout)
                record.update(status="PASS" if code == 0 else "FAIL", exit_code=code)
            except subprocess.TimeoutExpired:
                if os.name == "nt":
                    subprocess.run(["taskkill", "/PID", str(process.pid), "/T", "/F"],
                                   stdout=output, stderr=subprocess.STDOUT, check=False)
                else:
                    try:
                        os.killpg(process.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                process.wait()
                record.update(status="FAIL", exit_code=process.returncode, reason="Timeout")
                output.write(f"\nTIMEOUT after {timeout} seconds\n")
            except OSError as error:
                record.update(status="BLOCKED", reason=str(error))
                output.write(str(error) + "\n")
    record.update(finished_at=now(), duration_seconds=round(time.monotonic() - started, 3),
                  log_sha256=hashlib.sha256(log.read_bytes()).hexdigest())
    return record


def run_suite(commands: list[list[str]], root: Path, output: Path,
              timeout: float, dry_run: bool = False) -> dict:
    output.mkdir(parents=True, exist_ok=False)
    checks = []
    blocked = False
    for index, args in enumerate(commands, 1):
        if dry_run or blocked:
            record = {"command": args, "status": "NOT_RUN", "exit_code": None,
                      "reason": "Dry run" if dry_run else "Earlier check did not pass"}
        else:
            record = run_check(args, root, output / f"{index:02d}.log", timeout)
            blocked = record["status"] != "PASS"
        checks.append(record)
        print(f"[{record['status']}] {' '.join(args)}", flush=True)
    overall = next((r["status"] for r in checks if r["status"] != "PASS"), "NOT_RUN" if not checks else "PASS")
    return {"schema_version": 1, "status": overall, "checks": checks}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", choices=profiles(), default="cpu")
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument("--output", type=Path, help="New evidence directory; existing directories are never overwritten")
    parser.add_argument("--timeout", type=float, default=900, help="Seconds per check")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    root = args.root.resolve()
    if args.timeout <= 0 or not args.timeout < float("inf"):
        parser.error("--timeout must be finite and positive")
    if not (root / "Cargo.toml").is_file():
        parser.error("--root must contain Cargo.toml")
    output = (args.output or root / "target" / "sdk-evidence" / datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%S%fZ")).resolve()
    metadata = {"started_at": now(), "source_sha": probe(["git", "rev-parse", "HEAD"], root),
                "git_status": probe(["git", "status", "--porcelain"], root),
                "os": platform.platform(), "python": sys.version,
                "rustc": probe(["rustc", "--version", "--verbose"], root),
                "cargo": probe(["cargo", "--version"], root), "profile": args.profile,
                "manual_validation": "NOT_RUN", "independent_review": "NOT_RUN"}
    try:
        report = run_suite(profiles()[args.profile], root, output, args.timeout, args.dry_run)
    except OSError as error:
        print(f"BLOCKED: {error}", file=sys.stderr)
        return 2
    report.update(metadata, finished_at=now())
    (output / "report.json").write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"Evidence: {output / 'report.json'}")
    if args.dry_run:
        return 0
    return {"PASS": 0, "FAIL": 1}.get(report["status"], 2)


if __name__ == "__main__":
    raise SystemExit(main())
