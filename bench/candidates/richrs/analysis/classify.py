"""Failure attribution for the richrs correctness run (bench-design.md, pilot validity 5).

Usage: classify.py <candidate_out_dir> [--write]
For every failed or unsupported case: is the visible screen equal to the golden (only the SGR spelling
differs) or not, plus a root-cause tag from the case description. Writes classifications.json and
runs.jsonl next to the candidate output when --write is given. Supplementary analysis: the gate
remains byte equality (compare.py).
"""

import json
import sys
from collections import Counter, defaultdict
from pathlib import Path

import pyte

BENCH = Path(__file__).resolve().parents[3]
COLOR_NAMES = {"brown": "yellow", "brightbrown": "brightyellow"}


def screen(data: bytes, width: int):
    data = data.replace(b"\r\n", b"\n").replace(b"\n", b"\r\n")
    rows = max(60, data.count(b"\n") + 5)
    scr = pyte.Screen(width, rows)
    pyte.ByteStream(scr).feed(data)
    out = []
    for y in range(rows):
        line = []
        for x in range(width):
            ch = scr.buffer[y][x]
            line.append((ch.data, COLOR_NAMES.get(ch.fg, ch.fg), COLOR_NAMES.get(ch.bg, ch.bg), ch.bold, ch.italics,
                         ch.underscore, ch.strikethrough, ch.reverse))
        out.append(line)
    return out


def plain(data: bytes) -> str:
    scr = screen(data, 200)
    return "\n".join("".join(c[0] for c in row).rstrip() for row in scr).rstrip("\n")


def causes(case, got: bytes, want: bytes):
    r, feat = case["renderable"], case["feature"]
    tags = []
    if plain(got) != plain(want):
        tags.append("text-layout")
    if case["color_system"] in ("256", "standard") and (b"38;2" in got or b"48;2" in got):
        tags.append("color-downgrade-missing")
    if feat == "table":
        if any(c["justify"] in ("right", "center") for c in r["columns"]):
            tags.append("column-justify-ignored")
        if r["show_lines"]:
            tags.append("show-lines-ignored")
        if r["title"] or r["caption"]:
            tags.append("title-caption")
    if feat == "panel":
        tags.append("title-left-aligned") if r["title"] else None
    if feat == "tree":
        tags.append("tree-indent-2-cols")
    if feat == "progress":
        tags.append("progress-layout")
    if feat in ("panel", "error", "markup", "style", "table") and plain(got).count("\n") != plain(want).count("\n"):
        tags.append("no-wrap")
    return tags or ["style-rendering"]



def main():
    cand = Path(sys.argv[1])
    cases = [json.loads(l) for l in (BENCH / "cases" / "correctness.jsonl").read_text().splitlines()]
    cls, runs = {}, []
    summary = defaultdict(Counter)
    for c in cases:
        cid, feat = c["id"], c["feature"]
        want = (BENCH / "golden" / "correctness" / f"{cid}.ansi").read_bytes()
        if (cand / f"{cid}.unsupported").exists():
            reason = (cand / f"{cid}.unsupported").read_text()
            runs.append({"id": cid, "feature": feat, "result": "unsupported", "class": "candidate_failure", "reason": reason})
            summary[feat]["unsupported"] += 1
            continue
        got = (cand / f"{cid}.ansi").read_bytes()
        if got == want:
            runs.append({"id": cid, "feature": feat, "result": "pass"})
            summary[feat]["pass"] += 1
            continue
        same_screen = screen(got, c["width"]) == screen(want, c["width"])
        tags = ["sgr-spelling-only"] if same_screen else causes(c, got, want)
        cls[cid] = {"class": "candidate_bug", "reason": ",".join(tags)}
        runs.append({"id": cid, "feature": feat, "result": "fail", "class": "candidate_failure", "screen_equal": same_screen, "tags": tags})
        summary[feat]["screen_equal" if same_screen else "screen_differs"] += 1
    for feat, cnt in sorted(summary.items()):
        print(f"{feat:9s} {dict(cnt)}")
    tags = Counter(t for r in runs if r["result"] == "fail" for t in r["tags"])
    print("root-cause tags:", dict(tags.most_common()))
    if "--write" in sys.argv:
        (cand / "classifications.json").write_text(json.dumps(cls, indent=1) + "\n")
        (cand.parent / "runs.jsonl").write_text("".join(json.dumps(r) + "\n" for r in runs))


if __name__ == "__main__":
    main()
