"""Criterion S6 of v0.8: the 24 cell format matrix and the 6 override cells of spec/format.md, through a real
binary (candidates/hud/target/release/format_probe) on a pseudo-terminal and on a pipe.

  format_matrix.py [--out DIR]   writes pilot/hud/format_matrix.{json,txt}

Expected (spec/format.md): HUD_FORMAT unset, `rich` and `bogus` write rich (styled on a terminal and with
FORCE_COLOR, no escape on a plain pipe, where the text equals the plain table); `plain` never has an escape; `json`
(any ASCII case) is the document of golden/tasks/t09-structured/pipe_json.bytes. A console format chosen in code wins.
"""

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
from common import BENCH, ANSI_RE  # noqa: E402
from tasks import capture  # noqa: E402

PROBE = str(BENCH / "candidates" / "hud" / "target" / "release" / "format_probe")
GOLD = BENCH / "golden" / "tasks" / "t09-structured"
PLAIN = (GOLD / "pipe_plain.bytes").read_bytes()
JSON_DOC = (GOLD / "pipe_json.bytes").read_bytes()
BASE = {"TERM": "xterm-256color", "COLORTERM": "truecolor"}


def norm(data: bytes) -> bytes:
    return data.replace(b"\r\n", b"\n")


def run_cell(stream, env):
    run = {"stream": stream, "env": {**BASE, "COLUMNS": "100", **env}}
    return norm(capture([PROBE], run))


def check(expect, data):
    esc = b"\x1b" in data
    if expect == "styled":
        return esc and ANSI_RE.sub(b"", data) == PLAIN
    if expect == "unstyled":
        return not esc and data == PLAIN
    if expect == "plain":
        return not esc and data == PLAIN
    if expect == "json":
        return data == JSON_DOC
    raise AssertionError(expect)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=str(BENCH / "pilot" / "hud"))
    args = ap.parse_args()
    cells = []
    values = [None, "rich", "plain", "json", "JSON", "bogus"]
    for value in values:
        for stream in ("tty", "pipe"):
            for force in (None, "1"):
                if stream == "tty" and force:
                    continue  # a terminal needs no FORCE_COLOR; the matrix counts it once per row
                env = {}
                if value is not None:
                    env["HUD_FORMAT"] = value
                if force:
                    env["FORCE_COLOR"] = force
                if value in ("json", "JSON"):
                    expect = "json"
                elif value == "plain":
                    expect = "plain"
                else:
                    expect = "styled" if stream == "tty" or force else "unstyled"
                cells.append({"kind": "matrix", "format_env": value, "stream": stream, "force_color": force,
                              "expect": expect, "env": env})
    # The spec counts 6 x 2 x 2 = 24 cells: terminal x {unset, FORCE_COLOR=1}. FORCE_COLOR=1 on a terminal is
    # added so the product is complete.
    for value in values:
        env = {"FORCE_COLOR": "1"}
        if value is not None:
            env["HUD_FORMAT"] = value
        if value in ("json", "JSON"):
            expect = "json"
        elif value == "plain":
            expect = "plain"
        else:
            expect = "styled"
        cells.append({"kind": "matrix", "format_env": value, "stream": "tty", "force_color": "1",
                      "expect": expect, "env": env})
    for builder in ("rich", "plain", "json"):
        for env_value in ("json", "plain"):
            cells.append({"kind": "override", "format_env": env_value, "builder": builder, "stream": "pipe",
                          "force_color": None, "expect": {"rich": "unstyled", "plain": "plain", "json": "json"}[builder],
                          "env": {"HUD_FORMAT": env_value, "HUD_PROBE_FORMAT": builder}})
    results = []
    for cell in cells:
        data = run_cell(cell["stream"], cell["env"])
        ok = check(cell["expect"], data)
        results.append({**cell, "ok": ok, "bytes": len(data)})
    passed = sum(r["ok"] for r in results)
    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    (out / "format_matrix.json").write_text(json.dumps(results, indent=1) + "\n")
    lines = [f"format matrix: {passed}/{len(results)} cells ({sum(r['kind'] == 'matrix' for r in results)} matrix, "
             f"{sum(r['kind'] == 'override' for r in results)} override)"]
    for r in results:
        if not r["ok"]:
            lines.append(f"  FAIL {r['kind']} HUD_FORMAT={r['format_env']} stream={r['stream']} "
                         f"force={r['force_color']} builder={r.get('builder')} expect={r['expect']}")
    (out / "format_matrix.txt").write_text("\n".join(lines) + "\n")
    print("\n".join(lines))
    raise SystemExit(0 if passed == len(results) else 1)


if __name__ == "__main__":
    main()
