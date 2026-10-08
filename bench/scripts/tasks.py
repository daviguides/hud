"""DX task suite: run a solution, compare with the golden, build goldens from the Python references.

Check modes:
  bytes   exact stdout bytes (stream = pipe)
  screen  final screen of a 100 column terminal emulator (pyte), cell by cell: character, fg, bg,
          bold, italic, underline, strikethrough, reverse. Insensitive to how the SGR bytes were
          written, sensitive to what the user sees.
"""

import argparse
import fcntl
import json
import os
import pty
import struct
import subprocess
import sys
import termios
from pathlib import Path

import pyte

sys.path.insert(0, str(Path(__file__).parent))
from common import BENCH, read_pty  # noqa: E402

SPEC = json.loads((BENCH / "spec" / "tasks.json").read_text())
COLS, ROWS = SPEC["terminal"]["cols"], SPEC["terminal"]["rows"]
GOLDEN = BENCH / "golden" / "tasks"


def base_env(run_env):
    env = {"PATH": os.environ["PATH"], "HOME": os.environ.get("HOME", "/tmp"), "LANG": "en_US.UTF-8"}
    env.update(run_env)
    return env


def capture(cmd, run, cwd=None, timeout=120):
    env = base_env(run["env"])
    if run["stream"] == "pipe":
        proc = subprocess.run(cmd, env=env, cwd=cwd, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                              stderr=subprocess.DEVNULL, timeout=timeout)
        return proc.stdout
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", ROWS, COLS, 0, 0))
    proc = subprocess.Popen(cmd, env=env, cwd=cwd, stdin=subprocess.DEVNULL, stdout=slave,
                            stderr=subprocess.DEVNULL, close_fds=True)
    return read_pty(master, slave, proc, timeout)


COLOR_NAMES = {"brown": "yellow", "brightbrown": "brightyellow"}


def screen_snapshot(data: bytes):
    data = data.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")
    screen = pyte.Screen(COLS, ROWS)
    pyte.ByteStream(screen).feed(data)
    rows = []
    for y in range(ROWS):
        spans = []
        for x in range(COLS):
            ch = screen.buffer[y][x]
            attrs = (COLOR_NAMES.get(ch.fg, ch.fg), COLOR_NAMES.get(ch.bg, ch.bg), ch.bold, ch.italics, ch.underscore, ch.strikethrough, ch.reverse)
            if spans and spans[-1][1] == attrs:
                spans[-1][0] += ch.data
            else:
                spans.append([ch.data, attrs])
        while spans and spans[-1][0].strip() == "" and spans[-1][1][:2] == ("default", "default") and not any(spans[-1][1][2:]):
            spans.pop()
        rows.append([{"text": t, "fg": a[0], "bg": a[1], "bold": a[2], "italic": a[3], "underline": a[4],
                      "strike": a[5], "reverse": a[6]} for t, a in spans])
    while rows and not rows[-1]:
        rows.pop()
    return rows


def screen_text(rows):
    return "\n".join("".join(s["text"] for s in r).rstrip() for r in rows)


def golden_path(task_id, run):
    ext = "bytes" if run["check"] == "bytes" else "screen.json"
    return GOLDEN / task_id / f"{run['name']}.{ext}"


def load_golden(task_id, run):
    p = golden_path(task_id, run)
    return p.read_bytes() if run["check"] == "bytes" else json.loads(p.read_text())


def find_task(task_id):
    return next(t for t in SPEC["tasks"] if t["id"] == task_id)


def check_run(task, run, cmd, cwd=None):
    out = capture(cmd, run, cwd)
    want = load_golden(task["id"], run)
    if run["check"] == "bytes":
        return {"run": run["name"], "ok": out == want, "got_len": len(out), "want_len": len(want)}
    got = screen_snapshot(out)
    return {"run": run["name"], "ok": got == want, "got": screen_text(got), "want": screen_text(want)}


def make_golden():
    ref_python = [str(BENCH / ".venv" / "bin" / "python")]
    for task in SPEC["tasks"]:
        for run in task["runs"]:
            out = capture(ref_python + [str(BENCH / task["ref"])], run)
            p = golden_path(task["id"], run)
            p.parent.mkdir(parents=True, exist_ok=True)
            if run["check"] == "bytes":
                p.write_bytes(out)
            else:
                p.write_text(json.dumps(screen_snapshot(out), indent=1, ensure_ascii=False) + "\n")
            print("golden", p.relative_to(BENCH))


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="mode", required=True)
    sub.add_parser("make-golden")
    s = sub.add_parser("selftest", help="run the Python Rich references against their goldens")
    c = sub.add_parser("check", help="run one task's solution command against its goldens")
    c.add_argument("task_id")
    c.add_argument("cmd", nargs=argparse.REMAINDER)
    args = ap.parse_args()
    if args.mode == "make-golden":
        make_golden()
        return
    py = [str(BENCH / ".venv" / "bin" / "python")]
    tasks = SPEC["tasks"] if args.mode == "selftest" else [find_task(args.task_id)]
    bad = 0
    for task in tasks:
        cmd = py + [str(BENCH / task["ref"])] if args.mode == "selftest" else [a for a in args.cmd if a != "--"]
        for run in task["runs"]:
            r = check_run(task, run, cmd)
            bad += not r["ok"]
            print(f"{'PASS' if r['ok'] else 'FAIL'} {task['id']}/{r['run']}")
            if not r["ok"] and "got" in r:
                print("--- want\n" + r["want"] + "\n--- got\n" + r["got"])
    raise SystemExit(1 if bad else 0)


if __name__ == "__main__":
    main()
