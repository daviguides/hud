"""Supplementary analysis (not a gate): are byte-different outputs equal on screen?

Usage: uv run python candidates/rich_rs/analysis/screen_equiv.py <cases.jsonl> <golden_dir> <cand_dir> [--show N]

Byte identity is the pre-registered correctness metric. This script separates differences that are only SGR
spelling (same screen cells) from differences a user would see. It never changes a verdict.
"""

import argparse
import collections
import json
from pathlib import Path

import pyte

ROWS = 400


def snapshot(data: bytes, cols: int):
    data = data.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")
    screen = pyte.Screen(cols, ROWS)
    pyte.ByteStream(screen).feed(data)
    rows = []
    for y in range(ROWS):
        row = []
        for x in range(cols):
            c = screen.buffer[y][x]
            row.append((c.data, c.fg, c.bg, c.bold, c.italics, c.underscore, c.strikethrough, c.reverse))
        rows.append(row)
    while rows and all(c == (" ", "default", "default", False, False, False, False, False) for c in rows[-1]):
        rows.pop()
    return rows


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("cases", type=Path)
    ap.add_argument("golden", type=Path)
    ap.add_argument("cand", type=Path)
    ap.add_argument("--show", type=int, default=0)
    args = ap.parse_args()
    per = collections.defaultdict(lambda: collections.Counter())
    shown = 0
    for line in args.cases.read_text().splitlines():
        if not line.strip():
            continue
        c = json.loads(line)
        g = (args.golden / f"{c['id']}.ansi").read_bytes()
        o = (args.cand / f"{c['id']}.ansi").read_bytes()
        feat = c["feature"]
        if g == o:
            per[feat]["byte_identical"] += 1
            continue
        eq = snapshot(g, c["width"]) == snapshot(o, c["width"])
        per[feat]["same_screen_different_bytes" if eq else "different_screen"] += 1
        if not eq and shown < args.show:
            shown += 1
            print("==", c["id"], c["color_system"], c["width"], json.dumps(c["renderable"], ensure_ascii=False)[:300])
            print(" want", repr(g[:300]))
            print(" got ", repr(o[:300]))
    tot = collections.Counter()
    for feat in sorted(per):
        print(f"{feat:9s}", dict(per[feat]))
        tot.update(per[feat])
    print("all      ", dict(tot))


if __name__ == "__main__":
    main()
