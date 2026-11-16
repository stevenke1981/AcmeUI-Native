"""Replay pointer input in a real X11 CPU Gallery; not human acceptance."""
import hashlib
import json
import os
import platform
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path

from PIL import ImageGrab

root = Path(__file__).resolve().parents[4]
out = root / "docs/team/roadmap-054"
work = root / "target/roadmap-054/native-final"
work.mkdir(parents=True, exist_ok=True)
now = lambda: datetime.now(timezone.utc).isoformat()
record = {
    "started_at": now(),
    "source_sha": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip(),
    "git_status": subprocess.check_output(["git", "status", "--porcelain"], cwd=root, text=True),
    "os": platform.platform(),
    "execution": "single Codex agent; author inspection of automated captures",
    "manual_validation": "NOT_RUN",
    "commands": [],
    "captures": [],
}
server = app = None
with (out / "logs/native-window.txt").open("w") as log:
    try:
        args = ["Xvfb", "-displayfd", "1", "-screen", "0", "1100x1000x24", "-nolisten", "tcp"]
        record["commands"].append(args)
        server = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=log, text=True)
        display = ":" + server.stdout.readline().strip()
        env = {**os.environ, "DISPLAY": display, "WINIT_UNIX_BACKEND": "x11", "ACME_RENDER_BACKEND": "cpu"}
        record["environment"] = {k: env[k] for k in ["DISPLAY", "WINIT_UNIX_BACKEND", "ACME_RENDER_BACKEND"]}
        args = [str(root / "target/debug/acme-ui-gallery")]
        record["commands"].append(args)
        app = subprocess.Popen(args, cwd=work, env=env, stdout=log, stderr=log)
        for _ in range(100):
            p = subprocess.run(["xdotool", "search", "--name", "^AcmeUI Gallery$"], env=env, capture_output=True, text=True)
            if p.returncode == 0:
                wid = p.stdout.splitlines()[0]
                break
            if app.poll() is not None:
                raise RuntimeError("Gallery exited")
            time.sleep(.1)
        else:
            raise RuntimeError("No Gallery window")

        def run(*args):
            record["commands"].append(list(args))
            return subprocess.check_output(args, env=env, text=True, timeout=15)

        def click(x, y):
            run("xdotool", "mousemove", "--window", wid, str(x), str(y), "click", "1")
            time.sleep(.5)

        def capture(name):
            run("xdotool", "mousemove", "--window", wid, "1090", "990")
            time.sleep(.2)
            path = out / f"gallery-calendar-{name}.png"
            im = ImageGrab.grab(xdisplay=display)
            im.save(path)
            record["captures"].append({"captured_at": now(), "path": str(path.relative_to(root)), "dimensions": list(im.size), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()})

        run("xdotool", "windowsize", wid, "1100", "1000")
        run("xdotool", "windowfocus", "--sync", wid)
        time.sleep(.6)
        record["geometry"] = run("xdotool", "getwindowgeometry", "--shell", wid)
        click(80, 112)  # Foundations category.
        capture("range")
        click(650, 409)  # Clear range.
        capture("clear")
        click(650, 409)  # Restore range.
        click(265, 20)  # Dark theme.
        capture("range-dark")
        record["status"] = "CAPTURED"
    except Exception as error:
        record.update(status="FAIL", reason=str(error))
        raise
    finally:
        for process in [app, server]:
            if process is not None:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
        record["finished_at"] = now()
        (out / "logs/native-window.json").write_text(json.dumps(record, indent=2) + "\n")
print(record["status"])
