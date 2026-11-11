"""Automated X11 CPU-window evidence; compare toolbar pixels, not human sign-off."""
import hashlib
import json
import os
import platform
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path

from PIL import Image, ImageChops, ImageGrab

ROOT = Path(__file__).resolve().parents[4]
OUT = ROOT / 'docs/team/roadmap-047'
BAND = (200, 0, 1100, 40)


def now():
    return datetime.now(timezone.utc).isoformat()


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def compare(first, second):
    with Image.open(first) as a, Image.open(second) as b:
        a = a.crop(BAND).convert('RGB')
        b = b.crop(BAND).convert('RGB')
        equal = a.tobytes() == b.tobytes()
        return {
            'paths': [str(path.relative_to(ROOT)) for path in (first, second)],
            'region_xyxy': BAND,
            'identical': equal,
            'difference_bounds': ImageChops.difference(a, b).getbbox(),
            'pixel_sha256': [hashlib.sha256(im.tobytes()).hexdigest() for im in (a, b)],
        }


record = {
    'started_at': now(),
    'source_sha': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
    'git_status': subprocess.check_output(['git', 'status', '--porcelain'], cwd=ROOT, text=True),
    'os': platform.platform(),
    'binary_sha256': sha(ROOT / 'target/debug/acme-ui-gallery'),
    'manual_validation': 'NOT_RUN',
    'scenario': 'Start on Mobile, select Overlay, scroll, Next/Next/Finish in light and dark CPU windows',
    'commands': [], 'captures': [], 'comparisons': [],
}
server = app = None
log_path = OUT / 'logs/native-window.txt'
with log_path.open('w') as log:
    try:
        args = ['Xvfb', '-displayfd', '1', '-screen', '0', '1100x1000x24', '-nolisten', 'tcp']
        record['commands'].append(args)
        server = subprocess.Popen(args, stdout=subprocess.PIPE, stderr=log, text=True)
        display = ':' + server.stdout.readline().strip()
        env = {**os.environ, 'DISPLAY': display, 'WINIT_UNIX_BACKEND': 'x11', 'ACME_RENDER_BACKEND': 'cpu'}
        record['environment'] = {key: env[key] for key in ('DISPLAY', 'WINIT_UNIX_BACKEND', 'ACME_RENDER_BACKEND')}

        def run(*args):
            record['commands'].append(list(args))
            return subprocess.run(args, env=env, check=True, capture_output=True, text=True, timeout=15).stdout

        for mode in ('light', 'dark'):
            work = ROOT / f'target/roadmap-047/native-{mode}'
            work.mkdir(parents=True, exist_ok=True)
            (work / 'acme-gallery-theme.conf').write_text(f'pack=default\ndark={str(mode == "dark").lower()}\n')
            args = [str(ROOT / 'target/debug/acme-ui-gallery')]
            record['commands'].append({'argv': args, 'cwd': str(work), 'mode': mode})
            app = subprocess.Popen(args, cwd=work, env=env, stdout=log, stderr=log)
            wid = None
            for _ in range(100):
                found = subprocess.run(['xdotool', 'search', '--name', '^AcmeUI Gallery$'], env=env, capture_output=True, text=True)
                if found.returncode == 0:
                    wid = found.stdout.splitlines()[0]
                    break
                if app.poll() is not None:
                    raise RuntimeError('Gallery exited')
                time.sleep(.1)
            if wid is None:
                raise RuntimeError('No Gallery window')
            run('xdotool', 'windowsize', '--sync', wid, '1100', '900')
            run('xdotool', 'windowfocus', '--sync', wid)
            record.setdefault('geometry', {})[mode] = run('xdotool', 'getwindowgeometry', '--shell', wid)
            time.sleep(1)
            run('xdotool', 'mousemove', '--window', wid, '80', '238', 'click', '1')
            time.sleep(.5)
            run('xdotool', 'mousemove', '--window', wid, '1000', '800', 'click', '--repeat', '3', '--delay', '150', '5')
            time.sleep(.7)

            def capture(stage):
                run('xdotool', 'mousemove', '--window', wid, '1000', '750')
                time.sleep(.3)
                path = OUT / f'gallery-toolbar-{mode}-{stage}.png'
                im = ImageGrab.grab(xdisplay=display)
                im.save(path)
                record['captures'].append({'captured_at': now(), 'path': str(path.relative_to(ROOT)), 'dimensions': list(im.size), 'sha256': sha(path)})
                return path

            first = capture('first')
            for x, y in [(264, 632), (345, 632), (345, 632)]:
                run('xdotool', 'mousemove', '--window', wid, str(x), str(y), 'click', '1')
                time.sleep(.6)
            finished = capture('finished')
            result = compare(first, finished)
            record['comparisons'].append(result)
            assert result['identical'], f'{mode} toolbar pixels changed across Tour lifecycle'
            with Image.open(first) as im:
                path = OUT / f'gallery-toolbar-{mode}-detail.png'
                im.crop(BAND).save(path)
                record['captures'].append({'path': str(path.relative_to(ROOT)), 'dimensions': [900, 40], 'sha256': sha(path), 'derived_from': str(first.relative_to(ROOT)), 'region_xyxy': BAND})
            app.terminate()
            app.wait(timeout=5)
            app = None

        original = ROOT / 'docs/team/roadmap-046'
        record['comparisons'].append(compare(original / 'gallery-tour-first.png', original / 'gallery-tour-finished.png'))
        assert record['comparisons'][-1]['identical'], 'Historical toolbar bands differ'
        with Image.open(OUT / 'gallery-toolbar-light-detail.png') as a, Image.open(OUT / 'gallery-toolbar-dark-detail.png') as b:
            assert a.tobytes() != b.tobytes(), 'Theme scenarios must render different toolbar palettes'
        record['status'] = 'PASS'
    except Exception as error:
        record.update(status='FAIL', reason=str(error))
        raise
    finally:
        for process in (app, server):
            if process is not None:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
        record.update(finished_at=now(), log='logs/native-window.txt', log_sha256=sha(log_path))
        (OUT / 'logs/native-window.json').write_text(json.dumps(record, indent=2) + '\n')
print(record['status'], flush=True)
