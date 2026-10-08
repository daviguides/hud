"""Capability matrix: depth x environment variable x stream = 40 cells.

The expectation is written here, independent of Rich, from no-color.org
(NO_COLOR), bixense.com/clicolors (CLICOLOR, CLICOLOR_FORCE), the de-facto
FORCE_COLOR convention and TERM/COLORTERM depth detection. Where those
standards are silent the cell is marked `standards_silent` and follows Rich.

The test program is the one-line adapter every candidate provides: print
`x` in bold and #ff8800, then a newline, through the candidate's default
console (no explicit overrides).
"""

import argparse
import fcntl
import json
import os
import pty
import re
import struct
import subprocess
import sys
import termios
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from common import BENCH  # noqa: E402

DEPTHS = {
    "truecolor": {"TERM": "xterm-256color", "COLORTERM": "truecolor"},
    "256": {"TERM": "xterm-256color"},
    "16": {"TERM": "xterm"},
    "none": {"TERM": "dumb"},
}
ENVS = {
    "unset": {},
    "NO_COLOR": {"NO_COLOR": "1"},
    "FORCE_COLOR": {"FORCE_COLOR": "1"},
    "CLICOLOR": {"CLICOLOR": "0"},
    "CLICOLOR_FORCE": {"CLICOLOR_FORCE": "1"},
}
STREAMS = ["tty", "pipe"]

SGR_RE = re.compile(rb"\x1b\[([0-9;]*)m")


def expected(depth, env, stream):
    tty = stream == "tty"
    color_on = tty
    notes = []
    if env in ("FORCE_COLOR", "CLICOLOR_FORCE"):
        color_on = True
    if env == "CLICOLOR":
        color_on = False
        notes.append("CLICOLOR=0 disables color on a tty (bixense)")
    if depth == "none":
        if color_on and env in ("FORCE_COLOR", "CLICOLOR_FORCE"):
            notes.append("standards_silent: forced color on TERM=dumb; follows Rich (no escapes)")
        color_on = False
    if not color_on:
        return {"escapes": False, "bold": False, "color_class": "none", "notes": notes}
    if env == "NO_COLOR":
        notes.append("NO_COLOR removes color, keeps bold (no-color.org)")
        return {"escapes": True, "bold": True, "color_class": "none", "notes": notes}
    cls = {"truecolor": "truecolor", "256": "256", "16": "standard"}[depth]
    return {"escapes": True, "bold": True, "color_class": cls, "notes": notes}


def classify(out: bytes):
    """Observed {escapes, bold, color_class} from captured bytes."""
    seqs = SGR_RE.findall(out)
    params = [p.decode() for p in seqs]
    bold = color = False
    cls = "none"
    for p in params:
        parts = p.split(";") if p else ["0"]
        i = 0
        while i < len(parts):
            v = parts[i]
            if v == "1":
                bold = True
            elif v == "38" and i + 1 < len(parts):
                if parts[i + 1] == "2":
                    cls = "truecolor"
                    i += 4
                elif parts[i + 1] == "5":
                    cls = "256"
                    i += 2
            elif v.isdigit() and (30 <= int(v) <= 37 or 90 <= int(v) <= 97):
                cls = "standard" if cls == "none" else cls
            i += 1
    escapes = b"\x1b" in out
    return {"escapes": escapes, "bold": bold, "color_class": cls}


def matrix():
    cells = []
    for depth in DEPTHS:
        for env in ENVS:
            for stream in STREAMS:
                cell = {"id": f"cap-{depth}-{env}-{stream}", "depth": depth, "env": env, "stream": stream}
                cell["expect"] = expected(depth, env, stream)
                cells.append(cell)
    return cells


def run_cell(cmd, cell, cwd=None):
    env = {"PATH": os.environ["PATH"], "HOME": os.environ.get("HOME", "/tmp"), "LANG": "en_US.UTF-8"}
    env.update(DEPTHS[cell["depth"]])
    env.update(ENVS[cell["env"]])
    if cell["stream"] == "pipe":
        proc = subprocess.run(cmd, env=env, cwd=cwd, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                              stderr=subprocess.DEVNULL, timeout=60)
        return proc.stdout
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 100, 0, 0))
    proc = subprocess.Popen(cmd, env=env, cwd=cwd, stdin=subprocess.DEVNULL, stdout=slave,
                            stderr=subprocess.DEVNULL, close_fds=True)
    os.close(slave)
    chunks = []
    while True:
        try:
            data = os.read(master, 65536)
        except OSError:
            break
        if not data:
            break
        chunks.append(data)
    proc.wait(timeout=60)
    os.close(master)
    return b"".join(chunks).replace(b"\r\n", b"\n")


def check(cmd, cwd=None, outdir=None):
    results = []
    for cell in matrix():
        out = run_cell(cmd, cell, cwd)
        got = classify(out)
        want = {k: cell["expect"][k] for k in ("escapes", "bold", "color_class")}
        ok = got == want
        results.append({"id": cell["id"], "ok": ok, "want": want, "got": got,
                        "notes": cell["expect"]["notes"], "bytes": out.decode("utf-8", "replace")})
        if outdir:
            Path(outdir).mkdir(parents=True, exist_ok=True)
            (Path(outdir) / f"{cell['id']}.ansi").write_bytes(out)
    return results


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest="mode", required=True)
    sub.add_parser("write-matrix")
    c = sub.add_parser("check", help="run the adapter for all 40 cells and compare to the written expectation")
    c.add_argument("--out", type=Path, help="write the report JSON here")
    c.add_argument("--capture", type=Path, help="also save raw captures here")
    c.add_argument("cmd", nargs=argparse.REMAINDER, help="adapter command after --")
    args = ap.parse_args()
    if args.mode == "write-matrix":
        path = BENCH / "cases" / "capability_matrix.json"
        path.write_text(json.dumps(matrix(), indent=2, sort_keys=True) + "\n")
        print(f"{len(matrix())} cells -> {path}")
        return
    cmd = [a for a in args.cmd if a != "--"]
    results = check(cmd, outdir=args.capture)
    passed = sum(r["ok"] for r in results)
    for r in results:
        if not r["ok"]:
            print(f"MISMATCH {r['id']}: want {r['want']} got {r['got']}")
    print(f"capability matrix: {passed}/{len(results)} cells match the written expectation")
    if args.out:
        args.out.write_text(json.dumps(results, indent=2, ensure_ascii=False) + "\n")


if __name__ == "__main__":
    main()
