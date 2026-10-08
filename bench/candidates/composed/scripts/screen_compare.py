"""Supplementary view (not a gate): compare golden and candidate by what a terminal shows (pyte,
cell by cell with attributes), so SGR spelling differences are separated from layout differences."""

import json
import sys
from collections import defaultdict
from pathlib import Path

import pyte

BENCH = Path(__file__).resolve().parents[3]
NAMES = {"brown": "yellow", "brightbrown": "brightyellow"}


def snapshot(data: bytes, width: int):
    screen = pyte.Screen(width, 300)
    pyte.ByteStream(screen).feed(data.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n"))
    rows = []
    for y in range(300):
        row = []
        for x in range(width):
            c = screen.buffer[y][x]
            row.append((c.data, NAMES.get(c.fg, c.fg), NAMES.get(c.bg, c.bg), c.bold, c.italics, c.underscore, c.strikethrough, c.reverse))
        rows.append(row)
    while rows and all(c[0] == " " and c[1:3] == ("default", "default") and not any(c[3:]) for c in rows[-1]):
        rows.pop()
    return rows


def plain(rows):
    return "\n".join("".join(c[0] for c in r).rstrip() for r in rows)


def main():
    cand = Path(sys.argv[1])
    cls = json.loads((cand / "classifications.json").read_text()) if (cand / "classifications.json").exists() else {}
    per = defaultdict(lambda: defaultdict(int))
    detail = {}
    for line in (BENCH / "cases/correctness.jsonl").read_text().splitlines():
        case = json.loads(line)
        cid, feat, w = case["id"], case["feature"], case["width"]
        if cls.get(cid, {}).get("class") == "reference_quirk":
            per[feat]["excluded"] += 1
            continue
        p = cand / f"{cid}.ansi"
        if not p.exists():
            per[feat]["missing"] += 1
            continue
        want = snapshot((BENCH / "golden/correctness" / f"{cid}.ansi").read_bytes(), w)
        got = snapshot(p.read_bytes(), w)
        if want == got:
            per[feat]["screen_equal"] += 1
        elif plain(want) == plain(got):
            per[feat]["text_equal_attrs_differ"] += 1
            detail[cid] = "attrs"
        else:
            per[feat]["layout_differs"] += 1
            detail[cid] = "layout"
    for feat, d in sorted(per.items()):
        n = d["screen_equal"] + d["text_equal_attrs_differ"] + d["layout_differs"] + d["missing"]
        print(f"{feat:9s} screen_equal {d['screen_equal']:3d}/{n:3d}  text_equal_attrs_differ {d['text_equal_attrs_differ']:3d}  layout_differs {d['layout_differs']:3d}  excluded {d['excluded']}")
    tot_eq = sum(d["screen_equal"] for d in per.values())
    tot = sum(d["screen_equal"] + d["text_equal_attrs_differ"] + d["layout_differs"] + d["missing"] for d in per.values())
    print(f"overall   screen_equal {tot_eq}/{tot}")
    if len(sys.argv) > 2:
        Path(sys.argv[2]).write_text(json.dumps(detail, indent=1, sort_keys=True))


if __name__ == "__main__":
    main()
