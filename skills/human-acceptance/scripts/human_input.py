"""Human-like acceptance driver for native Windows apps.

Real pointer glides, real clicks, real key presses and window screenshots,
with no third-party packages (Python 3.8+ standard library + Win32 via ctypes).

Why this exists
---------------
Two failure modes make naive automation report false negatives:

1. DPI scaling. At 125%/150% scaling, a process that is not DPI aware moves
   the cursor in logical pixels while screenshots are in physical pixels, so
   clicks land at 1/scale of the target. This script makes the whole process
   per-monitor DPI aware (V2) before touching any coordinate, and verifies the
   cursor actually reached the target.
2. Extended keys. Arrows, Home/End, PageUp/PageDown, Insert/Delete and
   numpad Enter must be sent with KEYEVENTF_EXTENDEDKEY. Without it Windows
   reports numpad keys (with NumLock on: digits), and the app under test
   appears to ignore arrow keys.

Usage
-----
  python human_input.py --title "My App" STEP [STEP ...]
  python human_input.py --title "My App" --script steps.txt
  python human_input.py --list                 # visible top-level windows
  python human_input.py --status               # desktop/session readiness

Coordinates are physical pixels relative to the window's top-left corner,
which is exactly the pixel grid of the screenshots this script saves.

Steps
-----
  place:X,Y,W,H     move/resize the window (physical px) so it fits the screen
  move:X,Y          glide the pointer to X,Y
  click:X,Y         glide, settle, left click
  dclick:X,Y        double click
  rclick:X,Y        right click
  drag:X1,Y1,X2,Y2  press at X1,Y1, glide, release at X2,Y2
  wheel:X,Y,N       wheel N notches at X,Y (positive = up)
  key:NAME          press+release (see KEYS); chords with '+': key:ctrl+a
  type:TEXT         Unicode text (use type:"..." in a script file for spaces)
  wait:SECONDS
  shot:PATH.png     screenshot of the window rectangle
  park              move the pointer to a neutral spot (avoid hover artefacts)
  focus             re-activate the window
"""
import argparse
import ctypes
import os
import shlex
import struct
import sys
import time
import zlib
from ctypes import wintypes

if sys.platform != "win32":
    raise SystemExit("human_input.py drives the Windows desktop; run it on Windows.")

user32 = ctypes.windll.user32
gdi32 = ctypes.windll.gdi32

# --- DPI: must happen before any coordinate API -----------------------------
_PER_MONITOR_AWARE_V2 = ctypes.c_void_p(-4)
if not user32.SetProcessDpiAwarenessContext(_PER_MONITOR_AWARE_V2):
    try:
        ctypes.windll.shcore.SetProcessDpiAwareness(2)
    except OSError:
        user32.SetProcessDPIAware()

# --- SendInput structures ---------------------------------------------------
INPUT_MOUSE, INPUT_KEYBOARD = 0, 1
KEYEVENTF_EXTENDEDKEY, KEYEVENTF_KEYUP = 0x1, 0x2
KEYEVENTF_UNICODE, KEYEVENTF_SCANCODE = 0x4, 0x8
MOUSE = {"left": (0x2, 0x4), "right": (0x8, 0x10)}
MOUSEEVENTF_WHEEL = 0x800
ULONG_PTR = ctypes.c_size_t


class MOUSEINPUT(ctypes.Structure):
    _fields_ = [("dx", wintypes.LONG), ("dy", wintypes.LONG), ("mouseData", wintypes.DWORD),
                ("dwFlags", wintypes.DWORD), ("time", wintypes.DWORD), ("dwExtraInfo", ULONG_PTR)]


class KEYBDINPUT(ctypes.Structure):
    _fields_ = [("wVk", wintypes.WORD), ("wScan", wintypes.WORD), ("dwFlags", wintypes.DWORD),
                ("time", wintypes.DWORD), ("dwExtraInfo", ULONG_PTR)]


class _INPUTUNION(ctypes.Union):
    _fields_ = [("mi", MOUSEINPUT), ("ki", KEYBDINPUT), ("_pad", ctypes.c_byte * 32)]


class INPUT(ctypes.Structure):
    _fields_ = [("type", wintypes.DWORD), ("u", _INPUTUNION)]


def _send(*events):
    arr = (INPUT * len(events))(*events)
    sent = user32.SendInput(len(events), arr, ctypes.sizeof(INPUT))
    if sent != len(events):
        raise SystemExit("SendInput was blocked: target is elevated (UIPI), the desktop "
                         "is locked, or a secure desktop (UAC) is active.")


def _mouse(flags, data=0):
    return INPUT(INPUT_MOUSE, _INPUTUNION(mi=MOUSEINPUT(0, 0, data & 0xFFFFFFFF, flags, 0, 0)))


def _kbd(scan, flags, vk=0):
    return INPUT(INPUT_KEYBOARD, _INPUTUNION(ki=KEYBDINPUT(vk, scan, flags, 0, 0)))


# name -> (virtual key, needs KEYEVENTF_EXTENDEDKEY)
KEYS = {
    "left": (0x25, True), "up": (0x26, True), "right": (0x27, True), "down": (0x28, True),
    "home": (0x24, True), "end": (0x23, True), "pageup": (0x21, True), "pagedown": (0x22, True),
    "insert": (0x2D, True), "delete": (0x2E, True),
    "enter": (0x0D, False), "space": (0x20, False), "escape": (0x1B, False), "esc": (0x1B, False),
    "tab": (0x09, False), "backspace": (0x08, False),
    "shift": (0x10, False), "ctrl": (0x11, False), "alt": (0x12, False), "win": (0x5B, True),
}
KEYS.update({f"f{i}": (0x6F + i, False) for i in range(1, 25)})
KEYS.update({c: (ord(c.upper()), False) for c in "abcdefghijklmnopqrstuvwxyz0123456789"})


def _key_events(name, up):
    if name not in KEYS:
        raise SystemExit(f"unknown key '{name}'. Known: {', '.join(sorted(KEYS))}")
    vk, extended = KEYS[name]
    scan = user32.MapVirtualKeyW(vk, 0)  # MAPVK_VK_TO_VSC
    flags = KEYEVENTF_SCANCODE | (KEYEVENTF_EXTENDEDKEY if extended else 0)
    return _kbd(scan, flags | (KEYEVENTF_KEYUP if up else 0))


def key(chord):
    names = [n.strip().lower() for n in chord.split("+")]
    for n in names:
        _send(_key_events(n, up=False))
        time.sleep(0.03)
    time.sleep(0.05)
    for n in reversed(names):
        _send(_key_events(n, up=True))
        time.sleep(0.03)


def type_text(text):
    for ch in text:
        units = ch.encode("utf-16-le")
        for i in range(0, len(units), 2):
            code = int.from_bytes(units[i:i + 2], "little")
            _send(_kbd(code, KEYEVENTF_UNICODE), _kbd(code, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP))
        time.sleep(0.035)


# --- Windows -----------------------------------------------------------------
_WNDENUMPROC = ctypes.WINFUNCTYPE(wintypes.BOOL, wintypes.HWND, wintypes.LPARAM)


def list_windows():
    found = []

    def cb(hwnd, _):
        if user32.IsWindowVisible(hwnd):
            n = user32.GetWindowTextLengthW(hwnd)
            if n:
                buf = ctypes.create_unicode_buffer(n + 1)
                user32.GetWindowTextW(hwnd, buf, n + 1)
                found.append((hwnd, buf.value))
        return True

    user32.EnumWindows(_WNDENUMPROC(cb), 0)
    return found


def find_window(title):
    exact = [h for h, t in list_windows() if t == title]
    if len(exact) == 1:
        return exact[0]
    partial = [(h, t) for h, t in list_windows() if title.lower() in t.lower()]
    if len(exact) > 1 or len(partial) != 1:
        names = [t for _, t in partial] or ["<none>"]
        raise SystemExit(f"title '{title}' must match exactly one window; matches: {names}")
    return partial[0][0]


def window_rect(hwnd):
    r = wintypes.RECT()
    user32.GetWindowRect(hwnd, ctypes.byref(r))
    return r


def activate(hwnd):
    user32.ShowWindow(hwnd, 9)  # SW_RESTORE
    # Windows only lets the foreground process change focus; a no-op Alt tap
    # satisfies the "last input" rule so SetForegroundWindow is honoured.
    _send(_kbd(user32.MapVirtualKeyW(0x12, 0), KEYEVENTF_SCANCODE),
          _kbd(user32.MapVirtualKeyW(0x12, 0), KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP))
    user32.SetForegroundWindow(hwnd)
    time.sleep(0.3)
    if user32.GetForegroundWindow() != hwnd:
        raise SystemExit("could not bring the target window to the foreground")


def place(hwnd, x, y, w, h):
    user32.ShowWindow(hwnd, 9)
    user32.MoveWindow(hwnd, x, y, w, h, True)
    time.sleep(0.4)


# --- Pointer -------------------------------------------------------------------
def _cursor():
    p = wintypes.POINT()
    user32.GetCursorPos(ctypes.byref(p))
    return p.x, p.y


def move(hwnd, x, y, steps=14):
    r = window_rect(hwnd)
    tx, ty = r.left + x, r.top + y
    sx, sy = _cursor()
    for i in range(1, steps + 1):
        t = i / steps
        t = t * t * (3 - 2 * t)  # ease in/out like a hand
        user32.SetCursorPos(round(sx + (tx - sx) * t), round(sy + (ty - sy) * t))
        time.sleep(0.012)
    if _cursor() != (tx, ty):
        raise SystemExit(f"pointer at {_cursor()}, expected {(tx, ty)}: DPI or clipping problem")


def click(hwnd, x, y, button="left", count=1):
    move(hwnd, x, y)
    time.sleep(0.15)
    down, up = MOUSE[button]
    for _ in range(count):
        _send(_mouse(down))
        time.sleep(0.07)
        _send(_mouse(up))
        time.sleep(0.09)


def drag(hwnd, x1, y1, x2, y2):
    move(hwnd, x1, y1)
    time.sleep(0.12)
    _send(_mouse(MOUSE["left"][0]))
    time.sleep(0.1)
    move(hwnd, x2, y2, steps=30)
    time.sleep(0.1)
    _send(_mouse(MOUSE["left"][1]))


def wheel(hwnd, x, y, notches):
    move(hwnd, x, y)
    for _ in range(abs(notches)):
        _send(_mouse(MOUSEEVENTF_WHEEL, 120 if notches > 0 else -120))
        time.sleep(0.08)


def park(hwnd):
    r = window_rect(hwnd)
    user32.SetCursorPos(r.right + 40, r.bottom - 40)


# --- Screenshot (GDI -> PNG, standard library only) -----------------------------
class BITMAPINFOHEADER(ctypes.Structure):
    _fields_ = [("biSize", wintypes.DWORD), ("biWidth", wintypes.LONG), ("biHeight", wintypes.LONG),
                ("biPlanes", wintypes.WORD), ("biBitCount", wintypes.WORD),
                ("biCompression", wintypes.DWORD), ("biSizeImage", wintypes.DWORD),
                ("biXPelsPerMeter", wintypes.LONG), ("biYPelsPerMeter", wintypes.LONG),
                ("biClrUsed", wintypes.DWORD), ("biClrImportant", wintypes.DWORD)]


def _png(path, width, height, bgra):
    rows = bytearray()
    stride = width * 4
    for y in range(height):
        row = bgra[y * stride:(y + 1) * stride]
        rgb = bytearray(width * 3)
        rgb[0::3], rgb[1::3], rgb[2::3] = row[2::4], row[1::4], row[0::4]
        rows += b"\x00" + rgb

    def chunk(tag, data):
        return (struct.pack(">I", len(data)) + tag + data
                + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF))

    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n"
                + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 2, 0, 0, 0))
                + chunk(b"IDAT", zlib.compress(bytes(rows), 6))
                + chunk(b"IEND", b""))


def shot(hwnd, path):
    time.sleep(0.35)  # let the app finish its redraw
    r = window_rect(hwnd)
    w, h = r.right - r.left, r.bottom - r.top
    screen = user32.GetDC(0)
    mem = gdi32.CreateCompatibleDC(screen)
    bmp = gdi32.CreateCompatibleBitmap(screen, w, h)
    old = gdi32.SelectObject(mem, bmp)
    try:
        if not gdi32.BitBlt(mem, 0, 0, w, h, screen, r.left, r.top, 0x00CC0020 | 0x40000000):
            raise SystemExit("BitBlt failed (locked desktop or secure desktop?)")
        bih = BITMAPINFOHEADER(ctypes.sizeof(BITMAPINFOHEADER), w, -h, 1, 32, 0, 0, 0, 0, 0, 0)
        buf = (ctypes.c_ubyte * (w * h * 4))()
        gdi32.GetDIBits(mem, bmp, 0, h, buf, ctypes.byref(bih), 0)
    finally:
        gdi32.SelectObject(mem, old)
        gdi32.DeleteObject(bmp)
        gdi32.DeleteDC(mem)
        user32.ReleaseDC(0, screen)
    os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
    _png(path, w, h, bytes(buf))
    print(f"shot {path} ({w}x{h})")


# --- Readiness ---------------------------------------------------------------------
def status():
    desk = user32.OpenInputDesktop(0, False, 0x0100)  # DESKTOP_SWITCHDESKTOP
    unlocked = bool(desk) and bool(user32.SwitchDesktop(desk))
    if desk:
        user32.CloseDesktop(desk)
    numlock = user32.GetKeyState(0x90) & 1
    dpi = user32.GetDpiForSystem() if hasattr(user32, "GetDpiForSystem") else 96
    print(f"desktop unlocked: {unlocked}")
    print(f"system DPI: {dpi} (scale {dpi / 96:.0%})")
    print(f"NumLock: {'on' if numlock else 'off'} (arrows still work: extended flag is set)")
    print(f"screen: {user32.GetSystemMetrics(0)}x{user32.GetSystemMetrics(1)} physical px")
    return 0 if unlocked else 1


# --- Driver ----------------------------------------------------------------------
def _ints(arg, n):
    vals = [int(v) for v in arg.split(",")]
    if len(vals) != n:
        raise SystemExit(f"expected {n} comma-separated integers, got '{arg}'")
    return vals


def run(hwnd, steps):
    activate(hwnd)
    for step in steps:
        op, _, arg = step.partition(":")
        print(f"> {step}")
        if op == "place":
            place(hwnd, *_ints(arg, 4))
        elif op == "move":
            move(hwnd, *_ints(arg, 2))
        elif op == "click":
            click(hwnd, *_ints(arg, 2))
        elif op == "dclick":
            click(hwnd, *_ints(arg, 2), count=2)
        elif op == "rclick":
            click(hwnd, *_ints(arg, 2), button="right")
        elif op == "drag":
            drag(hwnd, *_ints(arg, 4))
        elif op == "wheel":
            wheel(hwnd, *_ints(arg, 3))
        elif op == "key":
            key(arg)
        elif op == "type":
            type_text(arg)
        elif op == "wait":
            time.sleep(float(arg))
        elif op == "shot":
            shot(hwnd, arg)
        elif op == "park":
            park(hwnd)
        elif op == "focus":
            activate(hwnd)
        else:
            raise SystemExit(f"unknown step '{step}'")
        time.sleep(0.12)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--title", help="exact (or uniquely partial) window title")
    ap.add_argument("--script", help="file with one step per line; '#' comments allowed")
    ap.add_argument("--list", action="store_true", help="list visible windows")
    ap.add_argument("--status", action="store_true", help="check the desktop is drivable")
    ap.add_argument("steps", nargs="*")
    a = ap.parse_args()
    if a.status:
        sys.exit(status())
    if a.list:
        for hwnd, title in list_windows():
            r = window_rect(hwnd)
            print(f"{hwnd:>10}  {r.right - r.left}x{r.bottom - r.top}+{r.left}+{r.top}  {title}")
        return
    if not a.title:
        ap.error("--title is required")
    steps = list(a.steps)
    if a.script:
        with open(a.script, encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#"):
                    steps.extend(shlex.split(line, posix=True))
    run(find_window(a.title), steps)


if __name__ == "__main__":
    main()
